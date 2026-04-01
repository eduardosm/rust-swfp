use crate::TestFloat;

pub(crate) fn test_sqrt_special<F: TestFloat + swfp::math::Sqrt>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.sqrt());
    assert_is_nan!((-F::INFINITY).sqrt());
    assert_is_nan!(f("-1").sqrt());
    assert_total_eq!(F::INFINITY.sqrt(), F::INFINITY);
    assert_total_eq!(F::ZERO.sqrt(), F::ZERO);
    assert_total_eq!((-F::ZERO).sqrt(), -F::ZERO);

    assert_total_eq!(f("4").sqrt(), f("2"));
}

pub(crate) fn test_cbrt_special<F: TestFloat + swfp::math::Cbrt>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.cbrt());
    assert_total_eq!(F::INFINITY.cbrt(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).cbrt(), -F::INFINITY);
    assert_total_eq!(F::ZERO.cbrt(), F::ZERO);
    assert_total_eq!((-F::ZERO).cbrt(), -F::ZERO);

    assert_total_eq!(f("8").cbrt(), f("2"));
    assert_total_eq!(f("-8").cbrt(), f("-2"));
    assert_total_eq!(f("42.875").cbrt(), f("3.5"));
    assert_total_eq!(f("-42.875").cbrt(), f("-3.5"));
}

pub(crate) fn test_hypot_special<F: TestFloat + swfp::math::Hypot>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);

    assert_is_nan!(F::NAN.hypot(F::NAN));
    assert_is_nan!(F::NAN.hypot(F::ZERO));
    assert_is_nan!(F::NAN.hypot(one));
    assert_is_nan!(F::ZERO.hypot(F::NAN));
    assert_is_nan!(one.hypot(F::NAN));
    assert_total_eq!(F::INFINITY.hypot(F::INFINITY), F::INFINITY);
    assert_total_eq!(F::INFINITY.hypot(-F::INFINITY), F::INFINITY);
    assert_total_eq!((-F::INFINITY).hypot(F::INFINITY), F::INFINITY);
    assert_total_eq!((-F::INFINITY).hypot(-F::INFINITY), F::INFINITY);
    assert_total_eq!(F::INFINITY.hypot(F::NAN), F::INFINITY);
    assert_total_eq!((-F::INFINITY).hypot(F::NAN), F::INFINITY);
    assert_total_eq!(F::NAN.hypot(F::INFINITY), F::INFINITY);
    assert_total_eq!(F::NAN.hypot(-F::INFINITY), F::INFINITY);
    assert_total_eq!(F::INFINITY.hypot(F::ZERO), F::INFINITY);
    assert_total_eq!(F::INFINITY.hypot(one), F::INFINITY);
    assert_total_eq!((-F::INFINITY).hypot(F::ZERO), F::INFINITY);
    assert_total_eq!((-F::INFINITY).hypot(one), F::INFINITY);
    assert_total_eq!(F::ZERO.hypot(F::INFINITY), F::INFINITY);
    assert_total_eq!(one.hypot(F::INFINITY), F::INFINITY);
    assert_total_eq!(F::ZERO.hypot(-F::INFINITY), F::INFINITY);
    assert_total_eq!(one.hypot(-F::INFINITY), F::INFINITY);
    assert_total_eq!(F::ZERO.hypot(F::ZERO), F::ZERO);
    assert_total_eq!((-F::ZERO).hypot(-F::ZERO), F::ZERO);
    assert_total_eq!(one.hypot(F::ZERO), one);
    assert_total_eq!(f("3").hypot(F::ZERO), f("3"));
    assert_total_eq!(F::ZERO.hypot(one), one);
    assert_total_eq!(F::ZERO.hypot(f("3")), f("3"));
    assert_total_eq!(f("3").hypot(f("4")), f("5"));
}

pub(crate) fn test_exp_special<F: TestFloat + swfp::math::Exp>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp());
    assert_total_eq!(F::INFINITY.exp(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp(), F::ZERO);
    assert_total_eq!(F::ZERO.exp(), one);
    assert_total_eq!((-F::ZERO).exp(), one);
}

