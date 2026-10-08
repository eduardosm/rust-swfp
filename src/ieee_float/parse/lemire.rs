//! Fast path of float parsing, based on the Eisel-Lemire algorithm.
//!
//! The algorithm is described in "Number Parsing at a Gigabyte per Second",
//! by Daniel Lemire, available at <https://arxiv.org/abs/2101.11408>.
//!
//! The original algorithm only supports rounding to nearest with ties to even,
//! and its correctness relies on a careful analysis of the error of the
//! truncated powers of five. Instead, this implementation keeps track of an
//! interval that is known to contain the exact value, which makes it valid for
//! any rounding mode and allows to tell whether the result is exact:
//!
//! 1. The decimal string is reduced to `w * 10^q`, where `w` holds up to
//!    `MAX_DIGITS` significant digits. When there are more significant digits
//!    and some of the dropped ones are not zero, the exact value is known to be
//!    in the open interval `(w * 10^q, (w + 1) * 10^q)`.
//! 2. When `0 <= q <= 55` and no non-zero digit has been dropped, the exact
//!    value is `(w * 5^q) * 2^q`, where `w * 5^q` fits in 256 bits, so it is
//!    calculated with integer arithmetic and rounded exactly.
//! 3. Otherwise, `5^q` is approximated as `t * 2^s`, where `t` has 128 bits,
//!    which gives an open interval that contains the exact value. When a
//!    conservative check proves that no rounding boundary (a representable
//!    value or the midpoint between two consecutive representable values, as
//!    if the exponent range were unbounded) lies inside that interval, the
//!    rounded value and the direction of the rounding error are known.
//! 4. Otherwise, when `q < 0` and no non-zero digit has been dropped, the exact
//!    value might be a rounding boundary itself, which requires `5^-q` to
//!    divide `w`. In that case, the exact value is `(w / 5^-q) * 2^q`, which is
//!    rounded exactly.
//! 5. Otherwise, the result cannot be determined by this algorithm.
//!
//! Step 5 is only reached when a rounding boundary is very close to the exact
//! value, which, for inputs with at most `MAX_DIGITS` significant digits, has
//! a probability of about `2^(PREC_BITS - 125)`.

use super::ParsedFinite;
use crate::ieee_float::{IeeeFloat, Semantics};
use crate::traits::{CastFrom as _, UInt as _};
use crate::utils::{RoundLoss, wide_shift_left, wide_shift_right};
use crate::{FpStatus, Round};

/// Maximum number of significant digits kept in the decimal significand.
///
/// Any integer with up to 38 decimal digits fits in a `u128`.
const MAX_DIGITS: usize = 38;

/// Attempts to convert `parsed` into a floating-point number, rounding
/// according to `round`.
///
/// Returns `None` when the result cannot be determined by this algorithm.
pub(super) fn try_convert<S: Semantics>(
    parsed: &ParsedFinite<'_>,
    round: Round,
) -> Option<(IeeeFloat<S>, FpStatus)> {
    // `round_interval` needs some bits below the rounding bit in the high
    // half of a 256-bit product.
    const {
        assert!(S::PREC_BITS <= 113);
    }

    let sign = parsed.sign;
    let (exp, mant, loss) = try_calc_significand(parsed, S::PREC_BITS)?;

    let Ok(exp) = S::Exp::try_from(exp) else {
        return Some(if exp > 0 {
            (
                IeeeFloat::make_overflow_value(sign, round),
                FpStatus::Overflow,
            )
        } else {
            (
                IeeeFloat::make_underflow_value(sign, round),
                FpStatus::Underflow,
            )
        });
    };
    Some(IeeeFloat::round_and_classify(
        sign,
        exp,
        S::Mant::cast_from(mant),
        loss,
        round,
    ))
}

/// Attempts to calculate `(exp, mant, loss)`, as expected by
/// `round_and_classify`, for the value of `parsed` and a significand of
/// `prec_bits` bits.
///
/// Returns `None` when the result cannot be determined by this algorithm.
// Not generic and never inlined, so the bulk of the algorithm is shared by
// all the formats.
#[inline(never)]
fn try_calc_significand(
    parsed: &ParsedFinite<'_>,
    prec_bits: u32,
) -> Option<(i32, u128, RoundLoss)> {
    let dec = Decimal::from_parsed(parsed);

    // The value is not known to underflow or overflow, so `q` is within the
    // range covered by `POW5_TABLE`.
    let q = i32::try_from(dec.q).ok()?;
    let w = dec.w;

    if !dec.truncated && matches!(q, 0..=MAX_POW5_EXACT) {
        // The value is `(w * 5^q) * 2^q`, where `w < 2^127` and `5^q < 2^128`,
        // so the product fits in 256 bits.
        let (n_lo, n_hi) = w.wide_mul(POW5_EXACT[q as usize]);
        Some(round_exact(n_lo, n_hi, q, prec_bits))
    } else {
        let lz = w.leading_zeros();
        let w_n = w << lz;
        // `t * 2^s <= 5^q < (t + 3) * 2^s`
        let (t, s) = pow5_approx(q)?;

        // The value is `X * 2^e2`, where `X` is in the open interval
        // `(w_n * t, w_n * t + delta)`.
        //
        // The lower bound is strict because either some non-zero digits have
        // been dropped or `5^q * 2^-s` is not an integer (`q < 0` or `q > 55`).
        let e2 = q + s - lz as i32;
        let (a_lo, a_hi) = w_n.wide_mul(t);
        let delta_hi = if dec.truncated {
            // `X < (w_n + 2^lz) * (t + 3)`, so
            // `delta = 2^lz * t + 3 * (w_n + 2^lz) < (2^lz + 3) * 2^128`,
            // given that `t < 2^128` and `w_n + 2^lz = (w + 1) * 2^lz <= 2^128`.
            //
            // `w` has `MAX_DIGITS` digits, so `lz` is small.
            debug_assert!(lz <= 5);
            (1 << lz) + 3
        } else {
            // `X < w_n * (t + 3)`, so `delta = 3 * w_n < 3 * 2^128`.
            3
        };

        if let Some(r) = round_interval(a_lo, a_hi, delta_hi, e2, prec_bits) {
            Some(r)
        } else if !dec.truncated
            && q < 0
            && let Some(&p5) = POW5_EXACT.get(q.unsigned_abs() as usize)
            && w.is_multiple_of(p5)
        {
            // The value is exactly `(w / 5^-q) * 2^q`.
            Some(round_exact(w / p5, 0, q, prec_bits))
        } else {
            None
        }
    }
}

/// Decimal number `w * 10^q`.
struct Decimal {
    /// Significand, made of the first `MAX_DIGITS` significant digits at most.
    w: u128,
    /// Exponent.
    q: i64,
    /// Whether non-zero digits have been dropped after the first `MAX_DIGITS`
    /// significant digits. If so, the exact value is in the open interval
    /// `(w * 10^q, (w + 1) * 10^q)`.
    truncated: bool,
}

impl Decimal {
    fn from_parsed(parsed: &ParsedFinite<'_>) -> Self {
        let digits0 = parsed.digits0;
        let digits1 = parsed.digits1;

        let mut w = 0;
        let num0 = accumulate_digits(&mut w, digits0, MAX_DIGITS);
        let num1 = accumulate_digits(&mut w, digits1, MAX_DIGITS - num0);
        debug_assert_ne!(w, 0);
        let rest0 = &digits0[num0..];
        let rest1 = &digits1[num1..];

        // The value is `(w * 10^num_dropped + dropped) * 10^parsed.exp`.
        let num_dropped = rest0.len() + rest1.len();
        let truncated = rest0.iter().chain(rest1).any(|&c| c != b'0');
        let q = parsed.exp.saturating_add(num_dropped as i64);

        Self { w, q, truncated }
    }
}

/// Appends up to `max` digits from `digits` to `w`, returning the number of
/// appended digits.
#[inline]
fn accumulate_digits(w: &mut u128, digits: &[u8], max: usize) -> usize {
    let n = digits.len().min(max);
    for &c in &digits[..n] {
        *w = *w * 10 + u128::from(c - b'0');
    }
    n
}

