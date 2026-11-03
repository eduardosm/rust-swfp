use swfp::{F64, Float as _};

use super::{mk_normal, mk_subnormal, to_rug};

const MANT_BITS: u32 = 52;
const MANT_MASK: u64 = (1 << MANT_BITS) - 1;
const MIN_EXP: i16 = -1022;
const MAX_EXP: i16 = 1023;
/// Exponent of the smallest positive subnormal value.
const MIN_SUB_EXP: i16 = MIN_EXP - MANT_BITS as i16;

/// Returns an iterator over a set of positive special `F64` values,
/// including NaN, infinity, and extreme values.
pub(super) fn specials() -> impl Iterator<Item = F64> {
    [
        F64::NAN,
        F64::INFINITY,
        mk_normal(MANT_MASK, MAX_EXP, false),
        mk_normal(0, MIN_EXP, false),
        F64::ZERO,
        mk_subnormal(1, false),
        mk_subnormal(2, false),
    ]
    .into_iter()
}

/// Returns `(1 + m * 2^-52) * 2^e` for `e` from `MIN_SUB_EXP` to `MAX_EXP`. When
/// `e < MIN_EXP`, the value is subnormal and the low bits of the mantissa that
/// do not fit are discarded.
fn mk_value(m: u64, e: i16) -> F64 {
    assert!(m <= MANT_MASK);
    if e >= MIN_EXP {
        mk_normal(m, e, false)
    } else {
        assert!(e >= MIN_SUB_EXP);
        mk_subnormal(((1 << MANT_BITS) | m) >> (MIN_EXP - e), false)
    }
}

