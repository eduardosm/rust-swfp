use swfp::{F64, Float as _, math::Hyperbolic as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_sinh_cosh() {
    crate::generic::test_sinh_cosh_special::<F64>();

    test_with(|x| {
        let actual_sinh = x.sinh();
        let actual_cosh = x.cosh();
        assert_total_eq!((-x).cosh(), actual_cosh);
        assert_total_eq!(x.sinh_cosh(), (actual_sinh, actual_cosh));
        if actual_sinh.is_nan() {
            assert_total_eq!((-x).sinh(), actual_sinh);
            assert_total_eq!((-x).sinh_cosh(), (actual_sinh, actual_cosh));
        } else {
            assert_total_eq!((-x).sinh(), -actual_sinh);
            assert_total_eq!((-x).sinh_cosh(), (-actual_sinh, actual_cosh));
        }

        let mut expected_sinh = to_rug(x);
        let round_cmp = expected_sinh.sinh_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_sinh,
            None,
            expected_sinh,
            round_cmp,
            rug::float::Round::Nearest,
        );

        let mut expected_cosh = to_rug(x);
        let round_cmp = expected_cosh.cosh_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_cosh,
            None,
            expected_cosh,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_tanh() {
    crate::generic::test_tanh_special::<F64>();

    test_with(|x| {
        let actual = x.tanh();
        if actual.is_nan() {
            assert_total_eq!((-x).tanh(), actual);
        } else {
            assert_total_eq!((-x).tanh(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.tanh_round(rug::float::Round::Nearest);
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

fn test_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 5_000) {
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
    for x in crate::data::read_data_file::<F64>("core-math/binary64/sinh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/cosh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/tanh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}
