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
use crate::ieee_float::pow5::{
    MAX_POW5_EXACT, POW5_APPROX_ERROR, POW5_EXACT, div_exact_pow5, pow5_approx, shr_interval,
};
use crate::ieee_float::{IeeeFloat, Semantics};
use crate::traits::{CastFrom as _, UInt as _};
use crate::utils::{RoundLoss, wide_shift_left};
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
            IeeeFloat::make_overflow_result(sign, round)
        } else {
            IeeeFloat::make_underflow_result(sign, round)
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
        // `t * 2^s <= 5^q < (t + err) * 2^s`, where `err` is
        // `POW5_APPROX_ERROR`.
        let (t, s) = pow5_approx(q)?;

        // The value is `X * 2^e2`, where `X` is in the open interval
        // `(w_n * t, w_n * t + delta)`.
        //
        // The lower bound is strict because either some non-zero digits have
        // been dropped or `5^q * 2^-s` is not an integer (`q < 0` or `q > 55`).
        let e2 = q + s - lz as i32;
        let (a_lo, a_hi) = w_n.wide_mul(t);
        let delta_hi = if dec.truncated {
            // `X < (w_n + 2^lz) * (t + err)`, so
            // `delta = 2^lz * t + err * (w_n + 2^lz) < (2^lz + err) * 2^128`,
            // given that `t < 2^128` and `w_n + 2^lz = (w + 1) * 2^lz <= 2^128`.
            //
            // `w` has `MAX_DIGITS` digits, so `lz` is small.
            debug_assert!(lz <= 5);
            (1 << lz) + POW5_APPROX_ERROR
        } else {
            // `X < w_n * (t + err)`, so `delta = err * w_n < err * 2^128`.
            POW5_APPROX_ERROR
        };

        if let Some(r) = round_interval(a_lo, a_hi, delta_hi, e2, prec_bits) {
            Some(r)
        } else if !dec.truncated
            && q < 0
            && let Some(w) = div_exact_pow5(w, q.unsigned_abs())
        {
            // The value is exactly `(w / 5^-q) * 2^q`.
            Some(round_exact(w, 0, q, prec_bits))
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

    // The significand is made of the `prec_bits` most significant bits of
    // `A`, so the rounding boundaries (representable values and midpoints
    // between them) are the integer and half-integer values of
    // `X / 2^(256 - prec_bits)`.
    let (mant, loss) = shr_interval(a_hi, delta_hi, 128 - prec_bits)?;
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
