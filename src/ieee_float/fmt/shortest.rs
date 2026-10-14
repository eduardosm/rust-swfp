//! Shortest decimal number that rounds to a float.
//!
//! Every real number in the rounding interval of a float `v`, whose bounds
//! are the midpoints between `v` and its two adjacent floats, rounds to `v`
//! when rounding to nearest. The bounds themselves round to `v` (with ties
//! to even) only when its significand is even.
//!
//! The result is `d * 10^k`, where `k` is the largest exponent for which a
//! multiple of `10^k` lies in the rounding interval, and `d * 10^k` is the
//! one closest to `v` among them (the largest one in case of tie). This is
//! the same result as the one of the Rust standard library, although it is
//! calculated like in the Schubfach algorithm, described in "The Schubfach
//! way to render doubles", by Raffaello Giulietti:
//!
//! 1. `j` is chosen so that `10^(j - 1) <= 2^e < 10^j`, where `2^e` is the
//!    distance between `v` and the next float, which is also the maximum
//!    width of the rounding interval. Hence, at most one multiple of `10^j`
//!    lies in the interval. If there is one, it is the result, after moving
//!    its trailing zeros to the exponent.
//! 2. Otherwise, at least one multiple of `10^(j - 1)` lies in the interval,
//!    because it is not narrower than `10^(j - 1)`, so the closest one to `v`
//!    is the result. The exception is when the lower half of the interval is
//!    narrower, in which case `10^(j - 2)` might be needed.
//!
//! In order to do that, the bounds of the interval and `v` are divided by the
//! smallest of those powers of ten (see `scale`), so the candidates of each
//! step are the multiples of 100, 10 or 1 in a range of integers.

use super::scale::{Pow10, Pow10U64, scale_slow};
use super::{Finite, MAX_PREC_BITS, floor_log10_pow2};
use crate::ieee_float::pow5::POW5_EXACT;
use crate::traits::UInt as _;
use crate::utils::{DEC_CHUNK_LEN, DEC_CHUNK_POW10, RoundLoss};

/// Returns `(d, k)` such that `d * 10^k` is the shortest decimal number that
/// rounds to `value`, where `d` is not a multiple of 10.
pub(super) fn shortest(value: &Finite) -> (u128, i32) {
    let m = value.mant;
    let e = value.exp;

    if matches!(e, -127..=0) && m.trailing_zeros() >= e.unsigned_abs() {
        // The value is an integer and the rounding interval is narrower
        // than 1, so no other integer lies in it.
        return strip_zeros_u128(m >> e.unsigned_abs(), 0);
    }

    // `m < 2^MAX_PREC_BITS`, so `4 * m + 2` fits in 128 bits. So does
    // everything that is divided by `10^fine_j` below: the results are less
    // than `100 * (m + 1)`, because `10^fine_j > 2^e / 100`.
    const {
        assert!(100 <= u128::MAX >> MAX_PREC_BITS);
    }

    // The rounding interval is `[(n_v - n_dist) * 2^e2, (n_v + 2) * 2^e2]`
    // and the value is `n_v * 2^e2`.
    let n_v = 4 * m;
    let n_dist = if value.asymmetric { 1 } else { 2 };
    let e2 = e - 2;
    let inclusive = m.is_multiple_of(2);

    // `10^(j - 1) <= 2^e < 10^j`
    let j = floor_log10_pow2(e) + 1;
    // The width of the interval is `2^e` or `3/4 * 2^e`, so it is not
    // narrower than `10^fine_j`.
    let num_levels = if value.asymmetric { 2 } else { 1 };
    let fine_j = j - num_levels;

    // Divide everything by `10^fine_j`.
    let scaled = Scaled::try_new_exact(n_v, n_dist, e2, fine_j)
        .unwrap_or_else(|| Scaled::new(n_v, n_dist, e2, fine_j, inclusive));
    debug_assert!(scaled.min_x <= scaled.max_x);
    debug_assert!(scaled.x_v <= scaled.max_x + 1);

    // 128-bit divisions are slow, so they are avoided when possible.
    if scaled.max_x < u128::from(u64::MAX) {
        let (d, level) = select_u64(
            scaled.min_x as u64,
            scaled.max_x as u64,
            scaled.x_v as u64,
            scaled.round_up,
            num_levels,
        );
        (u128::from(d), fine_j + level)
    } else {
        let (d, level) = select_u128(
            scaled.min_x,
            scaled.max_x,
            scaled.x_v,
            scaled.round_up,
            num_levels,
        );
        (d, fine_j + level)
    }
}

