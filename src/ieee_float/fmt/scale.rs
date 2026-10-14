//! Calculation of `floor(x)` and the rounding loss of `x = n * 2^e / 10^j`,
//! which is the basic operation of float formatting.
//!
//! `Pow10::try_scale` is a fast path that, like the fast path of float
//! parsing, uses 256-bit arithmetic and keeps track of an interval that is
//! known to contain the exact value:
//!
//! 1. When `0 <= -j <= 55`, `x` is `(n * 5^-j) * 2^(e - j)`, where `n * 5^-j`
//!    fits in 256 bits, so the result is calculated exactly.
//! 2. Otherwise, `5^-j` is approximated as `t * 2^s`, where `t` has 128 bits,
//!    which gives an open interval that contains `x`. When a conservative
//!    check proves that no integer or half-integer lies inside that interval,
//!    the result is known.
//! 3. Otherwise, when `j > 0`, `x` might be an integer or a half-integer
//!    itself, which requires `5^j` to divide `n`. In that case, `x` is
//!    `(n / 5^j) * 2^(e - j)`, so the result is calculated exactly.
//! 4. Otherwise, the result cannot be determined by the fast path.
//!
//! `Pow10::try_scale_around` does the same for three values that are close to
//! each other, which only needs one multiplication in step 2.
//!
//! `Pow10U64::try_scale` is a cheaper fast path for the formats up to
//! binary64, whose values and results fit in 64 bits, so 192-bit arithmetic
//! is enough. Besides, the integers of step 3 are recognized without a
//! division, because the interval is too narrow to contain any other
//! multiple of `1 / 5^j`.
//!
//! `scale_slow` is a slow path that always works, based on exact big integer
//! arithmetic.

use crate::ieee_float::biguint::BigUInt;
use crate::ieee_float::pow5::{
    MAX_POW5_EXACT, POW5_APPROX_ERROR, POW5_EXACT, div_exact_pow5, pow5_approx, shr_interval,
};
use crate::traits::UInt as _;
use crate::utils::{RoundLoss, wide_shift_left, wide_shift_right};

/// Power of ten `10^j` by which values are divided in the fast path.
pub(super) struct Pow10 {
    j: i32,
    pow5: Pow5,
}

enum Pow5 {
    /// `5^-j`, when `0 <= -j <= MAX_POW5_EXACT`.
    Exact(u128),
    /// The result of `pow5_approx(-j)`, otherwise.
    Approx(u128, i32),
}

impl Pow10 {
    /// Returns `None` if `10^j` is out of the range supported by the fast
    /// path.
    #[inline]
    pub(super) fn new(j: i32) -> Option<Self> {
        // `1 / 10^j = 5^q * 2^q`
        let q = j.checked_neg()?;
        let pow5 = if matches!(q, 0..=MAX_POW5_EXACT) {
            Pow5::Exact(POW5_EXACT[q as usize])
        } else {
            let (t, s) = pow5_approx(q)?;
            Pow5::Approx(t, s)
        };
        Some(Self { j, pow5 })
    }

    /// Attempts to calculate `floor(x)` and the rounding loss of `x`, where
    /// `x = n * 2^e / 10^j` and `n` is not zero.
    ///
    /// Returns `None` when the result cannot be determined by the fast path
    /// or when `floor(x)` does not fit in 128 bits.
    pub(super) fn try_scale(&self, n: u128, e: i32) -> Option<(u128, RoundLoss)> {
        debug_assert_ne!(n, 0);

        // `x = (n * 5^-j) * 2^shift`
        let shift = i64::from(e) - i64::from(self.j);
        match self.pow5 {
            Pow5::Exact(pow5) => {
                let (lo, hi) = n.wide_mul(pow5);
                shift_exact(lo, hi, shift)
            }
            Pow5::Approx(t, s) => {
                // `x = X / 2^(128 + frac_bits)`, where `X` is in the open
                // interval `(A, A + delta)`, `A = (n * 2^lz) * t` is in
                // `[2^254, 2^256)` and `delta = POW5_APPROX_ERROR * 2^lz * n`
                // is less than `POW5_APPROX_ERROR * 2^128`.
                //
                // The lower bound is strict because `5^-j` is not exact.
                let lz = n.leading_zeros();
                let (_, a_hi) = (n << lz).wide_mul(t);
                let frac_bits = i64::from(lz) - shift - i64::from(s) - 128;
                self.round_approx(n, shift, a_hi, frac_bits)
            }
        }
    }

