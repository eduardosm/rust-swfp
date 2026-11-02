use swfp::{F32, Float as _, math::Pow as _};

use super::{PREC, check_exact, from_rug, mk_normal, to_rug, to_rug_prec, values};

#[test]
fn test_pow() {
    crate::generic::test_pow_special::<F32>();

    test_pow_with(|x, y| {
        let actual = x.pow(y);
        check_neg_base(x, y);

        let mut expected = to_rug(x);
        let round_cmp = rug::ops::PowAssignRound::pow_assign_round(
            &mut expected,
            &to_rug(y),
            rug::float::Round::Nearest,
        );
        check_exact(
            (x, y),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

/// Checks `pow(-|x|, y)` against `pow(|x|, y)`: they are equal when `y` is an
/// even integer (or infinite) and opposite when it is an odd integer. When `y`
/// is not an integer, `pow(-|x|, y)` is NaN if `x` is finite and non-zero, and
/// equal to `pow(|x|, y)` otherwise.
fn check_neg_base(x: F32, y: F32) {
    let pos = x.abs().pow(y);
    let neg = (-x.abs()).pow(y);
    if y.is_infinite() || y % F32::from_uint(2) == F32::ZERO {
        assert_total_eq!(neg, pos);
    } else if y.trunc() == y {
        if pos.is_nan() {
            assert_total_eq!(neg, pos);
        } else {
            assert_total_eq!(neg, -pos);
        }
    } else if x.is_finite() && x != F32::ZERO {
        assert!(neg.is_nan(), "pow({:?}, {y:?}) = {neg:?}", -x.abs());
    } else {
        assert_total_eq!(neg, pos);
    }
}

#[test]
fn test_powi() {
    crate::generic::test_powi_special::<F32>();

    test_powi_with(|x, n| {
        let actual = x.powi(n);
        if let (y, swfp::FpStatus::OK) = F32::from_int_ex(n.into(), swfp::Round::NearestTiesToEven)
        {
            assert_total_eq!(x.pow(y), actual);
        }
        if n % 2 == 0 || actual.is_nan() {
            assert_total_eq!((-x).powi(n), actual);
        } else {
            assert_total_eq!((-x).powi(n), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = rug::ops::PowAssignRound::pow_assign_round(
            &mut expected,
            n,
            rug::float::Round::Nearest,
        );
        check_exact(
            (x, n),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

fn test_pow_with(mut f: impl FnMut(F32, F32)) {
    // Negative bases are tested by `check_neg_base`.

    // Test special values
    for x in values::specials().chain([F32::from_uint(1)]) {
        for y in values::specials().chain(special_exponents()) {
            f(x, y);
            f(x, -y);
        }
    }

    // Test halves and small integers as exponents with bases at every binade
    for x in values::binades(-126..=127, 100).chain(values::subnormals(1000)) {
        for y in [half(1), half(2), half(3), half(4), half(6)] {
            f(x, y);
            f(x, -y);
        }
    }

    // Test the largest exponents with bases close to one
    for x in values::binades([-1, 0], 1000) {
        for y in special_exponents() {
            f(x, y);
            f(x, -y);
        }
    }

    // Test results spread across the whole range, with bases at every binade,
    // including subnormals and bases close to one
    for x in test_bases() {
        for y in result_targets(x) {
            f(x, y);
        }
    }

    // Test exact results and ties
    for (x, y) in exact_results() {
        f(x, y);
    }

    // Test powers of two as exponents with two as base
    for ey in -126..=127 {
        let x = F32::from_uint(2);
        let y = mk_normal(0, ey, false);
        f(x, y);
        f(x, -y);
    }

    // Test data from core-math
    for [x, y] in crate::data::read_data_file::<[F32; 2]>("core-math/binary32/powf.wc") {
        f(x, y);
    }
}

fn test_powi_with(mut f: impl FnMut(F32, i32)) {
    // Negative bases are tested by the caller.

    // Test special values
    for x in values::specials().chain([F32::from_uint(1)]) {
        for n in [0, 1, 2, 3, i32::MAX - 1, i32::MAX] {
            f(x, n);
            f(x, -n);
        }
        f(x, i32::MIN);
    }

    // Test small exponents with bases at every binade
    for x in values::binades(-126..=127, 100).chain(values::subnormals(1000)) {
        for n in [1, 2, 3] {
            f(x, n);
            f(x, -n);
        }
    }

    // Test larger exponents with bases at every binade
    for x in values::binades(-126..=127, 3).chain(values::subnormals(10)) {
        for n in 4..=128 {
            f(x, n);
            f(x, -n);
        }
    }

    // Test results spread across the whole range, with bases at every binade,
    // including subnormals and bases close to one (where the exponents can
    // saturate)
    for x in test_bases() {
        for y in result_targets(x) {
            if y.trunc() == y {
                f(x, to_i32_saturating(y));
            }
        }
    }

    // Test exact results and ties
    for (x, y) in exact_results() {
        if y.trunc() == y {
            f(x, to_i32_saturating(y));
        }
    }

    // Test data from core-math
    for [x, y] in crate::data::read_data_file::<[F32; 2]>("core-math/binary32/powf.wc") {
        let (Some(n), swfp::FpStatus::OK) = y.to_int_ex(32, swfp::Round::TowardZero) else {
            continue;
        };
        f(x, n as i32);
    }
}

/// Returns `n / 2`.
fn half(n: u128) -> F32 {
    F32::from_uint(n) / F32::from_uint(2)
}

/// Returns positive exponents that need special handling: halves and small
/// integers, and the largest integers and non-integers, where the parity
/// matters for negative bases.
fn special_exponents() -> impl Iterator<Item = F32> {
    [
        half(1),
        half(2),
        half(3),
        half(4),
        half(6),
        // Largest non-integer
        half((1 << 24) - 1),
        // Largest odd integers
        F32::from_uint((1 << 23) - 1),
        F32::from_uint((1 << 23) + 1),
        F32::from_uint((1 << 24) - 1),
        // Even integers, all the larger ones are even
        F32::from_uint(1 << 23),
        F32::from_uint(1 << 24),
        mk_normal((1 << 23) - 1, 127, false),
    ]
    .into_iter()
}

/// Returns positive bases at every binade, including subnormals and bases
/// close to one, where `y` needs to be large.
fn test_bases() -> impl Iterator<Item = F32> {
    let one = F32::from_uint(1);
    let near_one = values::binades(-24..=-1, 5000).flat_map(move |v| [one + v, one - v]);
    values::binades(-126..=127, 2000)
        .chain(values::subnormals(10_000))
        .chain(near_one)
}

/// Returns exponents `y` such that `x^y` is spread across the whole range.
///
/// `log2(x^y) = y * log2(x)` takes evenly spread values in `[-160, 140]`
/// (beyond the overflow and underflow limits), tiny values (results close to
/// one), and the values at the limits of the range: the overflow threshold,
/// the smallest normal and subnormal values, and half the smallest subnormal
/// (which is rounded to zero). Each non-integer exponent is also rounded to an
/// integer, which tests negative bases too.
fn result_targets(x: F32) -> Vec<F32> {
    let log2x = to_rug_prec(x, 128).log2();
    if log2x.is_zero() {
        return Vec::new();
    }

    let seed = u64::from(x.to_bits());
    let unit = |s: u64| s as f64 / (1u64 << 32) as f64;
    let spread = crate::utils::spread(32, 8, seed).map(|s| -160.0 + 300.0 * unit(s));
    let tiny = crate::utils::spread(32, 2, !seed)
        .zip([1.0, -1.0])
        .map(|(s, sign)| sign * (-1.0 - 30.0 * unit(s)).exp2());
    let limits = [128.0, -126.0, -149.0, -150.0];

    let mut ys = Vec::new();
    for t in spread.chain(tiny).chain(limits) {
        let y = round_rug(&(rug::Float::with_val(128, t) / &log2x));
        ys.push(y);
        if y.round_ties_away() != y {
            ys.push(y.round_ties_away());
        }
    }
    ys
}

/// Rounds `value` to the nearest `F32`.
fn round_rug(value: &rug::Float) -> F32 {
    let (value, round_cmp) = rug::Float::with_val_round(PREC, value, rug::float::Round::Nearest);
    from_rug(value, round_cmp, rug::float::Round::Nearest).0
}

/// Returns pairs `(x, y)` with `x = m^d * 2^(d * k)` and `y = n / d`, with odd
/// `m`, so `x^y = m^n * 2^(n * k)` is exact when `m^n` fits in 24 bits, and a
/// tie between two consecutive values when it has 25 bits.
///
/// `k` is chosen so that the results are spread over the whole range, including
/// subnormals (where they are not exact anymore).
fn exact_results() -> Vec<(F32, F32)> {
    let mut cases = Vec::new();
    for d in [1, 2, 4] {
        for n in 1..=25 {
            if n == d || (d != 1 && n % 2 == 0) {
                // `y` is one, or `n / d` is not in lowest terms (that `y` is
                // tested with a smaller `d`)
                continue;
            }

            // `m^d` must fit in 24 bits, so `x` is exact, and `m^n` in 25 bits
            let max_m = max_root(d, 24).min(max_root(n, 25));
            if max_m < 3 {
                continue;
            }
            let ms = [3, 5, 7, max_m - 2, max_m]
                .into_iter()
                .chain(
                    crate::utils::spread(32, 32, (d * 100 + n).into()).map(|s| 3 + s % (max_m - 2)),
                )
                .map(|m| m | 1)
                .filter(|&m| m <= max_m);

            for m in ms {
                let md = m.pow(d);
                let mn = m.pow(n);
                let x_bits = 64 - md.leading_zeros() as i32;
                let r_bits = 64 - mn.leading_zeros() as i32;
                let (d, n) = (d as i32, n as i32);

                // `x` is normal and the result is not smaller than half the
                // smallest subnormal or larger than the largest finite value
                let k_lo = ((-126 - x_bits + 1) as f64 / d as f64)
                    .max((-150 - r_bits + 1) as f64 / n as f64)
                    .ceil() as i32;
                let k_hi = ((127 - x_bits + 1) as f64 / d as f64)
                    .min((127 - r_bits + 1) as f64 / n as f64)
                    .floor() as i32;
                if k_lo > k_hi {
                    continue;
                }

                let ks = [k_lo, k_lo + 1, 0, k_hi - 1, k_hi].into_iter().chain(
                    crate::utils::spread(32, 16, m)
                        .map(move |s| k_lo + (s % (k_hi - k_lo + 1) as u64) as i32),
                );
                for k in ks.filter(|k| (k_lo..=k_hi).contains(k)) {
                    let x = F32::from_uint(md.into()) * exp2i(d * k);
                    let y = F32::from_int(n.into()) / F32::from_int(d.into());
                    cases.push((x, y));
                }
            }
        }
    }

    // Powers of two with results at the limits of the range
    for t in [127, 128, -126, -127, -149, -150, -151] {
        for k in (-149..=127).filter(|&k| k != 0 && t % k == 0) {
            cases.push((exp2i(k), F32::from_int((t / k).into())));
        }
    }

    cases
}

/// Returns the largest `m` such that `m^n < 2^bits`.
fn max_root(n: u32, bits: u32) -> u64 {
    let mut m = 2f64.powf(f64::from(bits) / f64::from(n)) as u64 + 1;
    while m.checked_pow(n).is_none_or(|p| p >= 1 << bits) {
        m -= 1;
    }
    m
}

/// Returns `2^e`, for `e` from -149 to 127.
fn exp2i(e: i32) -> F32 {
    assert!(matches!(e, -149..=127));
    F32::from_uint(1).scalbn(e)
}

/// Converts the integer `y` to `i32`, saturating at the limits of the range.
fn to_i32_saturating(y: F32) -> i32 {
    match y.to_int(32) {
        Some(n) => n as i32,
        None if y < F32::ZERO => i32::MIN,
        None => i32::MAX,
    }
}
