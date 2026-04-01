use swfp::{F64, Float as _, math::Cbrt as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_cbrt() {
    crate::generic::test_cbrt_special::<F64>();

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

fn test_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in 0..=2 {
        for _ in 0..2_000_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for e in -1022..=1023 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 52) - 1, e, false));
        for _ in 0..2000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
    }

    for x in 1..=500_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/cbrt.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}