/// Rounds `X * 2^e2` to `prec_bits` bits, where `X` is only known to be in
/// the open interval `(A, A + delta)`, with `A = a_hi * 2^128 + a_lo` in
/// `[2^254, 2^256)` and `delta < delta_hi * 2^128`.
///
/// Returns `(exp, mant, loss)` as expected by `round_and_classify`, or `None`
/// if a rounding boundary lies in the interval.
#[inline]
fn round_interval(
    a_lo: u128,
    a_hi: u128,
    delta_hi: u128,
    e2: i32,
    prec_bits: u32,
) -> Option<(i32, u128, RoundLoss)> {
    debug_assert!(a_hi >> 126 != 0);

    // Normalize so the most significant bit of `A` is bit 255.
    let (a_hi, delta_hi, e2) = if a_hi >> 127 == 0 {
        ((a_hi << 1) | (a_lo >> 127), delta_hi << 1, e2 - 1)
    } else {
        (a_hi, delta_hi, e2)
    };

    // Bit of `a_hi` right below the `prec_bits` most significant bits.
    let round_bit = 127 - prec_bits;
    let low_mask = (1 << round_bit) - 1;

    // Rounding boundaries (representable values and midpoints between them) are
    // the multiples of `2^(128 + round_bit)`. Since
    // `A mod 2^(128 + round_bit) < ((a_hi & low_mask) + 1) * 2^128`, if
    // `(a_hi & low_mask) + delta_hi <= low_mask`, no boundary lies in
    // `(A, A + delta)`, so `X` is strictly between the same two consecutive
    // boundaries as `A`.
    if (a_hi & low_mask) + delta_hi > low_mask {
        return None;
    }

    let mant = a_hi >> (round_bit + 1);
    // `X` is not a boundary, so the result is neither exact nor a tie.
    let loss = if (a_hi >> round_bit) & 1 == 0 {
        RoundLoss::HalfDown
    } else {
        RoundLoss::HalfUp
    };
    Some((e2 + 255, mant, loss))
}

/// Rounds `N * 2^e2` to `prec_bits` bits, where `N = n_hi * 2^128 + n_lo` is
/// not zero.
///
/// Returns `(exp, mant, loss)` as expected by `round_and_classify`.
#[inline]
fn round_exact(n_lo: u128, n_hi: u128, e2: i32, prec_bits: u32) -> (i32, u128, RoundLoss) {
    let lz = if n_hi != 0 {
        n_hi.leading_zeros()
    } else {
        128 + n_lo.leading_zeros()
    };
    let (a_lo, a_hi) = wide_shift_left(n_lo, n_hi, lz);

    let shift = 128 - prec_bits;
    let mant = a_hi >> shift;
    let loss = RoundLoss::from_wide_shift(a_lo, a_hi, shift + 128);
    (e2 + 255 - lz as i32, mant, loss)
}

/// Largest `n` such that `5^n` fits in a `u128`.
const MAX_POW5_EXACT: i32 = 55;

/// `POW5_EXACT[n] = 5^n`
static POW5_EXACT: [u128; MAX_POW5_EXACT as usize + 1] = {
    let mut table = [1; MAX_POW5_EXACT as usize + 1];
    let mut i = 1;
    while i < table.len() {
        table[i] = table[i - 1] * 5;
        i += 1;
    }
    table
};

/// Distance between the exponents of consecutive entries of `POW5_TABLE`.
///
/// `5^(POW5_TABLE_STEP - 1)` must be in `POW5_EXACT`.
const POW5_TABLE_STEP: i32 = 28;

/// `POW5_TABLE[0]` corresponds to `5^(POW5_TABLE_STEP * POW5_TABLE_MIN_I)`.
const POW5_TABLE_MIN_I: i32 = -179;

/// Returns `(t, s)` such that `t * 2^s <= 5^q < (t + 3) * 2^s` and
/// `2^127 <= t < 2^128`.
///
/// Returns `None` if `q` is out of the range covered by `POW5_TABLE`.
#[inline]
fn pow5_approx(q: i32) -> Option<(u128, i32)> {
    // `5^q = 5^q_hi * 5^q_lo`
    let i = q.div_euclid(POW5_TABLE_STEP);
    let q_hi = i * POW5_TABLE_STEP;
    let q_lo = q.rem_euclid(POW5_TABLE_STEP);

    // `t_hi * 2^s_hi <= 5^q_hi < (t_hi + 1) * 2^s_hi`
    let t_hi = *POW5_TABLE.get(usize::try_from(i - POW5_TABLE_MIN_I).ok()?)?;
    let s_hi = floor_log2_pow5(q_hi) - 127;

    // `p = t_hi * 5^q_lo`, so `p * 2^s_hi <= 5^q < (p + 5^q_lo) * 2^s_hi`.
    let (p_lo, p_hi) = t_hi.wide_mul(POW5_EXACT[q_lo as usize]);

    // Truncate `p` to 128 bits, so `t * 2^shift <= p < (t + 1) * 2^shift`.
    // Given that `2^127 * 5^q_lo <= p < 2^(128 + shift)`, `5^q_lo < 2^(shift + 1)`,
    // so `5^q < (t + 1 + 5^q_lo / 2^shift) * 2^(s_hi + shift) < (t + 3) * 2^(s_hi + shift)`.
    let shift = 128 - p_hi.leading_zeros();
    let t = wide_shift_right(p_lo, p_hi, shift).0;

    Some((t, s_hi + shift as i32))
}

/// Calculates `floor(q * log2(5))`.
///
/// The result is exact for `-5100 <= q <= 5100`, which includes the range
/// covered by `POW5_TABLE`.
#[inline]
fn floor_log2_pow5(q: i32) -> i32 {
    // 9972605232 = ceil(log2(5) * 2^32)
    ((i64::from(q) * 9972605232) >> 32) as i32
}

