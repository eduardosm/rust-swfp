use swfp::{F32, Float as _, math::Sqrt as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_sqrt() {
    crate::generic::test_sqrt_special::<F32>();

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

fn test_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    // Exhaustive test of all mantissas
    for e in 0..=1 {
        for m in 0..(1 << 23) {
            f(mk_normal(m, e, false));
        }
    }

    // Exhaustive test of all subnormal numbers
    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
    }

    for e in -126..=127 {
        f(mk_normal(0, e, false));
        f(mk_normal((1 << 23) - 1, e, false));
        for _ in 0..1000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for x in 1..=500_000 {
        f(F32::from_uint(x));
    }
}
