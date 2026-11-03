use swfp::{F16, Float as _, math::Hypot as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug, values};

#[test]
fn test_hypot() {
    crate::generic::test_hypot_special::<F16>();

    let round_modes = [
        (swfp::Round::NearestTiesToEven, rug::float::Round::Nearest),
        (swfp::Round::TowardPositive, rug::float::Round::Up),
        (swfp::Round::TowardNegative, rug::float::Round::Down),
        (swfp::Round::TowardZero, rug::float::Round::Zero),
        (swfp::Round::AwayFromZero, rug::float::Round::AwayZero),
    ];

    test_with(|x, y| {
        let actual = x.hypot(y);
        assert_total_eq!((-x).hypot(y), actual);
        assert_total_eq!(x.hypot(-y), actual);
        assert_total_eq!((-x).hypot(-y), actual);
        assert_total_eq!(y.hypot(x), actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.hypot_round(&to_rug(y), rug::float::Round::Nearest);
        check_exact(
            (x, y),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );

        for &(swfp_round, rug_round) in round_modes.iter() {
            let (actual, status) = x.hypot_ex(y, swfp_round);
            assert_total_eq!((-x).hypot_ex(y, swfp_round), (actual, status));
            assert_total_eq!(x.hypot_ex(-y, swfp_round), (actual, status));
            assert_total_eq!((-x).hypot_ex(-y, swfp_round), (actual, status));
            assert_total_eq!(y.hypot_ex(x, swfp_round), (actual, status));

            let mut expected = to_rug(x);
            let round_cmp = expected.hypot_round(&to_rug(y), rug_round);
            check_exact((x, y), actual, Some(status), expected, round_cmp, rug_round);
        }
    });
}

fn test_with(mut f: impl FnMut(F16, F16)) {
    // Swapping the arguments and toggling their signs is done by the caller.

    // Test all the exponent differences at every scale, including subnormals
    for (x, y) in values::binade_pairs(values::exp_pairs(0..=39, 1), 5000) {
        f(x, y);
    }

    // Exhaustive test of all pairs of subnormal numbers
    for x in values::subnormals_full() {
        for y in values::subnormals_full() {
            f(x, y);
        }
    }

    let specials = [
        F16::INFINITY,
        F16::ZERO,
        F16::from_int(1),
        mk_normal(0, 14, false),
        mk_normal(0x3FF, 14, false),
        mk_normal(0, 15, false),
        mk_normal(0x3FF, 15, false),
        mk_normal(0, -14, false),
        mk_normal(0x3FF, -14, false),
        mk_subnormal(1, false),
        mk_subnormal(0x200, false),
        mk_subnormal(0x3FF, false),
    ];

    for x in specials {
        for y in specials {
            f(x, y);
        }
    }
}
