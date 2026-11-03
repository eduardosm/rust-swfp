use swfp::{F64, Float as _, math::Exp as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_exp() {
    crate::generic::test_exp_special::<F64>();

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
    crate::generic::test_exp_m1_special::<F64>();

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
    crate::generic::test_exp2_special::<F64>();

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
    crate::generic::test_exp2_m1_special::<F64>();

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
    crate::generic::test_exp10_special::<F64>();

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
    crate::generic::test_exp10_m1_special::<F64>();

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

fn test_with(func: Func, mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 10_000) {
        f(x);
        f(-x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
        f(-x);
    }

    for x in -2000..=2000 {
        f(F64::from_int(x));
    }

    // Test data from core-math
    let data_file = match func {
        Func::Exp => "core-math/binary64/exp.wc",
        Func::ExpM1 => "core-math/binary64/expm1.wc",
        Func::Exp2 => "core-math/binary64/exp2.wc",
        Func::Exp2M1 => "core-math/binary64/exp2m1.wc",
        Func::Exp10 => "core-math/binary64/exp10.wc",
        Func::Exp10M1 => "core-math/binary64/exp10m1.wc",
    };
    for x in crate::data::read_data_file::<F64>(data_file) {
        f(x);
    }
}