/// Mantissa patterns at the ends and the middle of the range.
fn basic_mantissa_patterns() -> [u64; 11] {
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

fn mantissa_patterns() -> impl Iterator<Item = u64> {
    let max = MANT_MASK;
    let mut patterns: Vec<u64> = basic_mantissa_patterns()
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

/// Returns an iterator over a set of positive subnormal `F64` values,
/// include `n_spread` evenly spread values.
pub(super) fn subnormals(n_spread: u32) -> impl Iterator<Item = F64> {
    mantissa_patterns()
        .filter(|&m| m != 0)
        .chain(spread(n_spread, 0xE43A))
        .map(|m| mk_subnormal(m, false))
}

/// Returns an iterator over a set of positive normal `F64` values at the given
/// binades. Each binade will include a set of mantissa patterns and `n_spread`
/// evenly spread values.
pub(super) fn binades(
    e: impl IntoIterator<Item = i16>,
    n_spread: u32,
) -> impl Iterator<Item = F64> {
    e.into_iter().flat_map(move |e| {
        mantissa_patterns()
            .chain(spread(n_spread, i64::from(e) as u64))
            .map(move |m| mk_normal(m, e, false))
    })
}

/// Evenly spread values in `[0, MANT_MASK]` (Weyl sequence based on the golden
/// ratio).
fn spread(n: u32, seed: u64) -> impl Iterator<Item = u64> {
    crate::utils::spread(MANT_BITS, n, seed)
}

/// Evenly spread pairs of values in `[0, MANT_MASK]^2` (see [`crate::utils::spread2`]).
fn spread2(n: u32, seed: u64) -> impl Iterator<Item = (u64, u64)> {
    crate::utils::spread2(MANT_BITS, n, seed)
}

/// Returns an iterator over `n` values evenly spread over `[lo, hi]`.
pub(super) fn interval(lo: F64, hi: F64, n: u32) -> impl Iterator<Item = F64> {
    assert!(lo <= hi);
    crate::utils::spread(53, n, lo.to_bits() ^ hi.to_bits())
        .map(move |u| lo + (hi - lo) * F64::from_uint(u.into()).scalbn(-53))
}

/// Returns an iterator over `x` and the `n` values before and after it.
pub(super) fn around(x: F64, n: u32) -> impl Iterator<Item = F64> {
    let max = ordinal(F64::INFINITY);
    let ord = ordinal(x);
    (ord - i64::from(n)..=ord + i64::from(n))
        .filter(move |o| o.abs() <= max)
        .map(from_ordinal)
}

/// Position of `x` in the ordered sequence of values (`+0` and `-0` are
/// both mapped to zero).
fn ordinal(x: F64) -> i64 {
    let bits = x.to_bits();
    let mag = (bits & !0x8000_0000_0000_0000) as i64;
    if (bits & 0x8000_0000_0000_0000) != 0 {
        -mag
    } else {
        mag
    }
}

/// Inverse of [`ordinal`] (zero is mapped to `+0`).
fn from_ordinal(ord: i64) -> F64 {
    if ord < 0 {
        F64::from_bits(ord.unsigned_abs() | 0x8000_0000_0000_0000)
    } else {
        F64::from_bits(ord as u64)
    }
}

/// Returns an iterator over the exponent pairs `(e1, e2)` with `e1 - e2 = d`
/// for each `d` in `diffs`, where both are exponents of positive finite values
/// (from -1074 to 1023, see [`binade_pairs`]).
///
/// For each difference, `e1` takes the two lowest and the two highest valid
/// values, the values that make `e1` or `e2` zero, and every `step`-th value
/// in between.
pub(super) fn exp_pairs(
    diffs: impl IntoIterator<Item = i16>,
    step: i16,
) -> impl Iterator<Item = (i16, i16)> {
    assert!(step > 0);
    diffs.into_iter().flat_map(move |d| {
        let lo = MIN_SUB_EXP.max(MIN_SUB_EXP + d);
        let hi = MAX_EXP.min(MAX_EXP + d);
        (lo..=hi)
            .filter(move |&e| e - lo < 2 || hi - e < 2 || e == 0 || e == d || (e - lo) % step == 0)
            .map(move |e| (e, e - d))
    })
}

/// Returns an iterator over pairs of positive finite `F64` values `(x, y)`
/// where `x` is in the binade with exponent `ex` and `y` in the binade with
/// exponent `ey`, for each `(ex, ey)` in `exps`. Exponents below -1022 give
/// subnormal values, with fewer mantissa bits.
///
/// Each exponent pair includes all the combinations of the basic mantissa
/// patterns, each mantissa pattern paired with itself, and `n_spread` mantissa
/// pairs evenly spread over all the combinations.
pub(super) fn binade_pairs(
    exps: impl IntoIterator<Item = (i16, i16)>,
    n_spread: u32,
) -> impl Iterator<Item = (F64, F64)> {
    let basic = basic_mantissa_patterns();
    let pattern_pairs: Vec<(u64, u64)> = basic
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
        let seed = (u64::from(ex as u16) << 16) | u64::from(ey as u16);
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
fn test_subnormals_not_zero() {
    assert!(subnormals(20_000_000).all(|x| x != F64::ZERO));
}

#[test]
fn test_mk_value() {
    for e in MIN_SUB_EXP..=MAX_EXP {
        for m in mantissa_patterns() {
            let x = to_rug(mk_value(m, e));
            // In the binade with exponent `e`
            assert!(x >= exp2i(e.into()));
            assert!(x < exp2i(i32::from(e) + 1));
            // `(1 + m * 2^-52) * 2^e` with the bits that do not fit discarded
            let exact = rug::Float::with_val(64, (1 << MANT_BITS) | m) << (i32::from(e) - 52);
            let ulp = exp2i((i32::from(e) - 52).max(-1074));
            assert!(x <= exact && exact < rug::Float::with_val(64, &x + &ulp));
        }
    }
}

#[test]
fn test_exp_pairs() {
    for d in [-2097, -2096, -1100, -1, 0, 1, 30, 2096, 2097] {
        // Valid exponents with `e1 - e2 = d`
        let lo = MIN_SUB_EXP.max(MIN_SUB_EXP + d);
        let hi = MAX_EXP.min(MAX_EXP + d);
        for step in [1, 7, 100, 5000] {
            let e1s: Vec<i16> = exp_pairs([d], step)
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
    let in_binade = |x: F64, e: i16| {
        let x = to_rug(x);
        x >= exp2i(e.into()) && x < exp2i(i32::from(e) + 1)
    };
    for (ex, ey) in [
        (MIN_SUB_EXP, MIN_SUB_EXP),
        (MIN_SUB_EXP, MAX_EXP),
        (-1023, -1022),
        (0, 0),
        (5, -3),
        (MAX_EXP, MAX_EXP),
    ] {
        let pairs: Vec<(F64, F64)> = binade_pairs([(ex, ey)], 100).collect();
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

#[test]
fn test_interval() {
    for (lo, hi) in [(0.0, 1.0), (-745.5, 709.5), (1e300, 1.5e300), (2.0, 2.0)] {
        let n = 1000;
        let mut xs: Vec<f64> = interval(F64::from_host(lo), F64::from_host(hi), n)
            .map(F64::to_host)
            .collect();
        assert_eq!(xs.len(), n as usize);
        assert!(xs.iter().all(|x| (lo..=hi).contains(x)));

        // Evenly spread
        xs.sort_by(f64::total_cmp);
        let max_gap = [xs[0] - lo, hi - xs[xs.len() - 1]]
            .into_iter()
            .chain(xs.windows(2).map(|w| w[1] - w[0]))
            .fold(0.0, f64::max);
        assert!(max_gap <= 2.0 * (hi - lo) / f64::from(n));
    }
}

#[test]
fn test_around() {
    let collect = |x: f64, n: u32| {
        around(F64::from_host(x), n)
            .map(F64::to_host)
            .collect::<Vec<f64>>()
    };
    let one_below = 1.0 - f64::EPSILON / 2.0;
    assert_eq!(
        collect(1.0, 2),
        [
            one_below - f64::EPSILON / 2.0,
            one_below,
            1.0,
            1.0 + f64::EPSILON,
            1.0 + 2.0 * f64::EPSILON
        ],
    );
    // Across zero (only one zero)
    let min_sub = mk_subnormal(1, false).to_host();
    assert_eq!(collect(0.0, 1), [-min_sub, 0.0, min_sub]);
    assert_eq!(collect(-0.0, 1), [-min_sub, 0.0, min_sub]);
    assert!(collect(-0.0, 1)[1].is_sign_positive());
    // Not beyond infinity
    let below_max = f64::from_bits(f64::MAX.to_bits() - 1);
    let below_below_max = f64::from_bits(f64::MAX.to_bits() - 2);
    assert_eq!(
        collect(f64::MAX, 2),
        [below_below_max, below_max, f64::MAX, f64::INFINITY]
    );
    assert_eq!(
        collect(f64::NEG_INFINITY, 1),
        [f64::NEG_INFINITY, -f64::MAX]
    );
}
