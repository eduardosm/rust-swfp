use swfp::{F64, Float as _, math::InvHyperbolic as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_asinh() {
    crate::generic::test_asinh_special::<F64>();

    test_asinh_with(|x| {
        let actual = x.asinh();
        assert_total_eq!((-x).asinh(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.asinh_round(rug::float::Round::Nearest);
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
fn test_acosh() {
    crate::generic::test_acosh_special::<F64>();

    test_acosh_with(|x| {
        let actual = x.acosh();

        let mut expected = to_rug(x);
        let round_cmp = expected.acosh_round(rug::float::Round::Nearest);
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
fn test_atanh() {
    crate::generic::test_atanh_special::<F64>();

    test_atanh_with(|x| {
        let actual = x.atanh();
        assert_total_eq!((-x).atanh(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.atanh_round(rug::float::Round::Nearest);
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

fn test_asinh_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in -1022..=1023 {
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, false));
        }
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/asinh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_acosh_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in 0..=1023 {
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, false));
        }
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for x in 1..=200_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/acosh.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}

fn test_atanh_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in -1022..=-1 {
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, false));
        }
        for _ in 0..10_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/atanh.wc") {
        if x.is_nan() || x.abs() > F64::from_uint(1) {
            continue;
        }
        f(x);
    }
}
