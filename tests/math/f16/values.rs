use swfp::{F16, Float as _};

use super::{mk_normal, mk_subnormal, to_rug};

const MANT_BITS: u32 = 10;
const MANT_MASK: u16 = (1 << MANT_BITS) - 1;
const MIN_EXP: i8 = -14;
const MAX_EXP: i8 = 15;
/// Exponent of the smallest positive subnormal value.
const MIN_SUB_EXP: i8 = MIN_EXP - MANT_BITS as i8;

/// Returns an iterator over a set of positive special `F16` values,
/// including NaN, infinity, and extreme values.
pub(super) fn specials() -> impl Iterator<Item = F16> {
    [
        F16::NAN,
        F16::INFINITY,
        mk_normal(MANT_MASK, MAX_EXP, false),
        mk_normal(0, MIN_EXP, false),
        F16::ZERO,
        mk_subnormal(1, false),
        mk_subnormal(2, false),
    ]
    .into_iter()
}

/// Returns `(1 + m * 2^-10) * 2^e` for `e` from `MIN_SUB_EXP` to `MAX_EXP`. When
/// `e < MIN_EXP`, the value is subnormal and the low bits of the mantissa that
/// do not fit are discarded.
fn mk_value(m: u16, e: i8) -> F16 {
    assert!(m <= MANT_MASK);
    if e >= MIN_EXP {
        mk_normal(m, e, false)
    } else {
        assert!(e >= MIN_SUB_EXP);
        mk_subnormal(((1 << MANT_BITS) | m) >> (MIN_EXP - e), false)
    }
}