pub(crate) fn test_exp_m1_special<F: TestFloat + swfp::math::Exp>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp_m1());
    assert_total_eq!(F::INFINITY.exp_m1(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp_m1(), -one);
    assert_total_eq!(F::ZERO.exp_m1(), F::ZERO);
    assert_total_eq!((-F::ZERO).exp_m1(), -F::ZERO);
}

pub(crate) fn test_exp2_special<F: TestFloat + swfp::math::Exp>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp2());
    assert_total_eq!(F::INFINITY.exp2(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp2(), F::ZERO);
    assert_total_eq!(F::ZERO.exp2(), one);
    assert_total_eq!((-F::ZERO).exp2(), one);
    assert_total_eq!(f("3").exp2(), f("8"));
    assert_total_eq!(f("-2").exp2(), f("0.25"));
}

pub(crate) fn test_exp2_m1_special<F: TestFloat + swfp::math::Exp>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp2_m1());
    assert_total_eq!(F::INFINITY.exp2_m1(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp2_m1(), -one);
    assert_total_eq!(F::ZERO.exp2_m1(), F::ZERO);
    assert_total_eq!((-F::ZERO).exp2_m1(), -F::ZERO);
}

pub(crate) fn test_exp10_special<F: TestFloat + swfp::math::Exp>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp10());
    assert_total_eq!(F::INFINITY.exp10(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp10(), F::ZERO);
    assert_total_eq!(F::ZERO.exp10(), one);
    assert_total_eq!((-F::ZERO).exp10(), one);
    assert_total_eq!(f("2").exp10(), f("100"));
}

pub(crate) fn test_exp10_m1_special<F: TestFloat + swfp::math::Exp>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.exp10_m1());
    assert_total_eq!(F::INFINITY.exp10_m1(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).exp10_m1(), -one);
    assert_total_eq!(F::ZERO.exp10_m1(), F::ZERO);
    assert_total_eq!((-F::ZERO).exp10_m1(), -F::ZERO);
}

pub(crate) fn test_ln_special<F: TestFloat + swfp::math::Log>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.ln());
    assert_is_nan!((-one).ln());
    assert_is_nan!((-F::INFINITY).ln());
    assert_total_eq!(F::ZERO.ln(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).ln(), -F::INFINITY);
    assert_total_eq!(F::INFINITY.ln(), F::INFINITY);
}

pub(crate) fn test_ln_1p_special<F: TestFloat + swfp::math::Log>() {
    let f = |s: &str| s.parse::<F>().unwrap();
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.ln_1p());
    assert_is_nan!(f("-1.5").ln_1p());
    assert_is_nan!((-F::INFINITY).ln_1p());
    assert_total_eq!((-one).ln_1p(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).ln_1p(), -F::ZERO);
    assert_total_eq!(F::ZERO.ln_1p(), F::ZERO);
    assert_total_eq!(F::INFINITY.ln_1p(), F::INFINITY);
}

pub(crate) fn test_log2_special<F: TestFloat + swfp::math::Log>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.log2());
    assert_is_nan!((-one).log2());
    assert_is_nan!((-F::INFINITY).log2());
    assert_total_eq!(F::ZERO.log2(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).log2(), -F::INFINITY);
    assert_total_eq!(F::INFINITY.log2(), F::INFINITY);
}

pub(crate) fn test_log2_1p_special<F: TestFloat + swfp::math::Log>() {
    let f = |s: &str| s.parse::<F>().unwrap();
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.log2_1p());
    assert_is_nan!(f("-1.5").log2_1p());
    assert_is_nan!((-F::INFINITY).log2_1p());
    assert_total_eq!((-one).log2_1p(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).log2_1p(), -F::ZERO);
    assert_total_eq!(F::ZERO.log2_1p(), F::ZERO);
    assert_total_eq!(F::INFINITY.log2_1p(), F::INFINITY);
}

pub(crate) fn test_log10_special<F: TestFloat + swfp::math::Log>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.log10());
    assert_is_nan!((-one).log10());
    assert_is_nan!((-F::INFINITY).log10());
    assert_total_eq!(F::ZERO.log10(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).log10(), -F::INFINITY);
    assert_total_eq!(F::INFINITY.log10(), F::INFINITY);
}

