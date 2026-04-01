use swfp::{F16, Float as _, math::InvTrigonometric as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_asin() {
    crate::generic::test_asin_special::<F16>();

    test_asin_with(|x| {
        let actual = x.asin();
        assert_total_eq!((-x).asin(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_round(rug::float::Round::Nearest);
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

#[test]
fn test_acos() {
    crate::generic::test_acos_special::<F16>();

    test_acos_with(|x| {
        let actual = x.acos();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_round(rug::float::Round::Nearest);
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

#[test]
fn test_atan() {
    crate::generic::test_atan_special::<F16>();

    test_atan_with(|x| {
        let actual = x.atan();
        assert_total_eq!((-x).atan(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_round(rug::float::Round::Nearest);
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

#[test]
fn test_atan2() {
    crate::generic::test_atan2_special::<F16>();

    test_atan2_with(|y, x| {
        let actual = y.atan2(x);
        assert_total_eq!((-y).atan2(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_round(&to_rug(x), rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_asind() {
    crate::generic::test_asind_special::<F16>();

    test_asin_with(|x| {
        let actual = x.asind();
        assert_total_eq!((-x).asind(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_u_round(360, rug::float::Round::Nearest);
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

#[test]
fn test_acosd() {
    crate::generic::test_acosd_special::<F16>();

    test_acos_with(|x| {
        let actual = x.acosd();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_u_round(360, rug::float::Round::Nearest);
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

#[test]
fn test_atand() {
    crate::generic::test_atand_special::<F16>();

    test_atan_with(|x| {
        let actual = x.atand();
        assert_total_eq!((-x).atand(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_u_round(360, rug::float::Round::Nearest);
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

#[test]
fn test_atan2d() {
    crate::generic::test_atan2d_special::<F16>();

    test_atan2_with(|y, x| {
        let actual = y.atan2d(x);
        assert_total_eq!((-y).atan2d(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_u_round(&to_rug(x), 360, rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_asinpi() {
    crate::generic::test_asinpi_special::<F16>();

    test_asin_with(|x| {
        let actual = x.asinpi();
        assert_total_eq!((-x).asinpi(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.asin_pi_round(rug::float::Round::Nearest);
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

#[test]
fn test_acospi() {
    crate::generic::test_acospi_special::<F16>();

    test_acos_with(|x| {
        let actual = x.acospi();

        let mut expected = to_rug(x);
        let round_cmp = expected.acos_pi_round(rug::float::Round::Nearest);
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

#[test]
fn test_atanpi() {
    crate::generic::test_atanpi_special::<F16>();

    test_atan_with(|x| {
        let actual = x.atanpi();
        assert_total_eq!((-x).atanpi(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.atan_pi_round(rug::float::Round::Nearest);
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

#[test]
fn test_atan2pi() {
    crate::generic::test_atan2pi_special::<F16>();

    test_atan2_with(|y, x| {
        let actual = y.atan2pi(x);
        assert_total_eq!((-y).atan2pi(x), -actual);

        let mut expected = to_rug(y);
        let round_cmp = expected.atan2_pi_round(&to_rug(x), rug::float::Round::Nearest);
        check_exact(
            (y, x),
            actual,
            None,
            expected,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

fn test_asin_with(mut f: impl FnMut(F16)) {
    for e in -14..=-1 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
    }
    f(F16::from_int(1));
}

fn test_acos_with(mut f: impl FnMut(F16)) {
    for e in -14..=-1 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
            f(mk_normal(m, e, true));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
        f(mk_subnormal(m, true));
    }
    f(F16::from_int(1));
    f(F16::from_int(-1));
}

fn test_atan_with(mut f: impl FnMut(F16)) {
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false));
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false));
    }
    f(F16::INFINITY);
}

fn test_atan2_with(mut f: impl FnMut(F16, F16)) {
    let mut rng = crate::create_prng();

    for ey in -14..=15 {
        for ex in -14..=15 {
            for _ in 0..1000 {
                let my = super::gen_mantissa(&mut rng);
                let mx = super::gen_mantissa(&mut rng);
                f(mk_normal(my, ey, false), mk_normal(mx, ex, false));
                f(mk_normal(my, ey, false), mk_normal(mx, ex, true));
            }
        }
    }

    for my in 0..(1 << 10) {
        for mx in 0..(1 << 10) {
            f(mk_subnormal(my, false), mk_subnormal(mx, false));
            f(mk_subnormal(my, false), mk_subnormal(mx, true));
        }
    }

    let specials = [F16::from_int(1), F16::ZERO, F16::INFINITY];
    for &y in specials.iter() {
        for &x in specials.iter() {
            f(y, x);
            f(y, -x);
        }
    }

    let one = F16::from_int(1);
    for e in -14..=15 {
        for m in 0..(1 << 10) {
            f(mk_normal(m, e, false), one);
        }
    }
    for m in 0..(1 << 10) {
        f(mk_subnormal(m, false), one);
    }

    // Generated test data
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2.txt") {
        f(y, x);
    }
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2d.txt") {
        f(y, x);
    }
    for [y, x] in crate::data::read_data_file::<[F16; 2]>("f16/atan2pi.txt") {
        f(y, x);
    }
}