/// The rounding interval and the value, divided by `10^fine_j`.
struct Scaled {
    /// `x * 10^fine_j` is in the interval when `min_x <= x <= max_x`.
    min_x: u128,
    max_x: u128,
    /// The value is `(x_v + frac) * 10^fine_j`, with `0 <= frac < 1`.
    x_v: u128,
    /// Whether `frac >= 1/2`.
    round_up: bool,
}

impl Scaled {
    /// Divides `(n_v - n_dist) * 2^e2`, `n_v * 2^e2` and `(n_v + 2) * 2^e2`
    /// by `10^fine_j`. The bounds of the interval are only included when
    /// `inclusive` is set.
    // Never inlined, so it does not affect the code of the most common values
    // in `shortest`, which do not need it.
    #[inline(never)]
    fn new(n_v: u128, n_dist: u128, e2: i32, fine_j: i32, inclusive: bool) -> Self {
        match u64::try_from(n_v + 2) {
            Ok(n_high) => Self::new_u64(n_high - 2, n_dist as u64, e2, fine_j, inclusive),
            Err(_) => Self::new_u128(n_v, n_dist, e2, fine_j, inclusive),
        }
    }

    /// Implementation of `new` for values that fit in 64 bits, which is the
    /// case of the formats up to binary64. `n_v + 2` must not overflow.
    // Never inlined, like `new_u128`, so the code of each of them is not
    // affected by the other one.
    #[inline(never)]
    fn new_u64(n_v: u64, n_dist: u64, e2: i32, fine_j: i32, inclusive: bool) -> Self {
        Self::try_new_u64(n_v, n_dist, e2, fine_j, inclusive)
            .unwrap_or_else(|| Self::new_u128(n_v.into(), n_dist.into(), e2, fine_j, inclusive))
    }

    /// Attempts to do the same as `new_u64` with a fast path, which fails
    /// when a result is too large or it cannot be determined. That is very
    /// unlikely for the formats up to binary64.
    #[inline]
    fn try_new_u64(n_v: u64, n_dist: u64, e2: i32, fine_j: i32, inclusive: bool) -> Option<Self> {
        let pow10 = Pow10U64::new(fine_j)?;
        let (x_low, loss_low) = pow10.try_scale(n_v - n_dist, e2)?;
        let (x_v, loss_v) = pow10.try_scale(n_v, e2)?;
        let (x_high, loss_high) = pow10.try_scale(n_v + 2, e2)?;

        Some(Self::from_parts(
            (x_low.into(), loss_low),
            (x_v.into(), loss_v),
            (x_high.into(), loss_high),
            inclusive,
        ))
    }

    /// Implementation of `new` for values of any size.
    #[inline(never)]
    fn new_u128(n_v: u128, n_dist: u128, e2: i32, fine_j: i32, inclusive: bool) -> Self {
        let [low, v, high] = Pow10::new(fine_j).map_or([None; 3], |pow10| {
            pow10.try_scale_around(n_v, n_dist, 2, e2)
        });
        Self::from_parts(
            low.unwrap_or_else(|| scale_slow(n_v - n_dist, e2, fine_j)),
            v.unwrap_or_else(|| scale_slow(n_v, e2, fine_j)),
            high.unwrap_or_else(|| scale_slow(n_v + 2, e2, fine_j)),
            inclusive,
        )
    }

