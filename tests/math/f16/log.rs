use swfp::{F16, math::Log as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_ln() {
    crate::generic::test_ln_special::<F16>();

    test_log_with(|x| {
        let actual = x.ln();

        let mut expected = to_rug(x);
        let round_cmp = expected.ln_round(rug::float::Round::Nearest);
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
fn test_ln_1p() {
    crate::generic::test_ln_1p_special::<F16>();

    test_log_1p_with(|x| {
        let actual = x.ln_1p();

        let mut expected = to_rug(x);
        let round_cmp = expected.ln_1p_round(rug::float::Round::Nearest);
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
fn test_log2() {
    crate::generic::test_log2_special::<F16>();

    test_log_with(|x| {
        let actual = x.log2();

        let mut expected = to_rug(x);
        let round_cmp = expected.log2_round(rug::float::Round::Nearest);
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
fn test_log2_1p() {
    crate::generic::test_log2_1p_special::<F16>();

    test_log_1p_with(|x| {
        let actual = x.log2_1p();

        let mut expected = to_rug(x);
        let round_cmp = expected.log2_1p_round(rug::float::Round::Nearest);
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
fn test_log10() {
    crate::generic::test_log10_special::<F16>();

    test_log_with(|x| {
        let actual = x.log10();

        let mut expected = to_rug(x);
        let round_cmp = expected.log10_round(rug::float::Round::Nearest);
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
fn test_log10_1p() {
    crate::generic::test_log10_1p_special::<F16>();

    test_log_1p_with(|x| {
        let actual = x.log10_1p();

        let mut expected = to_rug(x);
        let round_cmp = expected.log10_1p_round(rug::float::Round::Nearest);
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

fn test_log_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
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

fn test_log_1p_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all normal numbers
    for x in values::binades_full(-14..=15) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }

    // -1 < x < 0
    for x in values::binades_full(-14..=-1) {
        f(-x);
    }
}
