use swfp::{F16, Float as _, math::InvHyperbolic as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_asinh() {
    crate::generic::test_asinh_special::<F16>();

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
    crate::generic::test_acosh_special::<F16>();

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
    crate::generic::test_atanh_special::<F16>();

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

fn test_asinh_with(mut f: impl FnMut(F16)) {
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

fn test_acosh_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all normal numbers above one
    for x in values::binades_full(0..=15) {
        f(x);
    }
}

fn test_atanh_with(mut f: impl FnMut(F16)) {
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
}