// GENERATE: pow5_table 28 -179 176
// POW5_TABLE[i] = floor(5^q * 2^(127 - floor(q * log2(5)))), where q = 28 * (i - 179)
static POW5_TABLE: [u128; 356] = [
    0xB491165AC6B0AD766DE87D653E43DF31, // 5^-5012
    0xB6536903BF8F2BDA2B55C9E70E00C557, // 5^-4984
    0xB81A1EC0EBF12AF1BAD933E1F4E65074, // 5^-4956
    0xB9E5428330737362BDDB2DFDE3F8A6E3, // 5^-4928
    0xBBB4DF56BAF62972692AA2588216D185, // 5^-4900
    0xBD89006346A9A34D88227FDFC13AB53D, // 5^-4872
    0xBF61B0EC60C4F5DC8EE3A73EE750B831, // 5^-4844
    0xC13EFC51ADE7DF64E05FE4207CA3D508, // 5^-4816
    0xC320EE0F3029BB57FF5733244E3B6BAA, // 5^-4788
    0xC50791BD8DD72EDB3C55F3F947FEF0E9, // 5^-4760
    0xC6F2F31258E041C6AFDE347F46FDB9DF, // 5^-4732
    0xC8E31DE056F89C190915564D8AB057EE, // 5^-4704
    0xCAD81E17CA6BA42708B7D94AF9C24E41, // 5^-4676
    0xCCD1FFC6BBA63E21801E38463183FC88, // 5^-4648
    0xCED0CF194377F1EB77707CAB526FA3EB, // 5^-4620
    0xD0D49859D60D40A3CFADF6B2AA7C4F43, // 5^-4592
    0xD2DD67F18EA4F7BA6819FCBC5DBA0576, // 5^-4564
    0xD4EB4A687C0253E89E601E707A2C3488, // 5^-4536
    0xD6FE4C65ED9DCAF00910B187A046B5A4, // 5^-4508
    0xD9167AB0C1965798A8EDFFDCCFE4DB4B, // 5^-4480
    0xDB33E22FB36528099B246C227911DB44, // 5^-4452
    0xDD568FE9AB559344B17CD86E7FCECE75, // 5^-4424
    0xDF7E91060EC33F465AAFDC42CA320902, // 5^-4396
    0xE1ABF2CD112066101151250681D59705, // 5^-4368
    0xE3DEC2A805C62CB438B47F50C3E4979F, // 5^-4340
    0xE6170E21B2910457025A8E1E5DBB41D6, // 5^-4312
    0xE854E2E6A34B1200C9D524DFDFE4E2D9, // 5^-4284
    0xEA984EC57DE69F1366E849253E5DA0C2, // 5^-4256
    0xECE15FAF578A9935647E32D3C54DF9DD, // 5^-4228
    0xEF3023B80A732D93F5A7800F23EF67B8, // 5^-4200
    0xF184A9168CA8907707776B7971F752FD, // 5^-4172
    0xF3DEFE25478E074A0E85FC7F4EDBD3CA, // 5^-4144
    0xF63F3162704B507048FE1D3430B5E548, // 5^-4116
    0xF8A551706112897C4268A54F70BD28C4, // 5^-4088
    0xFB116D15F344B9B0953D136B9A19CDB5, // 5^-4060
    0xFD83933EDA772C0B5052E9289F0F2333, // 5^-4032
    0xFFFBD2FC005BC9862C9AF917DDC988C9, // 5^-4004
    0x813D1DC1F0C754D601B02378A405B421, // 5^-3976
    0x827F6E1975A58A93EC2CAA7B143CE01A, // 5^-3948
    0x83C4E245ED051DC1B782DB1FC6ABA49B, // 5^-3920
    0x850D821C0C86F175753F080DAB88EE0A, // 5^-3892
    0x86595584116CAF3C4250BE2EEBA87D15, // 5^-3864
    0x87A86479F14D8EA39031FECC0841642D, // 5^-3836
    0x88FAB70D8B44952A3F1F93F1943CA9B6, // 5^-3808
    0x8A505562D9997D8A268889F30FC7A120, // 5^-3780
    0x8BA947B223E5783E2C87F18B39478AA2, // 5^-3752
    0x8D05964831B4FA23ED1E8AD53278B981, // 5^-3724
    0x8E6549867DA7D11A4054F5360249EBD1, // 5^-3696
    0x8FC869E36910B987BDFB5DAA8751F12B, // 5^-3668
    0x912EFFEA7015B2C5C1187FA0C18ADBBE, // 5^-3640
    0x9299143C5E525385772CED20F3BE4933, // 5^-3612
    0x9406AF8F83FD62654B4DE34E0EBC3E06, // 5^-3584
    0x9577DAAFEB92FA158E08F0978AC01650, // 5^-3556
    0x96EC9E7F9004839BAC73F0226EFF5EA1, // 5^-3528
    0x986503F6936FD47BAE686CF29A7B688D, // 5^-3500
    0x99E11423765EC1D02184706EA46A4C38, // 5^-3472
    0x9B60D82B4F907CA1202C9C950E81F6F2, // 5^-3444
    0x9CE4594A044E0F1BDDADB80577B906BD, // 5^-3416
    0x9E6BA0D2814B55A51F2A6E9BA997D195, // 5^-3388
    0x9FF6B82EF415D22260DBD8AA443B560F, // 5^-3360
    0xA185A8E10512BB3F2D22A5F73DE44D43, // 5^-3332
    0xA3187C82120DACE67401C6F091F87727, // 5^-3304
    0xA4AF3CC3695962A29314C38AF248CEAC, // 5^-3276
    0xA649F36E8583E81A4D5B32F713D7F476, // 5^-3248
    0xA7E8AA65499FAF6D44ED06A6C73283F1, // 5^-3220
    0xA98B6BA23E2300C7B4B39DD9DDB8D317, // 5^-3192
    0xAB324138CE5F3A2343AB66AA259BB140, // 5^-3164
    0xACDD3555869159D1EC41C1793D69D0D1, // 5^-3136
    0xAE8C523E528D52202F9B11C68554E06E, // 5^-3108
    0xB03FA252BD05A8153CA5A7540D9D56C9, // 5^-3080
    0xB1F7300C2F70E31A6CC8610FE1204DB5, // 5^-3052
    0xB3B305FE328E571F92E1BC1FBB33F18D, // 5^-3024
    0xB5732ED6AF8BD6A72C9155C7F2F76A10, // 5^-2996
    0xB737B55E31CDDE04A908FD4A88728B6A, // 5^-2968
    0xB900A478295BCCFFC3BC70DAED20545D, // 5^-2940
    0xBACE07232DF1C8027C4C65D15C614C56, // 5^-2912
    0xBC9FE87942B9DDF3984B360DB52F4726, // 5^-2884
    0xBE7653B01AAE13E5EF84CC99CB4C5D17, // 5^-2856
    0xC05154195DA4FBD52112BEF1B26149FE, // 5^-2828
    0xC230F522EE0A7FC2CFC147ADE4843A24, // 5^-2800
    0xC41542572F468EAC4068E186399DC435, // 5^-2772
    0xC5FE475D4CD35CFF4668677D5F46C29B, // 5^-2744
    0xC7EC0FF98204EE6EEB22603AA63048D9, // 5^-2716
    0xC9DEA80D6283A34C474B3CB1FE1D6A7F, // 5^-2688
    0xCBD61B98237B87D6B23C80CFBE16ABC0, // 5^-2660
    0xCDD276B6E582284FD6EA3B733029EF0B, // 5^-2632
    0xCFD3C5A4FF34B104824F4075B7D3949B, // 5^-2604
    0xD1DA14BC489025EA3736730A9E47FEF8, // 5^-2576
    0xD3E57075670581EBDA84BEAC12680510, // 5^-2548
    0xD5F5E5681A4B92853D24E68DC1027246, // 5^-2520
    0xD80B804B89F068DE014DA5D423752D8B, // 5^-2492
    0xDA264DF693AC3E30742AB8F3864562C8, // 5^-2464
    0xDC465B601A77ADF08F5F77DFDC869AC6, // 5^-2436
    0xDE6BB59F56672CDA8C119F3680212413, // 5^-2408
    0xE09669EC254DA8CF60203BCBC6354D53, // 5^-2380
    0xE2C6859F5C28423043190B523F872B9C, // 5^-2352
    0xE4FC16331955144110EAA1481B149E5A, // 5^-2324
    0xE7372943179706FC2A0969BF88679396, // 5^-2296
    0xE977CC8D01E8A9B169D9C1F7D0B33E49, // 5^-2268
    0xEBBE0DF0C8201AC5131565BE33DDA91A, // 5^-2240
    0xEE09FB70F46605EB453DBEA8FF260AC2, // 5^-2212
    0xF05BA3330181C750CCFB1CC2EF1F44DE, // 5^-2184
    0xF2B3137FB1FCC7430AD3B225CC56A181, // 5^-2156
    0xF5105AC3681F27165F8385B3A882FF4C, // 5^-2128
    0xF773878E7EC7DD452B566EF4CAF507B0, // 5^-2100
    0xF9DCA895A3226409166C15F456786C27, // 5^-2072
    0xFC4BCCB22F3C23052B49C17CF287A651, // 5^-2044
    0xFEC102E2857BC1F96C656C3B1F2C9D91, // 5^-2016
    0x809E2D25367E4BF40CC90239661BB26E, // 5^-1988
    0x81DEF119B76837C8FA70B9A2CA60B004, // 5^-1960
    0x8322D5069A14EFDCD0BE910FA323527C, // 5^-1932
    0x8469E0B6F2B8BD9B6A22490E8E9EC98B, // 5^-1904
    0x85B41C09452411445015E086841D2C28, // 5^-1876
    0x87018EEFB53C632569138459B0FA72D4, // 5^-1848
    0x8852417037EDF7DA9A8A962EDA71E86D, // 5^-1820
    0x89A63BA4C497B50E6C83AD1260FF20F4, // 5^-1792
    0x8AFD85BB86F237279F2BBAD927B779D1, // 5^-1764
    0x8C5827F711735B46D82EF2860273DE8D, // 5^-1736
    0x8DB62AAE902F73F628E92E707150BC1E, // 5^-1708
    0x8F17964DFC3961F2416D7F9AB1E67580, // 5^-1680
    0x907C73564F82CD82C1E15A2C8FF4DF56, // 5^-1652
    0x91E4CA5DB93DBFEC56700866B85D57FE, // 5^-1624
    0x9350A40FD2C0DFA4352E1FC6A1AADA9A, // 5^-1596
    0x94C0092DD4EF951143CF71D5C4FD7868, // 5^-1568
    0x9633028ECE2760D3B070FBDE944761C0, // 5^-1540
    0x97A9991FD8B3AFC0387898A6E22F821B, // 5^-1512
    0x9923D5E451C97BF8C66B5979A2CE2EF5, // 5^-1484
    0x9AA1C1F6110C0DD08F8857E875E7774E, // 5^-1456
    0x9C236685A09C3276801125C857604CA5, // 5^-1428
    0x9DA8CCDA75B341B5A5C58D5F91A476D7, // 5^-1400
    0x9F31FE5329CB4F7877BB986469851F56, // 5^-1372
    0xA0BF0465B455E9216E1F7F1642EBAAC8, // 5^-1344
    0xA24FE89FA502C23968758CBF71B19436, // 5^-1316
    0xA3E4B4A65E97B76AFAD2BE1679765F27, // 5^-1288
    0xA57D7237525B9240F77D1A9FF40226F3, // 5^-1260
    0xA71A2B283C14FBA6800CFAB80C4E2EB1, // 5^-1232
    0xA8BAE9675E9F0EB7AD3CB74FD4CAC6DE, // 5^-1204
    0xAA5FB6FBC115010B850B0C5976B21027, // 5^-1176
    0xAC089E056C96594299DAEEEDE2E0EB1B, // 5^-1148
    0xADB5A8BDAAA5305161363686961A41E5, // 5^-1120
    0xAF66E177441FFDB22C638FCBB822F998, // 5^-1092
    0xB11C529EC0D87268C6F075C4B81FC72D, // 5^-1064
    0xB2D606BAA7C8EA892EB30A609088263E, // 5^-1036
    0xB494086BBFEA00C3B4E4BE5B6455EF96, // 5^-1008
    0xB656626D51A9D353384EFD538D690C57, // 5^-980
    0xB81D1F9569068D8E24D256C540A50309, // 5^-952
    0xB9E84AD5184DCD4894CDE1BA3CFCA943, // 5^-924
    0xBBB7EF38BB827F2D6D4AA5B50BB5DC0D, // 5^-896
    0xBD8C17E83C6AD135AEBCC797B23B9BB6, // 5^-868
    0xBF64D0275747DE70925624C0D7D93317, // 5^-840
    0xC1422355E038BB648035810006A8CFB6, // 5^-812
    0xC3241CF0094A8E708E5A2E5116BAF191, // 5^-784
    0xC50AC88EA93763C0249494D1BF7C86EC, // 5^-756
    0xC6F631E782D57096B0560C246F90E9E8, // 5^-728
    0xC8E664CD8D387DF81E2BD23627C69801, // 5^-700
    0xCADB6D313C8736FC2FFFF1289A804C5A, // 5^-672
    0xCCD55720CB861B6ED95729515330F114, // 5^-644
    0xCED42EC885D9DBBEA855E127113C887B, // 5^-616
    0xD0D800731302E7A4064B9E215703F17F, // 5^-588
    0xD2E0D889C213FD60E00BAD8DFC0D8C8E, // 5^-560
    0xD4EEC394D6258BF828E54542D9B56DC9, // 5^-532
    0xD701CE3BD387BF47C654D07271E6C39F, // 5^-504
    0xD91A0545CDB51185E287C2AD77EAD647, // 5^-476
    0xDB377599B607424484C663CEE6B86E7C, // 5^-448
    0xDD5A2C3EAB3097CBBD54467EEC6DD2BB, // 5^-420
    0xDF82365C497B5453CB285CEB2FED040D, // 5^-392
    0xE1AFA13AFBD14D6D82189C09A3A1EC21, // 5^-364
    0xE3E27A444D8D98B7FD1B1B2308169B25, // 5^-336
    0xE61ACF033D1A45DF6FB92487298E33BD, // 5^-308
    0xE858AD248F5C22C9D1B3400F8F9CFF68, // 5^-280
    0xEA9C227723EE8BCB465E15A979C1CADC, // 5^-252
    0xECE53CEC4A314EBDA4F8BF5635246428, // 5^-224
    0xEF340A98172AACE486FB897116C87C34, // 5^-196
    0xF18899B1BC3F8CA1DC44E6C3CB279AC1, // 5^-168
    0xF3E2F893DEC3F1265A89DBA3C3EFCCFA, // 5^-140
    0xF64335BCF065D37D4D4617B5FF4A16D5, // 5^-112
    0xF8A95FCF88747D9475A44C6397CE912A, // 5^-84
    0xFB158592BE068D2EEED6E2F0F0D56712, // 5^-56
    0xFD87B5F28300CA0D8BCA9D6E188853FC, // 5^-28
    0x80000000000000000000000000000000, // 5^0
    0x813F3978F89409844000000000000000, // 5^28
    0x82818F1281ED449FBFF8F10E7A8921A4, // 5^56
    0x83C7088E1AAB65DB792667C6DA79E0FA, // 5^84
    0x850FADC09923329E03E2CF6BC604DDB0, // 5^112
    0x865B86925B9BC5C20B8A2392BA45A9B2, // 5^140
    0x87AA9AFF7904228690FB44D2F05D0842, // 5^168
    0x88FCF317F22241E2441FECE3BDF81F03, // 5^196
    0x8A5296FFE33CC92F82BD6B70D99AAA6F, // 5^224
    0x8BAB8EEFB6409C1A1AD089B6C2F7548E, // 5^252
    0x8D07E33455637EB2DB0B487B6423E1E8, // 5^280
    0x8E679C2F5E44FF8F570F09EAA7EA7648, // 5^308
    0x8FCAC257558EE4E6213A4F0AA5E8A7B1, // 5^336
    0x91315E37DB165AA92C0DE8DD3D020C0C, // 5^364
    0x929B7871DE7F22B91C306F5D1B0B5FDF, // 5^392
    0x940919BBD4620B6D250535BCC387778E, // 5^420
    0x957A4AE1EBF7F3D3A7EA9C8838CE9437, // 5^448
    0x96EF14C6454AA8404CF76E8DF8D89498, // 5^476
    0x9867806127ECE4F4BF1D49CACCCD5E68, // 5^504
    0x99E396C13A3ACFF1B0C5560A402AC0B2, // 5^532
    0x9B63610BB9243E46655494C5C95D77F2, // 5^560
    0x9CE6E87CB0821C85C3BFBAE0F3E130E2, // 5^588
    0x9E6E366733F8556102E008393FD60B55, // 5^616
    0x9FF95435986594C96632249F8A06C2C6, // 5^644
    0xA1884B69ADE2496455E04DBA4B3BD4DD, // 5^672
    0xA31B259CFA50498F7478A3CBBA44EC48, // 5^700
    0xA4B1EC80F47C84AD44B222741EB1EBBF, // 5^728
    0xA64CA9DF3FD42CF68F96BEE42FDA4243, // 5^756
    0xA7EB6799E8AEC9991CF4A5C3BC09FA6F, // 5^784
    0xA98E2FABA12EA4818AF70B7BE4ECB750, // 5^812
    0xAB350C27FEB90ACC3C4A575151B294DC, // 5^840
    0xACE0073BB807DA808480950470D805ED, // 5^868
    0xAE8F2B2CE3D5DBE9870A8D87239D8F35, // 5^896
    0xB042825B38276899BCC0502652E7E71D, // 5^924
    0xB1FA17404A30E5E8DD929F09C3EFF5AC, // 5^952
    0xB3B5F46FCEDC9C8816C0208E3CC9E873, // 5^980
    0xB5762497DBF17A9E1931B583A9431D7E, // 5^1008
    0xB73AB28129DC51BBBF0F83FB9A0D7ED7, // 5^1036
    0xB903A90F561D25E2E30DB03E0F8DD286, // 5^1064
    0xBAD11341265A26CB9F7165AE2B921943, // 5^1092
    0xBCA2FC30CC19F0909EB5CB19647508C5, // 5^1120
    0xBE796F142926B4F18C9281465B0C0F44, // 5^1148
    0xC054773D149BF26B24BD4C00042AD125, // 5^1176
    0xC2342019A0A0627EEE1F4EA0CEC13421, // 5^1204
    0xC418753460CDCCA97EA30DBD7EA479E3, // 5^1232
    0xC6018234B1486FB546C1734E983D9305, // 5^1260
    0xC7EF52DEFE87B751764F4CF916B4DECE, // 5^1288
    0xC9E1F3150DD1F818A7C8570E77A19E03, // 5^1316
    0xCBD96ED6466CF081BEB7FBDC1CBE8B37, // 5^1344
    0xCDD5D23FFB84D18EE373203B69F2EB6A, // 5^1372
    0xCFD7298DB6CB9672DCE472C619AA3F63, // 5^1400
    0xD1DD811983D276D453C35AD3235D128C, // 5^1428
    0xD3E8E55C3C1F43D0E47DEFC14A406E4F, // 5^1456
    0xD5F962EDD3FF846769FD88C48E1AC6B1, // 5^1484
    0xD80F0685A81B2A81B7157C60A24A0569, // 5^1512
    0xDA29DCFACBC8BE7222FC05BE6269F878, // 5^1540
    0xDC49F3445824E360FB0B98F6BBC4F0CB, // 5^1568
    0xDE6F5679BBEF1BD935E3A416F04CA9AA, // 5^1596
    0xE09A13D30C2DBA62C6C6C1764E047E15, // 5^1624
    0xE2CA38A9559AEEE3C905DE537F07EC9B, // 5^1652
    0xE4FFD276EEDCE65887E8DCFC09DBC33A, // 5^1680
    0xE73AEED7CB8AF75545A4713B13D24707, // 5^1708
    0xE97B9B89D001DAB3B1A3642A8DA3CF4F, // 5^1736
    0xEBC1E66D2608F4C95A1B25540EB6B8AA, // 5^1764
    0xEE0DDD84924AB88C2D4070F33B21AB7B, // 5^1792
    0xF05F8EF5CAA2331E727544D538F3F31E, // 5^1820
    0xF2B70909CD3FD35CA2BF0C63A814E04E, // 5^1848
    0xF5145A2D38A7863551528E351ACE7C2B, // 5^1876
    0xF77790F0A48A45CE08F13995CF9C2747, // 5^1904
    0xF9E0BC08FB7D3EBFC167073AC21593D6, // 5^1932
    0xFC4FEA4FD590B40A7A37993EB21444FA, // 5^1960
    0xFEC52AC3D3C8CFC1BD4C24B2C0457430, // 5^1988
    0x80A046447E3D49F1B7B1ADA9CDEBA84D, // 5^2016
    0x81E10F748C479223C2CE91A881EDD191, // 5^2044
    0x8324F8AA08D7D4110CC6866C5D69B2CB, // 5^2072
    0x846C09B028AE039504F609974DD3FFE9, // 5^2100
    0x85B64A659077660E7FE2B4308DCBF1A3, // 5^2128
    0x8703C2BC85483E0738D0EF9AB8A8F2C8, // 5^2156
    0x88547ABB1D8E5BD91D73EF3EAAC3C964, // 5^2184
    0x89A87A7B727DC0D25C7015CD0E51679A, // 5^2212
    0x8AFFCA2BD1F885491E34291B1EF566C7, // 5^2240
    0x8C5A720EF0F3350711C0B3BACD7601B3, // 5^2268
    0x8DB87A7C1E56D8739E9383D73D486881, // 5^2296
    0x8F19EBDF7661E3E9AC89BFA5E79484A6, // 5^2324
    0x907ECEBA168949B39CC5EE51962C011A, // 5^2352
    0x91E72BA251DAEE3D564F722FCAA40DD4, // 5^2380
    0x93530B43E5E2C129413407CFEEAC9743, // 5^2408
    0x94C276603013C119C69F0B71EF89019E, // 5^2436
    0x963575CE63B6332D7EFA7D29C44E11B7, // 5^2464
    0x97AC127BC05C5A60B450373470F0746B, // 5^2492
    0x9926556BC8DEFE435A848859645D1C6F, // 5^2520
    0x9AA447B87AE313B72C95A08E49A4C15B, // 5^2548
    0x9C25F29286E9DDB651EDEA897B34601F, // 5^2576
    0x9DAB5F4188ECDF77DD5DAEBB2F169C8B, // 5^2604
    0x9F3497244186FCA4B50008D92529E91F, // 5^2632
    0xA0C1A3B0CFAC27B513E15517552A7BC7, // 5^2660
    0xA2528E74EAF101FCF09E780BCC8238D9, // 5^2688
    0xA3E761161E63D4643C85A6192EBF4818, // 5^2716
    0xA580255203F84B473A5828869701A165, // 5^2744
    0xA71CE4FE808763833033D77325DAF287, // 5^2772
    0xA8BDAA0A0064FA448B231A70EB5444CE, // 5^2800
    0xAA627E7BB48C74C54251FF2792301CE5, // 5^2828
    0xAC0B6C73D065F8CCFA1BDE1F473556A4, // 5^2856
    0xADB87E2BC825B2702A73F1628AA4208E, // 5^2884
    0xAF69BDF68FC6A7407730E00421DA4D55, // 5^2912
    0xB11F3640DAA29ADE9254AA6FBBB55F5C, // 5^2940
    0xB2D8F1915BA88CA57F959CB702329D14, // 5^2968
    0xB496FA89063359F7FC797C10226CDA5B, // 5^2996
    0xB6595BE34F82149340C3A071220F5567, // 5^3024
    0xB820207670D3A02E57854716B3F18898, // 5^3052
    0xB9EB5333AA272E9B11C48D02B8326BD3, // 5^3080
    0xBBBAFF2785A33595209D5496B884CCFF, // 5^3108
    0xBD8F2F7A1BA47D6D566765461BD2F61B, // 5^3136
    0xBF67EF6F5776EBCA7D7ACEBF8AADFB4B, // 5^3164
    0xC1454A673CB9B1CEB889018E4F6E9A52, // 5^3192
    0xC3274BDE2D7089101556481F9C26F53D, // 5^3220
    0xC50DFF6D30C3AEFCF85333A94848659F, // 5^3248
    0xC6F970CA3A70527967CE61CCFD48C510, // 5^3276
    0xC8E9ABC872EB2BC11A1AEAE7CF8A9D3D, // 5^3304
    0xCADEBC588036FAE39D3D9605B201EB8A, // 5^3332
    0xCCD8AE88CF70AD8412E29F09D9061609, // 5^3360
    0xCED78E85DF12F0E4EB3149759843E989, // 5^3388
    0xD0DB689A89F2F9B1DF7601457CA20B35, // 5^3416
    0xD2E4493052F84F6F45BEEBB8A6B94A98, // 5^3444
    0xD4F23CCFB1916DF5CBDCD02F23CC7690, // 5^3472
    0xD70550205EE713ECD67AEFFBFCACC7B9, // 5^3500
    0xD91D8FE9A3D019CC44289DD21B589D7A, // 5^3528
    0xDB3B0912A787B1904881D9E963E4CE8F, // 5^3556
    0xDD5DC8A2BF27F3F795AA118EC1D08317, // 5^3584
    0xDF85DBC1BDEAA4DD36D5B4A1A707195F, // 5^3612
    0xE1B34FB846321D0472C4D2CAD73B0A7B, // 5^3640
    0xE3E631F01B5C4C7DE6331D95A376B8C8, // 5^3668
    0xE61E8FF47461CDA9E20A88F1134F906D, // 5^3696
    0xE85C77724F4305C5158950EF08DE22BE, // 5^3724
    0xEA9FF638C54554E1C7C91D5C341ED39D, // 5^3752
    0xECE91A3960025C317CB5735C85C60AD7, // 5^3780
    0xEF37F1886F4B6690F659EDE2159A45EC, // 5^3808
    0xF18C8A5D5FE3046333A802CDAED28CF3, // 5^3836
    0xF3E6F313130EF0EF78D946BAB954B82F, // 5^3864
    0xF6473A2837045CAAB325712DD8C98916, // 5^3892
    0xF8AD6E3FA030BD15C9B1474D8F89C269, // 5^3920
    0xFB199E20A3614828C8C37010926872B0, // 5^3948
    0xFD8BD8B770CB469E6B1D2745340E7B14, // 5^3976
    0x8002168AB7FBB6EE3C67B6BBB284E49E, // 5^4004
    0x81415538CE493BD5F22E502FCDD4BCA2, // 5^4032
    0x8283B014721299BBD00832554D9149C7, // 5^4060
    0x83C92EDF425B292D7C1735FC3B813C8C, // 5^4088
    0x8511D96E362C1A73FA9D4D41A7042940, // 5^4116
    0x865DB7A9CCD2839E0367500A8E9A178F, // 5^4144
    0x87ACD18E3E95BEDA8F1672EC7D776C85, // 5^4172
    0x88FF2F2BADE74531C9AC50475E25293A, // 5^4200
    0x8A54D8A6590D3496E9CC6E8725EC5D92, // 5^4228
    0x8BADD636CC48B3410879B2E5F6EE8B1C, // 5^4256
    0x8D0A302A147965340DDC924865236FC7, // 5^4284
    0x8E69EEE1F23F2BE52F33C652BD12FAB7, // 5^4312
    0x8FCD1AD50D9B6AF062FE50CE55EED182, // 5^4340
    0x9133BC8F2A130FE5AD6A6308A8E8B557, // 5^4368
    0x929DDCB15B529E4E4B07B86F1DB31283, // 5^4396
    0x940B83F23A55842A9DBAA465EFE141A0, // 5^4424
    0x957CBB1E1B11FE526B3C9C8F4DA2A4D8, // 5^4452
    0x96F18B1742AAD751888C9AB2FC5B3437, // 5^4480
    0x9869FCD61E284E938E33034A7A9E5D55, // 5^4508
    0x99E6196979B978F1BA00864671D1053F, // 5^4536
    0x9B65E9F6B87F6EFEC7FDDFD9302C767D, // 5^4564
    0x9CE977BA0CE3A0BD61D59D402AAE4FEA, // 5^4592
    0x9E70CC06B17AA9C6DE85ADFE03E691B5, // 5^4620
    0x9FFBF04722750449803C1CD864033781, // 5^4648
    0xA18AEDFD579EFCAF40BBC431F624B546, // 5^4676
    0xA31DCEC2FEF14B30A28A151725A55E10, // 5^4704
    0xA4B49C49B7B3BC11FBB16E441EEC585A, // 5^4732
    0xA64F605B4E3352CD5B8452AF2302FE13, // 5^4760
    0xA7EE24D9F80D57F79D2ACF5772F77020, // 5^4788
    0xA990F3C09110C54482B84CABC828BF93, // 5^4816
    0xAB37D722D8B786ABEE2722AD5F60D16E, // 5^4844
    0xACE2D92DB0390B598D29DD5122E4278D, // 5^4872
    0xAE9204275937A4C0A8C91282E5AF94EA, // 5^4900
    0xB045626FB50A35E758F8FDE02C03A6C6, // 5^4928
];

