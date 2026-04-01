use swfp::{F32, Float as _, math::Hyperbolic as _};

use super::{check_exact, mk_normal, mk_subnormal, to_rug};

#[test]
fn test_sinh_cosh() {
    crate::generic::test_sinh_cosh_special::<F32>();

    test_with(|x| {
        let actual_sinh = x.sinh();
        let actual_cosh = x.cosh();
        assert_total_eq!((-x).sinh(), -actual_sinh);
        assert_total_eq!((-x).cosh(), actual_cosh);
        assert_total_eq!(x.sinh_cosh(), (actual_sinh, actual_cosh));
        assert_total_eq!((-x).sinh_cosh(), (-actual_sinh, actual_cosh));

        let mut expected_sinh = to_rug(x);
        let round_cmp = expected_sinh.sinh_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_sinh,
            None,
            expected_sinh,
            round_cmp,
            rug::float::Round::Nearest,
        );

        let mut expected_cosh = to_rug(x);
        let round_cmp = expected_cosh.cosh_round(rug::float::Round::Nearest);
        check_exact(
            x,
            actual_cosh,
            None,
            expected_cosh,
            round_cmp,
            rug::float::Round::Nearest,
        );
    });
}

#[test]
fn test_tanh() {
    crate::generic::test_tanh_special::<F32>();

    test_with(|x| {
        let actual = x.tanh();
        assert_total_eq!((-x).tanh(), -actual);

        let mut expected = to_rug(x);
        let round_cmp = expected.tanh_round(rug::float::Round::Nearest);
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

fn test_with(mut f: impl FnMut(F32)) {
    let mut rng = crate::create_prng();

    for e in -126..=127 {
        for &m in super::NORMAL_MANTISSAS.iter() {
            f(mk_normal(m, e, false));
        }
        for _ in 0..5000 {
            let m = super::gen_mantissa(&mut rng);
            f(mk_normal(m, e, false));
        }
    }

    for m in 0..(1 << 23) {
        f(mk_subnormal(m, false));
    }

    for x in 1..=200_000 {
        f(F32::from_uint(x));
    }

    // Generated test data
    for x in crate::data::read_data_file::<F32>("f32/sinh.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/cosh.txt") {
        f(x);
    }
    for x in crate::data::read_data_file::<F32>("f32/tanh.txt") {
        f(x);
    }
}
