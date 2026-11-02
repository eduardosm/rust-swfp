use swfp::{F16, Float as _, math::Trigonometric as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_sin_cos() {
    crate::generic::test_sin_cos_special::<F16>();

    test_with(|x| {
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
    crate::generic::test_tan_special::<F16>();

    test_with(|x| {
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
    crate::generic::test_sind_cosd_special::<F16>();

    test_with(|x| {
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
    crate::generic::test_tand_special::<F16>();

    test_with(|x| {
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
    crate::generic::test_sinpi_cospi_special::<F16>();

    test_with(|x| {
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
    crate::generic::test_tanpi_special::<F16>();

    test_with(|x| {
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

fn test_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Exhaustive test of all normal numbers
    for x in values::binades_full(-14..=15) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }
}