    /// Does the same as `try_scale` for `n - dist_low`, `n` and
    /// `n + dist_high` (in that order) with a single multiplication, given
    /// that their products by a power of five differ by multiples of it.
    ///
    /// `dist_low` and `dist_high` must be powers of two, and `dist_low` must
    /// be less than `n`.
    ///
    /// The three values are calculated with the precision that suits
    /// `n + dist_high`, so the results of the ones that have fewer bits are
    /// less likely to be determined than with `try_scale`.
    #[inline]
    pub(super) fn try_scale_around(
        &self,
        n: u128,
        dist_low: u128,
        dist_high: u128,
        e: i32,
    ) -> [Option<(u128, RoundLoss)>; 3] {
        debug_assert!(dist_low.is_power_of_two() && dist_low < n);
        debug_assert!(dist_high.is_power_of_two());
        let n_low = n - dist_low;
        let n_high = n + dist_high;

        match self.pow5 {
            // The product is not shared in this case, which is rare: most of
            // the values that have an exact power of five are handled by
            // `shortest::Scaled::try_new_exact` instead.
            Pow5::Exact(_) => [n_low, n, n_high].map(|n| self.try_scale(n, e)),
            Pow5::Approx(t, s) => {
                // Like in `try_scale`, but `lz` is the same for the three
                // values, so their `A` can be calculated together. Each of
                // them still has `A = (n * 2^lz) * t < 2^256` and
                // `delta < POW5_APPROX_ERROR * 2^128`, given that
                // `n_high * 2^lz < 2^128`.
                let shift = i64::from(e) - i64::from(self.j);
                let lz = n_high.leading_zeros();
                let [a_low_hi, a_hi, a_high_hi] = mul_around(n, dist_low, dist_high, lz, t);
                let frac_bits = i64::from(lz) - shift - i64::from(s) - 128;

                [
                    self.round_approx(n_low, shift, a_low_hi, frac_bits),
                    self.round_approx(n, shift, a_hi, frac_bits),
                    self.round_approx(n_high, shift, a_high_hi, frac_bits),
                ]
            }
        }
    }

    /// Attempts to calculate `floor(x)` and the rounding loss of
    /// `x = (n * 5^-j) * 2^shift`, given that `x = X / 2^(128 + frac_bits)`,
    /// where `X` is in the open interval `(A, A + delta)`,
    /// `A = a_hi * 2^128 + a_lo < 2^256` and
    /// `delta < POW5_APPROX_ERROR * 2^128`.
    #[inline]
    fn round_approx(
        &self,
        n: u128,
        shift: i64,
        a_hi: u128,
        frac_bits: i64,
    ) -> Option<(u128, RoundLoss)> {
        if frac_bits >= 130 {
            // `x < (2^256 + 2^130) / 2^258 < 1/2`
            const {
                assert!(POW5_APPROX_ERROR <= 4);
            }
            return Some((0, RoundLoss::HalfDown));
        }
        if matches!(frac_bits, 1..=127)
            && let Some(r) = shr_interval(a_hi, POW5_APPROX_ERROR, frac_bits as u32)
        {
            return Some(r);
        }

        self.try_div_exact(n, shift)
    }

    /// Attempts to calculate `floor(x)` and the rounding loss of
    /// `x = (n * 5^-j) * 2^shift`, which is only possible when `5^j` is an
    /// integer that fits in 128 bits and divides `n`.
    // Never inlined because it is rarely needed and it is large, which would
    // prevent its callers from being inlined.
    #[inline(never)]
    fn try_div_exact(&self, n: u128, shift: i64) -> Option<(u128, RoundLoss)> {
        if self.j > 0
            && let Some(n) = div_exact_pow5(n, self.j.unsigned_abs())
        {
            // `x` is exactly `(n / 5^j) * 2^shift`.
            shift_exact(n, 0, shift)
        } else {
            None
        }
    }
}

/// Power of ten `10^j` by which values that fit in 64 bits are divided in
/// the fast path, when their results also fit in 64 bits.
pub(super) struct Pow10U64 {
    j: i32,
    /// `5^-j` is `tau * 2^s`, where `tau` is `t` if `exact` is set, and it
    /// is in the open interval `(t, t + POW5_APPROX_ERROR)` otherwise.
    t: u128,
    s: i32,
    exact: bool,
}

impl Pow10U64 {
    /// Largest `j` such that `POW5_APPROX_ERROR * 5^j <= 2^65`. Larger
    /// powers of five do not fit in 64 bits, so they do not divide any `n`.
    const MAX_J_GAP: i32 = 27;

    /// Returns `None` if `10^j` is out of the range supported by the fast
    /// path.
    #[inline]
    pub(super) fn new(j: i32) -> Option<Self> {
        // `1 / 10^j = 5^q * 2^q`
        let q = j.checked_neg()?;
        let (t, s) = pow5_approx(q)?;
        Some(Self {
            j,
            t,
            s,
            exact: matches!(q, 0..=MAX_POW5_EXACT),
        })
    }