/// Mantissa patterns at the ends and the middle of the range.
fn basic_mantissa_patterns() -> [u16; 11] {
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

fn mantissa_patterns() -> impl Iterator<Item = u16> {
    let max = MANT_MASK;
    let mut patterns: Vec<u16> = basic_mantissa_patterns()
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

/// Returns an iterator over all positive subnormal `F16` values.
pub(super) fn subnormals_full() -> impl Iterator<Item = F16> {
    (1..=MANT_MASK).map(|m| mk_subnormal(m, false))
}

/// Returns an iterator over all positive normal `F16` values at the given binades.
pub(super) fn binades_full(e: impl IntoIterator<Item = i8>) -> impl Iterator<Item = F16> {
    e.into_iter()
        .flat_map(|e| (0..=MANT_MASK).map(move |m| mk_normal(m, e, false)))
}

/// Evenly spread pairs of values in `[0, MANT_MASK]^2` (see [`crate::utils::spread2`]).
fn spread2(n: u32, seed: u64) -> impl Iterator<Item = (u16, u16)> {
    crate::utils::spread2(MANT_BITS, n, seed).map(|(a, b)| (a as u16, b as u16))
}

/// Returns an iterator over the exponent pairs `(e1, e2)` with `e1 - e2 = d`
/// for each `d` in `diffs`, where both are exponents of positive finite values
/// (from -24 to 15, see [`binade_pairs`]).
///
/// For each difference, `e1` takes the two lowest and the two highest valid
/// values, the values that make `e1` or `e2` zero, and every `step`-th value
/// in between.
pub(super) fn exp_pairs(
    diffs: impl IntoIterator<Item = i8>,
    step: i8,
) -> impl Iterator<Item = (i8, i8)> {
    assert!(step > 0);
    diffs.into_iter().flat_map(move |d| {
        let lo = MIN_SUB_EXP.max(MIN_SUB_EXP + d);
        let hi = MAX_EXP.min(MAX_EXP + d);
        (lo..=hi)
            .filter(move |&e| e - lo < 2 || hi - e < 2 || e == 0 || e == d || (e - lo) % step == 0)
            .map(move |e| (e, e - d))
    })
}

/// Returns an iterator over pairs of positive finite `F16` values `(x, y)`
/// where `x` is in the binade with exponent `ex` and `y` in the binade with
/// exponent `ey`, for each `(ex, ey)` in `exps`. Exponents below -14 give
/// subnormal values, with fewer mantissa bits.
///
/// Each exponent pair includes all the combinations of the basic mantissa
/// patterns, each mantissa pattern paired with itself, and `n_spread` mantissa
/// pairs evenly spread over all the combinations.
pub(super) fn binade_pairs(
    exps: impl IntoIterator<Item = (i8, i8)>,
    n_spread: u32,
) -> impl Iterator<Item = (F16, F16)> {
    let basic = basic_mantissa_patterns();
    let pattern_pairs: Vec<(u16, u16)> = basic
        .iter()
        .flat_map(|&mx| basic.iter().map(move |&my| (mx, my)))
        // The basic ones are already paired with themselves
        .chain(
            mantissa_patterns()
                .filter(|m| !basic.contains(m))
                .map(|m| (m, m)),
        )
        .collect();

    exps.into_iter().flat_map(move |(ex, ey)| {
        let seed = (u64::from(ex as u8) << 8) | u64::from(ey as u8);
        pattern_pairs
            .clone()
            .into_iter()
            .chain(spread2(n_spread, seed))
            .map(move |(mx, my)| (mk_value(mx, ex), mk_value(my, ey)))
    })
}

/// Returns `2^e`.
fn exp2i(e: i32) -> rug::Float {
    rug::Float::with_val(8, 1) << e
}

#[test]
fn test_mk_value() {
    for e in MIN_SUB_EXP..=MAX_EXP {
        for m in mantissa_patterns() {
            let x = to_rug(mk_value(m, e));
            // In the binade with exponent `e`
            assert!(x >= exp2i(e.into()));
            assert!(x < exp2i(i32::from(e) + 1));
            // `(1 + m * 2^-10) * 2^e` with the bits that do not fit discarded
            let exact = rug::Float::with_val(64, (1 << MANT_BITS) | m) << (i32::from(e) - 10);
            let ulp = exp2i((i32::from(e) - 10).max(-24));
            assert!(x <= exact && exact < rug::Float::with_val(64, &x + &ulp));
        }
    }
}

#[test]
fn test_exp_pairs() {
    for d in [-39, -38, -20, -1, 0, 1, 10, 38, 39] {
        // Valid exponents with `e1 - e2 = d`
        let lo = MIN_SUB_EXP.max(MIN_SUB_EXP + d);
        let hi = MAX_EXP.min(MAX_EXP + d);
        for step in [1, 3, 10, 100] {
            let e1s: Vec<i8> = exp_pairs([d], step)
                .map(|(e1, e2)| {
                    assert_eq!(e1 - e2, d);
                    e1
                })
                .collect();
            assert!(e1s.windows(2).all(|w| w[0] < w[1]));
            assert!(e1s.iter().all(|e| (lo..=hi).contains(e)));

            // The edges, the ones that make `e1` or `e2` zero, and every
            // `step`-th one
            let expected = [lo, lo + 1, hi - 1, hi, 0, d]
                .into_iter()
                .chain((lo..=hi).step_by(step as usize))
                .filter(|e| (lo..=hi).contains(e));
            for e in expected {
                assert!(e1s.contains(&e), "d = {d}, step = {step}, e = {e}");
            }
            if step == 1 {
                assert_eq!(e1s.len(), (hi - lo + 1) as usize);
            }
        }
    }
}

#[test]
fn test_binade_pairs() {
    let n_patterns = mantissa_patterns().count();
    let in_binade = |x: F16, e: i8| {
        let x = to_rug(x);
        x >= exp2i(e.into()) && x < exp2i(i32::from(e) + 1)
    };
    for (ex, ey) in [
        (MIN_SUB_EXP, MIN_SUB_EXP),
        (MIN_SUB_EXP, MAX_EXP),
        (-15, -14),
        (0, 0),
        (5, -3),
        (MAX_EXP, MAX_EXP),
    ] {
        let pairs: Vec<(F16, F16)> = binade_pairs([(ex, ey)], 100).collect();
        // All the combinations of the 11 basic patterns, the other patterns
        // paired with themselves, and the spread pairs
        assert_eq!(pairs.len(), 11 * 11 + (n_patterns - 11) + 100);
        assert!(
            pairs
                .iter()
                .all(|&(x, y)| in_binade(x, ex) && in_binade(y, ey))
        );
    }
}
