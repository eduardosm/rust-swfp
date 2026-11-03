use swfp::{F32, Float as _, math::Gamma as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_gamma() {
    crate::generic::test_gamma_special::<F32>();

    test_with(Func::Gamma, |x| {
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
    crate::generic::test_ln_gamma_special::<F32>();

    test_with(Func::LnGamma, |x| {
        let (actual, actual_sign) = x.ln_gamma();

        let mut expected = to_rug(x);
        let (ord, round_cmp) = expected.ln_abs_gamma_round(rug::float::Round::Nearest);
        if x == -F32::INFINITY {
            // The gamma function is not defined at -∞ (MPFR returns +∞)
            expected = to_rug(F32::NAN);
        }
        let expected_sign = if x.is_nan() || (x < F32::ZERO && x.trunc() == x) {
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

/// A function of the gamma family.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Func {
    Gamma,
    LnGamma,
}

fn test_with(func: Func, mut f: impl FnMut(F32)) {
    // Test special values
    for x in values::specials() {
        f(x);
        f(-x);
    }

    // Test across a wide range of normal numbers, with more values at the
    // binades below 2^7
    for x in values::binades(-126..=6, 10_000).chain(values::binades(7..=127, 2000)) {
        f(x);
        f(-x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
        f(-x);
    }

    for x in 1..=20_000 {
        let x = F32::from_uint(x) / F32::from_uint(100);
        f(x);
        f(-x);
    }

    // Test the arguments around the negative integers (poles), whose results
    // are large or overflow
    for n in 1..=50 {
        for x in values::around(F32::from_int(-n), 10_000) {
            f(x);
        }
    }

    // Generated test data
    let data_file = match func {
        Func::Gamma => "f32/gamma.txt",
        Func::LnGamma => "f32/ln_gamma.txt",
    };
    for x in crate::data::read_data_file::<F32>(data_file) {
        f(x);
    }
}