    /// Attempts to calculate `floor(x)` and the rounding loss of `x`, where
    /// `x = n * 2^e / 10^j` and `n` is not zero.
    ///
    /// Returns `None` when the result cannot be determined by the fast path,
    /// which is always the case when `x < n / 4` or `x >= 2^63`.
    #[inline]
    pub(super) fn try_scale(&self, n: u64, e: i32) -> Option<(u64, RoundLoss)> {
        debug_assert_ne!(n, 0);

        // `x = (n * 5^-j) * 2^shift = (n * 2^pre) * tau / 2^129`
        let shift = i64::from(e) - i64::from(self.j);
        let pre = shift + i64::from(self.s) + 129;
        if !matches!(pre, 0..=63) || n.leading_zeros() < pre as u32 {
            return None;
        }

        // `P = (n * 2^pre) * t = hi * 2^64 + lo`, which is less than `2^192`.
        let m = u128::from(n << pre);
        let lo = m * u128::from(self.t as u64);
        let hi = m * (self.t >> 64) + (lo >> 64);
        let lo = lo as u64;

        if self.exact {
            // `x = P / 2^129`
            let loss = RoundLoss::from_wide_shift((hi << 64) | u128::from(lo), hi >> 64, 129);
            return Some(((hi >> 65) as u64, loss));
        }

        // `x = X / 2^(128 + 65)`, where `X = (n * 2^pre) * tau * 2^64` is in
        // the open interval `(A, A + delta)`, with
        // `A = P * 2^64 = hi * 2^128 + lo * 2^64` and
        // `delta < POW5_APPROX_ERROR * 2^128`, given that `n * 2^pre < 2^64`.
        if let Some((floor, loss)) = shr_interval(hi, POW5_APPROX_ERROR, 65) {
            return Some((floor as u64, loss));
        }

        // Otherwise, the rounding boundary that follows `A`, where `x` is
        // `bound / 2`, is from `(hi + 1) * 2^128` to
        // `(hi + POW5_APPROX_ERROR) * 2^128`, so `X` is less than
        // `POW5_APPROX_ERROR * 2^128` away from it and `x` is less than
        // `POW5_APPROX_ERROR / 2^65` away from `bound / 2`.
        let bound = (hi >> 64) + 1;
        const {
            assert!(POW5_APPROX_ERROR * 5u128.pow(Pow10U64::MAX_J_GAP as u32) <= 1 << 65);
        }
        if bound.is_multiple_of(2) && matches!(self.j, 1..=Self::MAX_J_GAP) {
            // `x >= n / 4` because `tau >= 2^127`, so `2^shift >= 5^j / 4`,
            // which makes `shift` positive. Hence, `x = n * 2^shift / 5^j`
            // is a multiple of `1 / 5^j`, like the integer `bound / 2`. Two
            // different multiples are not closer than `1 / 5^j`, which is
            // not less than `POW5_APPROX_ERROR / 2^65`, so `x` is that
            // integer.
            debug_assert!(shift > 0);
            Some(((bound / 2) as u64, RoundLoss::Zero))
        } else {
            None
        }
    }
}

/// Calculates `floor(P / 2^128)` for the products `P` of `t` by
/// `(n - dist_low) * 2^lz`, `n * 2^lz` and `(n + dist_high) * 2^lz` (in that
/// order) with a single multiplication.
///
/// `dist_low` and `dist_high` must be powers of two, `dist_low` must not be
/// greater than `n` and `(n + dist_high) * 2^lz` must be less than `2^128`.
#[inline]
fn mul_around(n: u128, dist_low: u128, dist_high: u128, lz: u32, t: u128) -> [u128; 3] {
    let (p_lo, p_hi) = (n << lz).wide_mul(t);

    // The other products are `P - D` and `P + D`, where
    // `D = (dist * 2^lz) * t` is `t` shifted, because each `dist` is a
    // power of two.
    let (d_lo, d_hi) = wide_shift_left(t, 0, lz + dist_low.trailing_zeros());
    let low_hi = p_hi - d_hi - u128::from(p_lo < d_lo);
    let (d_lo, d_hi) = wide_shift_left(t, 0, lz + dist_high.trailing_zeros());
    let high_hi = p_hi + d_hi + u128::from(p_lo.overflowing_add(d_lo).1);

    [low_hi, p_hi, high_hi]
}

/// Calculates `floor(x)` and the rounding loss of `x = N * 2^shift`, where
/// `N = hi * 2^128 + lo` is not zero.
///
/// Returns `None` if `floor(x)` does not fit in 128 bits.
#[inline]
fn shift_exact(lo: u128, hi: u128, shift: i64) -> Option<(u128, RoundLoss)> {
    if shift >= 0 {
        if hi != 0 || shift > i64::from(lo.leading_zeros()) {
            return None;
        }
        Some((lo << shift, RoundLoss::Zero))
    } else if shift > -256 {
        let shift = shift.unsigned_abs() as u32;
        let (q_lo, q_hi) = wide_shift_right(lo, hi, shift);
        if q_hi != 0 {
            return None;
        }
        // `frac / 2^256` is the fractional part of `x`.
        let (frac_lo, frac_hi) = wide_shift_left(lo, hi, 256 - shift);
        Some((q_lo, RoundLoss::from_wide_frac(frac_lo, frac_hi)))
    } else if shift == -256 {
        Some((0, RoundLoss::from_wide_frac(lo, hi)))
    } else {
        // `x < 1/2`
        Some((0, RoundLoss::HalfDown))
    }
}

