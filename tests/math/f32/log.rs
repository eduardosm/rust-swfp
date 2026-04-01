use swfp::{F32, Float as _, math::Log as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_ln() {
    crate::generic::test_ln_special::<F32>();

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
    crate::generic::test_ln_1p_special::<F32>();

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
    crate::generic::test_log2_special::<F32>();

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
    crate::generic::test_log2_1p_special::<F32>();

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
    crate::generic::test_log10_special::<F32>();

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
    crate::generic::test_log10_1p_special::<F32>();

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

fn test_log_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    for e in -126..=127 {
        if matches!(e, -1 | 0) {
            for m in 0..(1 << 23) {
                f(mk_normal(m, e, false));
            }
        } else {
            f(mk_normal(0, e, false));
            f(mk_normal((1 << 23) - 1, e, false));
            for _ in 0..10_000 {
                let m = super::gen_mantissa(&mut rng);
                f(mk_normal(m, e, false));
            }
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/ln.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/log2.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/log10.txt") {
        f(x);
    }
}

fn test_log_1p_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    for e in -126..=127 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 23) - 1, e, false));
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // 1 < x < 0
    for e in -126..=-1 {
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, true));
        }
        for _ in 0..5000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, true));
        }
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/ln_1p.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/log2_1p.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/log10_1p.txt") {
        f(x);
    }
}
