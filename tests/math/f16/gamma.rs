use swfp::{F16, Float as _, math::Gamma as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_gamma() {
    crate::generic::test_gamma_special::<F16>();

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
    crate::generic::test_ln_gamma_special::<F16>();

    test_with(|x| {
        let (actual, actual_sign) = x.ln_gamma();

        let mut expected = to_rug(x);
        let (ord, round_cmp) = expected.ln_abs_gamma_round(rug::float::Round::Nearest);
        if x == -F16::INFINITY {
            // The gamma function is not defined at -∞ (MPFR returns +∞)
            expected = to_rug(F16::NAN);
        }
        let expected_sign = if x.is_nan() || (x < F16::ZERO && x.trunc() == x) {
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

fn test_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Exhaustive test of all normal numbers
    for x in values::binades_full(-14..=15) {
        f(x);
        f(-x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }
}