pub(crate) fn test_log10_1p_special<F: TestFloat + swfp::math::Log>() {
    let f = |s: &str| s.parse::<F>().unwrap();
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.log10_1p());
    assert_is_nan!(f("-1.5").log10_1p());
    assert_is_nan!((-F::INFINITY).log10_1p());
    assert_total_eq!((-one).log10_1p(), -F::INFINITY);
    assert_total_eq!((-F::ZERO).log10_1p(), -F::ZERO);
    assert_total_eq!(F::ZERO.log10_1p(), F::ZERO);
    assert_total_eq!(F::INFINITY.log10_1p(), F::INFINITY);
}

pub(crate) fn test_pow_special<F: TestFloat + swfp::math::Pow>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);
    let two = F::from_int(2);

    assert_is_nan!(F::NAN.pow(F::NAN));
    assert_is_nan!(F::ZERO.pow(F::NAN));
    assert_is_nan!((-F::ZERO).pow(F::NAN));
    assert_is_nan!(two.pow(F::NAN));
    assert_is_nan!(F::INFINITY.pow(F::NAN));
    assert_is_nan!((-F::INFINITY).pow(F::NAN));
    assert_is_nan!(F::NAN.pow(one));
    assert_is_nan!(F::NAN.pow(F::INFINITY));
    assert_is_nan!(F::NAN.pow(-F::INFINITY));
    assert_is_nan!(f("-3").pow(f("0.5")));
    assert_total_eq!(F::ZERO.pow(f("-33")), F::INFINITY);
    assert_total_eq!((-F::ZERO).pow(f("-33")), -F::INFINITY);
    assert_total_eq!(F::ZERO.pow(f("-33.5")), F::INFINITY);
    assert_total_eq!((-F::ZERO).pow(f("-33.5")), F::INFINITY);
    assert_total_eq!(F::ZERO.pow(f("-34")), F::INFINITY);
    assert_total_eq!((-F::ZERO).pow(f("-34")), F::INFINITY);
    assert_total_eq!(F::ZERO.pow(f("33")), F::ZERO);
    assert_total_eq!((-F::ZERO).pow(f("33")), -F::ZERO);
    assert_total_eq!(F::ZERO.pow(f("33.5")), F::ZERO);
    assert_total_eq!((-F::ZERO).pow(f("33.5")), F::ZERO);
    assert_total_eq!(F::ZERO.pow(f("34.0")), F::ZERO);
    assert_total_eq!((-F::ZERO).pow(f("34.0")), F::ZERO);
    assert_total_eq!(F::ZERO.pow(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).pow(F::INFINITY), F::ZERO);
    assert_total_eq!(F::ZERO.pow(-F::INFINITY), F::INFINITY);
    assert_total_eq!((-F::ZERO).pow(-F::INFINITY), F::INFINITY);
    assert_total_eq!(one.pow(F::ZERO), one);
    assert_total_eq!(one.pow(-F::ZERO), one);
    assert_total_eq!(one.pow(f("33")), one);
    assert_total_eq!(one.pow(f("-33")), one);
    assert_total_eq!(one.pow(f("33.5")), one);
    assert_total_eq!(one.pow(f("-33.5")), one);
    assert_total_eq!(one.pow(f("34")), one);
    assert_total_eq!(one.pow(f("-34.0")), one);
    assert_total_eq!(one.pow(F::INFINITY), one);
    assert_total_eq!(one.pow(-F::INFINITY), one);
    assert_total_eq!(one.pow(F::NAN), one);
    assert_total_eq!((-one).pow(F::INFINITY), one);
    assert_total_eq!((-one).pow(-F::INFINITY), one);
    assert_total_eq!(f("0.5").pow(F::INFINITY), F::ZERO);
    assert_total_eq!(f("0.5").pow(-F::INFINITY), F::INFINITY);
    assert_total_eq!(f("-0.5").pow(F::INFINITY), F::ZERO);
    assert_total_eq!(f("-0.5").pow(-F::INFINITY), F::INFINITY);
    assert_total_eq!(f("1.5").pow(F::INFINITY), F::INFINITY);
    assert_total_eq!(f("1.5").pow(-F::INFINITY), F::ZERO);
    assert_total_eq!(f("-1.5").pow(F::INFINITY), F::INFINITY);
    assert_total_eq!(f("-1.5").pow(-F::INFINITY), F::ZERO);
    assert_total_eq!(F::INFINITY.pow(F::ZERO), one);
    assert_total_eq!(F::INFINITY.pow(-F::ZERO), one);
    assert_total_eq!(F::INFINITY.pow(f("33")), F::INFINITY);
    assert_total_eq!(F::INFINITY.pow(f("-33")), F::ZERO);
    assert_total_eq!(F::INFINITY.pow(f("33.5")), F::INFINITY);
    assert_total_eq!(F::INFINITY.pow(f("-33.5")), F::ZERO);
    assert_total_eq!(F::INFINITY.pow(f("34.0")), F::INFINITY);
    assert_total_eq!(F::INFINITY.pow(f("-34.0")), F::ZERO);
    assert_total_eq!((-F::INFINITY).pow(F::ZERO), one);
    assert_total_eq!((-F::INFINITY).pow(-F::ZERO), one);
    assert_total_eq!((-F::INFINITY).pow(f("33")), -F::INFINITY);
    assert_total_eq!((-F::INFINITY).pow(f("-33")), -F::ZERO);
    assert_total_eq!((-F::INFINITY).pow(f("33.5")), F::INFINITY);
    assert_total_eq!((-F::INFINITY).pow(f("-33.5")), F::ZERO);
    assert_total_eq!((-F::INFINITY).pow(f("34.0")), F::INFINITY);
    assert_total_eq!((-F::INFINITY).pow(f("-34.0")), F::ZERO);
    assert_total_eq!(two.pow(two), f("4"));
    assert_total_eq!(two.pow(-two), f("0.25"));
    assert_total_eq!((-two).pow(f("3")), f("-8"));
    assert_total_eq!((-two).pow(f("-3")), f("-0.125"));
    assert_total_eq!(f("3.5").pow(f("3")), f("42.875"));
    assert_total_eq!(f("10").pow(f("4")), f("10000"));
}

