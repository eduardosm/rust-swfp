use swfp::{F64, Float as _, math::Gamma as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_gamma() {
    crate::generic::test_gamma_special::<F64>();

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
    crate::generic::test_ln_gamma_special::<F64>();

    test_with(|x| {
        let (actual, actual_sign) = x.ln_gamma();

        let mut expected = to_rug(x);
        let (ord, round_cmp) = expected.ln_abs_gamma_round(rug::float::Round::Nearest);
        let expected_sign = if x < F64::ZERO && x.trunc() == x {
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

fn test_with(mut f: impl FnMut(F64)) {
    let mut rng = crate::create_prng();

    for e in -1022..=1023 {
        f(mk_normal(0, e, false));
        f(mk_normal(0, e, true));
        f(mk_normal((1 << 23) - 1, e, false));
        f(mk_normal((1 << 23) - 1, e, true));
        for _ in 0..2000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
            f(mk_normal(m, e, true));
        }
    }

    for &m in super::SUBNORMAL_MANTISSAS.iter() {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }

    for x in 1..=20_000 {
        let x = F64::from_uint(x) / F64::from_uint(100);
        f(x);
        f(-x);
    }

    // Roots of ln_gamma (i.e., where abs(gamma(x)) = 1 and ln_gamma(x) = 0)
    let roots = [
        F64::from_int(1),
        F64::from_int(2),
        F64::from_host(-2.4570247382208006),
        F64::from_host(-2.7476826467274127),
        F64::from_host(-3.14358088834998),
        F64::from_host(-3.955294284858598),
        F64::from_host(-4.039361839740537),
        F64::from_host(-4.991544640560048),
        F64::from_host(-5.0082181683225935),
        F64::from_host(-5.998607480080875),
        F64::from_host(-6.001385294453155),
        F64::from_host(-6.999801507890638),
        F64::from_host(-7.000198333407325),
        F64::from_host(-7.999975197095821),
        F64::from_host(-8.000024800270682),
        F64::from_host(-8.999997244250977),
        F64::from_host(-9.000002755714823),
        F64::from_host(-9.99999972442663),
        F64::from_host(-10.000000275573013),
        F64::from_host(-10.99999997494789),
        F64::from_host(-11.000000025052106),
        F64::from_host(-11.999999997912324),
        F64::from_host(-12.000000002087676),
        F64::from_host(-12.99999999983941),
        F64::from_host(-13.00000000016059),
        F64::from_host(-13.99999999998853),
        F64::from_host(-14.00000000001147),
        F64::from_host(-14.999999999999236),
        F64::from_host(-15.000000000000764),
        F64::from_host(-15.999999999999952),
        F64::from_host(-16.000000000000046),
        F64::from_host(-16.999999999999996),
        F64::from_host(-17.000000000000004),
    ];

    for root in roots {
        f(root);

        for bump in 1..=10_000 {
            let x = F64::from_bits(root.to_bits() + bump);
            f(x);

            let x = F64::from_bits(root.to_bits() - bump);
            f(x);
        }
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/tgamma.wc") {
        f(x);
    }
    for x in crate::data::read_data_file::<F64>("core-math/binary64/lgamma.wc") {
        f(x);
    }
}
