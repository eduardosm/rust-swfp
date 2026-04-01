use swfp::{F16, math::Log as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

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
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
    }
}

fn test_log_1p_with(mut f: impl FnMut(F16)) {
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    // 1 < x < 0
    for e in -14..=-1 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, true));
        }
    }
}
