//! Slow path of float parsing, based on exact big integer arithmetic.
//!
//! It is only used when the fast path cannot determine the result, which
//! happens when the exact value is very close to a rounding boundary (a
//! representable value or the midpoint between two consecutive representable
//! values), or to a value that would be one if the exponent range were
//! unbounded.
//!
//! 1. The decimal string is reduced to `w * 10^q`, where `w` holds up to
//!    `max_digits::<S>()` significant digits. When there are more significant
//!    digits and some of the dropped ones are not zero, the exact value is in
//!    the open interval `(w * 10^q, (w + 1) * 10^q)`. Any number in that
//!    interval has more significant digits than any rounding boundary, so no
//!    boundary lies in it, and the exact value rounds like `w * 10^q` plus an
//!    infinitesimal amount.
//! 2. `w * 10^q` is written as `(num / den) * 2^e2`, where `num` and `den` are
//!    big integers: `num = w * 5^q` and `den = 1` when `q >= 0`, or `num = w`
//!    and `den = 5^-q` when `q < 0`. In both cases, `e2 = q`.
//! 3. `num` or `den` is shifted left, adjusting `e2` accordingly, so that
//!    `den / 2 <= num < den`.
//! 4. The significand, `floor(num * 2^PREC_BITS / den)`, is calculated with a
//!    long division that produces up to 64 bits per step (Algorithm D from
//!    "The Art of Computer Programming", Vol. 2, Section 4.3.1, by Donald E.
//!    Knuth). The remainder of the division gives the exact rounding loss.

use super::ParsedFinite;
use crate::ieee_float::biguint::BigUInt;
use crate::ieee_float::{IeeeFloat, Semantics};
use crate::traits::CastFrom as _;
use crate::utils::RoundLoss;
use crate::{FpStatus, Round};

/// Converts `parsed` into a floating-point number, rounding according to
/// `round`.
pub(super) fn convert<S: Semantics>(
    parsed: &ParsedFinite<'_>,
    round: Round,
) -> (IeeeFloat<S>, FpStatus) {
    // The significand is accumulated in a `u128`.
    const {
        assert!(S::PREC_BITS <= 128);
    }

    let sign = parsed.sign;

    let digits0 = parsed.digits0;
    let digits1 = parsed.digits1;
    let num_digits = digits0.len() + digits1.len();
    debug_assert_ne!(num_digits, 0);

    let num_kept = num_digits.min(max_digits::<S>());
    let num_dropped = num_digits - num_kept;

    // The value is `(w * 10^num_dropped + dropped) * 10^parsed.exp`.
    let q = parsed.exp.saturating_add(num_dropped as i64);

    // `w` is made of the first `num_kept` significant digits.
    let truncated = digits0
        .iter()
        .chain(digits1)
        .skip(num_kept)
        .any(|&c| c != b'0');

    // Using `BigUInt` sized for the worst case has a noticeable cost.
    let (mant, e2, loss) = match required_limbs(num_kept, q) {
        0..=32 => calc_significand::<32>(digits0, digits1, num_kept, q, S::PREC_BITS),
        33..=128 => calc_significand::<128>(digits0, digits1, num_kept, q, S::PREC_BITS),
        _ => calc_significand::<MAX_LIMBS>(digits0, digits1, num_kept, q, S::PREC_BITS),
    };
    let loss = match loss {
        RoundLoss::Zero if truncated => RoundLoss::HalfDown,
        RoundLoss::Halfway if truncated => RoundLoss::HalfUp,
        _ => loss,
    };

    // The most significant bit of `mant` has a weight of `2^(e2 - 1)`.
    let exp = e2 - 1;
    let Ok(exp) = S::Exp::try_from(exp) else {
        if exp > 0 {
            return (
                IeeeFloat::make_overflow_value(sign, round),
                FpStatus::Overflow,
            );
        } else {
            return (
                IeeeFloat::make_underflow_value(sign, round),
                FpStatus::Underflow,
            );
        }
    };
    IeeeFloat::round_and_classify(sign, exp, S::Mant::cast_from(mant), loss, round)
}