#[cfg(test)]
mod tests {
    use core::ops::RangeInclusive;
    use std::format;
    use std::string::String;

    use super::super::tests::{
        ALL_ROUND_MODES, create_prng, format_input, gen_arbitrary, gen_boundary, parse_string_for,
    };
    use super::super::{ParsedFloat, slow};
    use super::{MAX_DIGITS, try_convert};
    use crate::ieee_float::Semantics;

    /// Checks the fast path against the slow path.
    ///
    /// Returns whether the fast path was able to handle `s`. Zeros and values
    /// that are known to underflow or overflow do not reach the fast path, so
    /// they count as handled.
    fn check<S: Semantics>(s: &str) -> bool {
        let ParsedFloat::Finite(parsed) = parse_string_for::<S>(s) else {
            return true;
        };

        let mut num_handled = 0;
        for round in ALL_ROUND_MODES {
            let Some((actual, actual_status)) = try_convert::<S>(&parsed, round) else {
                continue;
            };
            num_handled += 1;

            let (expected, expected_status) = slow::convert::<S>(&parsed, round);
            let actual_bits = actual.to_bits();
            let expected_bits = expected.to_bits();
            assert_eq!(
                actual_bits, expected_bits,
                "input = {s:?}, round = {round:?}",
            );
            assert_eq!(
                actual_status, expected_status,
                "input = {s:?}, round = {round:?}",
            );
        }

        assert!(
            num_handled == 0 || num_handled == ALL_ROUND_MODES.len(),
            "input = {s:?}",
        );
        num_handled != 0
    }

