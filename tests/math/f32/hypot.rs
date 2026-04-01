use swfp::{F32, Float as _, math::Hypot as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_hypot() {
    crate::generic::test_hypot_special::<F32>();

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

fn test_with(mut f: impl FnMut(F32, F32)) {
    let mut rng = crate::create_prng();

    for ex in -126..=127 {
        for ey in -126..=127 {
            for _ in 0..200 {
                let mx = super::gen_mantissa(&mut rng);
                let my = super::gen_mantissa(&mut rng);
                f(mk_normal(mx, ex, false), mk_normal(my, ey, false));
            }
        }
    }

    for _ in 0..1_000_000 {
        let mx = super::gen_mantissa(&mut rng);
        let my = super::gen_mantissa(&mut rng);
        f(mk_subnormal(mx, false), mk_subnormal(my, false));
    }

    let specials = [
        F32::INFINITY,
        F32::ZERO,
        F32::from_int(1),
        mk_normal(0, 126, false),
        mk_normal(0x3FFFFF, 126, false),
        mk_normal(0, 127, false),
        mk_normal(0x3FFFFF, 127, false),
        mk_normal(0, -126, false),
        mk_normal(0x3FFFFF, -126, false),
        mk_subnormal(1, false),
        mk_subnormal(0x200000, false),
        mk_subnormal(0x3FFFFF, false),
    ];

    for x in specials {
        for y in specials {
            f(x, y);
        }
    }

    // Test data from core-math
    for [x, y] in crate::data::read_data_file::<[F32; 2]>("core-math/binary32/hypotf.wc") {
        if x.is_nan() || y.is_nan() {
            continue;
        }
        f(x, y);
    }
}
