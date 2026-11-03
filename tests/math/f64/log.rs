use swfp::{F64, Float as _, math::Log as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_ln() {
    crate::generic::test_ln_special::<F64>();

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
    crate::generic::test_ln_1p_special::<F64>();

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
    crate::generic::test_log2_special::<F64>();

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
    crate::generic::test_log2_1p_special::<F64>();

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
    crate::generic::test_log10_special::<F64>();

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
    crate::generic::test_log10_1p_special::<F64>();

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

fn test_log_with(func: Func, mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades((-1022..=-2).chain(1..=1023), 10_000) {
        f(x);
    }

    // Test more values at the binades around one
    for x in values::binades(-1..=0, 2_000_000) {
        f(x);
    }

    // Test values close to one (`1 + v` and `1 - v` with `v` at each binade
    // below one), whose results are close to zero
    let one = F64::from_uint(1);
    for v in values::binades(-53..=-1, 10_000) {
        f(one + v);
        f(one - v);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    let data_file = match func {
        Func::Ln => "core-math/binary64/log.wc",
        Func::Log2 => "core-math/binary64/log2.wc",
        Func::Log10 => "core-math/binary64/log10.wc",
        Func::Ln1p | Func::Log2_1p | Func::Log10_1p => unreachable!(),
    };
    for x in crate::data::read_data_file::<F64>(data_file) {
        f(x);
    }
}

fn test_log_1p_with(func: Func, mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 10_000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(10_000) {
        f(x);
        f(-x);
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // -1 < x < 0
    for x in values::binades(-1022..=-1, 10_000) {
        f(-x);
    }

    // Test data from core-math
    let data_file = match func {
        Func::Ln1p => "core-math/binary64/log1p.wc",
        Func::Log2_1p => "core-math/binary64/log2p1.wc",
        Func::Log10_1p => "core-math/binary64/log10p1.wc",
        Func::Ln | Func::Log2 | Func::Log10 => unreachable!(),
    };
    for x in crate::data::read_data_file::<F64>(data_file) {
        f(x);
    }
}
