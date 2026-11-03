use swfp::{F32, Float as _, math::Exp as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_exp() {
    crate::generic::test_exp_special::<F32>();

    test_with(Func::Exp, |x| {
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
    crate::generic::test_exp_m1_special::<F32>();

    test_with(Func::ExpM1, |x| {
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
    crate::generic::test_exp2_special::<F32>();

    test_with(Func::Exp2, |x| {
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

    test_with(Func::Exp2M1, |x| {
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
    crate::generic::test_exp10_special::<F32>();

    test_with(Func::Exp10, |x| {
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

    test_with(Func::Exp10M1, |x| {
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

/// A function of the `exp` family.
#[derive(Copy, Clone)]
enum Func {
    Exp,
    ExpM1,
    Exp2,
    Exp2M1,
    Exp10,
    Exp10M1,
}

fn test_with(func: Func, mut f: impl FnMut(F32)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-126..=127, 20_000) {
        f(x);
        f(-x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }

    for x in -200..=200 {
        f(F32::from_int(x));
    }

    // Generated test data
    let data_file = match func {
        Func::Exp => "f32/exp.txt",
        Func::ExpM1 => "f32/exp_m1.txt",
        Func::Exp2 => "f32/exp2.txt",
        Func::Exp2M1 => "f32/exp2_m1.txt",
        Func::Exp10 => "f32/exp10.txt",
        Func::Exp10M1 => "f32/exp10_m1.txt",
    };
    for x in crate::data::read_data_file::<F32>(data_file) {
        f(x);
    }
}
