use swfp::{F64, Float as _, math::Cbrt as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_cbrt() {
    crate::generic::test_cbrt_special::<F64>();

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

fn test_with(mut f: impl FnMut(F64)) {
    // Test special values
    for x in values::specials() {
        f(x);
    }

    // Test more values at three consecutive binades. The implementation only
    // depends on the mantissa and on the exponent modulo 3 (the result is
    // scaled exactly).
    for x in values::binades(0..=2, 2_000_000) {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-1022..=1023, 2000) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(1_000_000) {
        f(x);
    }

    for x in 1..=500_000 {
        f(F64::from_uint(x));
    }

    // Test data from core-math
    for x in crate::data::read_data_file::<F64>("core-math/binary64/cbrt.wc") {
        if x.is_nan() {
            continue;
        }
        f(x);
    }
}
