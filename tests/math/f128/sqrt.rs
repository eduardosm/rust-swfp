use swfp::{F128, Float as _, math::Sqrt as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_sqrt() {
    crate::generic::test_sqrt_special::<F128>();

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

            let mut expected = to_rug(x);
            let round_cmp = expected.sqrt_round(rug_round);
            check_exact(x, actual, Some(status), expected, round_cmp, rug_round);
        }
    });
}

fn test_with(mut f: impl FnMut(F128)) {
    let mut rng = crate::create_prng();

    for e in 0..=1 {
        for _ in 0..2_000_000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for e in -16382..=16383 {
        f(mk_normal(0, e, false));
        f(mk_normal(!(u128::MAX << 112), e, false));
        for _ in 0..100 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for _ in 0..1_000_000 {
        let m = super::gen_mantissa(&mut rng);
        f(mk_subnormal(m, false));
    }

    for x in 1..=500_000 {
        f(F128::from_uint(x));
    }
}