    /// Builds it from `floor(x)` and the rounding loss of `x`, for the
    /// lower bound of the interval, the value and the upper bound, divided
    /// by `10^fine_j`.
    #[inline]
    fn from_parts(
        (x_low, loss_low): (u128, RoundLoss),
        (x_v, loss_v): (u128, RoundLoss),
        (x_high, loss_high): (u128, RoundLoss),
        inclusive: bool,
    ) -> Self {
        Self {
            min_x: x_low + u128::from(!(inclusive && loss_low == RoundLoss::Zero)),
            max_x: x_high - u128::from(!inclusive && loss_high == RoundLoss::Zero),
            x_v,
            round_up: matches!(loss_v, RoundLoss::Halfway | RoundLoss::HalfUp),
        }
    }

    /// Does the same as `new` in a cheaper way, with a single exact
    /// multiplication, which is only possible when `5^-fine_j` fits in 128
    /// bits and the results are neither too large nor too small. That is the
    /// case of the most common values.
    #[inline]
    fn try_new_exact(n_v: u128, n_dist: u128, e2: i32, fine_j: i32) -> Option<Self> {
        // `n * 2^e2 / 10^fine_j = n * pow5 / 2^shift`
        let pow5 = *POW5_EXACT.get(usize::try_from(-i64::from(fine_j)).ok()?)?;
        let shift = fine_j - e2;
        if !matches!(shift, 2..=120) {
            return None;
        }
        let shift = shift as u32;
        let mask = (1 << shift) - 1;
        // `pow5 / 2^shift` is a quarter of `2^e / 10^fine_j`, which is less
        // than 100, so `pow5 < 2^125`.
        debug_assert!(pow5 >> shift < 25);

        // `n_v * pow5 = x_v * 2^shift + rem_v`
        let (lo, hi) = n_v.wide_mul(pow5);
        debug_assert!(hi >> shift == 0);
        let x_v = (lo >> shift) | (hi << (128 - shift));
        let rem_v = lo & mask;

        // `n_v - n_dist` and `n_v + 2` are not multiples of 4 and `pow5` is
        // odd, so the bounds of the interval are not multiples of
        // `10^fine_j`. Hence, it does not matter whether they are included.

        // `max_x = floor((n_v + 2) * pow5 / 2^shift)`
        let max_x = x_v + ((rem_v + 2 * pow5) >> shift);
        // `min_x = floor((n_v - n_dist) * pow5 / 2^shift) + 1`
        let dist_low = n_dist * pow5;
        let min_x = if let Some(borrow) = dist_low.checked_sub(rem_v + 1) {
            // `rem_v - dist_low = -(borrow + 1)`, whose floor after dividing
            // by `2^shift` is `-floor(borrow / 2^shift) - 1`.
            x_v - (borrow >> shift)
        } else {
            x_v + 1
        };

        Some(Self {
            min_x,
            max_x,
            x_v,
            round_up: rem_v >> (shift - 1) != 0,
        })
    }
}

macro_rules! impl_select {
    ($select:ident, $closest_multiple:ident, $strip_zeros:ident, $ty:ty) => {
        /// Returns `(d, level)` such that `d * 10^level` is the closest
        /// integer to `x_v + frac` (the largest one in case of tie) among
        /// the multiples of `10^level` in `min_x..=max_x`, `level` is as
        /// large as possible and `d` is not a multiple of 10.
        ///
        /// `frac` is in `[0, 1)` and `round_up` tells whether it is at least
        /// 1/2. There must not be more than one multiple of
        /// `10^num_levels` in `min_x..=max_x`.
        fn $select(
            min_x: $ty,
            max_x: $ty,
            x_v: $ty,
            round_up: bool,
            num_levels: i32,
        ) -> ($ty, i32) {
            // A multiple of a larger power of ten would have been found in a
            // higher level, except in the first one.
            if num_levels == 2
                && let Some(d) = $closest_multiple::<100>(min_x, max_x, x_v)
            {
                $strip_zeros(d, 2)
            } else if let Some(d) = $closest_multiple::<10>(min_x, max_x, x_v) {
                $strip_zeros(d, 1)
            } else {
                ((x_v + <$ty>::from(round_up)).clamp(min_x, max_x), 0)
            }
        }

        /// Returns `d` such that `d * UNIT` is the closest integer to
        /// `x_v + frac` (the largest one in case of tie) among the multiples
        /// of `UNIT` in `min_x..=max_x`, if there is any.
        ///
        /// `frac` is in `[0, 1)` and `UNIT` is even, so `frac` cannot break
        /// a tie.
        // `UNIT` is a constant because divisions by constants are faster.
        #[inline]
        fn $closest_multiple<const UNIT: $ty>(min_x: $ty, max_x: $ty, x_v: $ty) -> Option<$ty> {
            let first = min_x.div_ceil(UNIT);
            let last = max_x / UNIT;
            if first > last {
                return None;
            }
            let below = x_v / UNIT;
            let nearest = if x_v - below * UNIT >= UNIT / 2 {
                below + 1
            } else {
                below
            };
            Some(nearest.clamp(first, last))
        }
    };
}

