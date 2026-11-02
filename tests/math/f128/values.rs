use swfp::{F128, Float as _};

use super::{mk_normal, mk_subnormal};

const MANT_BITS: u32 = 112;
const MANT_MASK: u128 = (1 << MANT_BITS) - 1;
const MIN_EXP: i16 = -16382;
const MAX_EXP: i16 = 16383;

/// Returns an iterator over a set of positive special `F128` values,
/// including NaN, infinity, and extreme values.
pub(super) fn specials() -> impl Iterator<Item = F128> {
    [
        F128::NAN,
        F128::INFINITY,
        mk_normal(MANT_MASK, MAX_EXP, false),
        mk_normal(0, MIN_EXP, false),
        F128::ZERO,
        mk_subnormal(1, false),
        mk_subnormal(2, false),
    ]
    .into_iter()
}

/// Mantissa patterns at the ends and the middle of the range.
fn basic_mantissa_patterns() -> [u128; 11] {
    let max = MANT_MASK;
    let alt = max / 3; // 0b0101...
    [
        0,
        1,
        2,
        max,
        max - 1,
        max - 2,
        1 << (MANT_BITS - 1),
        (1 << (MANT_BITS - 1)) - 1,
        (1 << (MANT_BITS - 1)) + 1,
        alt,
        alt << 1,
    ]
}

fn mantissa_patterns() -> impl Iterator<Item = u128> {
    let max = MANT_MASK;
    let mut patterns: Vec<u128> = basic_mantissa_patterns()
        .into_iter()
        .chain((2..=4).flat_map(move |i| {
            // .01b, .11b, .001b, .111b, .0001b, .1111b
            [1 << (MANT_BITS - i), max - ((1 << (MANT_BITS - i)) - 1)]
        }))
        .chain((0..MANT_BITS).flat_map(|i| {
            // .100000...b, .111111...b, .010000...b, .011111...b, ...
            [1 << i, (1 << (i + 1)) - 1]
        }))
        .collect();
    // Some are repeated (such as `1 << (MANT_BITS - 1)`)
    patterns.sort_unstable();
    patterns.dedup();
    patterns.into_iter()
}

/// Returns an iterator over a set of positive subnormal `F128` values,
/// include `n_spread` evenly spread values.
pub(super) fn subnormals(n_spread: u32) -> impl Iterator<Item = F128> {
    mantissa_patterns()
        .filter(|&m| m != 0)
        .chain(spread(n_spread, 0xE43A))
        .map(|m| mk_subnormal(m, false))
}

/// Returns an iterator over a set of positive normal `F128` values at the given
/// binades. Each binade will include a set of mantissa patterns and `n_spread`
/// evenly spread values.
pub(super) fn binades(
    e: impl IntoIterator<Item = i16>,
    n_spread: u32,
) -> impl Iterator<Item = F128> {
    e.into_iter().flat_map(move |e| {
        mantissa_patterns()
            .chain(spread(n_spread, i64::from(e) as u64))
            .map(move |m| mk_normal(m, e, false))
    })
}

/// Evenly spread values in `[0, MANT_MASK]` (Weyl sequence based on the golden
/// ratio).
fn spread(n: u32, seed: u64) -> impl Iterator<Item = u128> {
    crate::utils::spread_u128(MANT_BITS, n, seed)
}

#[test]
fn test_subnormals_not_zero() {
    assert!(subnormals(20_000_000).all(|x| x != F128::ZERO));
}
