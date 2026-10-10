use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_from_str_exact, check_from_str_round,
    check_round_int_exact, check_round_int_round, mk_f64, test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for hi in 0..=0xFFFF {
        for lo in 0..=0xF {
            let bits = (hi << (64 - 16)) | lo;
            let new_bits = swfp::F64::from_bits(bits).to_bits();
            if new_bits != bits {
                panic!("0x{new_bits:04X} != 0x{bits:04X}");
            }
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F64>(true, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        for hi in 0..=255 {
            for lo in 0..=255 {
                let m = (hi << (52 - 8)) | lo;
                let category = if m == 0 {
                    FpCategory::Zero
                } else {
                    FpCategory::Subnormal
                };
                check_category(mk_f64(s, -1023, m), category, s);
            }
        }

        for e in -1022..=1023 {
            for hi in 0..=255 {
                for lo in 0..=255 {
                    let m = (hi << (52 - 8)) | lo;
                    check_category(mk_f64(s, e, m), FpCategory::Normal, s);
                }
            }
        }

        for hi in 0..=255 {
            for lo in 0..=255 {
                let m = (hi << (52 - 8)) | lo;
                let category = if m == 0 {
                    FpCategory::Infinite
                } else {
                    FpCategory::Nan
                };
                check_category(mk_f64(s, 1024, m), category, s);
            }
        }
    }
}

#[test]
fn test_convert_to_self() {
    for hi in 0..=0xFFFF {
        for lo in 0..=0xF {
            let bits = (hi << (64 - 16)) | lo;
            let value = swfp::F64::from_bits(bits);
            for round in ALL_ROUND_MODES {
                let (new_value, status) = swfp::F64::convert_from_ex(value, round);
                let new_bits = new_value.to_bits();

                let (expected_status, expected_bits) = if value.is_nan() && bits & (1 << 51) == 0 {
                    (FpStatus::INVALID, bits | (1 << 51))
                } else {
                    (FpStatus::OK, bits)
                };
                assert_eq!(status, expected_status);
                assert_eq!(new_bits, expected_bits);
            }
        }
    }
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::F64>();

    let exact = [
        (
            "1.79769313486231570814527423731704356798070567525844996598917476803157260780\
             0285387605895586327668781715404589535143824642343213268894641827684675467035\
             3751698604991057655128207624549009038932894407586850845513394230458323690322\
             2948165808559332123348274797826204144723168738177180919299881250404026184124\
             858368e308",
            swfp::F64::from_bits(0x7FEFFFFFFFFFFFFF),
        ),
        ("9007199254740991", swfp::F64::from_bits(0x433FFFFFFFFFFFFF)),
        ("9007199254740992", swfp::F64::from_bits(0x4340000000000000)),
        (
            "4503599627370495.5",
            swfp::F64::from_bits(0x432FFFFFFFFFFFFF),
        ),
        (
            "0.50000000000000011102230246251565404236316680908203125",
            swfp::F64::from_bits(0x3FE0000000000001),
        ),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F64::from_int(n));
    }

    for e in -1074..=1023 {
        let s = format!("{:.1000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F64::from_int(1).scalbn(e));
    }

    // (input, rounded toward zero, rounded away from zero, loss)
    let inexact = [
        // Around the overflow threshold
        (
            "1.79769313486231580793728971405303415079934132710037826936173778980444968292\
             7647509466490179775872070963302864166928879109465555478519404026306574886715\
             0582068190890200070838367627385484581771153176447573027006985557136695962284\
             2914819860834936475292719074168444365510704342711559699508093042880177904174\
             497792e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfOdd,
        ),
        (
            "1.797693134862315807937289714053034150799e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfDown,
        ),
        (
            "1.797693134862315807937289714053034150800e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfUp,
        ),
        (
            "1.7976931348623157e308",
            0x7FEFFFFFFFFFFFFE,
            0x7FEFFFFFFFFFFFFF,
            Loss::HalfUp,
        ),
        (
            "1.7976931348623158e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfDown,
        ),
        (
            "1.7976931348623159e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfUp,
        ),
        (
            "1.797693134862315907729305190789024733617e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::HalfUp,
        ),
        (
            "1.797693134862315907729305190789024733618e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        (
            "1.79769313486231590772930519078902473361797697894230657273430081157732675805\
             5009631327084773224075360211201138798713933576587897688144166224928474306394\
             7412437776789342486548527630221960124609411945308295208500576883815068234246\
             2881473913110540827237163350510684586298239947245938479716304835356329624224\
             137216e308",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        (
            "1e309",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        (
            "1111111111111111111111111111111111111111111111111111111111111111111111111111\
             1111111111111111111111111111111111111111111111111111111111111111111111111111\
             1111111111111111111111111111111111111111111111111111111111111111111111111111\
             1111111111111111111111111111111111111111111111111111111111111111111111111111\
             111111",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        (
            "1e99999999999999999999",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        (
            "1e9223372036854775807",
            0x7FEFFFFFFFFFFFFF,
            0x7FF0000000000000,
            Loss::Overflow,
        ),
        // Around the underflow threshold
        (
            "2.47032822920623272088284396434110686182529901307162382212792841250337753635\
             1043759326499181808179961898982823477228588654633283551779698981993873980053\
             9093906315035659515570226392290858392449105184435931802849936536152500319370\
             4576782492193656236698636584807570015857692699037063119282795585513329278343\
             3840935197801553124659726357957462276646527282722005637400648549997709659947\
             0454020828166226237857393450736339007967761930577506740176324673600968951340\
             5355374585166611342237666786041621596804619144672918403005300575308490487653\
             9171138659164623952491262365388187963623937328042389101867234849766823508986\
             3388587925628302755995657524455507255189313690836254779186948667994968324049\
             705821028513185451396213837722826145437693412532098591327667236328125e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfEven,
        ),
        (
            "2.470328229206232720882843964341106861825e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfDown,
        ),
        (
            "2.470328229206232720882843964341106861826e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfUp,
        ),
        (
            "2.4703282292062327e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfDown,
        ),
        (
            "2.4703282292062328e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfUp,
        ),
        (
            "4.9406564584124654e-324",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfUp,
        ),
        (
            "7.410984687618698162648531893023320585475e-324",
            0x0000000000000001,
            0x0000000000000002,
            Loss::HalfDown,
        ),
        (
            "7.410984687618698162648531893023320585476e-324",
            0x0000000000000001,
            0x0000000000000002,
            Loss::HalfUp,
        ),
        (
            "2.22507385850720113605740979670913197593481954635164564802342610972482222202\
             1076945516529523908135087914149158913039621106870086438694594645527657207407\
             8206217433799881410632673292535522868813721490129811224514518898490572223072\
             8525513315575501591439747639798341180199932396254828901710708185069063066665\
             5994938275772572015763062690663332647565300009245888316433037779791869612049\
             4973903778297049050510806099407302629371289589500035837999672072543043602840\
             7889577179615094551674824347103070260914462157228988025818254518032570701886\
             0872113128079512233426288368622321503775666622503982534335974568884423900265\
             4981983854879482922068947216898310996983658468140228542433306603398508864458\
             0400103493397042756718644338377048603786162277173854562306587467901408672332\
             763671875e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfOdd,
        ),
        (
            "2.22507385850720113605740979670913197593481954635164564802342610972482222202\
             1076945516529523908135087914149158913039621106870086438694594645527657207407\
             8206217433799881410632673292535522868813721490129811224514518898490572223072\
             8525513315575501591439747639798341180199932396254828901710708185069063066665\
             5994938275772572015763062690663332647565300009245888316433037779791869612049\
             4973903778297049050510806099407302629371289589500035837999672072543043602840\
             7889577179615094551674824347103070260914462157228988025818254518032570701886\
             0872113128079512233426288368622321503775666622503982534335974568884423900265\
             4981983854879482922068947216898310996983658468140228542433306603398508864458\
             0400103493397042756718644338377048603786162277173854562306587467901408672332\
             763671875000000000000000000001e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfUp,
        ),
        (
            "2.225073858507201136057409796709131975934e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfDown,
        ),
        (
            "2.225073858507201136057409796709131975935e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfUp,
        ),
        (
            "2.2250738585072011e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfDown,
        ),
        (
            "2.2250738585072012e-308",
            0x000FFFFFFFFFFFFF,
            0x0010000000000000,
            Loss::HalfUp,
        ),
        (
            "2.2250738585072014e-308",
            0x0010000000000000,
            0x0010000000000001,
            Loss::HalfDown,
        ),
        (
            "1e-400",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfDown,
        ),
        (
            "1e-99999999999999999999",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfDown,
        ),
        (
            "1e-9223372036854775808",
            0x0000000000000000,
            0x0000000000000001,
            Loss::HalfDown,
        ),
        // Ties in the normal range
        (
            "9007199254740992.999999999999999999999999999999999999999999999",
            0x4340000000000000,
            0x4340000000000001,
            Loss::HalfDown,
        ),
        (
            "9007199254740993",
            0x4340000000000000,
            0x4340000000000001,
            Loss::HalfEven,
        ),
        (
            "9007199254740993.0000000000000000000000000000000000000000000001",
            0x4340000000000000,
            0x4340000000000001,
            Loss::HalfUp,
        ),
        (
            "9007199254740995",
            0x4340000000000001,
            0x4340000000000002,
            Loss::HalfOdd,
        ),
        (
            "4503599627370496.5",
            0x4330000000000000,
            0x4330000000000001,
            Loss::HalfEven,
        ),
        (
            "4503599627370497.5",
            0x4330000000000001,
            0x4330000000000002,
            Loss::HalfOdd,
        ),
        (
            "1e23",
            0x44B52D02C7E14AF6,
            0x44B52D02C7E14AF7,
            Loss::HalfEven,
        ),
        (
            "1.00000000000000011102230246251565404236316680908203124999999999999999999999\
             999999999999999999999999",
            0x3FF0000000000000,
            0x3FF0000000000001,
            Loss::HalfDown,
        ),
        (
            "1.00000000000000011102230246251565404236316680908203125",
            0x3FF0000000000000,
            0x3FF0000000000001,
            Loss::HalfEven,
        ),
        (
            "1.00000000000000011102230246251565404236316680908203125000000000000000000000\
             0000000000000000000000001",
            0x3FF0000000000000,
            0x3FF0000000000001,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000033306690738754696212708950042724609375",
            0x3FF0000000000001,
            0x3FF0000000000002,
            Loss::HalfOdd,
        ),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3FEFFFFFFFFFFFFF,
            0x3FF0000000000000,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x3FF0000000000000,
            0x3FF0000000000001,
            Loss::HalfDown,
        ),
        // Very close to the midpoint between two consecutive representable values
        (
            "9e-265",
            0x091D05244FE50669,
            0x091D05244FE5066A,
            Loss::HalfUp,
        ),
        (
            "85e-37",
            0x38A698CCDC600159,
            0x38A698CCDC60015A,
            Loss::HalfUp,
        ),
        (
            "623e+100",
            0x554640A62F3A83DE,
            0x554640A62F3A83DF,
            Loss::HalfUp,
        ),
        (
            "3571e+263",
            0x77462644C61D41A9,
            0x77462644C61D41AA,
            Loss::HalfUp,
        ),
        (
            "81661e+153",
            0x60B7CA8E3D68578D,
            0x60B7CA8E3D68578E,
            Loss::HalfUp,
        ),
        (
            "920657e-23",
            0x3C653A9985DBDE6B,
            0x3C653A9985DBDE6C,
            Loss::HalfUp,
        ),
        (
            "87575437e-309",
            0x016E07320602056B,
            0x016E07320602056C,
            Loss::HalfUp,
        ),
        (
            "245540327e+122",
            0x5B01B6231E18C5CA,
            0x5B01B6231E18C5CB,
            Loss::HalfUp,
        ),
        (
            "83356057653e+193",
            0x6A4544E6DAEE2A17,
            0x6A4544E6DAEE2A18,
            Loss::HalfUp,
        ),
        (
            "619534293513e+124",
            0x5C210C20303FE0F0,
            0x5C210C20303FE0F1,
            Loss::HalfUp,
        ),
        (
            "2335141086879e+218",
            0x6FC340A1C932C1ED,
            0x6FC340A1C932C1EE,
            Loss::HalfUp,
        ),
        (
            "36167929443327e-159",
            0x21BCE77C2B3328FB,
            0x21BCE77C2B3328FC,
            Loss::HalfUp,
        ),
        (
            "609610927149051e-255",
            0x0E104273B18918B0,
            0x0E104273B18918B1,
            Loss::HalfUp,
        ),
        (
            "3743626360493413e-165",
            0x20E8823A57ADBEF8,
            0x20E8823A57ADBEF9,
            Loss::HalfUp,
        ),
        (
            "94080055902682397e-242",
            0x11364981E39E66C9,
            0x11364981E39E66CA,
            Loss::HalfUp,
        ),
        (
            "899810892172646163e+283",
            0x7E6ADF51FA055E02,
            0x7E6ADF51FA055E03,
            Loss::HalfUp,
        ),
        (
            "7120190517612959703e+120",
            0x5CC3220DCD5899FC,
            0x5CC3220DCD5899FD,
            Loss::HalfUp,
        ),
        (
            "25188282901709339043e-252",
            0x0FA4059AF3DB2A83,
            0x0FA4059AF3DB2A84,
            Loss::HalfUp,
        ),
        (
            "308984926168550152811e-52",
            0x39640DE48676653A,
            0x39640DE48676653B,
            Loss::HalfUp,
        ),
        (
            "6372891218502368041059e+64",
            0x51C067047DBB38FD,
            0x51C067047DBB38FE,
            Loss::HalfUp,
        ),
        // Other values
        ("0.1", 0x3FB9999999999999, 0x3FB999999999999A, Loss::HalfUp),
        ("0.2", 0x3FC9999999999999, 0x3FC999999999999A, Loss::HalfUp),
        (
            "0.3",
            0x3FD3333333333333,
            0x3FD3333333333334,
            Loss::HalfDown,
        ),
        (
            "0.7",
            0x3FE6666666666666,
            0x3FE6666666666667,
            Loss::HalfDown,
        ),
        (
            "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
            0x3FD5555555555555,
            0x3FD5555555555556,
            Loss::HalfDown,
        ),
        (
            "1.50000000000000000000000000000000000000000000000000000000000000000001",
            0x3FF8000000000000,
            0x3FF8000000000001,
            Loss::HalfDown,
        ),
        (
            "3.14159265358979323846264338327950288",
            0x400921FB54442D18,
            0x400921FB54442D19,
            Loss::HalfDown,
        ),
        (
            "8.98846567431158e307",
            0x7FE0000000000000,
            0x7FE0000000000001,
            Loss::HalfDown,
        ),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::F64::from_bits(tz);
        let az = swfp::F64::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }
}

#[test]
fn test_fmt() {
    fn test(host_value: f64) {
        let expected = format!("{host_value}");
        let actual = format!("{}", swfp::F64::from_host(host_value));
        assert_eq!(expected, actual);

        let expected = format!("{host_value:e}");
        let actual = format!("{:e}", swfp::F64::from_host(host_value));
        assert_eq!(expected, actual);

        let expected = format!("{host_value:.100e}");
        let actual = format!("{:.100e}", swfp::F64::from_host(host_value));
        assert_eq!(expected, actual);

        let expected = format!("{host_value:.3e}");
        let actual = format!("{:.3e}", swfp::F64::from_host(host_value));
        assert_eq!(expected, actual);
    }

    test(f64::MIN_POSITIVE);
    test(f64::MAX);
    test(f64::MIN);
    test(f64::EPSILON);

    for i in 0..=100_000 {
        test(i as f64);
        test(-(i as f64));
    }

    for i in 0..52 {
        test(f64::from_bits(1 << i));
    }

    for i in 0..51 {
        test(f64::from_bits((1 << i) | (1 << 52)));
    }
}

#[test]
fn test_fmt_display() {
    let v = swfp::F64::from_int(1);
    assert_eq!(format!("{v}"), "1");

    let v = swfp::F64::from_bits(0x3FF0000000000001);
    assert_eq!(format!("{v}"), "1.0000000000000002");

    let v = swfp::F64::from_bits(1); // min subnormal
    assert_eq!(format!("{v}"), format!("0.{}5", "0".repeat(323)));

    let v = swfp::F64::from_host(f64::MIN_POSITIVE);
    assert_eq!(
        format!("{v}"),
        format!("0.{}22250738585072014", "0".repeat(307))
    );

    let v = swfp::F64::from_host(f64::MAX);
    assert_eq!(
        format!("{v}"),
        format!("17976931348623157{}", "0".repeat(292))
    );
}

#[test]
fn test_fmt_exp() {
    let v = swfp::F64::from_bits(1);
    assert_eq!(format!("{v:e}"), "5e-324"); // min subnormal

    let v = swfp::F64::from_bits(2);
    assert_eq!(format!("{v:e}"), "1e-323");

    let v = swfp::F64::from_bits(3);
    assert_eq!(format!("{v:e}"), "1.5e-323");
    assert_eq!(format!("{v:.0e}"), "1e-323");

    let v = swfp::F64::from_host(f64::MIN_POSITIVE);
    assert_eq!(format!("{v:e}"), "2.2250738585072014e-308");
    assert_eq!(format!("{v:.8e}"), "2.22507386e-308");

    let v = swfp::F64::from_host(f64::MAX);
    assert_eq!(format!("{v:e}"), "1.7976931348623157e308");
    assert_eq!(format!("{v:.8e}"), "1.79769313e308");
}

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::F64::NAN);
    check_round_int_exact(swfp::F64::INFINITY);
    check_round_int_exact(-swfp::F64::INFINITY);
    check_round_int_exact(swfp::F64::ZERO);
    check_round_int_exact(-swfp::F64::ZERO);

    for s in [false, true] {
        check_round_int_exact(mk_f64(s, 0, 0));
        check_round_int_exact(mk_f64(s, 2, 1 << 51));
        check_round_int_exact(mk_f64(s, 52, (1 << 52) - 1));

        let zero = mk_f64(s, -1023, 0);
        let one = mk_f64(s, 0, 0);
        check_round_int_round(mk_f64(s, -1023, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f64(s, -2, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f64(s, -1, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_f64(s, -1, 1 << 51), zero, one, Loss::HalfUp);
        check_round_int_round(mk_f64(s, -1, (1 << 52) - 1), zero, one, Loss::HalfUp);
    }
}