pub(crate) fn test_powi_special<F: TestFloat + swfp::math::Pow>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);
    let two = F::from_int(2);

    assert_is_nan!(F::NAN.powi(1));
    assert_total_eq!(F::ZERO.powi(-33), F::INFINITY);
    assert_total_eq!((-F::ZERO).powi(-33), -F::INFINITY);
    assert_total_eq!(F::ZERO.powi(-34), F::INFINITY);
    assert_total_eq!((-F::ZERO).powi(-34), F::INFINITY);
    assert_total_eq!(F::ZERO.powi(33), F::ZERO);
    assert_total_eq!((-F::ZERO).powi(33), -F::ZERO);
    assert_total_eq!(F::ZERO.powi(34), F::ZERO);
    assert_total_eq!((-F::ZERO).powi(34), F::ZERO);
    assert_total_eq!(one.powi(0), one);
    assert_total_eq!(one.powi(33), one);
    assert_total_eq!(one.powi(-33), one);
    assert_total_eq!(one.powi(34), one);
    assert_total_eq!(one.powi(-34), one);
    assert_total_eq!(one.powi(i32::MAX), one);
    assert_total_eq!(one.powi(i32::MIN), one);
    assert_total_eq!(F::INFINITY.powi(0), one);
    assert_total_eq!(F::INFINITY.powi(33), F::INFINITY);
    assert_total_eq!(F::INFINITY.powi(-33), F::ZERO);
    assert_total_eq!(F::INFINITY.powi(34), F::INFINITY);
    assert_total_eq!(F::INFINITY.powi(-34), F::ZERO);
    assert_total_eq!((-F::INFINITY).powi(0), one);
    assert_total_eq!((-F::INFINITY).powi(-0), one);
    assert_total_eq!((-F::INFINITY).powi(33), -F::INFINITY);
    assert_total_eq!((-F::INFINITY).powi(-33), -F::ZERO);
    assert_total_eq!((-F::INFINITY).powi(34), F::INFINITY);
    assert_total_eq!((-F::INFINITY).powi(-34), F::ZERO);
    assert_total_eq!(two.powi(2), f("4"));
    assert_total_eq!(two.powi(-2), f("0.25"));
    assert_total_eq!((-two).powi(3), f("-8"));
    assert_total_eq!((-two).powi(-3), f("-0.125"));
    assert_total_eq!(f("3.5").powi(3), f("42.875"));
    assert_total_eq!(f("10").powi(4), f("10000"));
}

