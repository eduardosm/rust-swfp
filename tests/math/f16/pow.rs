use swfp::{F16, Float as _, math::Pow as _};

use super::{PREC, check_exact, from_rug, mk_normal, to_rug, to_rug_prec, values};

#[test]
fn test_pow() {
    crate::generic::test_pow_special::<F16>();

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
fn check_neg_base(x: F16, y: F16) {
    let pos = x.abs().pow(y);
    let neg = (-x.abs()).pow(y);
    if y.is_infinite() || y % F16::from_uint(2) == F16::ZERO {
        assert_total_eq!(neg, pos);
    } else if y.trunc() == y {
        if pos.is_nan() {
            assert_total_eq!(neg, pos);
        } else {
            assert_total_eq!(neg, -pos);
        }
    } else if x.is_finite() && x != F16::ZERO {
        assert!(neg.is_nan(), "pow({:?}, {y:?}) = {neg:?}", -x.abs());
    } else {
        assert_total_eq!(neg, pos);
    }
}

#[test]
fn test_powi() {
    crate::generic::test_powi_special::<F16>();

    test_powi_with(|x, n| {
        let actual = x.powi(n);
        if let (y, swfp::FpStatus::OK) = F16::from_int_ex(n.into(), swfp::Round::NearestTiesToEven)
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

fn test_pow_with(mut f: impl FnMut(F16, F16)) {
    // Negative bases are tested by `check_neg_base`.

    // Test special values
    for x in values::specials().chain([F16::from_uint(1)]) {
        for y in values::specials().chain(special_exponents()) {
            f(x, y);
            f(x, -y);
        }
    }

    // Test halves and small integers as exponents with all the bases
    for x in test_bases() {
        for y in [half(1), half(2), half(3), half(4), half(6)] {
            f(x, y);
            f(x, -y);
        }
    }

    // Test the largest exponents with bases close to one
    for x in values::binades_full(-1..=0) {
        for y in special_exponents() {
            f(x, y);
            f(x, -y);
        }
    }

    // Test results spread across the whole range, with all the bases
    for x in test_bases() {
        for y in result_targets(x) {
            f(x, y);
        }
    }

    // Test exact results and ties
    for (x, y) in exact_results() {
        f(x, y);
    }

    // Generated test data
    for [x, y] in crate::data::read_data_file::<[F16; 2]>("f16/pow.txt") {
        f(x, y);
    }
}

fn test_powi_with(mut f: impl FnMut(F16, i32)) {
    // Negative bases are tested by the caller.

    // Test special values
    for x in values::specials().chain([F16::from_uint(1)]) {
        for n in [0, 1, 2, 3, i32::MAX - 1, i32::MAX] {
            f(x, n);
            f(x, -n);
        }
        f(x, i32::MIN);
    }

    // Test small and larger exponents with all the bases
    for x in test_bases() {
        for n in 1..=128 {
            f(x, n);
            f(x, -n);
        }
    }

    // Test results spread across the whole range, with all the bases (where
    // the exponents can saturate)
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

    // Generated test data
    for [x, y] in crate::data::read_data_file::<[F16; 2]>("f16/pow.txt") {
        let (Some(n), swfp::FpStatus::OK) = y.to_int_ex(32, swfp::Round::TowardZero) else {
            continue;
        };
        f(x, n as i32);
    }
}

/// Returns `n / 2`.
fn half(n: u128) -> F16 {
    F16::from_uint(n) / F16::from_uint(2)
}

/// Returns positive exponents that need special handling: halves and small
/// integers, and the largest integers and non-integers, where the parity
/// matters for negative bases.
fn special_exponents() -> impl Iterator<Item = F16> {
    [
        half(1),
        half(2),
        half(3),
        half(4),
        half(6),
        // Largest non-integer
        half((1 << 11) - 1),
        // Largest odd integers
        F16::from_uint((1 << 10) - 1),
        F16::from_uint((1 << 10) + 1),
        F16::from_uint((1 << 11) - 1),
        // Even integers, all the larger ones are even
        F16::from_uint(1 << 10),
        F16::from_uint(1 << 11),
        mk_normal((1 << 10) - 1, 15, false),
    ]
    .into_iter()
}

/// Returns all the positive finite non-zero bases.
fn test_bases() -> impl Iterator<Item = F16> {
    values::binades_full(-14..=15).chain(values::subnormals_full())
}

/// Returns exponents `y` such that `x^y` is spread across the whole range.
///
/// `log2(x^y) = y * log2(x)` takes evenly spread values in `[-30, 20]`
/// (beyond the overflow and underflow limits), tiny values (results close to
/// one), and the values at the limits of the range: the overflow threshold,
/// the smallest normal and subnormal values, and half the smallest subnormal
/// (which is rounded to zero). Each non-integer exponent is also rounded to an
/// integer, which tests negative bases too.
fn result_targets(x: F16) -> Vec<F16> {
    let log2x = to_rug_prec(x, 128).log2();
    if log2x.is_zero() {
        return Vec::new();
    }

    let seed = u64::from(x.to_bits());
    let unit = |s: u64| s as f64 / (1u64 << 32) as f64;
    let spread = crate::utils::spread(32, 8, seed).map(|s| -30.0 + 50.0 * unit(s));
    let tiny = crate::utils::spread(32, 2, !seed)
        .zip([1.0, -1.0])
        .map(|(s, sign)| sign * (-1.0 - 15.0 * unit(s)).exp2());
    let limits = [16.0, -14.0, -24.0, -25.0];

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

/// Rounds `value` to the nearest `F16`.
fn round_rug(value: &rug::Float) -> F16 {
    let (value, round_cmp) = rug::Float::with_val_round(PREC, value, rug::float::Round::Nearest);
    from_rug(value, round_cmp, rug::float::Round::Nearest).0
}

/// Returns pairs `(x, y)` with `x = m^d * 2^(d * k)` and `y = n / d`, with odd
/// `m`, so `x^y = m^n * 2^(n * k)` is exact when `m^n` fits in 11 bits, and a
/// tie between two consecutive values when it has 12 bits.
///
/// `k` is chosen so that the results are spread over the whole range, including
/// subnormals (where they are not exact anymore).
fn exact_results() -> Vec<(F16, F16)> {
    let mut cases = Vec::new();
    for d in [1, 2, 4] {
        for n in 1..=12 {
            if n == d || (d != 1 && n % 2 == 0) {
                // `y` is one, or `n / d` is not in lowest terms (that `y` is
                // tested with a smaller `d`)
                continue;
            }

            // `m^d` must fit in 11 bits, so `x` is exact, and `m^n` in 12 bits
            let max_m = max_root(d, 11).min(max_root(n, 12));
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
                let k_lo = ((-14 - x_bits + 1) as f64 / d as f64)
                    .max((-25 - r_bits + 1) as f64 / n as f64)
                    .ceil() as i32;
                let k_hi = ((15 - x_bits + 1) as f64 / d as f64)
                    .min((15 - r_bits + 1) as f64 / n as f64)
                    .floor() as i32;
                if k_lo > k_hi {
                    continue;
                }

                let ks = [k_lo, k_lo + 1, 0, k_hi - 1, k_hi].into_iter().chain(
                    crate::utils::spread(32, 16, m)
                        .map(move |s| k_lo + (s % (k_hi - k_lo + 1) as u64) as i32),
                );
                for k in ks.filter(|k| (k_lo..=k_hi).contains(k)) {
                    let x = F16::from_uint(md.into()) * exp2i(d * k);
                    let y = F16::from_int(n.into()) / F16::from_int(d.into());
                    cases.push((x, y));
                }
            }
        }
    }

    // Powers of two with results at the limits of the range
    for t in [15, 16, -14, -15, -24, -25, -26] {
        for k in (-24..=15).filter(|&k| k != 0 && t % k == 0) {
            cases.push((exp2i(k), F16::from_int((t / k).into())));
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

/// Returns `2^e`, for `e` from -24 to 15.
fn exp2i(e: i32) -> F16 {
    assert!(matches!(e, -24..=15));
    F16::from_uint(1).scalbn(e)
}

/// Converts the integer `y` to `i32`, saturating at the limits of the range.
fn to_i32_saturating(y: F16) -> i32 {
    match y.to_int(32) {
        Some(n) => n as i32,
        None if y < F16::ZERO => i32::MIN,
        None => i32::MAX,
    }
}
