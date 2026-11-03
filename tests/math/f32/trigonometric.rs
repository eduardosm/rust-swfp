use swfp::{F32, Float as _, math::Exp as _, math::Trigonometric as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_sin_cos() {
    crate::generic::test_sin_cos_special::<F32>();

    test_with(Unit::Radians, |x| {
        let actual_sin = x.sin();
        let actual_cos = x.cos();
        assert_total_eq!((-x).cos(), actual_cos);
        assert_total_eq!(x.sin_cos(), (actual_sin, actual_cos));
        if actual_sin.is_nan() {
            assert_total_eq!((-x).sin(), actual_sin);
            assert_total_eq!((-x).sin_cos(), (actual_sin, actual_cos));
        } else {
            assert_total_eq!((-x).sin(), -actual_sin);
            assert_total_eq!((-x).sin_cos(), (-actual_sin, actual_cos));
        }

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

    test_with(Unit::Radians, |x| {
        let actual = x.tan();
        if actual.is_nan() {
            assert_total_eq!((-x).tan(), actual);
        } else {
            assert_total_eq!((-x).tan(), -actual);
        }

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

    test_with(Unit::Degrees, |x| {
        let actual_sin = x.sind();
        let actual_cos = x.cosd();
        assert_total_eq!((-x).cosd(), actual_cos);
        assert_total_eq!(x.sind_cosd(), (actual_sin, actual_cos));
        if actual_sin.is_nan() {
            assert_total_eq!((-x).sind(), actual_sin);
            assert_total_eq!((-x).sind_cosd(), (actual_sin, actual_cos));
        } else {
            assert_total_eq!((-x).sind(), -actual_sin);
            assert_total_eq!((-x).sind_cosd(), (-actual_sin, actual_cos));
        }

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

    test_with(Unit::Degrees, |x| {
        let actual = x.tand();
        if actual.is_nan() {
            assert_total_eq!((-x).tand(), actual);
        } else {
            assert_total_eq!((-x).tand(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.tan_u_round(360, rug::float::Round::Nearest);
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

    test_with(Unit::HalfRevs, |x| {
        let actual_sin = x.sinpi();
        let actual_cos = x.cospi();
        assert_total_eq!((-x).cospi(), actual_cos);
        assert_total_eq!(x.sinpi_cospi(), (actual_sin, actual_cos));
        if actual_sin.is_nan() {
            assert_total_eq!((-x).sinpi(), actual_sin);
            assert_total_eq!((-x).sinpi_cospi(), (actual_sin, actual_cos));
        } else {
            assert_total_eq!((-x).sinpi(), -actual_sin);
            assert_total_eq!((-x).sinpi_cospi(), (-actual_sin, actual_cos));
        }

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

    test_with(Unit::HalfRevs, |x| {
        let actual = x.tanpi();
        if actual.is_nan() {
            assert_total_eq!((-x).tanpi(), actual);
        } else {
            assert_total_eq!((-x).tanpi(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.tan_pi_round(rug::float::Round::Nearest);
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

/// Angle unit of the trigonometric functions.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Unit {
    Radians,
    Degrees,
    HalfRevs,
}

fn test_with(unit: Unit, mut f: impl FnMut(F32)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-126..=127, 5000) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // Problematic value in
    // "ARGUMENT REDUCTION FOR HUGE ARGUMENTS: Good to the Last Bit"
    f(F32::from_host(1.0e22));

    // Test the multiples of an eighth of a half revolution (π/8 radians, 22.5
    // degrees or 1/8 half revolution) at several scales, and the arguments
    // slightly away from them
    let one = F32::from_uint(1);
    let step = match unit {
        Unit::Radians => F32::from_host(std::f32::consts::FRAC_PI_8),
        Unit::Degrees => F32::from_host(22.5),
        Unit::HalfRevs => F32::from_host(0.125),
    };
    for f1 in 1..=100 {
        let f1 = F32::from_uint(f1);
        for f2 in 0..=20 {
            let f2 = F32::from_uint(1 << f2);

            f(step * f1 * f2);
            for f3 in 1..=20 {
                let f3 = F32::from_int(-f3);
                f(step * f1 * f2 * (one + f3.exp()));
                f(step * f1 * f2 * (one - f3.exp()));
                f(step * f1 * f2 * (one + f3.exp2()));
                f(step * f1 * f2 * (one - f3.exp2()));
                f(step * f1 * f2 * (one + f3.exp10()));
                f(step * f1 * f2 * (one - f3.exp10()));
            }
        }
    }

    // Generated test data
    let data_files = match unit {
        Unit::Radians => ["f32/sin.txt", "f32/cos.txt", "f32/tan.txt"],
        Unit::Degrees => ["f32/sind.txt", "f32/cosd.txt", "f32/tand.txt"],
        Unit::HalfRevs => ["f32/sinpi.txt", "f32/cospi.txt", "f32/tanpi.txt"],
    };
    for data_file in data_files {
        for x in crate::data::read_data_file::<F32>(data_file) {
            f(x);
        }
    }
}