    /// Returns the range of `e2` for which the decimal expansion of
    /// `m * 2^e2`, where `m` has `PREC_BITS + 1` bits, has at most
    /// `MAX_DIGITS` digits.
    ///
    /// The values generated by `gen_boundary` with this range can only be
    /// handled by the exact steps of the algorithm.
    fn boundary_e2_range<S: Semantics>() -> RangeInclusive<i32> {
        let m_bits = S::PREC_BITS + 1;

        // Keep `m * 2^e2` in `[2^(min_subnormal_exp - 1), 2^(max_normal_exp + 1))`.
        let min_exp: i32 = S::min_subnormal_exp().into();
        let max_exp: i32 = S::max_normal_exp().into();
        let min_e2 = min_exp - 1 - S::PREC_BITS as i32;
        let max_e2 = max_exp - S::PREC_BITS as i32;

        // Keep `m * 2^e2` (when `e2 >= 0`) or `m * 5^-e2` (when `e2 < 0`)
        // below `2^126 < 10^MAX_DIGITS`. Since `log2(5) < 7 / 3`,
        // `5^-e2 < 2^(126 - m_bits)` when `-e2 * 7 / 3 <= 126 - m_bits`.
        let lim = 126 - m_bits as i32;
        let min_e2 = min_e2.max(-(lim * 3 / 7));
        let max_e2 = max_e2.min(lim);

        min_e2..=max_e2
    }

