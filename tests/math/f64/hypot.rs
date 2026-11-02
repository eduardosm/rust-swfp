use swfp::{F64, Float as _, math::Hypot as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug, values};

#[test]
fn test_hypot() {
    crate::generic::test_hypot_special::<F64>();

    let round_modes = [
        (swfp::Round::NearestTiesToEven, rug::float::Round::Nearest),
        (swfp::Round::TowardPositive, rug::float::Round::Up),
        (swfp::Round::TowardNegative, rug::float::Round::Down),
        (swfp::Round::TowardZero, rug::float::Round::Zero),
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

fn test_with(mut f: impl FnMut(F64, F64)) {
    // Swapping the arguments and toggling their signs is done by the caller.

    // Test the exponent differences where both arguments affect the result
    // when rounding to nearest (the smaller one stops affecting it when it is
    // less than 2^-27 times the larger one), at many scales, including
    // subnormals
    for (x, y) in values::binade_pairs(values::exp_pairs(0..=30, 8), 1000) {
        f(x, y);
    }

    // Test larger exponent differences at some scales
    for (x, y) in values::binade_pairs(values::exp_pairs((31..=2097).step_by(7), 29), 1) {
        f(x, y);
    }

    // Test more pairs of subnormal numbers, at every combination of their
    // binades
    let subnormal_exps = (-1074..=-1023).flat_map(|ex| (-1074..=ex).map(move |ey| (ex, ey)));
    for (x, y) in values::binade_pairs(subnormal_exps, 1000) {
        f(x, y);
    }

    let specials = [
        F64::INFINITY,
        F64::ZERO,
        F64::from_int(1),
        mk_normal(0, 1022, false),
        mk_normal(0xF_FFFF_FFFF_FFFF, 1022, false),
        mk_normal(0, 1023, false),
        mk_normal(0xF_FFFF_FFFF_FFFF, 1023, false),
        mk_normal(0, -1022, false),
        mk_normal(0xF_FFFF_FFFF_FFFF, -1022, false),
        mk_subnormal(1, false),
        mk_subnormal(0x8_0000_0000_0000, false),
        mk_subnormal(0xF_FFFF_FFFF_FFFF, false),
    ];

    for x in specials {
        for y in specials {
            f(x, y);
        }
    }

    // Test data from core-math
    for [x, y] in crate::data::read_data_file::<[F64; 2]>("core-math/binary64/hypot.wc") {
        if x.is_nan() || y.is_nan() {
            continue;
        }
        f(x, y);
    }
}
