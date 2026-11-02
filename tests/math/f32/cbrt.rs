use swfp::{F32, Float as _, math::Cbrt as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_cbrt() {
    crate::generic::test_cbrt_special::<F32>();

    test_with(|x| {
        let actual = x.cbrt();
        if actual.is_nan() {
            assert_total_eq!((-x).cbrt(), actual);
        } else {
            assert_total_eq!((-x).cbrt(), -actual);
        }

        let mut expected = to_rug(x);
        let round_cmp = expected.cbrt_round(rug::float::Round::Nearest);
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
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Exhaustive test of all mantissas
    for x in values::binades_full(0..=2) {
        f(x);
    }

    // Exhaustive test of all subnormal numbers
    for x in values::subnormals_full() {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-126..=127, 1000) {
        f(x);
    }

    for x in 1..=500_000 {
        f(F32::from_uint(x));
    }
}
