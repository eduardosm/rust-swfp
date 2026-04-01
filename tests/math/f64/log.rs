use swfp::{F64, Float as _, math::Log as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_ln() {
    crate::generic::test_ln_special::<F64>();

    test_log_with(|x| {
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

    test_log_1p_with(|x| {
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

    test_log_with(|x| {
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

    test_log_1p_with(|x| {
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

    test_log_with(|x| {
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

    test_log_1p_with(|x| {
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

fn test_log_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in -1022..=1023 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 52) - 1, e, false));
        let n = if matches!(e, -1 | 0) {
            2_000_000
        } else {
            10_000
        };
        for _ in 0..n {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for i in 1..52 {
        let max_mant = (1 << 52) - 1;

        f(mk_normal(1 << i, 0, false));
        f(mk_normal(max_mant ^ (1 << i), -1, false));
        f(mk_normal(max_mant ^ ((1 << (i + 1)) - 1), -1, false));
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log.wc") {
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log2.wc") {
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log10.wc") {
        f(x);
    }
}

fn test_log_1p_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in -1022..=1023 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 52) - 1, e, false));
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // 1 < x < 0
    for e in -1022..=-1 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 52) - 1, e, false));
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, true));
        }
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log1p.wc") {
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log2p1.wc") {
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/log10p1.wc") {
        f(x);
    }
}
