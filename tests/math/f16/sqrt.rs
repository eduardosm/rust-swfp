use swfp::{F16, Float as _, math::Sqrt as _};

use super::{check_exact, to_rug, values};

#[test]
fn test_sqrt() {
    crate::generic::test_sqrt_special::<F16>();

    let round_modes = [
        (swfp::Round::NearestTiesToEven, rug::float::Round::Nearest),
        (swfp::Round::NearestTiesToAway, rug::float::Round::Nearest),
        (swfp::Round::TowardPositive, rug::float::Round::Up),
        (swfp::Round::TowardNegative, rug::float::Round::Down),
        (swfp::Round::TowardZero, rug::float::Round::Zero),
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
                assert_total_eq!((actual, status), (F16::NAN, swfp::FpStatus::OK));
            } else {
                let mut expected = to_rug(x);
                let round_cmp = expected.sqrt_round(rug_round);
                check_exact(x, actual, Some(status), expected, round_cmp, rug_round);
            }
        }
    });
}

fn test_with(mut f: impl FnMut(F16)) {
    // Test special values
    for x in values::specials().chain([-F16::ZERO]) {
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