pub(crate) fn test_sin_cos_special<F: TestFloat + swfp::math::Trigonometric>() {
    let one = F::from_int(1);

    let test_nan = |arg: F| {
        let sin1 = arg.sin();
        let cos1 = arg.cos();
        let (sin2, cos2) = arg.sin_cos();
        assert_is_nan!(sin1);
        assert_is_nan!(cos1);
        assert_is_nan!(sin2);
        assert_is_nan!(cos2);
    };

    let test_value = |arg: F, expected_sin: F, expected_cos: F| {
        let sin1 = arg.sin();
        let cos1 = arg.cos();
        let (sin2, cos2) = arg.sin_cos();
        assert_total_eq!(sin1, expected_sin);
        assert_total_eq!(cos1, expected_cos);
        assert_total_eq!(sin2, expected_sin);
        assert_total_eq!(cos2, expected_cos);
    };

    test_nan(F::NAN);
    test_nan(F::INFINITY);
    test_nan(-F::INFINITY);
    test_value(F::ZERO, F::ZERO, one);
    test_value(-F::ZERO, -F::ZERO, one);
}

pub(crate) fn test_tan_special<F: TestFloat + swfp::math::Trigonometric>() {
    assert_is_nan!(F::NAN.tan());
    assert_is_nan!(F::INFINITY.tan());
    assert_is_nan!((-F::INFINITY).tan());
    assert_total_eq!(F::ZERO.tan(), F::ZERO);
    assert_total_eq!((-F::ZERO).tan(), -F::ZERO);
}

pub(crate) fn test_sind_cosd_special<F: TestFloat + swfp::math::Trigonometric>() {
    let one = F::from_int(1);

    let test_nan = |arg: F| {
        let sin1 = arg.sind();
        let cos1 = arg.cosd();
        let (sin2, cos2) = arg.sind_cosd();
        assert_is_nan!(sin1);
        assert_is_nan!(cos1);
        assert_is_nan!(sin2);
        assert_is_nan!(cos2);
    };

    let test_value = |arg: F, expected_sin: F, expected_cos: F| {
        let sin1 = arg.sind();
        let cos1 = arg.cosd();
        let (sin2, cos2) = arg.sind_cosd();
        assert_total_eq!(sin1, expected_sin);
        assert_total_eq!(cos1, expected_cos);
        assert_total_eq!(sin2, expected_sin);
        assert_total_eq!(cos2, expected_cos);
    };

    test_nan(F::NAN);
    test_nan(F::INFINITY);
    test_nan(-F::INFINITY);
    test_value(F::ZERO, F::ZERO, one);
    test_value(-F::ZERO, -F::ZERO, one);
}

pub(crate) fn test_tand_special<F: TestFloat + swfp::math::Trigonometric>() {
    assert_is_nan!(F::NAN.tand());
    assert_is_nan!(F::INFINITY.tand());
    assert_is_nan!((-F::INFINITY).tand());
    assert_total_eq!(F::ZERO.tand(), F::ZERO);
    assert_total_eq!((-F::ZERO).tand(), -F::ZERO);
}

