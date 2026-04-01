use swfp::{F16, Float as _, math::Gamma as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_gamma() {
    crate::generic::test_gamma_special::<F16>();

    test_with(|x| {
        let actual = x.gamma();

        let mut expected = to_rug(x);
        let round_cmp = expected.gamma_round(rug::float::Round::Nearest);
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
fn test_ln_gamma() {
    crate::generic::test_ln_gamma_special::<F16>();

    test_with(|x| {
        let (actual, actual_sign) = x.ln_gamma();

        let mut expected = to_rug(x);
        let (ord, round_cmp) = expected.ln_abs_gamma_round(rug::float::Round::Nearest);
        let expected_sign = if x < F16::ZERO && x.trunc() == x {
            0
        } else {
            ord as i8
        };
        assert_eq!(actual_sign, expected_sign, "input = {x:?}");
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
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
            f(mk_normal(m, e, true));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }
}