/// Calculates `floor(x)` and the rounding loss of `x`, where
/// `x = n * 2^e / 10^j`, `n` is not zero and `x < 2^128`.
///
/// `required_limbs(n, e, j)` must not be greater than `MAX_LIMBS`.
pub(super) fn scale_slow(n: u128, e: i32, j: i32) -> (u128, RoundLoss) {
    // Using `BigUInt` sized for the worst case has a noticeable cost.
    match required_limbs(n, e, j) {
        0..=SMALL_LIMBS => scale_slow_impl::<SMALL_LIMBS>(n, e, j),
        _ => scale_slow_impl::<MAX_LIMBS>(n, e, j),
    }
}

/// Implementation of `scale_slow` with big integers of `N` limbs, which must
/// be at least `required_limbs(n, e, j)`.
// Never inlined, so the stack frame of `scale_slow` does not include the
// large `BigUInt`s of every size, only the frame of the called instance does.
#[inline(never)]
fn scale_slow_impl<const N: usize>(n: u128, e: i32, j: i32) -> (u128, RoundLoss) {
    debug_assert!(N >= required_limbs(n, e, j));

    let (mut num, mut den) = fraction::<N>(n, e, j);

    // Shift `den` by whole limbs (which keeps it normalized) until it is
    // greater than `num`. Each shifted limb is a 64-bit digit of the
    // quotient.
    let mut num_digits = 0;
    while num >= den {
        den.shl(64);
        num_digits += 1;
    }
    debug_assert!(num_digits <= 2);

    // Calculate `q = floor(x)` with a long division. After each step, `num`
    // holds the remainder of the division calculated so far, which is less
    // than `den`.
    let mut q = 0;
    for _ in 0..num_digits {
        q = (q << 64) | u128::from(num.shl_div_rem(64, &den));
    }

    // `x = q + num / den`
    (q, num.frac_round_loss(&den))
}

/// Returns `(num, den)` such that `num / den = n * 2^e / 10^j`, where `den`
/// is normalized, as required by `BigUInt::shl_div_rem`. When `j <= 0`, `den`
/// is a power of two.
///
/// `N` must be at least `required_limbs(n, e, j)`.
pub(super) fn fraction<const N: usize>(n: u128, e: i32, j: i32) -> (BigUInt<N>, BigUInt<N>) {
    let mut num = BigUInt::<N>::from_u128(n);
    let mut den = BigUInt::<N>::from_u64(1);

    // `n * 2^e / 10^j = (n * 5^-j) * 2^(e - j)`
    if j <= 0 {
        num.mul_pow5(j.unsigned_abs().into());
    } else {
        den.mul_pow5(j.unsigned_abs().into());
    }
    let shift = i64::from(e) - i64::from(j);
    let (num_shift, den_shift) = if shift >= 0 {
        (shift as usize, 0)
    } else {
        (0, shift.unsigned_abs() as usize)
    };

    BigUInt::shl_normalized(&mut num, &mut den, num_shift, den_shift);

    (num, den)
}

/// Returns the number of limbs needed by the big integers returned by
/// `fraction`, including the room needed to calculate up to 128 bits of
/// `floor(num / den)` with `BigUInt::shl_div_rem`.
pub(super) fn required_limbs(n: u128, e: i32, j: i32) -> usize {
    // A positive integer `x` has `floor(log2(x)) + 1` bits and
    // `log2(5) < 2.32193`.
    let n_bits = u64::from(128 - n.leading_zeros());
    let pow5_bits = u64::from(j.unsigned_abs()) * 232193 / 100000 + 1;
    let (num_bits, den_bits) = if j <= 0 {
        (n_bits + pow5_bits, 1)
    } else {
        (n_bits, pow5_bits)
    };
    let shift = i64::from(e) - i64::from(j);
    let (num_bits, den_bits) = if shift >= 0 {
        (num_bits + shift.unsigned_abs(), den_bits)
    } else {
        (num_bits, den_bits + shift.unsigned_abs())
    };

    // `den` is normalized to a whole number of limbs, which shifts `num` by
    // 63 bits at most.
    let num_limbs = (num_bits + 63).div_ceil(64);
    let den_limbs = den_bits.div_ceil(64);
    // `den` might be shifted by two more limbs to be greater than `num`, and
    // the remainder shifted in `shl_div_rem` (as well as twice the final
    // remainder) needs one more limb.
    (num_limbs.max(den_limbs + 2) + 1) as usize
}

/// Number of limbs that is enough for the values of any format with
/// moderate exponents, and for any value of the formats up to binary64.
pub(super) const SMALL_LIMBS: usize = 32;

/// Largest number of limbs needed to format a value within the limits of
/// `Finite`, see `tests::test_max_limbs`.
pub(super) const MAX_LIMBS: usize = 184;

#[cfg(test)]
mod tests {
    use rand::RngExt as _;