    /// Checks the fast path against the slow path with random inputs.
    fn test_random<S: Semantics>(num_inputs: u32) {
        let mut rng = create_prng();

        let mut num_handled = 0;
        for _ in 0..num_inputs {
            // Some values have more than `MAX_DIGITS` significant digits.
            let (digits, exp) = gen_arbitrary::<S>(&mut rng, MAX_DIGITS + 10);
            let s = format_input(&mut rng, &digits, exp);
            if check::<S>(&s) {
                num_handled += 1;
            }
        }
        // Falling back should be very rare.
        assert!(
            num_handled >= num_inputs - num_inputs / 1000,
            "only {num_handled} of {num_inputs} arbitrary inputs handled",
        );

        for _ in 0..num_inputs {
            let (digits, exp) = gen_boundary::<S>(&mut rng, boundary_e2_range::<S>());
            let s = format_input(&mut rng, &digits, exp);
            // Exact values with at most `MAX_DIGITS` significant digits and
            // `|q| <= 55` are always handled by steps 2 or 4.
            assert!(check::<S>(&s), "boundary input {s:?} not handled");
        }
    }

    /// Checks inputs around the boundary between those that the fast path can
    /// handle and those that need the slow path.
    ///
    /// Each case is `(input, handled)`, where `handled` tells whether
    /// `try_convert` is expected to determine the result.
    fn check_boundary_cases<S: Semantics>(cases: &[(&str, bool)]) {
        for &(input, handled) in cases {
            for sign in [false, true] {
                let s = if sign {
                    format!("-{input}")
                } else {
                    String::from(input)
                };
                assert_eq!(check::<S>(&s), handled, "input = {s:?}");
            }
        }
    }

    #[test]
    fn test_f8e5m2_random() {
        test_random::<crate::f8e5m2::F8E5M2Semantics>(100_000);
    }

    #[test]
    fn test_f8e4m3b8nnz_random() {
        test_random::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>(100_000);
    }

    #[test]
    fn test_f8e4m3nao_random() {
        test_random::<crate::f8e4m3nao::F8E4M3NaoSemantics>(100_000);
    }

    #[test]
    fn test_f16_random() {
        test_random::<crate::f16::F16Semantics>(100_000);
    }

    #[test]
    fn test_f32_random() {
        test_random::<crate::f32::F32Semantics>(100_000);
    }

