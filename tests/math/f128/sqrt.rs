use swfp::{F128, Float as _, math::Sqrt as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_sqrt() {
    crate::generic::test_sqrt_special::<F128>();

    let round_modes = [
        (swfp::Round::NearestTiesToEven, rug::float::Round::Nearest),
        (swfp::Round::NearestTiesToAway, rug::float::Round::Nearest),
        (swfp::Round::TowardPositive, rug::float::Round::Up),
        (swfp::Round::TowardNegative, rug::float::Round::Down),
        (swfp::Round::TowardZero, rug::float::Round::Zero),
        (swfp::Round::AwayFromZero, rug::float::Round::AwayZero),
    ];

    test_with(|x| {
        let actual = x.sqrt();

        let mut expected = to_rug(x);
        let round_cmp = expected.sqrt_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );

        for &(swfp_round, rug_round) in round_modes.iter() {
            let (actual, status) = x.sqrt_ex(swfp_round);
            if x.is_nan() {
                // `from_rug` assumes that NaN results come from invalid
                // operations, but a NaN argument does not raise any exception
                assert_total_eq!((actual, status), (F128::NAN, swfp::FpStatus::OK));
            } else {
                let mut expected = to_rug(x);
                let round_cmp = expected.sqrt_round(rug_round);
                check_exact(x, actual, Some(status), expected, round_cmp, rug_round);
            }
        }
    });
}

fn test_with(mut f: impl FnMut(F128)) {
    // Test special values
    for x in values::specials().chain([-F128::ZERO]) {
        f(x);
    }

    // Test more values at two consecutive binades. The implementation only
    // depends on the mantissa and on the parity of the exponent (the result
    // is scaled exactly).
    for x in values::binades(0..=1, 2_000_000) {
        f(x);
    }

    // Test across a wide range of normal numbers
    for x in values::binades(-16382..=16383, 100) {
        f(x);
    }

    // Test a set of subnormal values
    for x in values::subnormals(1_000_000) {
        f(x);
    }

    for x in 1..=500_000 {
        f(F128::from_uint(x));
    }
}
