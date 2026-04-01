use swfp::{F32, Float as _, math::Exp as _, math::Trigonometric as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_sin_cos() {
    crate::generic::test_sin_cos_special::<F32>();

    test_with(|x| {
        let actual_sin = x.sin();
        let actual_cos = x.cos();
        assert_total_eq!((-x).sin(), -actual_sin);
        assert_total_eq!((-x).cos(), actual_cos);
        assert_total_eq!(x.sin_cos(), (actual_sin, actual_cos));
        assert_total_eq!((-x).sin_cos(), (-actual_sin, actual_cos));

        let mut expected_sin = to_rug(x);
        let round_cmp = expected_sin.sin_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_sin,
            None,
            expected_sin,
            round_cmp,
            rug::float::Round::Nearest,
        );

        let mut expected_cos = to_rug(x);
        let round_cmp = expected_cos.cos_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_cos,
            None,
            expected_cos,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_tan() {
    crate::generic::test_tan_special::<F32>();

    test_with(|x| {
        let actual = x.tan();
        assert_total_eq!((-x).tan(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.tan_round(rug::float::Round::Nearest);
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
fn test_sind_cosd() {
    crate::generic::test_sind_cosd_special::<F32>();

    test_with(|x| {
        let actual_sin = x.sind();
        let actual_cos = x.cosd();
        assert_total_eq!((-x).sind(), -actual_sin);
        assert_total_eq!((-x).cosd(), actual_cos);
        assert_total_eq!(x.sind_cosd(), (actual_sin, actual_cos));
        assert_total_eq!((-x).sind_cosd(), (-actual_sin, actual_cos));

        let mut expected_sin = to_rug(x);
        let round_cmp = expected_sin.sin_u_round(360, rug::float::Round::Nearest);
        check_exact(
            x,
            actual_sin,
            None,
            expected_sin,
            round_cmp,
            rug::float::Round::Nearest,
        );

        let mut expected_cos = to_rug(x);
        let round_cmp = expected_cos.cos_u_round(360, rug::float::Round::Nearest);
        check_exact(
            x,
            actual_cos,
            None,
            expected_cos,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_tand() {
    crate::generic::test_tand_special::<F32>();

    test_with(|x| {
        let actual = x.tand();
        assert_total_eq!((-x).tand(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.tan_u_round(360, rug::float::Round::Nearest);
        if expected.is_zero() {
            rug::Assign::assign(
                &mut expected,
                if x.is_sign_positive() {
                    rug::float::Special::Zero
                } else {
                    rug::float::Special::NegZero
                },
            );
        } else if expected.is_infinite() {
            rug::Assign::assign(
                &mut expected,
                if x.is_sign_positive() {
                    rug::float::Special::Infinity
                } else {
                    rug::float::Special::NegInfinity
                },
            );
        }
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
fn test_sinpi_cospi() {
    crate::generic::test_sinpi_cospi_special::<F32>();

    test_with(|x| {
        let actual_sin = x.sinpi();
        let actual_cos = x.cospi();
        assert_total_eq!((-x).sinpi(), -actual_sin);
        assert_total_eq!((-x).cospi(), actual_cos);
        assert_total_eq!(x.sinpi_cospi(), (actual_sin, actual_cos));
        assert_total_eq!((-x).sinpi_cospi(), (-actual_sin, actual_cos));

        let mut expected_sin = to_rug(x);
        let round_cmp = expected_sin.sin_pi_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_sin,
            None,
            expected_sin,
            round_cmp,
            rug::float::Round::Nearest,
        );

        let mut expected_cos = to_rug(x);
        let round_cmp = expected_cos.cos_pi_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_cos,
            None,
            expected_cos,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_tanpi() {
    crate::generic::test_tanpi_special::<F32>();

    test_with(|x| {
        let actual = x.tanpi();
        assert_total_eq!((-x).tanpi(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.tan_pi_round(rug::float::Round::Nearest);
        if expected.is_zero() {
            rug::Assign::assign(
                &mut expected,
                if x.is_sign_positive() {
                    rug::float::Special::Zero
                } else {
                    rug::float::Special::NegZero
                },
            );
        } else if expected.is_infinite() {
            rug::Assign::assign(
                &mut expected,
                if x.is_sign_positive() {
                    rug::float::Special::Infinity
                } else {
                    rug::float::Special::NegInfinity
                },
            );
        }
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
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, false));
        }
        for _ in 0..5000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // Problematic value in
    // "ARGUMENT REDUCTION FOR HUGE ARGUMENTS: Good to the Last Bit"
    f(F32::from_host(1.0e22));

    let one = F32::from_uint(1);
    let frac_pi_8 = F32::from_host(std::f32::consts::FRAC_PI_8);
    for f1 in 1..=100 {
        let f1 = F32::from_uint(f1);
        for f2 in 0..=20 {
            let f2 = F32::from_uint(1 << f2);

            f(frac_pi_8 * f1 * f2);
            for f3 in 1..=20 {
                let f3 = F32::from_int(-f3);
                f(frac_pi_8 * f1 * f2 * (one + f3.exp()));
                f(frac_pi_8 * f1 * f2 * (one - f3.exp()));
                f(frac_pi_8 * f1 * f2 * (one + f3.exp2()));
                f(frac_pi_8 * f1 * f2 * (one - f3.exp2()));
                f(frac_pi_8 * f1 * f2 * (one + f3.exp10()));
                f(frac_pi_8 * f1 * f2 * (one - f3.exp10()));
            }
        }
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/sin.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/cos.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/tan.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/sind.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/cosd.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/tand.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/sinpi.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/cospi.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/tanpi.txt") {
        f(x);
    }
}
