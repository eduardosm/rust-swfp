use swfp::{F32, Float as _, math::Gamma as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_gamma() {
    crate::generic::test_gamma_special::<F32>();

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
    crate::generic::test_ln_gamma_special::<F32>();

    test_with(|x| {
        let (actual, actual_sign) = x.ln_gamma();

        let mut expected = to_rug(x);
        let (ord, round_cmp) = expected.ln_abs_gamma_round(rug::float::Round::Nearest);
        let expected_sign = if x < F32::ZERO && x.trunc() == x {
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

fn test_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    for e in -126..=127 {
        f(mk_normal(0, e, false));
        f(mk_normal(0, e, true));
        f(mk_normal((1 << 23) - 1, e, false));
        f(mk_normal((1 << 23) - 1, e, true));
        let n = if e <= 6 { 10_000 } else { 2000 };
        for _ in 0..n {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
            f(mk_normal(m, e, true));
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    for x in 1..=20_000 {
        let x = F32::from_uint(x) / F32::from_uint(100);
        f(x);
        f(-x);
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/gamma.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/ln_gamma.txt") {
        f(x);
    }
}
