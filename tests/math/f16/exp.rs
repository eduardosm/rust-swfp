use swfp::{F16, math::Exp as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_exp() {
    crate::generic::test_exp_special::<F16>();

    test_with(|x| {
        let actual = x.exp();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp_round(rug::float::Round::Nearest);
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
fn test_exp_m1() {
    crate::generic::test_exp_m1_special::<F16>();

    test_with(|x| {
        let actual = x.exp_m1();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp_m1_round(rug::float::Round::Nearest);
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
fn test_exp2() {
    crate::generic::test_exp2_special::<F16>();

    test_with(|x| {
        let actual = x.exp2();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp2_round(rug::float::Round::Nearest);
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
fn test_exp2_m1() {
    crate::generic::test_exp2_m1_special::<F16>();

    test_with(|x| {
        let actual = x.exp2_m1();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp2_m1_round(rug::float::Round::Nearest);
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
fn test_exp10() {
    crate::generic::test_exp10_special::<F16>();

    test_with(|x| {
        let actual = x.exp10();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp10_round(rug::float::Round::Nearest);
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
fn test_exp10_m1() {
    crate::generic::test_exp10_m1_special::<F16>();

    test_with(|x| {
        let actual = x.exp10_m1();

        let mut expected = to_rug(x);
        let round_cmp = expected.exp10_m1_round(rug::float::Round::Nearest);
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

fn test_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all normal numbers
    for x in values::binades_full(-14..=15) {
        f(x);
        f(-x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }
}