    use super::super::exact::calc_k_up;
    use super::super::floor_log10_pow2;
    use super::super::tests::{create_prng, for_each_max_finite, scale_reference};
    use super::{
        MAX_LIMBS, Pow10, Pow10U64, mul_around, required_limbs, scale_slow, scale_slow_impl,
    };
    use crate::ieee_float::pow5::POW5_APPROX_ERROR;
    use crate::traits::UInt as _;
    use crate::utils::RoundLoss;

    /// Generates `(n, e, j)` such that `n` has `n_bits` bits and
    /// `x = n * 2^e / 10^j` is in `[2^(x_bits - 2), 2^(x_bits + 1))`.
    fn gen_input(
        rng: &mut impl rand::RngExt,
        n_bits: u32,
        j: i32,
        x_bits: i32,
    ) -> (u128, i32, i32) {
        let n = (rng.random::<u128>() >> (128 - n_bits)) | (1 << (n_bits - 1));
        // `x_bits = n_bits + e - j * log2(10)`
        let e = x_bits - n_bits as i32 + (i64::from(j) * 3321928).div_euclid(1000000) as i32;
        (n, e, j)
    }

    /// Checks both paths with `x = n * 2^e / 10^j`, which must be less than
    /// `2^128`.
    ///
    /// Returns whether the fast path was able to handle it.
    fn check(n: u128, e: i32, j: i32) -> bool {
        let (expected_x, expected_loss) = scale_reference(&n.into(), e, j);
        let expected = (expected_x.to_u128().unwrap(), expected_loss);

        // Only the values needed to format a float are supported.
        if required_limbs(n, e, j) <= MAX_LIMBS {
            assert_eq!(scale_slow(n, e, j), expected, "n = {n}, e = {e}, j = {j}");
        }

        let actual = Pow10::new(j).and_then(|pow10| pow10.try_scale(n, e));
        if let Some(actual) = actual {
            assert_eq!(actual, expected, "n = {n}, e = {e}, j = {j}");
        }
        actual.is_some()
    }

    #[test]
    fn test_random() {
        let mut rng = create_prng();

        // Any `n` and `j`, with results from zero to almost `2^128`.
        for _ in 0..20_000 {
            let n_bits = rng.random_range(1..=116);
            let j = if rng.random() {
                rng.random_range(-5020..=4960)
            } else {
                rng.random_range(-70..=70)
            };
            let x_bits = if rng.random_ratio(1, 4) {
                rng.random_range(-600..=0)
            } else {
                rng.random_range(-2..=126)
            };
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            check(n, e, j);
        }

        // Values like the ones needed to format up to binary64 and x87
        // extended, which the fast path should almost always handle.
        let num_inputs = 20_000;
        let mut num_handled = 0;
        for _ in 0..num_inputs {
            let n_bits = rng.random_range(1..=66);
            let j = rng.random_range(-4990..=4930);
            let x_bits = rng.random_range(1..=70);
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            if check(n, e, j) {
                num_handled += 1;
            }
        }
        assert!(
            num_handled >= num_inputs - num_inputs / 1000,
            "only {num_handled} of {num_inputs} inputs handled",
        );
    }

    /// Checks values that are exactly an integer or a small fraction, which
    /// the fast path must always handle when the power of five is exact.
    #[test]
    fn test_exact() {
        let mut rng = create_prng();

        let mut has_exact = false;
        let mut has_tie = false;
        for _ in 0..20_000 {
            let j: i32 = rng.random_range(-50..=50);
            let pow5 = 5u128.pow(j.unsigned_abs());
            // Keep `c * 5^|j| < 2^124`.
            let max_c_bits = pow5.leading_zeros() - 4;
            let c_bits = rng.random_range(1..=max_c_bits.min(64));
            let c = (rng.random::<u128>() >> (128 - c_bits)) | 1;

            // `x = c * 5^max(-j, 0) * 2^d`
            let d = rng.random_range(-3..=3);
            let n = if j > 0 { c * pow5 } else { c };
            let e = j + d;

            let expected_loss = scale_reference(&n.into(), e, j).1;
            has_exact |= expected_loss == RoundLoss::Zero;
            has_tie |= expected_loss == RoundLoss::Halfway;
            assert!(check(n, e, j), "n = {n}, e = {e}, j = {j}");
        }
        assert!(has_exact && has_tie);
    }

    #[test]
    fn test_mul_around() {
        let mut rng = create_prng();
        for _ in 0..100_000 {
            let n_bits = rng.random_range(2..=127);
            let n = (rng.random::<u128>() >> (128 - n_bits)) | (1 << (n_bits - 1));
            // `dist_low < 2^(n_bits - 1) <= n`
            let dist_low = 1 << rng.random_range(0..(n_bits - 1));
            let dist_high = 1 << rng.random_range(0..n_bits);
            let lz = rng.random_range(0..=(n + dist_high).leading_zeros());
            let t = rng.random();

            let expected = [n - dist_low, n, n + dist_high].map(|n| (n << lz).wide_mul(t).1);
            assert_eq!(
                mul_around(n, dist_low, dist_high, lz, t),
                expected,
                "n = {n}, dist_low = {dist_low}, dist_high = {dist_high}, lz = {lz}, t = {t}",
            );
        }
    }

