use swfp::{F64, Float as _, math::InvTrigonometric as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_asin() {
    crate::generic::test_asin_special::<F64>();

    test_asin_with(|x| {
        let actual = x.asin();
        if actual.is_nan() {
            assert_total_eq!((-x).asin(), actual);
        } else {
            assert_total_eq!((-x).asin(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_acos() {
    crate::generic::test_acos_special::<F64>();

    test_acos_with(|x| {
        let actual = x.acos();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atan() {
    crate::generic::test_atan_special::<F64>();

    test_atan_with(|x| {
        let actual = x.atan();
        if actual.is_nan() {
            assert_total_eq!((-x).atan(), actual);
        } else {
            assert_total_eq!((-x).atan(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atan2() {
    crate::generic::test_atan2_special::<F64>();

    test_atan2_with(|y, x| {
        let actual = y.atan2(x);
        assert_total_eq!((-y).atan2(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_round(&to_rug(x), rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_asind() {
    crate::generic::test_asind_special::<F64>();

    test_asin_with(|x| {
        let actual = x.asind();
        if actual.is_nan() {
            assert_total_eq!((-x).asind(), actual);
        } else {
            assert_total_eq!((-x).asind(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_u_round(360, rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_acosd() {
    crate::generic::test_acosd_special::<F64>();

    test_acos_with(|x| {
        let actual = x.acosd();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_u_round(360, rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atand() {
    crate::generic::test_atand_special::<F64>();

    test_atan_with(|x| {
        let actual = x.atand();
        if actual.is_nan() {
            assert_total_eq!((-x).atand(), actual);
        } else {
            assert_total_eq!((-x).atand(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_u_round(360, rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atan2d() {
    crate::generic::test_atan2d_special::<F64>();

    test_atan2_with(|y, x| {
        let actual = y.atan2d(x);
        assert_total_eq!((-y).atan2d(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_u_round(&to_rug(x), 360, rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_asinpi() {
    crate::generic::test_asinpi_special::<F64>();

    test_asin_with(|x| {
        let actual = x.asinpi();
        if actual.is_nan() {
            assert_total_eq!((-x).asinpi(), actual);
        } else {
            assert_total_eq!((-x).asinpi(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_pi_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_acospi() {
    crate::generic::test_acospi_special::<F64>();

    test_acos_with(|x| {
        let actual = x.acospi();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_pi_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atanpi() {
    crate::generic::test_atanpi_special::<F64>();

    test_atan_with(|x| {
        let actual = x.atanpi();
        if actual.is_nan() {
            assert_total_eq!((-x).atanpi(), actual);
        } else {
            assert_total_eq!((-x).atanpi(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_pi_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_atan2pi() {
    crate::generic::test_atan2pi_special::<F64>();

    test_atan2_with(|y, x| {
        let actual = y.atan2pi(x);
        assert_total_eq!((-y).atan2pi(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_pi_round(&to_rug(x), rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

fn test_asin_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers below one
    for x in values::binades(-1022..=-1, 5000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
    }

    f(F64::from_int(1));

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/asin.wc") {
        if x.is_nan() || x.abs() > F64::from_uint(1) {
            continue;
        }
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/asinpi.wc") {
        if x.is_nan() || x.abs() > F64::from_uint(1) {
            continue;
        }
        f(x);
    }
}

fn test_acos_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers with magnitudes below
    // one
    for x in values::binades(-1022..=-1, 5000) {
        f(x);
        f(-x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
        f(-x);
    }

    f(F64::from_int(1));
    f(F64::from_int(-1));

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/acos.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/acospi.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_atan_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 5000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
    }

    f(F64::from_int(1));

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atan.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atanpi.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_atan2_with(mut f: impl FnMut(F64, F64)) {
    // Toggling the sign of `y` is done by the callers.

    // Test the ratios `|y / x|` between 2^-64 and 2^64, at many scales,
    // including subnormals
    for (y, x) in values::binade_pairs(values::exp_pairs(-64..=64, 32), 4) {
        f(y, x);
        f(y, -x);
    }

    // Test more values with ratios between 2^-8 and 2^8 at a few scales
    // (since only the ratio matters away from the limits of the range)
    for (y, x) in values::binade_pairs(values::exp_pairs(-8..=8, 256), 20_000) {
        f(y, x);
        f(y, -x);
    }

    // Test larger ratios at fewer scales
    let large_diffs = (65..=2097).step_by(18).flat_map(|d| [d, -d]);
    for (y, x) in values::binade_pairs(values::exp_pairs(large_diffs, 61), 1) {
        f(y, x);
        f(y, -x);
    }

    // Test more pairs of subnormal numbers, at every combination of their
    // binades
    let subnormal_exps = (-1074..=-1023).flat_map(|ey| (-1074..=-1023).map(move |ex| (ey, ex)));
    for (y, x) in values::binade_pairs(subnormal_exps, 200) {
        f(y, x);
        f(y, -x);
    }

    let specials = [F64::from_int(1), F64::ZERO, F64::INFINITY];
    for &y in specials.iter() {
        for &x in specials.iter() {
            f(y, x);
            f(y, -x);
        }
    }

    let one = F64::from_int(1);

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atan.wc") {
        if x.is_nan() {
            continue;
        }
        f(x, one);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atanpi.wc") {
        if x.is_nan() {
            continue;
        }
        f(x, one);
    }

    for [y, x] in crate::data::read_data_file::<[F64; 2]>("core-math/binary64/atan2.wc") {
        if y.is_nan() || x.is_nan() {
            continue;
        }
        f(y, x);
    }
    for [y, x] in crate::data::read_data_file::<[F64; 2]>("core-math/binary64/atan2pi.wc") {
        if y.is_nan() || x.is_nan() {
            continue;
        }
        f(y, x);
    }
}