/// Calculates `(mant, e2, loss)` such that
/// `w * 10^q = (mant + frac) * 2^(e2 - prec_bits)`, where `mant` has exactly
/// `prec_bits` bits, `0 <= frac < 1` and `loss` describes `frac`.
///
/// `w` is made of the first `num_digits` digits of `digits0` followed by
/// `digits1`, the first of which is not zero. The big integers have `N`
/// limbs, which must be at least `required_limbs(num_digits, q)`.
// Never inlined, so the stack frame of `convert` does not include the large
// `BigUInt`s of every size, only the frame of the called instance does.
#[inline(never)]
fn calc_significand<const N: usize>(
    digits0: &[u8],
    digits1: &[u8],
    num_digits: usize,
    q: i64,
    prec_bits: u32,
) -> (u128, i64, RoundLoss) {
    debug_assert!(N >= required_limbs(num_digits, q));

    let digits = digits0.iter().chain(digits1).take(num_digits);
    let mut num = BigUInt::<N>::from_digits(digits.map(|&c| c - b'0'));
    let mut den = BigUInt::<N>::from_u64(1);

    // The value is `(num / den) * 2^e2`.
    if q >= 0 {
        num.mul_pow5(q.unsigned_abs());
    } else {
        den.mul_pow5(q.unsigned_abs());
    }
    let mut e2 = q;

    // Make `den / 2 <= num < den`.
    let num_bits = num.bit_len();
    let den_bits = den.bit_len();
    if num_bits < den_bits {
        num.shl(den_bits - num_bits);
        e2 -= (den_bits - num_bits) as i64;
    } else {
        den.shl(num_bits - den_bits);
        e2 += (num_bits - den_bits) as i64;
    }
    // Both have the same bit length, so `den / 2 < num < den * 2`.
    if num >= den {
        den.shl(1);
        e2 += 1;
    }

    // Normalize `den` for `shl_div_rem`, which does not change `num / den`.
    let norm_shift = (64 - den.bit_len() % 64) % 64;
    num.shl(norm_shift);
    den.shl(norm_shift);

    // Calculate `mant = floor(num * 2^prec_bits / den)`, which has exactly
    // `prec_bits` bits, up to 64 bits at a time. After each step, `num` holds
    // the remainder of the division calculated so far, which is less than
    // `den`.
    let mut mant = 0u128;
    let mut rem_bits = prec_bits;
    while rem_bits != 0 {
        let bits = rem_bits.min(64);
        mant = (mant << bits) | u128::from(num.shl_div_rem(bits, &den));
        rem_bits -= bits;
    }

    // `w * 10^q = (mant + num / den) * 2^(e2 - prec_bits)`
    let loss = if num.is_zero() {
        RoundLoss::Zero
    } else {
        num.shl(1);
        match num.cmp(&den) {
            core::cmp::Ordering::Less => RoundLoss::HalfDown,
            core::cmp::Ordering::Equal => RoundLoss::Halfway,
            core::cmp::Ordering::Greater => RoundLoss::HalfUp,
        }
    };
    (mant, e2, loss)
}

/// Returns the number of limbs that `calc_significand` needs for each big
/// integer.
fn required_limbs(num_digits: usize, q: i64) -> usize {
    // When `q >= 0`, `num = w * 5^q` and `den = 1`. Otherwise, `num = w` and
    // `den = 5^-q`. A positive integer `x` has `floor(log2(x)) + 1` bits,
    // `w < 10^num_digits`, `log2(10) < 3.32193` and `log2(5) < 2.32193`.
    let w_log2 = num_digits as u64 * 332193;
    let (num_log2, den_log2) = if q >= 0 {
        (w_log2 + q.unsigned_abs() * 232193, 0)
    } else {
        (w_log2, q.unsigned_abs() * 232193)
    };
    let max_bits = num_log2.max(den_log2) / 100000 + 1;
    // Making `den / 2 <= num < den` might add one bit to `den`, which is then
    // normalized to a whole number of limbs. The remainder shifted in
    // `shl_div_rem` (as well as twice the final remainder) needs one more
    // limb.
    ((max_bits + 1).div_ceil(64) + 1) as usize
}

/// Returns the maximum number of significant digits of a rounding boundary
/// (a representable value or the midpoint between two consecutive
/// representable values) of `S`, including `2^(max_normal_exp + 1)`, which
/// separates finite results from overflows.
fn max_digits<S: Semantics>() -> usize {
    let prec = i64::from(S::PREC_BITS);
    let min_exp = i64::from(S::min_normal_exp().into());
    let max_exp = i64::from(S::max_normal_exp().into());

    // Rounding boundaries are `m * 2^(e - prec)`, where `m < 2^(prec + 1)` and
    // `min_exp <= e <= max_exp` (subnormal ones are included with
    // `e = min_exp`).
    //
    // When `e < prec`, the boundary is `(m * 5^(prec - e)) * 10^(e - prec)`,
    // so it does not have more significant digits than
    // `m * 5^(prec - e) < 2^(prec + 1) * 5^(prec - min_exp)`.
    //
    // When `e >= prec`, the boundary is an integer not greater than
    // `2^(max_exp + 1)`.
    //
    // A positive integer less than or equal to `x` has at most
    // `floor(log10(x)) + 1` digits, `log10(2) < 0.30103` and
    // `log10(5) < 0.69898`.
    let frac_digits = ((prec + 1) * 30103 + (prec - min_exp) * 69898) / 100000 + 1;
    let int_digits = (max_exp + 1) * 30103 / 100000 + 1;
    frac_digits.max(int_digits) as usize
}