    #[test]
    fn test_f64_random() {
        test_random::<crate::f64::F64Semantics>(100_000);
    }

    #[test]
    fn test_f128_random() {
        test_random::<crate::f128::F128Semantics>(50_000);
    }

    #[test]
    fn test_x87f80_random() {
        test_random::<crate::x87f80::X87F80Semantics>(50_000);
    }

    #[test]
    fn test_f8e5m2_boundary() {
        check_boundary_cases::<crate::f8e5m2::F8E5M2Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            // `A < boundary < X` is not possible without truncation.
            ("6.8664550781249999999999999999999999999e-5", true), // below, slack 0
            ("4.3945312499999999999999999999999999999e-3", false), // below, slack -1
            ("8.7890625000000000000000000000000000001e-3", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("6.103515624999999999999999999999999999899999e-4", true), // below, slack 0
            ("7.629394531249999999999999999999999999899999e-5", false), // below, slack -1
            ("6.103515625000000000000000000000000000099999e-5", false), // `A < boundary < X`
            ("9.765625000000000000000000000000000000199999e-4", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("2.5390625000000000000000000000000000000e-2", true), // exact
            ("2.53906250000000000000000000000000000000e-2", true), // exact, 39 digits
            ("2.5390625000000000000000000000000000000000001e-2", false), // `A < boundary < X`
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("6.1440000000000000000000000000000000000e4", true), // exact
            ("6.1439999999999999999999999999999999999e4", true), // below, slack 1
            ("6.1440000000000000000000000000000000000000001e4", false), // `A < boundary < X`
            ("6.1440000000000000000000000000000000001e4", true), // above, `a_hi & low_mask == 3`
            // Around half the smallest subnormal.
            ("7.6293945312500000000000000000000000000e-6", true), // exact
            ("7.6293945312499999999999999999999999999e-6", false), // below, slack -1
            ("7.6293945312500000000000000000000000000000001e-6", false), // `A < boundary < X`
            ("7.6293945312500000000000000000000000001e-6", true), // above, `a_hi & low_mask == 1`
        ]);
    }

    #[test]
    fn test_f8e4m3b8nnz_boundary() {
        check_boundary_cases::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            // `A < boundary < X` is not possible without truncation.
            ("8.3007812499999999999999999999999999999e-3", true), // below, slack 0
            ("9.2773437499999999999999999999999999998e-3", false), // below, slack -1
            ("8.3007812500000000000000000000000000001e-3", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("9.765624999999999999999999999999999999699999e-3", true), // below, slack 0
            ("8.789062499999999999999999999999999999699999e-3", false), // below, slack -1
            ("7.812500000000000000000000000000000000099999e-3", false), // `A < boundary < X`
            ("8.789062500000000000000000000000000000199999e-3", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("1.3500000000000000000000000000000000000e1", true), // exact
            ("1.35000000000000000000000000000000000000e1", true), // exact, 39 digits
            ("1.3500000000000000000000000000000000000000001e1", false), // `A < boundary < X`
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("2.4800000000000000000000000000000000000e2", true), // exact
            ("2.4799999999999999999999999999999999999e2", true), // below, slack 8
            ("2.4800000000000000000000000000000000000000001e2", false), // `A < boundary < X`
            ("2.4800000000000000000000000000000000001e2", true), // above, `a_hi & low_mask == 11`
            // Around half the smallest subnormal.
            ("4.8828125000000000000000000000000000000e-4", true), // exact
            ("4.8828124999999999999999999999999999999e-4", true), // below, slack 2
            ("4.8828125000000000000000000000000000000000001e-4", false), // `A < boundary < X`
            ("4.8828125000000000000000000000000000001e-4", true), // above, `a_hi & low_mask == 2`
        ]);
    }

