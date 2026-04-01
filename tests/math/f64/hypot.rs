use swfp::{F64, Float as _, math::Hypot as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

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
    let mut rng = crate::create_prng();

    for ex in -1022..=1023 {
        if matches!(ex, -900..=900) && (ex & 3) != 3 {
            continue; // speed up tests
        }
        for ey in -1022..=1023 {
            if matches!(ey, -900..=900) && (ey & 3) != 3 {
                continue; // speed up tests
            }
            for _ in 0..20 {
                let mx = super::gen_mantissa(&mut rng);
                let my = super::gen_mantissa(&mut rng);
                f(mk_normal(mx, ex, false), mk_normal(my, ey, false));
            }
        }
    }

    for _ in 0..2_000_000 {
        let mx = super::gen_mantissa(&mut rng);
        let my = super::gen_mantissa(&mut rng);
        f(mk_subnormal(mx, false), mk_subnormal(my, false));
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