pub(crate) fn test_sinpi_cospi_special<F: TestFloat + swfp::math::Trigonometric>() {
    let one = F::from_int(1);

    let test_nan = |arg: F| {
        let sin1 = arg.sinpi();
        let cos1 = arg.cospi();
        let (sin2, cos2) = arg.sinpi_cospi();
        assert_is_nan!(sin1);
        assert_is_nan!(cos1);
        assert_is_nan!(sin2);
        assert_is_nan!(cos2);
    };

    let test_value = |arg: F, expected_sin: F, expected_cos: F| {
        let sin1 = arg.sinpi();
        let cos1 = arg.cospi();
        let (sin2, cos2) = arg.sinpi_cospi();
        assert_total_eq!(sin1, expected_sin);
        assert_total_eq!(cos1, expected_cos);
        assert_total_eq!(sin2, expected_sin);
        assert_total_eq!(cos2, expected_cos);
    };

    test_nan(F::NAN);
    test_nan(F::INFINITY);
    test_nan(-F::INFINITY);
    test_value(F::ZERO, F::ZERO, one);
    test_value(-F::ZERO, -F::ZERO, one);
}

pub(crate) fn test_tanpi_special<F: TestFloat + swfp::math::Trigonometric>() {
    assert_is_nan!(F::NAN.tanpi());
    assert_is_nan!(F::INFINITY.tanpi());
    assert_is_nan!((-F::INFINITY).tanpi());
    assert_total_eq!(F::ZERO.tanpi(), F::ZERO);
    assert_total_eq!((-F::ZERO).tanpi(), -F::ZERO);
}

pub(crate) fn test_asin_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.asin());
    assert_is_nan!(f("1.5").asin());
    assert_is_nan!(f("-1.5").asin());
    assert_is_nan!(F::INFINITY.asin());
    assert_is_nan!((-F::INFINITY).asin());
    assert_total_eq!(F::ZERO.asin(), F::ZERO);
    assert_total_eq!((-F::ZERO).asin(), -F::ZERO);
}

pub(crate) fn test_acos_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.acos());
    assert_is_nan!(f("1.5").acos());
    assert_is_nan!(f("-1.5").acos());
    assert_is_nan!(F::INFINITY.acos());
    assert_is_nan!((-F::INFINITY).acos());
}

pub(crate) fn test_atan_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    assert_is_nan!(F::NAN.atan());
    assert_total_eq!(F::ZERO.atan(), F::ZERO);
    assert_total_eq!((-F::ZERO).atan(), -F::ZERO);
}

pub(crate) fn test_atan2_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.atan2(one));
    assert_is_nan!(F::NAN.atan2(F::ZERO));
    assert_is_nan!(F::NAN.atan2(F::INFINITY));
    assert_is_nan!(F::NAN.atan2(F::NAN));
    assert_is_nan!(F::INFINITY.atan2(F::NAN));
    assert_is_nan!(F::ZERO.atan2(F::NAN));
    assert_is_nan!(one.atan2(F::NAN));
    assert_total_eq!(F::ZERO.atan2(F::ZERO), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2(F::ZERO), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2(one), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2(one), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2(F::INFINITY), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2(F::INFINITY), -F::ZERO);
    assert_total_eq!(one.atan2(F::INFINITY), F::ZERO);
    assert_total_eq!((-one).atan2(F::INFINITY), -F::ZERO);
}

pub(crate) fn test_asind_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.asind());
    assert_is_nan!(f("1.5").asind());
    assert_is_nan!(f("-1.5").asind());
    assert_is_nan!(F::INFINITY.asind());
    assert_is_nan!((-F::INFINITY).asind());
    assert_total_eq!(F::ZERO.asind(), F::ZERO);
    assert_total_eq!((-F::ZERO).asind(), -F::ZERO);
}

pub(crate) fn test_acosd_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.acosd());
    assert_is_nan!(f("1.5").acosd());
    assert_is_nan!(f("-1.5").acosd());
    assert_is_nan!(F::INFINITY.acosd());
    assert_is_nan!((-F::INFINITY).acosd());
}

pub(crate) fn test_atand_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    assert_is_nan!(F::NAN.atand());
    assert_total_eq!(F::ZERO.atand(), F::ZERO);
    assert_total_eq!((-F::ZERO).atand(), -F::ZERO);
}