    #[test]
    fn test_f8e4m3nao_boundary() {
        check_boundary_cases::<crate::f8e4m3nao::F8E4M3NaoSemantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            // `A < boundary < X` is not possible without truncation.
            ("4.4921874999999999999999999999999999999e-2", true), // below, slack 0
            ("4.7499999999999999999999999999999999999e0", false), // below, slack -1
            ("8.5000000000000000000000000000000000001e0", true),  // above, `a_hi & low_mask == 1`
            // Truncated, around representable values.
            ("2.148437499999999999999999999999999999899999e-2", true), // below, slack 0
            ("4.499999999999999999999999999999999999799999e0", false), // below, slack -1
            ("1.562500000000000000000000000000000000099999e-2", false), // `A < boundary < X`
            ("8.000000000000000000000000000000000000199999e0", true), // above, `a_hi & low_mask == 1`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("8.2031250000000000000000000000000000000e-2", true), // exact
            ("8.20312500000000000000000000000000000000e-2", true), // exact, 39 digits
            ("8.2031250000000000000000000000000000000000001e-2", false), // `A < boundary < X`
            // Around the midpoint between the largest finite value and
            // the next value, whose encoding is used for NaN.
            ("4.6400000000000000000000000000000000000e2", true), // exact
            ("4.6399999999999999999999999999999999999e2", true), // below, slack 2
            ("4.6400000000000000000000000000000000000000001e2", false), // `A < boundary < X`
            ("4.6400000000000000000000000000000000001e2", true), // above, `a_hi & low_mask == 5`
            // Around half the smallest subnormal.
            ("9.7656250000000000000000000000000000000e-4", true), // exact
            ("9.7656249999999999999999999999999999999e-4", false), // below, slack -1
            ("9.7656250000000000000000000000000000000000001e-4", false), // `A < boundary < X`
            ("9.7656250000000000000000000000000000001e-4", true), // above, `a_hi & low_mask == 0`
        ]);
    }

    #[test]
    fn test_f16_boundary() {
        check_boundary_cases::<crate::f16::F16Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            // `A < boundary < X` is not possible without truncation.
            ("4.7134399414062499999999999999999999999e-2", true), // below, slack 0
            ("5.3059999999999999999999999999999999999e3", false), // below, slack -1
            ("9.7465515136718750000000000000000000001e-3", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("6.607055664062499999999999999999999999799999e-3", true), // below, slack 0
            ("7.306249999999999999999999999999999999899999e1", false), // below, slack -1
            ("1.735839843750000000000000000000000000099999e-1", false), // `A < boundary < X`
            ("9.391784667968750000000000000000000000199999e-3", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("1.5193750000000000000000000000000000000e2", true), // exact
            ("1.51937500000000000000000000000000000000e2", true), // exact, 39 digits
            ("1.5193750000000000000000000000000000000000001e2", false), // `A < boundary < X`
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("6.5520000000000000000000000000000000000e4", true), // exact
            ("6.5519999999999999999999999999999999999e4", true), // below, slack 1
            ("6.5520000000000000000000000000000000000000001e4", false), // `A < boundary < X`
            ("6.5520000000000000000000000000000000001e4", true), // above, `a_hi & low_mask == 2`
            // Around half the smallest subnormal.
            ("2.9802322387695312500000000000000000000e-8", true), // exact
            ("2.9802322387695312499999999999999999999e-8", true), // below, slack 7
            ("2.9802322387695312500000000000000000000000001e-8", false), // `A < boundary < X`
            ("2.9802322387695312500000000000000000001e-8", true), // above, `a_hi & low_mask == 4`
        ]);
    }

    #[test]
    fn test_f32_boundary() {
        check_boundary_cases::<crate::f32::F32Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            ("6.4122845644451658301034461769724723832e-27", true), // below, slack 0
            ("5.5047699715942144393920898437499999999e-3", false), // below, slack -1
            ("5.2788870348011336652849223582961712964e-14", false), // `A < boundary < X`
            ("3.3704784839841406274274504539789631963e-12", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("5.943685971897324158525640266968750941799999e-27", true), // below, slack 0
            ("5.264719871999999999999999999999999999899999e11", false), // below, slack -1
            ("1.331858640000000000000000000000000000099999e8", false),  // `A < boundary < X`
            ("5.137772873808899021241813898086547851699999e-9", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("9.0435885795159265398979187011718750000e-5", true), // exact
            ("9.04358857951592653989791870117187500000e-5", true), // exact, 39 digits
            ("9.0435885795159265398979187011718750000000001e-5", false), // `A < boundary < X`
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("3.4028235677973366163753939545814256844e38", true), // below, exact
            ("3.4028235677973366163753939545814256844800001e38", false), // `A < boundary < X`
            ("3.4028235677973366163753939545814256845e38", true), // above, exact
            // Around half the smallest subnormal.
            ("7.0064923216240853546186479164495806562e-46", true), // below, slack 4
            ("7.0064923216240853546186479164495806563e-46", false), // below, slack -1
            ("7.0064923216240853546186479164495806564013098e-46", false), // `A < boundary < X`
            ("7.0064923216240853546186479164495806565e-46", true), // above, `a_hi & low_mask == 2`
        ]);
    }

    #[test]
    fn test_f64_boundary() {
        check_boundary_cases::<crate::f64::F64Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            ("9.5360163765436844812535547986155602911e-29", true), // below, slack 0
            ("9.0742173951775103799000099513705208663e-3", false), // below, slack -1
            ("9.0742173951775103799000099513705208665e-3", false), // `A < boundary < X`
            ("9.5360163765436844812535547986155602913e-29", true), // above, `a_hi & low_mask == 0`
            // Not truncated, with `q > 55`, around representable values.
            ("9.5710351487234787053146198898079439302e232", true), // below, slack 0
            ("5.9537818888525143138772690553770426048e267", false), // below, slack -1
            ("5.9537818888525143138772690553770426049e267", false), // `A < boundary < X`
            ("9.5710351487234787053146198898079439304e232", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("8.048928720193022003937867242640276668399999e170", true), // below, slack 0
            ("1.915542691464724574760876511678258518199999e71", false), // below, slack -1
            ("8.048928720193022003937867242640276668499999e170", false), // `A < boundary < X`
            ("6.326179213335336108903633835857244182399999e92", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("6.2012382213881383650004863739013671875e6", true), // exact
            ("6.20123822138813836500048637390136718750e6", true), // exact, 39 digits
            ("6.2012382213881383650004863739013671875000001e6", false), // `A < boundary < X`
            // Not truncated, close enough to a boundary that `round_interval` fails.
            // With `q = 55`, the value is calculated exactly.
            ("4.6087635413807508825121603501708982614e92", true), // `q = 55`, exact
            ("4.0759305305090500596355345631514601621e93", false), // `q = 56`, slack -3
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("1.7976931348623158079372897140530341507e308", true), // below, slack 13
            ("1.7976931348623158079372897140530341508e308", false), // `A < boundary < X`
            ("1.7976931348623158079372897140530341507993414e308", false), // `A < boundary < X`
            ("1.7976931348623158079372897140530341509e308", true), // above, `a_hi & low_mask == 17`
            // Around half the smallest subnormal.
            ("2.4703282292062327208828439643411068617e-324", true), // below, slack 12
            ("2.4703282292062327208828439643411068618e-324", false), // below, slack -2
            ("2.4703282292062327208828439643411068618252991e-324", false), // `A < boundary < X`
            ("2.4703282292062327208828439643411068619e-324", true), // above, `a_hi & low_mask == 4`
        ]);
    }

    #[test]
    fn test_f128_boundary() {
        check_boundary_cases::<crate::f128::F128Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            ("5.2000473241184237850696689318265720884e15", true), // below, slack 0
            ("6.1979502644200898914750756515102858288e-29", false), // below, slack -1
            ("3.6584767335133677263693491168199918747e28", false), // `A < boundary < X`
            ("5.2000473241184237850696689318265720885e15", true), // above, `a_hi & low_mask == 0`
            // Not truncated, with `q > 55`, around representable values.
            ("1.0282676188537473151720214971328953379e1010", true), // below, slack 0
            ("9.0943263575400549683417330683557664538e455", false), // below, slack -1
            ("4.1838893941702078317304685466038555140e1348", false), // `A < boundary < X`
            ("9.0943263575400549683417330683557664540e455", true),  // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("2.886007449944421170678653149929987192199999e1814", true), // below, slack 0
            ("7.744592036280809354325971861510140513999999e-2182", false), // below, slack -1
            ("2.037806143990840739498189264758727750999999e-3225", false), // `A < boundary < X`
            ("2.037806143990840739498189264758727751099999e-3225", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("3.7563070530003676828166186292263627500e33", true), // exact
            ("3.75630705300036768281661862922636275000e33", true), // exact, 39 digits
            ("3.7563070530003676828166186292263627500000001e33", false), // `A < boundary < X`
            // Not truncated, close enough to a boundary that `round_interval` fails.
            // With `q = 55`, the value is calculated exactly.
            ("2.7848414003313788953986591288089834522e92", true), // `q = 55`, exact
            ("7.7568794625254626064082465468081838955e93", false), // `q = 56`, slack -2
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("1.1897314953572317650857593266280070734e4932", true), // below, slack 17
            ("1.1897314953572317650857593266280070734799569e4932", false), // `A < boundary < X`
            ("1.1897314953572317650857593266280070735e4932", true), // above, `a_hi & low_mask == 4`
            // Around half the smallest subnormal.
            ("3.2375875597190125554622194791138232762e-4966", true), // below, slack 0
            ("3.2375875597190125554622194791138232762497847e-4966", false), // `A < boundary < X`
            ("3.2375875597190125554622194791138232763e-4966", true), // above, `a_hi & low_mask == 2`
        ]);
    }

    #[test]
    fn test_x87f80_boundary() {
        check_boundary_cases::<crate::x87f80::X87F80Semantics>(&[
            // Not truncated, with `q < 0`, around midpoints.
            ("6.6004461961062020303124999999999999999e17", true), // below, slack 0
            ("3.1673449653035951843790112915379541199e-11", false), // below, slack -1
            ("1.0828359583869991800338496789069226841e-15", false), // `A < boundary < X`
            ("6.1801495182977789461934429692522609268e-5", true), // above, `a_hi & low_mask == 0`
            // Not truncated, with `q > 55`, around representable values.
            ("4.3069294257632654240351636039665587650e780", true), // below, slack 0
            ("8.4382894018908618193753606410083516836e243", false), // below, slack -1
            ("8.4382894018908618193753606410083516837e243", false), // `A < boundary < X`
            ("1.0470315446210360190769442165895898018e877", true), // above, `a_hi & low_mask == 0`
            // Truncated, around representable values.
            ("3.828514195682578285635389419790292495499999e2045", true), // below, slack 0
            ("6.086654209886217554670065309165829535899999e-1508", false), // below, slack -1
            ("3.505084985371937562107803480146007100699999e2503", false), // `A < boundary < X`
            ("3.895600404025249745543247491961682194499999e-1568", true), // above, `a_hi & low_mask == 0`
            // A midpoint with 38 significant digits, which is handled exactly, followed
            // by a zero, which is dropped, or by a non-zero digit.
            ("1.7492326343094659414672851562500000000e15", true), // exact
            ("1.74923263430946594146728515625000000000e15", true), // exact, 39 digits
            ("1.7492326343094659414672851562500000000000001e15", false), // `A < boundary < X`
            // Not truncated, close enough to a boundary that `round_interval` fails.
            // With `q = 55`, the value is calculated exactly.
            ("2.4965684665753362604960559986961057431e92", true), // `q = 55`, exact
            ("2.2636878099687176213908701554212400759e93", false), // `q = 56`, slack -5
            // Around the midpoint between the largest finite value and
            // `2^(max_normal_exp + 1)`.
            ("1.1897314953572317650535115898294886679e4932", true), // below, slack 13
            ("1.1897314953572317650535115898294886679662541e4932", false), // `A < boundary < X`
            ("1.1897314953572317650535115898294886680e4932", true), // above, `a_hi & low_mask == 8`
            // Around half the smallest subnormal.
            ("1.8225997659412373012642029668097099081e-4951", true), // below, slack 14
            ("1.8225997659412373012642029668097099082e-4951", false), // `A < boundary < X`
            ("1.8225997659412373012642029668097099081995255e-4951", false), // `A < boundary < X`
            ("1.8225997659412373012642029668097099083e-4951", true), // above, `a_hi & low_mask == 8`
        ]);
    }
}
