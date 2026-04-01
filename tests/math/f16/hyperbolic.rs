use swfp::{F16, math::Hyperbolic as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_sinh_cosh() {
    crate::generic::test_sinh_cosh_special::<F16>();

    test_with(|x| {
        let actual_sinh = x.sinh();
        let actual_cosh = x.cosh();
        assert_total_eq!((-x).sinh(), -actual_sinh);
        assert_total_eq!((-x).cosh(), actual_cosh);
        assert_total_eq!(x.sinh_cosh(), (actual_sinh, actual_cosh));
        assert_total_eq!((-x).sinh_cosh(), (-actual_sinh, actual_cosh));

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
    crate::generic::test_tanh_special::<F16>();

    test_with(|x| {
        let actual = x.tanh();
        assert_total_eq!((-x).tanh(), -actual);

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

fn test_with(mut f: impl FnMut(F16)) {
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
    }
}
