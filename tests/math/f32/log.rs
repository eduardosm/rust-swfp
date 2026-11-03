use swfp::{F32, Float as _, math::Log as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_ln() {
    crate::generic::test_ln_special::<F32>();

    test_log_with(Func::Ln, |x| {
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
    crate::generic::test_ln_1p_special::<F32>();

    test_log_1p_with(Func::Ln1p, |x| {
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
    crate::generic::test_log2_special::<F32>();

    test_log_with(Func::Log2, |x| {
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
    crate::generic::test_log2_1p_special::<F32>();

    test_log_1p_with(Func::Log2_1p, |x| {
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
    crate::generic::test_log10_special::<F32>();

    test_log_with(Func::Log10, |x| {
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
    crate::generic::test_log10_1p_special::<F32>();

    test_log_1p_with(Func::Log10_1p, |x| {
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

/// A function of the `log` family.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Func {
    Ln,
    Ln1p,
    Log2,
    Log2_1p,
    Log10,
    Log10_1p,
}

fn test_log_with(func: Func, mut f: impl FnMut(F32)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all mantissas at the binades around one
    for x in values::binades_full(-1..=0) {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades((-126..=-2).chain(1..=127), 10_000) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // Generated test data
    let data_file = match func {
        Func::Ln => "f32/ln.txt",
        Func::Log2 => "f32/log2.txt",
        Func::Log10 => "f32/log10.txt",
        Func::Ln1p | Func::Log2_1p | Func::Log10_1p => unreachable!(),
    };
    for x in crate::data::read_data_file::<F32>(data_file) {
        f(x);
    }
}

fn test_log_1p_with(func: Func, mut f: impl FnMut(F32)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-126..=127, 10_000) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // -1 < x < 0
    for x in values::binades(-126..=-1, 5000) {
        f(-x);
    }

    // Generated test data
    let data_file = match func {
        Func::Ln1p => "f32/ln_1p.txt",
        Func::Log2_1p => "f32/log2_1p.txt",
        Func::Log10_1p => "f32/log10_1p.txt",
        Func::Ln | Func::Log2 | Func::Log10 => unreachable!(),
    };
    for x in crate::data::read_data_file::<F32>(data_file) {
        f(x);
    }
}
