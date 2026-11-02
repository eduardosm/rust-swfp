use swfp::{F64, Float as _, math::InvHyperbolic as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_asinh() {
    crate::generic::test_asinh_special::<F64>();

    test_asinh_with(|x| {
        let actual = x.asinh();
        if actual.is_nan() {
            assert_total_eq!((-x).asinh(), actual);
        } else {
            assert_total_eq!((-x).asinh(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.asinh_round(rug::float::Round::Nearest);
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
fn test_acosh() {
    crate::generic::test_acosh_special::<F64>();

    test_acosh_with(|x| {
        let actual = x.acosh();

        let mut expected = to_rug(x);
        let round_cmp = expected.acosh_round(rug::float::Round::Nearest);
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
fn test_atanh() {
    crate::generic::test_atanh_special::<F64>();

    test_atanh_with(|x| {
        let actual = x.atanh();
        if actual.is_nan() {
            assert_total_eq!((-x).atanh(), actual);
        } else {
            assert_total_eq!((-x).atanh(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.atanh_round(rug::float::Round::Nearest);
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

fn test_asinh_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 10_000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/asinh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_acosh_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers above one
    for x in values::binades(0..=1023, 10_000) {
        f(x);
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/acosh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_atanh_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers below one
    for x in values::binades(-1022..=-1, 10_000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atanh.wc") {
        if x.is_nan() || x.abs() > F64::from_uint(1) {
            continue;
        }
        f(x);
    }
}
