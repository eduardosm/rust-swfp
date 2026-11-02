use swfp::{F16, Float as _, math::InvTrigonometric as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_asin() {
    crate::generic::test_asin_special::<F16>();

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
    crate::generic::test_acos_special::<F16>();

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
    crate::generic::test_atan_special::<F16>();

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
    crate::generic::test_atan2_special::<F16>();

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
    crate::generic::test_asind_special::<F16>();

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
    crate::generic::test_acosd_special::<F16>();

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
    crate::generic::test_atand_special::<F16>();

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
    crate::generic::test_atan2d_special::<F16>();

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
    crate::generic::test_asinpi_special::<F16>();

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
    crate::generic::test_acospi_special::<F16>();

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
    crate::generic::test_atanpi_special::<F16>();

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
    crate::generic::test_atan2pi_special::<F16>();

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

fn test_asin_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Exhaustive test of all normal numbers below one
    for x in values::binades_full(-14..=-1) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }

    f(F16::from_int(1));
}

fn test_acos_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all normal numbers with magnitudes below one
    for x in values::binades_full(-14..=-1) {
        f(x);
        f(-x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }

    f(F16::from_int(1));
    f(F16::from_int(-1));
}

fn test_atan_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Exhaustive test of all normal numbers
    for x in values::binades_full(-14..=15) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }
}

fn test_atan2_with(mut f: impl FnMut(F16, F16)) {
    // Toggling the sign of `y` is done by the callers.

    // Test all the ratios `|y / x|` at every scale, including subnormals
    for (y, x) in values::binade_pairs(values::exp_pairs(-39..=39, 1), 1000) {
        f(y, x);
        f(y, -x);
    }

    // Exhaustive test of all pairs of subnormal numbers
    for y in values::subnormals_full() {
        for x in values::subnormals_full() {
            f(y, x);
            f(y, -x);
        }
    }

    let specials = [F16::from_int(1), F16::ZERO, F16::INFINITY];
    for &y in specials.iter() {
        for &x in specials.iter() {
            f(y, x);
            f(y, -x);
        }
    }

    // Exhaustive test of all positive numbers with one as `x`
    let one = F16::from_int(1);
    for y in values::binades_full(-14..=15).chain(values::subnormals_full()) {
        f(y, one);
    }

    // Generated test data
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2.txt") {
        f(y, x);
    }
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2d.txt") {
        f(y, x);
    }
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2pi.txt") {
        f(y, x);
    }
}