    /// Checks `Pow10::try_scale_around`, which calculates three values at
    /// once.
    #[test]
    fn test_around() {
        /// Returns whether the three values were handled.
        fn check(n: u128, dist_low: u128, dist_high: u128, e: i32, j: i32) -> bool {
            let Some(pow10) = Pow10::new(j) else {
                return false;
            };
            let actual = pow10.try_scale_around(n, dist_low, dist_high, e);
            for (n, actual) in [n - dist_low, n, n + dist_high].into_iter().zip(actual) {
                let (expected_x, expected_loss) = scale_reference(&n.into(), e, j);
                if let Some((actual_x, actual_loss)) = actual {
                    assert_eq!(
                        (Some(actual_x), actual_loss),
                        (expected_x.to_u128(), expected_loss),
                        "n = {n}, e = {e}, j = {j}",
                    );
                }
            }
            actual.iter().all(Option::is_some)
        }

        let mut rng = create_prng();

        // Any `n`, `j` and distances, with results up to almost `2^128`,
        // which are the ones that depend on every bit of the product.
        for _ in 0..20_000 {
            let n_bits = rng.random_range(2..=116);
            let j = if rng.random() {
                rng.random_range(-5020..=4960)
            } else {
                rng.random_range(-70..=70)
            };
            let x_bits = if rng.random() {
                rng.random_range(100..=124)
            } else {
                rng.random_range(-200..=124)
            };
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            // `dist_low < 2^(n_bits - 1) <= n`
            let dist_low = 1 << rng.random_range(0..(n_bits - 1));
            let dist_high = 1 << rng.random_range(0..n_bits);
            check(n, dist_low, dist_high, e, j);
        }

        // Values like the ones needed to find the shortest representation
        // of up to binary64 and x87 extended, which should almost always be
        // handled.
        let num_inputs = 20_000;
        let mut num_handled = 0;
        for _ in 0..num_inputs {
            let n_bits = rng.random_range(3..=66);
            let j = rng.random_range(-4990..=4930);
            let x_bits = rng.random_range(4..=70);
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            // The low bound of the interval has fewer bits than the value
            // when the latter is a power of two.
            let n = if rng.random_ratio(1, 8) {
                1 << (n_bits - 1)
            } else {
                n & !3
            };
            let dist_low = rng.random_range(1..=2);
            if check(n, dist_low, 2, e, j) {
                num_handled += 1;
            }
        }
        assert!(
            num_handled >= num_inputs - num_inputs / 1000,
            "only {num_handled} of {num_inputs} inputs handled",
        );
    }

    /// Checks `Pow10U64::try_scale` with `x = n * 2^e / 10^j`.
    ///
    /// Returns whether it was able to handle it.
    fn check_u64(n: u64, e: i32, j: i32) -> bool {
        let actual = Pow10U64::new(j).and_then(|pow10| pow10.try_scale(n, e));
        if let Some((actual_x, actual_loss)) = actual {
            let (expected_x, expected_loss) = scale_reference(&n.into(), e, j);
            assert_eq!(
                (rug::Integer::from(actual_x), actual_loss),
                (expected_x, expected_loss),
                "n = {n}, e = {e}, j = {j}",
            );
        }
        actual.is_some()
    }