impl_select!(select_u64, closest_multiple_u64, strip_zeros_u64, u64);
impl_select!(select_u128, closest_multiple_u128, strip_zeros_u128, u128);

/// Returns `(d2, k2)` such that `d2 * 10^k2 = d * 10^k` and `d2` is not a
/// multiple of 10, where `d` is not zero.
#[inline]
fn strip_zeros_u64(mut d: u64, mut k: i32) -> (u64, i32) {
    debug_assert_ne!(d, 0);
    while d.is_multiple_of(100_000_000) {
        d /= 100_000_000;
        k += 8;
    }
    if d.is_multiple_of(10_000) {
        d /= 10_000;
        k += 4;
    }
    if d.is_multiple_of(100) {
        d /= 100;
        k += 2;
    }
    if d.is_multiple_of(10) {
        d /= 10;
        k += 1;
    }
    (d, k)
}

/// Like `strip_zeros_u64`, for `u128`.
fn strip_zeros_u128(mut d: u128, mut k: i32) -> (u128, i32) {
    debug_assert_ne!(d, 0);
    let mut is_first = true;
    loop {
        if let Ok(d) = u64::try_from(d) {
            let (d, k) = strip_zeros_u64(d, k);
            return (u128::from(d), k);
        }
        if !d.is_multiple_of(10) {
            return (d, k);
        }
        // 128-bit divisions are slow, so the zeros are not stripped one by
        // one, except the first one: most of the values that have any do
        // not have more.
        if !is_first {
            return strip_many_zeros_u128(d, k);
        }
        is_first = false;
        d /= 10;
        k += 1;
    }
}