pub(crate) fn test_atan2d_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.atan2d(one));
    assert_is_nan!(F::NAN.atan2d(F::ZERO));
    assert_is_nan!(F::NAN.atan2d(F::INFINITY));
    assert_is_nan!(F::NAN.atan2d(F::NAN));
    assert_is_nan!(F::INFINITY.atan2d(F::NAN));
    assert_is_nan!(F::ZERO.atan2d(F::NAN));
    assert_is_nan!(one.atan2d(F::NAN));
    assert_total_eq!(F::ZERO.atan2d(F::ZERO), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2d(F::ZERO), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2d(one), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2d(one), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2d(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2d(F::INFINITY), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2d(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2d(F::INFINITY), -F::ZERO);
    assert_total_eq!(one.atan2d(F::INFINITY), F::ZERO);
    assert_total_eq!((-one).atan2d(F::INFINITY), -F::ZERO);
}

pub(crate) fn test_asinpi_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.asinpi());
    assert_is_nan!(f("1.5").asinpi());
    assert_is_nan!(f("-1.5").asinpi());
    assert_is_nan!(F::INFINITY.asinpi());
    assert_is_nan!((-F::INFINITY).asinpi());
    assert_total_eq!(F::ZERO.asinpi(), F::ZERO);
    assert_total_eq!((-F::ZERO).asinpi(), -F::ZERO);
}

pub(crate) fn test_acospi_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    assert_is_nan!(F::NAN.acospi());
    assert_is_nan!(f("1.5").acospi());
    assert_is_nan!(f("-1.5").acospi());
    assert_is_nan!(F::INFINITY.acospi());
    assert_is_nan!((-F::INFINITY).acospi());
}

pub(crate) fn test_atanpi_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    assert_is_nan!(F::NAN.atanpi());
    assert_total_eq!(F::ZERO.atanpi(), F::ZERO);
    assert_total_eq!((-F::ZERO).atanpi(), -F::ZERO);
}

pub(crate) fn test_atan2pi_special<F: TestFloat + swfp::math::InvTrigonometric>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.atan2pi(one));
    assert_is_nan!(F::NAN.atan2pi(F::ZERO));
    assert_is_nan!(F::NAN.atan2pi(F::INFINITY));
    assert_is_nan!(F::NAN.atan2pi(F::NAN));
    assert_is_nan!(F::INFINITY.atan2pi(F::NAN));
    assert_is_nan!(F::ZERO.atan2pi(F::NAN));
    assert_is_nan!(one.atan2pi(F::NAN));
    assert_total_eq!(F::ZERO.atan2pi(F::ZERO), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2pi(F::ZERO), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2pi(one), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2pi(one), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2pi(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2pi(F::INFINITY), -F::ZERO);
    assert_total_eq!(F::ZERO.atan2pi(F::INFINITY), F::ZERO);
    assert_total_eq!((-F::ZERO).atan2pi(F::INFINITY), -F::ZERO);
    assert_total_eq!(one.atan2pi(F::INFINITY), F::ZERO);
    assert_total_eq!((-one).atan2pi(F::INFINITY), -F::ZERO);
}

pub(crate) fn test_sinh_cosh_special<F: TestFloat + swfp::math::Hyperbolic>() {
    let one = F::from_int(1);

    let test_nan = |arg: F| {
        let sinh1 = arg.sinh();
        let cosh1 = arg.cosh();
        let (sinh2, cosh2) = arg.sinh_cosh();
        assert_is_nan!(sinh1);
        assert_is_nan!(cosh1);
        assert_is_nan!(sinh2);
        assert_is_nan!(cosh2);
    };

    let test_value = |arg: F, expected_sinh: F, expected_cosh: F| {
        let sinh1 = arg.sinh();
        let cosh1 = arg.cosh();
        let (sinh2, cosh2) = arg.sinh_cosh();
        assert_total_eq!(sinh1, expected_sinh);
        assert_total_eq!(cosh1, expected_cosh);
        assert_total_eq!(sinh2, expected_sinh);
        assert_total_eq!(cosh2, expected_cosh);
    };

    test_nan(F::NAN);
    test_value(F::INFINITY, F::INFINITY, F::INFINITY);
    test_value(-F::INFINITY, -F::INFINITY, F::INFINITY);
    test_value(F::ZERO, F::ZERO, one);
    test_value(-F::ZERO, -F::ZERO, one);
}

