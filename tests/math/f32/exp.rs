use swfp::{F32, Float as _, math::Exp as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_exp() {
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
    crate::generic::test_exp2_m1_special::<F32>();

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
    crate::generic::test_exp10_m1_special::<F32>();

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

fn test_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    for e in -126..=127 {
        f(mk_normal(0, e, false));
        f(mk_normal(0, e, true));
        f(mk_normal((1 << 23) - 1, e, false));
        f(mk_normal((1 << 23) - 1, e, true));
        for _ in 0..20_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
            f(mk_normal(m, e, true));
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    for x in -200..=200 {
        f(F32::from_int(x));
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/exp.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/exp_m1.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/exp2.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/exp2_m1.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/exp10.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/exp10_m1.txt") {
        f(x);
    }
}
