use rand::RngExt as _;

use swfp::{F16, Float as _, math::Pow as _};

use super::{check_exact, mk_normal, to_rug};

#[test]
fn test_pow() {
    crate::generic::test_pow_special::<F16>();

    test_pow_with(|x, y| {
        let actual = x.pow(y);

        let mut expected = to_rug(x);
        let round_cmp = rug::ops::PowAssignRound::pow_assign_round(
            &mut expected,
            &to_rug(y),
            rug::float::Round::Nearest,
        );
        check_exact(
            (x, y),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_powi() {
    crate::generic::test_powi_special::<F16>();

    test_powi_with(|x, n| {
        let actual = x.powi(n);

        let mut expected = to_rug(x);
        let round_cmp = rug::ops::PowAssignRound::pow_assign_round(
            &mut expected,
            n,
            rug::float::Round::Nearest,
        );
        check_exact(
            (x, n),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

fn test_pow_with(mut f: impl FnMut(F16, F16)) {
    let mut rng = crate::create_prng();

    for ex in -14..=15 {
        for ey in -14..=15 {
            for _ in 0..5000 {
                let mx = super::gen_mantissa(&mut rng);
                let my = super::gen_mantissa(&mut rng);
                let x = mk_normal(mx, ex, false);
                let y = mk_normal(my, ey, false);
                f(x, y);
                f(x, -y);
                f(-x, y);
                f(-x, -y);
            }
        }
    }

    let one = F16::from_uint(1);
    for ex in -9..=-1 {
        for ey in 1..=15 {
            for _ in 0..5000 {
                let mx = super::gen_mantissa(&mut rng);
                let x = one + mk_normal(mx, ex, false);

                let my = super::gen_mantissa(&mut rng);
                let y = mk_normal(my, ey, false);

                f(x, y);
                f(x, -y);
            }
        }
    }

    // Generated test data
    for [x, y] in crate::data::read_data_file::<[F16; 2]>("f16/pow.txt") {
        f(x, y);
    }
}

fn test_powi_with(mut f: impl FnMut(F16, i32)) {
    let mut rng = crate::create_prng();

    for ex in -14..=15 {
        for n in -128..=128 {
            for _ in 0..2000 {
                let mx = super::gen_mantissa(&mut rng);
                let x = mk_normal(mx, ex, false);
                f(x, n);
                f(-x, n);
            }
        }
        for shift in 0..=25 {
            for _ in 0..5000 {
                let mx = super::gen_mantissa(&mut rng);
                let x = mk_normal(mx, ex, false);
                let n = rng.random::<i32>() >> shift;
                f(x, n);
                f(-x, n);
            }
        }
    }

    let one = F16::from_uint(1);
    for ex in -9..=-1 {
        for shift in 0..=30 {
            for _ in 0..5000 {
                let mx = super::gen_mantissa(&mut rng);
                let x = one + mk_normal(mx, ex, false);
                let n = rng.random::<i32>() >> shift;

                f(x, n);
            }
        }
    }

    // Generated test data
    for [x, y] in crate::data::read_data_file::<[F16; 2]>("f16/pow.txt") {
        let (Some(n), swfp::FpStatus::Ok) = y.to_int_ex(32, swfp::Round::TowardZero) else {
            continue;
        };
        f(x, n as i32);
    }
}
