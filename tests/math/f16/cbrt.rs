use swfp::{F16, math::Cbrt as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_cbrt() {
    crate::generic::test_cbrt_special::<F16>();

    test_with(|x| {
        let actual = x.cbrt();
        assert_total_eq!((-x).cbrt(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.cbrt_round(rug::float::Round::Nearest);
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