/// Does the same as `strip_zeros_u128`, in a way that is only cheaper when
/// `d` has many trailing zeros.
// Never inlined, so its size does not affect the code of the most common
// values in `shortest`, which do not need it.
#[inline(never)]
fn strip_many_zeros_u128(mut d: u128, mut k: i32) -> (u128, i32) {
    loop {
        if let Ok(d) = u64::try_from(d) {
            let (d, k) = strip_zeros_u64(d, k);
            return (u128::from(d), k);
        }

        // A single division splits `d` into `hi * 10^19 + lo`, and the
        // zeros are stripped from the part that has them, which fits in 64
        // bits.
        let hi = d / u128::from(DEC_CHUNK_POW10);
        let lo = (d % u128::from(DEC_CHUNK_POW10)) as u64;
        if lo != 0 {
            // The trailing zeros of `d` are the ones of `lo`, which are
            // fewer than 19.
            let (lo, num_zeros) = strip_zeros_u64(lo, 0);
            let hi_scale = 10u64.pow(DEC_CHUNK_LEN as u32 - num_zeros as u32);
            return (hi * u128::from(hi_scale) + u128::from(lo), k + num_zeros);
        }
        // `d` is `hi * 10^19`.
        d = hi;
        k += DEC_CHUNK_LEN as i32;
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt as _;

    use super::super::Finite;
    use super::super::tests::{create_prng, exp_range, for_each_semantics, gen_finite};
    use super::{Scaled, shortest, strip_many_zeros_u128, strip_zeros_u128};
    use crate::ieee_float::Semantics;

    /// Calculates the shortest representation of `value` with exact integer
    /// arithmetic, straight from its definition.
    fn reference(value: &Finite) -> (u128, i32) {
        let n_low = rug::Integer::from(4 * value.mant - if value.asymmetric { 1 } else { 2 });
        let n_v = rug::Integer::from(4 * value.mant);
        let n_high = rug::Integer::from(4 * value.mant + 2);
        let e = value.exp - 2;
        let inclusive = value.mant.is_multiple_of(2);

        // Start with a power of ten that is greater than the upper bound
        // (`log10(2) < 0.30103`), so no multiple of it is in the interval.
        let high_bits = n_high.significant_bits() as i32 + e;
        let mut k = (high_bits * 30103).div_euclid(100000) + 1;
        loop {
            // `n * 2^e / 10^k = n * num / den`
            let mut num = rug::Integer::from(1);
            let mut den = rug::Integer::from(1);
            if e >= 0 {
                num <<= e.unsigned_abs();
            } else {
                den <<= e.unsigned_abs();
            }
            let pow10 = rug::Integer::from(rug::Integer::u_pow_u(10, k.unsigned_abs()));
            if k >= 0 {
                den *= pow10;
            } else {
                num *= pow10;
            }

            // `d * 10^k` is in the interval when `min_d <= d <= max_d`.
            let (low_floor, low_rem) = (n_low.clone() * &num).div_rem(den.clone());
            let (high_floor, high_rem) = (n_high.clone() * &num).div_rem(den.clone());
            let min_d = if inclusive && low_rem == 0 {
                low_floor
            } else {
                low_floor + 1u32
            };
            let max_d = if !inclusive && high_rem == 0 {
                high_floor - 1u32
            } else {
                high_floor
            };

            if min_d <= max_d {
                // `floor(x + 1/2)` is the closest integer to `x`, the largest
                // one in case of tie.
                let nearest = (n_v.clone() * &num * 2u32 + &den) / (den * 2u32);
                let d = Ord::clamp(nearest, min_d, max_d);
                return (d.to_u128().unwrap(), k);
            }
            k -= 1;
        }
    }

    fn check(value: &Finite) {
        assert_eq!(
            shortest(value),
            reference(value),
            "mant = {}, exp = {}, asymmetric = {}",
            value.mant,
            value.exp,
            value.asymmetric,
        );
    }

    #[test]
    fn test_random() {
        fn test<S: Semantics>(rng: &mut impl rand::RngExt) {
            for _ in 0..2000 {
                check(&gen_finite::<S>(rng));
            }
        }

        let mut rng = create_prng();
        for_each_semantics!(test(&mut rng));
    }

    /// Checks every value of the formats up to binary16.
    #[test]
    fn test_exhaustive() {
        fn test<S: Semantics>() {
            let (min_exp, max_exp) = exp_range::<S>();
            let min_mant = 1u128 << (S::PREC_BITS - 1);
            let max_mant = 2 * min_mant - 1;

            // Subnormal values
            for mant in 1..min_mant {
                check(&Finite {
                    mant,
                    exp: min_exp,
                    asymmetric: false,
                });
            }
            // Normal values
            for exp in min_exp..=max_exp {
                for mant in min_mant..=max_mant {
                    check(&Finite {
                        mant,
                        exp,
                        asymmetric: mant == min_mant,
                    });
                }
            }
        }

        test::<crate::f8e5m2::F8E5M2Semantics>();
        test::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>();
        test::<crate::f8e4m3nao::F8E4M3NaoSemantics>();
        test::<crate::f16::F16Semantics>();
    }

    /// Checks that `Scaled::new_u64` returns the same as `Scaled::new_u128`,
    /// including the values that its fast path is not able to handle.
    #[test]
    fn test_new_u64() {
        fn fields(scaled: &Scaled) -> (u128, u128, u128, bool) {
            (scaled.min_x, scaled.max_x, scaled.x_v, scaled.round_up)
        }

        let mut rng = create_prng();
        let mut num_fast = 0;
        let mut num_fallback = 0;
        for _ in 0..20_000 {
            // `n_v = 4 * mant`, like in `shortest`.
            let mant_bits: i32 = rng.random_range(1..=62);
            let mant = (rng.random::<u64>() >> (64 - mant_bits)) | (1 << (mant_bits - 1));
            let n_v = 4 * mant;
            let n_dist = rng.random_range(1..=2);
            let inclusive = rng.random();

            // `x = n_v * 2^e2 / 10^fine_j` has around `x_bits` bits, which
            // might be too few or too many for the fast path.
            let fine_j = rng.random_range(-60..=60);
            let x_bits = mant_bits + rng.random_range(-10..=42);
            let e2 =
                x_bits - (mant_bits + 2) + (i64::from(fine_j) * 3321928).div_euclid(1000000) as i32;

            let actual = Scaled::new_u64(n_v, n_dist, e2, fine_j, inclusive);
            let expected = Scaled::new_u128(n_v.into(), n_dist.into(), e2, fine_j, inclusive);
            assert_eq!(
                fields(&actual),
                fields(&expected),
                "n_v = {n_v}, n_dist = {n_dist}, e2 = {e2}, fine_j = {fine_j}",
            );

            if Scaled::try_new_u64(n_v, n_dist, e2, fine_j, inclusive).is_some() {
                num_fast += 1;
            } else {
                num_fallback += 1;
            }
        }
        assert!(num_fast >= 2000 && num_fallback >= 2000);
    }

    /// Checks `strip_zeros_u128` and `strip_many_zeros_u128` with any number
    /// of digits and of trailing zeros.
    #[test]
    fn test_strip_zeros() {
        let mut rng = create_prng();
        for _ in 0..50_000 {
            // `d = c * 10^num_zeros`, where `c` is not a multiple of 10.
            let num_zeros = rng.random_range(0..=38);
            let max_c = u128::MAX / 10u128.pow(num_zeros);
            let c_bits = rng.random_range(1..=(128 - max_c.leading_zeros()));
            let mut c = (rng.random::<u128>() >> (128 - c_bits)).clamp(1, max_c);
            if c.is_multiple_of(10) {
                c -= 1;
            }
            let d = c * 10u128.pow(num_zeros);
            let k = rng.random_range(-100..=100);

            let expected = (c, k + num_zeros as i32);
            assert_eq!(strip_zeros_u128(d, k), expected, "d = {d}");
            assert_eq!(strip_many_zeros_u128(d, k), expected, "d = {d}");
        }
    }

    /// Checks integers and their neighbors, whose shortest representations
    /// have many trailing zeros or are just next to one that has them.
    #[test]
    fn test_integers() {
        fn test<S: Semantics>() {
            let (min_exp, max_exp) = exp_range::<S>();
            let min_mant = 1u128 << (S::PREC_BITS - 1);
            let max_mant = 2 * min_mant - 1;

            for exp10 in 0..=37 {
                for digit in 1..=9 {
                    // `mant * 2^exp <= digit * 10^exp10 < (mant + 1) * 2^exp`
                    let int = digit * 10u128.pow(exp10);
                    let int_bits = 128 - int.leading_zeros();
                    let (mant, exp) = if int_bits <= S::PREC_BITS {
                        let shift = S::PREC_BITS - int_bits;
                        (int << shift, -(shift as i32))
                    } else {
                        let shift = int_bits - S::PREC_BITS;
                        (int >> shift, shift as i32)
                    };
                    if !(min_exp..=max_exp).contains(&exp) {
                        continue;
                    }

                    for mant in (mant - 2)..=(mant + 2) {
                        if (min_mant..=max_mant).contains(&mant) {
                            check(&Finite {
                                mant,
                                exp,
                                asymmetric: mant == min_mant,
                            });
                        }
                    }
                }
            }
        }

        for_each_semantics!(test());
    }
}