/// Largest number of limbs needed by `calc_significand` for any format.
///
/// This is enough for binary128, which needs the largest integers, see
/// `tests::test_max_limbs`.
const MAX_LIMBS: usize = 602;

#[cfg(test)]
mod tests {
    use core::ops::RangeInclusive;
    use std::format;
    use std::string::{String, ToString as _};

    use rand::RngExt as _;

    use super::super::tests::{
        ALL_ROUND_MODES, create_prng, format_input, gen_arbitrary, gen_boundary, parse_string_for,
        reference,
    };
    use super::super::{
        ParsedFloat, max_underflow_sci_exp, min_overflow_sci_exp, parse, take_sign,
    };
    use super::{MAX_LIMBS, calc_significand, convert, max_digits, required_limbs};
    use crate::ieee_float::Semantics;

    /// Checks the slow path against MPFR for `s`, whose value is
    /// `±digits * 10^exp`.
    ///
    /// Zeros and values that are known to underflow or overflow do not reach
    /// the slow path, so the whole parser is checked for them instead.
    fn check<S: Semantics>(s: &str, digits: &str, exp: i64) {
        let (sign, _) = take_sign(s.as_bytes());
        let parsed = parse_string_for::<S>(s);
        let value = format!("{digits}e{exp}");

        for round in ALL_ROUND_MODES {
            let (actual, actual_status) = match &parsed {
                ParsedFloat::Finite(parsed) => convert::<S>(parsed, round),
                _ => parse::<S>(s.as_bytes(), round).unwrap(),
            };
            let (expected, expected_status) = reference::<S>(sign, &value, round);
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "input = {s:?}, round = {round:?}",
            );
            assert_eq!(
                actual_status, expected_status,
                "input = {s:?}, round = {round:?}",
            );
        }
    }

    /// Returns the range of `e2` that keeps `m * 2^e2`, where `m` has
    /// `PREC_BITS + 1` bits, around
    /// `[2^(min_subnormal_exp - 1), 2^(max_normal_exp + 1))`.
    ///
    /// Unlike in the tests of the fast path, the number of digits of the
    /// values generated by `gen_boundary` is not limited.
    fn boundary_e2_range<S: Semantics>() -> RangeInclusive<i32> {
        let min_exp: i32 = S::min_subnormal_exp().into();
        let max_exp: i32 = S::max_normal_exp().into();
        let min_e2 = min_exp - 2 - S::PREC_BITS as i32;
        let max_e2 = max_exp + 1 - S::PREC_BITS as i32;

        min_e2..=max_e2
    }

    /// Moves `digits * 10^exp` slightly up or down by a random amount, which
    /// may be beyond the number of digits kept by the slow path.
    fn perturb<S: Semantics>(rng: &mut impl rand::RngExt, digits: &str, exp: i64) -> (String, i64) {
        let extra = rng.random_range(1..=(max_digits::<S>() + 10));
        let w: rug::Integer = digits.parse().unwrap();
        let w = w * rug::Integer::from(rug::Integer::u_pow_u(10, extra as u32));
        let w = if rng.random() { w + 1u32 } else { w - 1u32 };
        (w.to_string(), exp - extra as i64)
    }

    fn test_random<S: Semantics>(num_inputs: u32) {
        let mut rng = create_prng();

        for _ in 0..num_inputs {
            let max_num_digits = if rng.random_ratio(1, 10) {
                max_digits::<S>() + 10
            } else {
                40
            };
            let (digits, exp) = gen_arbitrary::<S>(&mut rng, max_num_digits);
            let s = format_input(&mut rng, &digits, exp);
            check::<S>(&s, &digits, exp);
        }

        for _ in 0..num_inputs {
            let (digits, exp) = gen_boundary::<S>(&mut rng, boundary_e2_range::<S>());
            let s = format_input(&mut rng, &digits, exp);
            check::<S>(&s, &digits, exp);

            let (digits, exp) = perturb::<S>(&mut rng, &digits, exp);
            let s = format_input(&mut rng, &digits, exp);
            check::<S>(&s, &digits, exp);
        }
    }

    /// Checks values with the largest number of significant digits at both
    /// ends of the range, which need the largest big integers.
    fn test_extremes<S: Semantics>() {
        let mut rng = create_prng();

        let max_digits = max_digits::<S>();
        for num_digits in [max_digits - 1, max_digits, max_digits + 1, max_digits + 50] {
            for sci_exp in [
                max_underflow_sci_exp::<S>(),
                max_underflow_sci_exp::<S>() + 1,
                min_overflow_sci_exp::<S>() - 1,
                min_overflow_sci_exp::<S>(),
            ] {
                for fill in ['0', '9'] {
                    let mut digits = String::with_capacity(num_digits);
                    digits.push('1');
                    for _ in 2..num_digits {
                        digits.push(fill);
                    }
                    digits.push('1');

                    let exp = sci_exp - (num_digits as i64 - 1);
                    let s = format_input(&mut rng, &digits, exp);
                    check::<S>(&s, &digits, exp);
                }
            }
        }
    }

    fn test_type<S: Semantics>(num_inputs: u32) {
        test_random::<S>(num_inputs);
        test_extremes::<S>();
    }

    #[test]
    fn test_f8e5m2() {
        test_type::<crate::f8e5m2::F8E5M2Semantics>(2_000);
    }

    #[test]
    fn test_f8e4m3b8nnz() {
        test_type::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>(2_000);
    }

    #[test]
    fn test_f8e4m3nao() {
        test_type::<crate::f8e4m3nao::F8E4M3NaoSemantics>(2_000);
    }

    #[test]
    fn test_f16() {
        test_type::<crate::f16::F16Semantics>(2_000);
    }

    #[test]
    fn test_f32() {
        test_type::<crate::f32::F32Semantics>(2_000);
    }

    #[test]
    fn test_f64() {
        test_type::<crate::f64::F64Semantics>(1_000);
    }

    #[test]
    fn test_f128() {
        test_type::<crate::f128::F128Semantics>(100);
    }

    #[test]
    fn test_x87f80() {
        test_type::<crate::x87f80::X87F80Semantics>(100);
    }

    /// Checks that `MAX_LIMBS` is enough for every format.
    #[test]
    fn test_max_limbs() {
        fn check<S: Semantics>() {
            // Values that reach `calc_significand` have
            // `max_underflow_sci_exp < q + num_digits - 1 < min_overflow_sci_exp`,
            // and `required_limbs` is largest at the extreme values of `q`.
            for num_digits in 1..=max_digits::<S>() {
                let min_q = max_underflow_sci_exp::<S>() + 2 - num_digits as i64;
                let max_q = min_overflow_sci_exp::<S>() - num_digits as i64;
                assert!(required_limbs(num_digits, min_q) <= MAX_LIMBS);
                assert!(required_limbs(num_digits, max_q) <= MAX_LIMBS);
            }
        }

        check::<crate::f8e5m2::F8E5M2Semantics>();
        check::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>();
        check::<crate::f8e4m3nao::F8E4M3NaoSemantics>();
        check::<crate::f16::F16Semantics>();
        check::<crate::f32::F32Semantics>();
        check::<crate::f64::F64Semantics>();
        check::<crate::f128::F128Semantics>();
        check::<crate::x87f80::X87F80Semantics>();
    }

    /// Checks that `required_limbs` is enough by running `calc_significand`
    /// with exactly that number of limbs (which panics if it is too small).
    #[test]
    fn test_required_limbs() {
        fn check<const N: usize>(digits: &str, q: i64) {
            assert_eq!(required_limbs(digits.len(), q), N);
            calc_significand::<N>(digits.as_bytes(), &[], digits.len(), q, 113);
        }

        // `w` is the largest integer with the given number of digits.
        let nines = "9".repeat(3000);

        // Every `q` for small numbers of digits.
        macro_rules! check_any {
            ($digits:expr, $q:expr; $($n:literal)*) => {
                match required_limbs($digits.len(), $q) {
                    $($n => check::<$n>($digits, $q),)*
                    n => panic!("unexpected number of limbs: {n}"),
                }
            };
        }
        for num_digits in 1..=64 {
            for q in -1000..=1000 {
                check_any!(
                    &nines[..num_digits], q;
                    2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21
                    22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41
                );
            }
        }

        // The extreme values of `q` that need exactly `N` limbs, for larger
        // numbers of limbs.
        fn check_extremes<const N: usize>(nines: &str) {
            for num_digits in [1, 10, 100, 1000, 3000] {
                if required_limbs(num_digits, 0) > N {
                    continue;
                }
                for step in [1, -1] {
                    let mut q = 0;
                    while required_limbs(num_digits, q + step) <= N {
                        q += step;
                    }
                    check::<N>(&nines[..num_digits], q);
                }
            }
        }
        check_extremes::<64>(&nines);
        check_extremes::<128>(&nines);
        check_extremes::<256>(&nines);
        check_extremes::<MAX_LIMBS>(&nines);
    }
}