    #[test]
    fn test_u64_random() {
        let mut rng = create_prng();

        // Any `n` and `j`, with results that might be too small or too large
        // to be handled.
        let mut num_handled = 0;
        for _ in 0..20_000 {
            let n_bits = rng.random_range(1..=64);
            let j = if rng.random() {
                rng.random_range(-5020..=4960)
            } else {
                rng.random_range(-70..=70)
            };
            let x_bits = n_bits as i32 + rng.random_range(-6..=12);
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            if check_u64(n as u64, e, j) {
                num_handled += 1;
            }
        }
        assert!(num_handled >= 5000);

        // Values whose results are from `n / 4` to `2^62`, like the ones
        // needed to find the shortest representation of up to binary64,
        // which should always be handled.
        for _ in 0..20_000 {
            let n_bits = rng.random_range(1..=56);
            let j = rng.random_range(-4990..=4930);
            let x_bits = rng.random_range(n_bits..=61) as i32;
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);
            assert!(check_u64(n as u64, e, j), "n = {n}, e = {e}, j = {j}");
        }
    }

    /// Checks values that are exactly an integer or a small fraction with
    /// `Pow10U64`, which must always handle them.
    #[test]
    fn test_u64_exact() {
        let mut rng = create_prng();

        // `x = c * 5^-j * 2^d`, where the power of five is exact.
        let mut has_exact = false;
        let mut has_tie = false;
        for _ in 0..20_000 {
            let j: i32 = rng.random_range(-24..=0);
            let d = rng.random_range(-2..=3);
            // Keep `x < 2^61`.
            let pow5_bits = 64 - 5u64.pow(j.unsigned_abs()).leading_zeros();
            let c_bits = rng.random_range(1..=(58 - pow5_bits));
            let c = (rng.random::<u64>() >> (64 - c_bits)) | 1;
            let e = j + d;

            let expected_loss = scale_reference(&c.into(), e, j).1;
            has_exact |= expected_loss == RoundLoss::Zero;
            has_tie |= expected_loss == RoundLoss::Halfway;
            assert!(check_u64(c, e, j), "n = {c}, e = {e}, j = {j}");
        }
        assert!(has_exact && has_tie);

        // `x = (c * 5^j + r) * 2^d / 5^j`, which is the integer `c * 2^d`
        // when `r` is zero and very close to it otherwise, for every `j`
        // such that `5^j` fits in 64 bits.
        for j in 1..=27i32 {
            let pow5 = 5u64.pow(j.unsigned_abs());
            // The smallest `d` such that `x >= n / 4`.
            let min_d = 64 - pow5.leading_zeros() as i32 - 2;
            for _ in 0..500 {
                // Any number of bits such that `c * 5^j` fits in 64 bits.
                let max_c = (u64::MAX / pow5).min(1 << rng.random_range(1..=63));
                let c = rng.random_range(1..=max_c);
                let n = c * pow5;
                // Keep `n * 2^d / 5^j < 2^62`.
                let max_d = min_d.max(61 - (64 - c.leading_zeros() as i32));
                let d = rng.random_range(min_d..=max_d);
                let e = j + d;

                assert_eq!(
                    Pow10U64::new(j).unwrap().try_scale(n, e),
                    Some((c << d, RoundLoss::Zero)),
                    "n = {n}, e = {e}, j = {j}",
                );
                for n in [n - 1, n + 1] {
                    check_u64(n, e, j);
                }
            }
        }
    }

    /// Checks that `Pow10U64` knows when its power of five is exact.
    #[test]
    fn test_u64_exact_flag() {
        let mut num_exact = 0;
        for j in -5100..=5100 {
            let Some(pow10) = Pow10U64::new(j) else {
                continue;
            };
            // `5^-j` is not a multiple of a power of two when `j > 0`.
            let is_exact = j <= 0 && {
                let pow5 = rug::Integer::from(rug::Integer::u_pow_u(5, j.unsigned_abs()));
                let t = rug::Integer::from(pow10.t);
                if pow10.s >= 0 {
                    pow5 == t << pow10.s.unsigned_abs()
                } else {
                    pow5 << pow10.s.unsigned_abs() == t
                }
            };
            assert_eq!(pow10.exact, is_exact, "j = {j}");
            num_exact += usize::from(is_exact);
        }
        assert_eq!(num_exact, 56);
    }

    /// Checks the arithmetic of `Pow10U64::try_scale` with arbitrary
    /// multipliers in place of the power of five, including the ones that
    /// leave the result as close as possible to a rounding boundary.
    #[test]
    fn test_u64_arith() {
        /// Calculates `floor(x)` and the rounding loss of `x = X / 2^129`.
        fn exact(x: &rug::Integer) -> (rug::Integer, RoundLoss) {
            let frac = x.clone().keep_bits(129);
            let half = rug::Integer::from(1) << 128;
            let loss = if frac.is_zero() {
                RoundLoss::Zero
            } else {
                match frac.cmp(&half) {
                    core::cmp::Ordering::Less => RoundLoss::HalfDown,
                    core::cmp::Ordering::Equal => RoundLoss::Halfway,
                    core::cmp::Ordering::Greater => RoundLoss::HalfUp,
                }
            };
            (x.clone() >> 129, loss)
        }

        fn widen(r: Option<(u64, RoundLoss)>) -> Option<(rug::Integer, RoundLoss)> {
            r.map(|(x, loss)| (x.into(), loss))
        }

        let mut rng = create_prng();
        let mut num_some = 0;
        let mut num_none = 0;
        let mut num_zero = 0;
        let mut num_tie = 0;
        let mut num_near_int = 0;
        for _ in 0..100_000 {
            let n_bits = rng.random_range(2..=64);
            let mut n = (rng.random::<u64>() >> (64 - n_bits)) | (1 << (n_bits - 1));
            let pre = rng.random_range(0..=n.leading_zeros().min(63));

            // `P = (n * 2^pre) * t`
            let t: u128 = match rng.random_range(0..3) {
                0 => rng.random(),
                1 => {
                    // The smallest `t` such that `floor(P / 2^64)` is `dist`
                    // below a multiple of `2^64`, which is `bound * 2^64`.
                    let m = rug::Integer::from(n) << pre;
                    let bound = rng.random_range(1..(n << pre));
                    let dist = rng.random_range(0..=2 * POW5_APPROX_ERROR as u32);
                    let target = ((rug::Integer::from(bound) << 64u32) - dist) << 64u32;
                    let t: rug::Integer = (target + &m - 1u32) / &m;
                    t.to_u128().unwrap()
                }
                _ => {
                    // `P` has from 127 to 130 trailing zeros, so it is at a
                    // rounding boundary or next to one.
                    n = 1 << (n_bits - 1);
                    let p_zeros = rng.random_range(127..=130);
                    let t_zeros = (p_zeros - (n_bits - 1) - pre).min(127);
                    (rng.random::<u128>() | 1) << t_zeros
                }
            };
            let m = rug::Integer::from(n) << pre;
            let p = m.clone() * t;

            // With `x = n * 2^(pre - 129) * tau`, `tau = t` and positive `shift`.
            let new = |j, exact| Pow10U64 {
                j,
                t,
                s: pre as i32 - 130,
                exact,
            };
            let scale = |j, exact| widen(new(j, exact).try_scale(n, j + 1));

            // `x = P / 2^129`
            assert_eq!(
                scale(0, true),
                Some(exact(&p)),
                "n = {n}, pre = {pre}, t = {t}"
            );
            num_zero += usize::from(exact(&p).1 == RoundLoss::Zero);
            num_tie += usize::from(exact(&p).1 == RoundLoss::Halfway);

            // `x = X / 2^129`, with `X` in the open interval
            // `(P, P + POW5_APPROX_ERROR * n * 2^pre)`. A rounding boundary
            // lies in it if and only if the results of the first and the
            // last integer are different.
            let first = exact(&(p.clone() + 1u32));
            let last = exact(&(p.clone() + m * POW5_APPROX_ERROR - 1u32));
            let actual = scale(0, false);
            if let Some(actual) = &actual {
                assert_eq!(
                    (&first, &last),
                    (actual, actual),
                    "n = {n}, pre = {pre}, t = {t}"
                );
                num_some += 1;
            } else {
                num_none += 1;
            }

            // The powers of ten whose multiples are recognized as integers
            // when `x` is too close to one: `bound / 2`.
            let hi = (p >> 64u32).to_u128().unwrap();
            let bound = (hi >> 64) + 1;
            let near_int = bound.is_multiple_of(2) && (hi + POW5_APPROX_ERROR) >> 64 == bound;
            num_near_int += usize::from(near_int);
            for j in [-56, -1, 1, 27, 28] {
                let expected = if near_int && matches!(j, 1..=27) {
                    Some(((bound / 2).into(), RoundLoss::Zero))
                } else {
                    actual.clone()
                };
                assert_eq!(
                    scale(j, false),
                    expected,
                    "n = {n}, pre = {pre}, t = {t}, j = {j}"
                );
            }
        }
        assert!(num_some >= 10_000 && num_none >= 10_000 && num_near_int >= 1000);
        assert!(num_zero >= 1000 && num_tie >= 1000);
    }

    /// Checks that `required_limbs` is enough by running `scale_slow_impl`
    /// with exactly that number of limbs (which panics if it is too small).
    #[test]
    fn test_required_limbs() {
        macro_rules! check_any {
            ($n:expr, $e:expr, $j:expr; $($limbs:literal)*) => {
                match required_limbs($n, $e, $j) {
                    $($limbs => scale_slow_impl::<$limbs>($n, $e, $j),)*
                    limbs => panic!("unexpected number of limbs: {limbs}"),
                }
            };
        }

        let mut rng = create_prng();
        for _ in 0..50_000 {
            let n_bits = rng.random_range(1..=116);
            let j = rng.random_range(-250..=250);
            let x_bits = rng.random_range(-130..=127);
            let (n, e, j) = gen_input(&mut rng, n_bits, j, x_bits);

            // All ones is the largest `n` for its number of bits.
            for n in [n, u128::MAX >> (128 - n_bits)] {
                let (expected_x, expected_loss) = scale_reference(&n.into(), e, j);
                let Some(expected_x) = expected_x.to_u128() else {
                    continue;
                };
                let actual = check_any!(n, e, j; 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18);
                assert_eq!(actual, (expected_x, expected_loss));
            }
        }
    }

    /// Checks that `MAX_LIMBS` is enough for any value within the limits of
    /// `Finite`.
    #[test]
    fn test_max_limbs() {
        let mut max_limbs = 0;
        // `required_limbs` only depends on the number of bits of the integer,
        // which is the largest one of each size here.
        for_each_max_finite(|value| {
            // `shortest::shortest`
            for num_levels in [1, 2] {
                let fine_j = floor_log10_pow2(value.exp) + 1 - num_levels;
                let limbs = required_limbs(4 * value.mant + 2, value.exp - 2, fine_j);
                max_limbs = max_limbs.max(limbs);
            }
            // `exact::exact_slow`
            let limbs = required_limbs(value.mant, value.exp, calc_k_up(value));
            max_limbs = max_limbs.max(limbs);
        });
        assert_eq!(max_limbs, MAX_LIMBS);
    }
}