pub(crate) fn test_tanh_special<F: TestFloat + swfp::math::Hyperbolic>() {
    let one = F::from_int(1);

    assert_is_nan!(F::NAN.tanh());
    assert_total_eq!(F::INFINITY.tanh(), one);
    assert_total_eq!((-F::INFINITY).tanh(), -one);
    assert_total_eq!(F::ZERO.tanh(), F::ZERO);
    assert_total_eq!((-F::ZERO).tanh(), -F::ZERO);
}

pub(crate) fn test_asinh_special<F: TestFloat + swfp::math::InvHyperbolic>() {
    assert_is_nan!(F::NAN.asinh());
    assert_total_eq!(F::INFINITY.asinh(), F::INFINITY);
    assert_total_eq!((-F::INFINITY).asinh(), -F::INFINITY);
    assert_total_eq!(F::ZERO.asinh(), F::ZERO);
    assert_total_eq!((-F::ZERO).asinh(), -F::ZERO);
}

pub(crate) fn test_acosh_special<F: TestFloat + swfp::math::InvHyperbolic>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);

    assert_is_nan!(F::NAN.acosh());
    assert_is_nan!((-F::INFINITY).acosh());
    assert_is_nan!((-one).acosh());
    assert_is_nan!(F::ZERO.acosh());
    assert_is_nan!(f("0.5").acosh());
    assert_total_eq!(F::INFINITY.acosh(), F::INFINITY);
    assert_total_eq!(one.acosh(), F::ZERO);
}

pub(crate) fn test_atanh_special<F: TestFloat + swfp::math::InvHyperbolic>() {
    let f = |s: &str| s.parse::<F>().unwrap();

    let one = F::from_int(1);

    assert_is_nan!(F::NAN.atanh());
    assert_is_nan!(f("1.5").atanh());
    assert_is_nan!(f("-1.5").atanh());
    assert_is_nan!(F::INFINITY.atanh());
    assert_is_nan!((-F::INFINITY).atanh());
    assert_total_eq!(F::ZERO.atanh(), F::ZERO);
    assert_total_eq!((-F::ZERO).atanh(), -F::ZERO);
    assert_total_eq!((one).atanh(), F::INFINITY);
    assert_total_eq!((-one).atanh(), -F::INFINITY);
}

pub(crate) fn test_gamma_special<F: TestFloat + swfp::math::Gamma>() {
    let one = F::from_int(1);
    let two = F::from_int(2);

    assert_is_nan!(F::NAN.gamma());
    assert_is_nan!((-F::INFINITY).gamma());
    assert_is_nan!((-one).gamma());
    assert_is_nan!((-two).gamma());
    assert_total_eq!(F::INFINITY.gamma(), F::INFINITY);
    assert_total_eq!(F::ZERO.gamma(), F::INFINITY);
    assert_total_eq!((-F::ZERO).gamma(), -F::INFINITY);
    assert_total_eq!(one.gamma(), one);
    assert_total_eq!(two.gamma(), one);
}

pub(crate) fn test_ln_gamma_special<F: TestFloat + swfp::math::Gamma>() {
    let one = F::from_int(1);
    let two = F::from_int(2);

    let test_nan = |x: F| {
        let (r, sign) = x.ln_gamma();
        assert_is_nan!(r);
        assert_eq!(sign, 0);
    };
    let test_value = |x: F, r: F, sign: i8| {
        let (res, res_sign) = x.ln_gamma();
        assert_total_eq!(res, r);
        assert_eq!(res_sign, sign);
    };

    test_nan(F::NAN);
    test_nan(-F::INFINITY);
    test_value(F::INFINITY, F::INFINITY, 1);
    test_value(F::ZERO, F::INFINITY, 1);
    test_value(-F::ZERO, F::INFINITY, -1);
    test_value(-one, F::INFINITY, 0);
    test_value(-two, F::INFINITY, 0);
    test_value(one, F::ZERO, 1);
    test_value(two, F::ZERO, 1);
}
