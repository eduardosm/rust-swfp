use std::num::FpCategory;

use swfp::math::Exp as _;
use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{
    ALL_ROUND_MODES, Loss, check_add_sub_exact, check_add_sub_round, check_category,
    check_div_exact, check_div_round, check_frexp, check_from_int_exact, check_from_int_round,
    check_from_str_exact, check_from_str_round, check_from_uint_exact, check_from_uint_round,
    check_mul_exact, check_mul_round, check_rem, check_round_int_exact, check_round_int_round,
    check_scalbn_exact, check_scalbn_round, check_to_int_exact, check_to_int_round,
    check_to_uint_exact, check_to_uint_round, mk_f32, test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for hi in 0..=0xFFFF {
        for lo in 0..=0xF {
            let bits = (hi << (32 - 16)) | lo;
            let new_bits = swfp::F32::from_bits(bits).to_bits();
            if new_bits != bits {
                panic!("0x{new_bits:04X} != 0x{bits:04X}");
            }
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F32>(true, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        for hi in 0..=255 {
            for lo in 0..=255 {
                let m = (hi << (23 - 8)) | lo;
                let category = if m == 0 {
                    FpCategory::Zero
                } else {
                    FpCategory::Subnormal
                };
                check_category(mk_f32(s, -127, m), category, s);
            }
        }

        for e in -126..=127 {
            for hi in 0..=255 {
                for lo in 0..=255 {
                    let m = (hi << (23 - 8)) | lo;
                    check_category(mk_f32(s, e, m), FpCategory::Normal, s);
                }
            }
        }

        for hi in 0..=255 {
            for lo in 0..=255 {
                let m = (hi << (23 - 8)) | lo;
                let category = if m == 0 {
                    FpCategory::Infinite
                } else {
                    FpCategory::Nan
                };
                check_category(mk_f32(s, 128, m), category, s);
            }
        }
    }
}

#[test]
fn test_convert_to_self() {
    for hi in 0..=0xFFFF {
        for lo in 0..=0xF {
            let bits = (hi << (32 - 16)) | lo;
            let value = swfp::F32::from_bits(bits);
            for round in ALL_ROUND_MODES {
                let (new_value, status) = swfp::F32::convert_from_ex(value, round);
                let new_bits = new_value.to_bits();

                let (expected_status, expected_bits) = if value.is_nan() && bits & (1 << 22) == 0 {
                    (FpStatus::INVALID, bits | (1 << 22))
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
fn test_from_uint() {
    check_from_uint_exact(0, mk_f32(false, -127, 0));

    for i in 0..127 {
        check_from_uint_exact(1 << i, mk_f32(false, i, 0));
    }
    for i in 0..126 {
        check_from_uint_exact(0b11 << i, mk_f32(false, i + 1, 1 << 22));
    }

    for m in 0..(1 << 23) {
        check_from_uint_exact(u128::from(m) | (1 << 23), mk_f32(false, 23, m));
    }

    check_from_uint_round(
        u128::MAX >> 1,
        mk_f32(false, 126, 0x7FFFFF),
        mk_f32(false, 127, 0),
        Loss::HalfUp,
    );
    check_from_uint_round(
        u128::MAX,
        mk_f32(false, 127, 0x7FFFFF),
        swfp::F32::INFINITY,
        Loss::HalfUp,
    );
    check_from_uint_round(
        (1 << 24) | 1,
        mk_f32(false, 24, 0),
        mk_f32(false, 24, 1),
        Loss::HalfEven,
    );
    check_from_uint_round(
        (1 << 24) | 0b11,
        mk_f32(false, 24, 1),
        mk_f32(false, 24, 0b10),
        Loss::HalfOdd,
    );
    check_from_uint_round(
        (1 << 25) | 1,
        mk_f32(false, 25, 0),
        mk_f32(false, 25, 1),
        Loss::HalfDown,
    );
    check_from_uint_round(
        (1 << 25) | 0b11,
        mk_f32(false, 25, 0),
        mk_f32(false, 25, 1),
        Loss::HalfUp,
    );
}

#[test]
fn test_from_int() {
    check_from_int_exact(0, mk_f32(false, -127, 0));

    for i in 0..126 {
        check_from_int_exact(1 << i, mk_f32(false, i, 0));
        check_from_int_exact(-(1 << i), mk_f32(true, i, 0));
    }
    check_from_int_exact(i128::MIN, mk_f32(true, 127, 0));
    for i in 0..125 {
        check_from_int_exact(0b11 << i, mk_f32(false, i + 1, 1 << 22));
        check_from_int_exact(-(0b11 << i), mk_f32(true, i + 1, 1 << 22));
    }
    for m in 0..(1 << 23) {
        check_from_int_exact(i128::from(m | (1 << 23)), mk_f32(false, 23, m));
        check_from_int_exact(-i128::from(m | (1 << 23)), mk_f32(true, 23, m));
    }

    check_from_int_round(
        i128::MAX,
        mk_f32(false, 126, 0x7FFFFF),
        mk_f32(false, 127, 0),
        Loss::HalfUp,
    );
    check_from_int_round(
        -i128::MAX,
        mk_f32(true, 126, 0x7FFFFF),
        mk_f32(true, 127, 0),
        Loss::HalfUp,
    );
}

#[test]
fn test_to_uint() {
    check_to_uint_exact(swfp::F32::NAN, (None, FpStatus::INVALID));
    check_to_uint_exact(swfp::F32::INFINITY, (None, FpStatus::INVALID));
    check_to_uint_exact(-swfp::F32::INFINITY, (None, FpStatus::INVALID));

    check_to_uint_exact(swfp::F32::ZERO, (Some(0), FpStatus::OK));
    check_to_uint_exact(-swfp::F32::ZERO, (Some(0), FpStatus::OK));

    for e in 0..=127 {
        check_to_uint_exact(mk_f32(false, e, 0), (Some(1 << e), FpStatus::OK));
        check_to_uint_exact(mk_f32(true, e, 0), (None, FpStatus::INVALID));
    }

    for hi in 0..=255 {
        for lo in 0..=255 {
            let m = (hi << (23 - 8)) | lo;
            check_to_uint_exact(
                mk_f32(false, 23, m),
                (Some(u128::from(m | (1 << 23))), FpStatus::OK),
            );
            check_to_uint_exact(mk_f32(true, 23, m), (None, FpStatus::INVALID));
        }
    }

    for e in 23..=127 {
        let m = 0x7FEFEF;
        check_to_uint_exact(
            mk_f32(false, e, m),
            (Some(u128::from(m | (1 << 23)) << (e - 23)), FpStatus::OK),
        );
        check_to_uint_exact(mk_f32(true, e, m), (None, FpStatus::INVALID));
    }

    check_to_uint_round(
        mk_f32(false, -1, 0),
        (Some(0), FpStatus::INEXACT),
        (Some(1), FpStatus::INEXACT),
        Loss::HalfEven,
    );
    check_to_uint_round(
        mk_f32(true, -1, 0),
        (Some(0), FpStatus::INEXACT),
        (None, FpStatus::INVALID),
        Loss::HalfEven,
    );
    check_to_uint_round(
        mk_f32(false, 0, 1 << 22),
        (Some(1), FpStatus::INEXACT),
        (Some(2), FpStatus::INEXACT),
        Loss::HalfOdd,
    );
    check_to_uint_round(
        mk_f32(true, 0, 1 << 22),
        (None, FpStatus::INVALID),
        (None, FpStatus::INVALID),
        Loss::HalfOdd,
    );
    check_to_uint_round(
        mk_f32(false, -1, 1 << 22),
        (Some(0), FpStatus::INEXACT),
        (Some(1), FpStatus::INEXACT),
        Loss::HalfUp,
    );
    check_to_uint_round(
        mk_f32(true, -1, 1 << 22),
        (Some(0), FpStatus::INEXACT),
        (None, FpStatus::INVALID),
        Loss::HalfUp,
    );
    for e in -126..=-2 {
        check_to_uint_round(
            mk_f32(false, e, 0),
            (Some(0), FpStatus::INEXACT),
            (Some(1), FpStatus::INEXACT),
            Loss::HalfDown,
        );
        check_to_uint_round(
            mk_f32(true, e, 0),
            (Some(0), FpStatus::INEXACT),
            (None, FpStatus::INVALID),
            Loss::HalfDown,
        );
    }
}

#[test]
fn test_to_int() {
    check_to_int_exact(swfp::F32::NAN, (None, FpStatus::INVALID));
    check_to_int_exact(swfp::F32::INFINITY, (None, FpStatus::INVALID));
    check_to_int_exact(-swfp::F32::INFINITY, (None, FpStatus::INVALID));

    check_to_int_exact(swfp::F32::ZERO, (Some(0), FpStatus::OK));
    check_to_int_exact(-swfp::F32::ZERO, (Some(0), FpStatus::OK));

    for e in 0..=127 {
        check_to_int_exact(
            mk_f32(false, e, 0),
            if e == 127 {
                (None, FpStatus::INVALID)
            } else {
                (Some(1 << e), FpStatus::OK)
            },
        );
        check_to_int_exact(mk_f32(true, e, 0), (Some(-1 << e), FpStatus::OK));
    }

    for hi in 0..=255 {
        for lo in 0..=255 {
            let m = (hi << (23 - 8)) | lo;
            check_to_int_exact(
                mk_f32(false, 23, m),
                (Some(i128::from(m | (1 << 23))), FpStatus::OK),
            );
            check_to_int_exact(
                mk_f32(true, 23, m),
                (Some(-i128::from(m | (1 << 23))), FpStatus::OK),
            );
        }
    }

    for e in 23..=126 {
        let m = 0x7FEFEF;
        check_to_int_exact(
            mk_f32(false, e, m),
            (Some(i128::from(m | (1 << 23)) << (e - 23)), FpStatus::OK),
        );
        check_to_int_exact(
            mk_f32(true, e, m),
            (Some(-i128::from(m | (1 << 23)) << (e - 23)), FpStatus::OK),
        );
    }

    check_to_int_round(
        mk_f32(false, -1, 0),
        (Some(0), FpStatus::INEXACT),
        (Some(1), FpStatus::INEXACT),
        Loss::HalfEven,
    );
    check_to_int_round(
        mk_f32(true, -1, 0),
        (Some(0), FpStatus::INEXACT),
        (Some(-1), FpStatus::INEXACT),
        Loss::HalfEven,
    );
    check_to_int_round(
        mk_f32(false, 0, 1 << 22),
        (Some(1), FpStatus::INEXACT),
        (Some(2), FpStatus::INEXACT),
        Loss::HalfOdd,
    );
    check_to_int_round(
        mk_f32(true, 0, 1 << 22),
        (Some(-1), FpStatus::INEXACT),
        (Some(-2), FpStatus::INEXACT),
        Loss::HalfOdd,
    );
    check_to_int_round(
        mk_f32(false, -1, 1 << 22),
        (Some(0), FpStatus::INEXACT),
        (Some(1), FpStatus::INEXACT),
        Loss::HalfUp,
    );
    check_to_int_round(
        mk_f32(true, -1, 1 << 22),
        (Some(0), FpStatus::INEXACT),
        (Some(-1), FpStatus::INEXACT),
        Loss::HalfUp,
    );
    for e in -126..=-2 {
        check_to_int_round(
            mk_f32(false, e, 0),
            (Some(0), FpStatus::INEXACT),
            (Some(1), FpStatus::INEXACT),
            Loss::HalfDown,
        );
        check_to_int_round(
            mk_f32(true, e, 0),
            (Some(0), FpStatus::INEXACT),
            (Some(-1), FpStatus::INEXACT),
            Loss::HalfDown,
        );
    }
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::F32>();

    let exact = [
        (
            "2.3283064365386962890625e-10",
            swfp::F32::from_bits(0x2F800000),
        ),
        ("1.8446744073709551616e19", swfp::F32::from_bits(0x5F800000)),
        (
            "340282346638528859811704183484516925440",
            swfp::F32::from_bits(0x7F7FFFFF),
        ),
        (
            "3.4028234663852885981170418348451692544e38",
            swfp::F32::from_bits(0x7F7FFFFF),
        ),
        (
            "1.17549421069244107548702944484928734882705242874589333385717453057158887047\
             5618904265502351336181163787841796875e-38",
            swfp::F32::from_bits(0x007FFFFF),
        ),
        (
            "1.17549435082228750796873653722224567781866555677208752150875170627841725945\
             47271728515625e-38",
            swfp::F32::from_bits(0x00800000),
        ),
        (
            "1.00000011920928955078125",
            swfp::F32::from_bits(0x3F800001),
        ),
        (
            "0.5000000596046447753906250000000000000000000000000000",
            swfp::F32::from_bits(0x3F000001),
        ),
        ("8388607.5", swfp::F32::from_bits(0x4AFFFFFF)),
        ("16777216", swfp::F32::from_bits(0x4B800000)),
        ("4294967040", swfp::F32::from_bits(0x4F7FFFFF)),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F32::from_int(n));
    }

    for e in -149..=127 {
        let s = format!("{:.1000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F32::from_int(1).scalbn(e));
    }

    // (input, rounded toward zero, rounded away from zero, loss)
    let inexact = [
        // Around the overflow threshold
        (
            "340282356779733661637539395458142568447.999999999999999999999999999999999999999999999",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfDown,
        ),
        (
            "340282356779733661637539395458142568448",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfOdd,
        ),
        (
            "340282356779733661637539395458142568448.000000000000000000000000000000000000\
             0000000001",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfUp,
        ),
        (
            "3.4028235677973366e38",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfDown,
        ),
        (
            "3.4028235677973367e38",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfUp,
        ),
        (
            "340282366920938463463374607431768211455.999999999999999999999999999999999999999999999",
            0x7F7FFFFF,
            0x7F800000,
            Loss::HalfUp,
        ),
        (
            "340282366920938463463374607431768211456",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        ("1e39", 0x7F7FFFFF, 0x7F800000, Loss::Overflow),
        (
            "1111111111111111111111111111111111111111",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        (
            "1e99999999999999999999",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        (
            "1e9223372036854775807",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        (
            "1e9223372036854775808",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        (
            "1e18446744073709551616",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        (
            "0.00000000000000000001e99999999999999999999",
            0x7F7FFFFF,
            0x7F800000,
            Loss::Overflow,
        ),
        // Around the underflow threshold
        (
            "7.00649232162408535461864791644958065640130970938257885878534141944895541342\
             930300743319094181060791015624999999999999999999999999999999999999999999999e-46",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "7.00649232162408535461864791644958065640130970938257885878534141944895541342\
             930300743319094181060791015625e-46",
            0x00000000,
            0x00000001,
            Loss::HalfEven,
        ),
        (
            "7.00649232162408535461864791644958065640130970938257885878534141944895541342\
             9303007433190941810607910156250000000000000000000000000000000000000000000001e-46",
            0x00000000,
            0x00000001,
            Loss::HalfUp,
        ),
        (
            "7.006492321624085e-46",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "7.0064923216240854e-46",
            0x00000000,
            0x00000001,
            Loss::HalfUp,
        ),
        ("1e-45", 0x00000000, 0x00000001, Loss::HalfUp),
        ("1.4e-45", 0x00000000, 0x00000001, Loss::HalfUp),
        (
            "2.10194769648722560638559437493487419692039291281477365763560242583468662402\
             8790902229957282543182373046875e-45",
            0x00000001,
            0x00000002,
            Loss::HalfOdd,
        ),
        (
            "1.17549428075736429172788299103576651332285899275899042768296311842500306496\
             5173038558532425668090581893920898437499999999999999999999e-38",
            0x007FFFFF,
            0x00800000,
            Loss::HalfDown,
        ),
        (
            "1.17549428075736429172788299103576651332285899275899042768296311842500306496\
             51730385585324256680905818939208984375e-38",
            0x007FFFFF,
            0x00800000,
            Loss::HalfOdd,
        ),
        (
            "1.17549428075736429172788299103576651332285899275899042768296311842500306496\
             51730385585324256680905818939208984375000000000000000000001e-38",
            0x007FFFFF,
            0x00800000,
            Loss::HalfUp,
        ),
        (
            "1.17549435e-38",
            0x007FFFFF,
            0x00800000,
            Loss::ThreeQuartersUp,
        ),
        ("1e-50", 0x00000000, 0x00000001, Loss::HalfDown),
        (
            "1e-99999999999999999999",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "1e-9223372036854775808",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "1e-9223372036854775809",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "1e-18446744073709551616",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        (
            "100000000000000000000e-99999999999999999999",
            0x00000000,
            0x00000001,
            Loss::HalfDown,
        ),
        // Ties in the normal range
        ("10528446.5", 0x4B20A6BE, 0x4B20A6BF, Loss::HalfEven),
        ("10528445.5", 0x4B20A6BD, 0x4B20A6BE, Loss::HalfOdd),
        (
            "16777216.999999999999999999999999999999999999999999999",
            0x4B800000,
            0x4B800001,
            Loss::HalfDown,
        ),
        ("16777217", 0x4B800000, 0x4B800001, Loss::HalfEven),
        (
            "16777217.0000000000000000000000000000000000000000000001",
            0x4B800000,
            0x4B800001,
            Loss::HalfUp,
        ),
        ("16777219", 0x4B800001, 0x4B800002, Loss::HalfOdd),
        (
            "1.000000059604644775390624999999999999999999999999999999999999999999999",
            0x3F800000,
            0x3F800001,
            Loss::HalfDown,
        ),
        (
            "1.000000059604644775390625",
            0x3F800000,
            0x3F800001,
            Loss::HalfEven,
        ),
        (
            "1.0000000596046447753906250000000000000000000000000000000000000000000001",
            0x3F800000,
            0x3F800001,
            Loss::HalfUp,
        ),
        (
            "1.00000017881393432617187499",
            0x3F800001,
            0x3F800002,
            Loss::HalfDown,
        ),
        (
            "1.000000178813934326171875",
            0x3F800001,
            0x3F800002,
            Loss::HalfOdd,
        ),
        (
            "1.00000017881393432617187501",
            0x3F800001,
            0x3F800002,
            Loss::HalfUp,
        ),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3F7FFFFF,
            0x3F800000,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x3F800000,
            0x3F800001,
            Loss::HalfDown,
        ),
        // Very close to the midpoint between two consecutive representable values
        ("5e-20", 0x1F6C1E4A, 0x1F6C1E4B, Loss::HalfDown),
        ("67e+14", 0x59BE6CEA, 0x59BE6CEB, Loss::HalfDown),
        ("985e+15", 0x5D5AB6C4, 0x5D5AB6C5, Loss::HalfDown),
        ("55895e-16", 0x2CC4A9BD, 0x2CC4A9BE, Loss::HalfDown),
        ("7038531e-32", 0x15AE43FD, 0x15AE43FE, Loss::HalfDown),
        ("702990899e-20", 0x2CF757CA, 0x2CF757CB, Loss::HalfDown),
        ("25933168707e+13", 0x665BA998, 0x665BA999, Loss::HalfDown),
        ("596428896559e+20", 0x743C3324, 0x743C3325, Loss::HalfDown),
        ("3e-23", 0x1A111233, 0x1A111234, Loss::HalfUp),
        ("57e+18", 0x6045C22B, 0x6045C22C, Loss::HalfUp),
        ("789e-35", 0x0A23DE6F, 0x0A23DE70, Loss::HalfUp),
        ("2539e-18", 0x2736F448, 0x2736F449, Loss::HalfUp),
        ("76173e+28", 0x76163989, 0x7616398A, Loss::HalfUp),
        ("887745e-11", 0x3714F05B, 0x3714F05C, Loss::HalfUp),
        ("5382571e-37", 0x0D2EACA6, 0x0D2EACA7, Loss::HalfUp),
        ("82381273e-35", 0x128289D0, 0x128289D1, Loss::HalfUp),
        ("750486563e-38", 0x0F18377D, 0x0F18377E, Loss::HalfUp),
        ("3752432815e-39", 0x0E98377D, 0x0E98377E, Loss::HalfUp),
        ("75224575729e-45", 0x06C7FB31, 0x06C7FB32, Loss::HalfUp),
        ("459926601011e+15", 0x6BBE389F, 0x6BBE38A0, Loss::HalfUp),
        // Other values
        ("0.1", 0x3DCCCCCC, 0x3DCCCCCD, Loss::HalfUp),
        ("0.3", 0x3E999999, 0x3E99999A, Loss::HalfUp),
        ("0.7", 0x3F333333, 0x3F333334, Loss::HalfDown),
        (
            "3.14159265358979323846264338327950288",
            0x40490FDA,
            0x40490FDB,
            Loss::HalfUp,
        ),
        (
            "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
            0x3EAAAAAA,
            0x3EAAAAAB,
            Loss::HalfUp,
        ),
        (
            "1.50000000000000000000000000000000000000000000000000000000000000000001",
            0x3FC00000,
            0x3FC00001,
            Loss::HalfDown,
        ),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::F32::from_bits(tz);
        let az = swfp::F32::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }

    check_from_str_round(
        &format!("1.50{}01", "0".repeat(100_000)),
        swfp::F32::from_bits(0x3FC00000),
        swfp::F32::from_bits(0x3FC00001),
        Loss::HalfDown,
    );
}

#[test]
fn test_fmt_debug() {
    let v = swfp::F32::ZERO;
    assert_eq!(format!("{v:?}"), "0.0");
    assert_eq!(format!("{v:+?}"), "+0.0");
    assert_eq!(format!("{v:.3?}"), "0.000");

    let v = -swfp::F32::ZERO;
    assert_eq!(format!("{v:?}"), "-0.0");
    assert_eq!(format!("{v:+?}"), "-0.0");
    assert_eq!(format!("{v:.3?}"), "-0.000");

    let v = swfp::F32::from_int(1);
    assert_eq!(format!("{v:?}"), "1.0");
    assert_eq!(format!("{v:+?}"), "+1.0");

    let v = swfp::F32::from_int(-1);
    assert_eq!(format!("{v:?}"), "-1.0");
    assert_eq!(format!("{v:+?}"), "-1.0");

    let v = swfp::F32::from_host(f32::MAX);
    assert_eq!(format!("{v:?}"), "3.4028235e38");

    let v = swfp::F32::from_host(f32::MIN_POSITIVE);
    assert_eq!(format!("{v:?}"), "1.1754944e-38");

    let v = swfp::F32::from_bits(1);
    assert_eq!(format!("{v:?}"), "1e-45");

    let v = swfp::F32::from_host(1.41);
    assert_eq!(format!("{v:?}"), "1.41");
    assert_eq!(format!("{v:+?}"), "+1.41");
    assert_eq!(format!("{v:.4?}"), "1.4100");
    assert_eq!(format!("{v:07?}"), "0001.41");
    assert_eq!(format!("{v:+07?}"), "+001.41");
    assert_eq!(format!("{v:+08.3?}"), "+001.410");
    assert_eq!(format!("{v:8?}"), "    1.41");
    assert_eq!(format!("{v:+8?}"), "   +1.41");
    assert_eq!(format!("{v:8.4?}"), "  1.4100");
    assert_eq!(format!("{v:+8.4?}"), " +1.4100");
    assert_eq!(format!("{v:<8?}"), "1.41    ");
    assert_eq!(format!("{v:<+8?}"), "+1.41   ");
    assert_eq!(format!("{v:<8.4?}"), "1.4100  ");
    assert_eq!(format!("{v:<+8.4?}"), "+1.4100 ");

    let v = swfp::F32::from_host(-1.41);
    assert_eq!(format!("{v:?}"), "-1.41");
    assert_eq!(format!("{v:+?}"), "-1.41");
    assert_eq!(format!("{v:.4?}"), "-1.4100");
    assert_eq!(format!("{v:07?}"), "-001.41");
    assert_eq!(format!("{v:+07?}"), "-001.41");
    assert_eq!(format!("{v:+08.3?}"), "-001.410");
    assert_eq!(format!("{v:8?}"), "   -1.41");
    assert_eq!(format!("{v:+8?}"), "   -1.41");
    assert_eq!(format!("{v:8.4?}"), " -1.4100");
    assert_eq!(format!("{v:+8.4?}"), " -1.4100");
    assert_eq!(format!("{v:<8?}"), "-1.41   ");
    assert_eq!(format!("{v:<+8?}"), "-1.41   ");
    assert_eq!(format!("{v:<8.4?}"), "-1.4100 ");
    assert_eq!(format!("{v:<+8.4?}"), "-1.4100 ");

    let v = swfp::F32::from_host(10.41);
    assert_eq!(format!("{v:?}"), "10.41");
    assert_eq!(format!("{v:+?}"), "+10.41");
    assert_eq!(format!("{v:.4?}"), "10.4100");
    assert_eq!(format!("{v:07?}"), "0010.41");
    assert_eq!(format!("{v:+07?}"), "+010.41");
    assert_eq!(format!("{v:+08.3?}"), "+010.410");
    assert_eq!(format!("{v:8?}"), "   10.41");
    assert_eq!(format!("{v:+8?}"), "  +10.41");
    assert_eq!(format!("{v:8.3?}"), "  10.410");
    assert_eq!(format!("{v:+8.3?}"), " +10.410");
    assert_eq!(format!("{v:<8?}"), "10.41   ");
    assert_eq!(format!("{v:<+8?}"), "+10.41  ");
    assert_eq!(format!("{v:<8.3?}"), "10.410  ");
    assert_eq!(format!("{v:<+8.3?}"), "+10.410 ");

    let v = swfp::F32::from_host(-10.41);
    assert_eq!(format!("{v:?}"), "-10.41");
    assert_eq!(format!("{v:+?}"), "-10.41");
    assert_eq!(format!("{v:.4?}"), "-10.4100");
    assert_eq!(format!("{v:07?}"), "-010.41");
    assert_eq!(format!("{v:+07?}"), "-010.41");
    assert_eq!(format!("{v:+08.3?}"), "-010.410");
    assert_eq!(format!("{v:8?}"), "  -10.41");
    assert_eq!(format!("{v:+8?}"), "  -10.41");
    assert_eq!(format!("{v:8.3?}"), " -10.410");
    assert_eq!(format!("{v:+8.3?}"), " -10.410");
    assert_eq!(format!("{v:<8?}"), "-10.41  ");
    assert_eq!(format!("{v:<+8?}"), "-10.41  ");
    assert_eq!(format!("{v:<8.3?}"), "-10.410 ");
    assert_eq!(format!("{v:<+8.3?}"), "-10.410 ");

    let v = swfp::F32::from_host(1e3);
    assert_eq!(format!("{v:?}"), "1000.0");

    let v = swfp::F32::from_host(1.0e15);
    assert_eq!(format!("{v:?}"), "1000000000000000.0");

    let v = swfp::F32::from_host(1.2e15);
    assert_eq!(format!("{v:?}"), "1200000000000000.0");

    let v = swfp::F32::from_host(1.0e16);
    assert_eq!(format!("{v:?}"), "1e16");

    let v = swfp::F32::from_host(1.2e16);
    assert_eq!(format!("{v:?}"), "1.2e16");
}

#[test]
fn test_fmt_display() {
    let v = swfp::F32::ZERO;
    assert_eq!(format!("{v}"), "0");
    assert_eq!(format!("{v:+}"), "+0");
    assert_eq!(format!("{v:.3}"), "0.000");

    let v = -swfp::F32::ZERO;
    assert_eq!(format!("{v:}"), "-0");
    assert_eq!(format!("{v:+}"), "-0");
    assert_eq!(format!("{v:.3}"), "-0.000");

    let v = swfp::F32::from_int(1);
    assert_eq!(format!("{v}"), "1");
    assert_eq!(format!("{v:+}"), "+1");

    let v = swfp::F32::from_int(-1);
    assert_eq!(format!("{v}"), "-1");
    assert_eq!(format!("{v:+}"), "-1");

    let v = swfp::F32::from_host(f32::MAX);
    assert_eq!(format!("{v}"), "340282350000000000000000000000000000000");

    let v = swfp::F32::from_host(f32::MIN_POSITIVE);
    assert_eq!(
        format!("{v}"),
        "0.000000000000000000000000000000000000011754944",
    );

    let v = swfp::F32::from_bits(1);
    assert_eq!(
        format!("{v}"),
        "0.000000000000000000000000000000000000000000001",
    );

    let v = swfp::F32::from_host(1.41);
    assert_eq!(format!("{v}"), "1.41");
    assert_eq!(format!("{v:+}"), "+1.41");
    assert_eq!(format!("{v:.4}"), "1.4100");
    assert_eq!(format!("{v:07}"), "0001.41");
    assert_eq!(format!("{v:+07}"), "+001.41");
    assert_eq!(format!("{v:+08.3}"), "+001.410");
    assert_eq!(format!("{v:8}"), "    1.41");
    assert_eq!(format!("{v:+8}"), "   +1.41");
    assert_eq!(format!("{v:8.4}"), "  1.4100");
    assert_eq!(format!("{v:+8.4}"), " +1.4100");
    assert_eq!(format!("{v:<8}"), "1.41    ");
    assert_eq!(format!("{v:<+8}"), "+1.41   ");
    assert_eq!(format!("{v:<8.4}"), "1.4100  ");
    assert_eq!(format!("{v:<+8.4}"), "+1.4100 ");

    let v = swfp::F32::from_host(-1.41);
    assert_eq!(format!("{v}"), "-1.41");
    assert_eq!(format!("{v:+}"), "-1.41");
    assert_eq!(format!("{v:.4}"), "-1.4100");
    assert_eq!(format!("{v:07}"), "-001.41");
    assert_eq!(format!("{v:+07}"), "-001.41");
    assert_eq!(format!("{v:+08.3}"), "-001.410");
    assert_eq!(format!("{v:8}"), "   -1.41");
    assert_eq!(format!("{v:+8}"), "   -1.41");
    assert_eq!(format!("{v:8.4}"), " -1.4100");
    assert_eq!(format!("{v:+8.4}"), " -1.4100");
    assert_eq!(format!("{v:<8}"), "-1.41   ");
    assert_eq!(format!("{v:<+8}"), "-1.41   ");
    assert_eq!(format!("{v:<8.4}"), "-1.4100 ");
    assert_eq!(format!("{v:<+8.4}"), "-1.4100 ");

    let v = swfp::F32::from_host(10.41);
    assert_eq!(format!("{v}"), "10.41");
    assert_eq!(format!("{v:+}"), "+10.41");
    assert_eq!(format!("{v:.4}"), "10.4100");
    assert_eq!(format!("{v:07}"), "0010.41");
    assert_eq!(format!("{v:+07}"), "+010.41");
    assert_eq!(format!("{v:+08.3}"), "+010.410");
    assert_eq!(format!("{v:8}"), "   10.41");
    assert_eq!(format!("{v:+8}"), "  +10.41");
    assert_eq!(format!("{v:8.3}"), "  10.410");
    assert_eq!(format!("{v:+8.3}"), " +10.410");
    assert_eq!(format!("{v:<8}"), "10.41   ");
    assert_eq!(format!("{v:<+8}"), "+10.41  ");
    assert_eq!(format!("{v:<8.3}"), "10.410  ");
    assert_eq!(format!("{v:<+8.3}"), "+10.410 ");

    let v = swfp::F32::from_host(-10.41);
    assert_eq!(format!("{v}"), "-10.41");
    assert_eq!(format!("{v:+}"), "-10.41");
    assert_eq!(format!("{v:.4}"), "-10.4100");
    assert_eq!(format!("{v:07}"), "-010.41");
    assert_eq!(format!("{v:+07}"), "-010.41");
    assert_eq!(format!("{v:+08.3}"), "-010.410");
    assert_eq!(format!("{v:8}"), "  -10.41");
    assert_eq!(format!("{v:+8}"), "  -10.41");
    assert_eq!(format!("{v:8.3}"), " -10.410");
    assert_eq!(format!("{v:+8.3}"), " -10.410");
    assert_eq!(format!("{v:<8}"), "-10.41  ");
    assert_eq!(format!("{v:<+8}"), "-10.41  ");
    assert_eq!(format!("{v:<8.3}"), "-10.410 ");
    assert_eq!(format!("{v:<+8.3}"), "-10.410 ");

    let v = swfp::F32::from_host(1.2e15);
    assert_eq!(format!("{v}"), "1200000000000000");

    let v = swfp::F32::from_host(1.2e16);
    assert_eq!(format!("{v}"), "12000000000000000");

    for i in 1..=38 {
        let v = swfp::F32::from_int(i).exp10();
        assert_eq!(format!("{v}"), format!("1{}", "0".repeat(i as usize)));
    }

    for i in 1..=45 {
        let v = swfp::F32::from_int(-i).exp10();
        assert_eq!(
            format!("{v}"),
            format!("0.{}1", "0".repeat((i - 1) as usize)),
        );
    }

    for i in 1..=100_000 {
        let v = swfp::F32::from_int(i);
        assert_eq!(format!("{v}"), format!("{i}"));
        assert_eq!(format!("{v:.3}"), format!("{i}.000"));

        let v = swfp::F32::from_int(-i);
        assert_eq!(format!("{v}"), format!("-{i}"));
        assert_eq!(format!("{v:.3}"), format!("-{i}.000"));
    }
}

#[test]
fn test_fmt_exp() {
    let v = swfp::F32::ZERO;
    assert_eq!(format!("{v:e}"), "0e0");
    assert_eq!(format!("{v:E}"), "0E0");
    assert_eq!(format!("{v:+e}"), "+0e0");
    assert_eq!(format!("{v:+E}"), "+0E0");
    assert_eq!(format!("{v:.3e}"), "0.000e0");
    assert_eq!(format!("{v:.3E}"), "0.000E0");

    let v = -swfp::F32::ZERO;
    assert_eq!(format!("{v:e}"), "-0e0");
    assert_eq!(format!("{v:+e}"), "-0e0");
    assert_eq!(format!("{v:.3e}"), "-0.000e0");

    let v = swfp::F32::from_int(1);
    assert_eq!(format!("{v:e}"), "1e0");
    assert_eq!(format!("{v:E}"), "1E0");
    assert_eq!(format!("{v:+e}"), "+1e0");
    assert_eq!(format!("{v:+E}"), "+1E0");

    let v = swfp::F32::from_int(-1);
    assert_eq!(format!("{v:e}"), "-1e0");
    assert_eq!(format!("{v:+e}"), "-1e0");

    let v = swfp::F32::from_host(f32::MAX);
    assert_eq!(format!("{v:e}"), "3.4028235e38");

    let v = swfp::F32::from_host(f32::MIN_POSITIVE);
    assert_eq!(format!("{v:e}"), "1.1754944e-38");

    let v = swfp::F32::from_bits(1);
    assert_eq!(format!("{v:e}"), "1e-45");

    let v = swfp::F32::from_host(1.41);
    assert_eq!(format!("{v:e}"), "1.41e0");
    assert_eq!(format!("{v:+e}"), "+1.41e0");
    assert_eq!(format!("{v:.4e}"), "1.4100e0");
    assert_eq!(format!("{v:09e}"), "0001.41e0");
    assert_eq!(format!("{v:+09e}"), "+001.41e0");
    assert_eq!(format!("{v:+09.3e}"), "+01.410e0");
    assert_eq!(format!("{v:9e}"), "   1.41e0");
    assert_eq!(format!("{v:+9e}"), "  +1.41e0");
    assert_eq!(format!("{v:9.3e}"), "  1.410e0");
    assert_eq!(format!("{v:+9.3e}"), " +1.410e0");
    assert_eq!(format!("{v:<9e}"), "1.41e0   ");
    assert_eq!(format!("{v:<+9e}"), "+1.41e0  ");
    assert_eq!(format!("{v:<9.3e}"), "1.410e0  ");
    assert_eq!(format!("{v:<+9.3e}"), "+1.410e0 ");

    let v = swfp::F32::from_host(-1.41);
    assert_eq!(format!("{v:e}"), "-1.41e0");
    assert_eq!(format!("{v:+e}"), "-1.41e0");
    assert_eq!(format!("{v:.4e}"), "-1.4100e0");
    assert_eq!(format!("{v:09e}"), "-001.41e0");
    assert_eq!(format!("{v:+09e}"), "-001.41e0");
    assert_eq!(format!("{v:+09.3e}"), "-01.410e0");
    assert_eq!(format!("{v:9e}"), "  -1.41e0");
    assert_eq!(format!("{v:+9e}"), "  -1.41e0");
    assert_eq!(format!("{v:9.3e}"), " -1.410e0");
    assert_eq!(format!("{v:+9.3e}"), " -1.410e0");
    assert_eq!(format!("{v:<9e}"), "-1.41e0  ");
    assert_eq!(format!("{v:<+9e}"), "-1.41e0  ");
    assert_eq!(format!("{v:<9.3e}"), "-1.410e0 ");
    assert_eq!(format!("{v:<+9.3e}"), "-1.410e0 ");

    let v = swfp::F32::from_host(10.41);
    assert_eq!(format!("{v:e}"), "1.041e1");
    assert_eq!(format!("{v:+e}"), "+1.041e1");
    assert_eq!(format!("{v:.5e}"), "1.04100e1");
    assert_eq!(format!("{v:09e}"), "001.041e1");
    assert_eq!(format!("{v:+09e}"), "+01.041e1");
    assert_eq!(format!("{v:+010.4e}"), "+01.0410e1");
    assert_eq!(format!("{v:10e}"), "   1.041e1");
    assert_eq!(format!("{v:+10e}"), "  +1.041e1");
    assert_eq!(format!("{v:10.4e}"), "  1.0410e1");
    assert_eq!(format!("{v:+10.4e}"), " +1.0410e1");
    assert_eq!(format!("{v:<10e}"), "1.041e1   ");
    assert_eq!(format!("{v:<+10e}"), "+1.041e1  ");
    assert_eq!(format!("{v:<10.4e}"), "1.0410e1  ");
    assert_eq!(format!("{v:<+10.4e}"), "+1.0410e1 ");

    let v = swfp::F32::from_host(-10.41);
    assert_eq!(format!("{v:e}"), "-1.041e1");
    assert_eq!(format!("{v:+e}"), "-1.041e1");
    assert_eq!(format!("{v:.5e}"), "-1.04100e1");
    assert_eq!(format!("{v:09e}"), "-01.041e1");
    assert_eq!(format!("{v:+09e}"), "-01.041e1");
    assert_eq!(format!("{v:+010.4e}"), "-01.0410e1");
    assert_eq!(format!("{v:10e}"), "  -1.041e1");
    assert_eq!(format!("{v:+10e}"), "  -1.041e1");
    assert_eq!(format!("{v:10.4e}"), " -1.0410e1");
    assert_eq!(format!("{v:+10.4e}"), " -1.0410e1");
    assert_eq!(format!("{v:<10e}"), "-1.041e1  ");
    assert_eq!(format!("{v:<+10e}"), "-1.041e1  ");
    assert_eq!(format!("{v:<10.4e}"), "-1.0410e1 ");
    assert_eq!(format!("{v:<+10.4e}"), "-1.0410e1 ");

    let v = swfp::F32::from_host(1.2e15);
    assert_eq!(format!("{v:e}"), "1.2e15");
    assert_eq!(format!("{v:E}"), "1.2E15");

    let v = swfp::F32::from_host(1.2e16);
    assert_eq!(format!("{v:e}"), "1.2e16");
    assert_eq!(format!("{v:E}"), "1.2E16");

    for i in 1..=38 {
        let v = swfp::F32::from_int(i).exp10();
        assert_eq!(format!("{v:e}"), format!("1e{i}"));
        assert_eq!(format!("{v:E}"), format!("1E{i}"));
    }

    for i in 1..=45 {
        let v = swfp::F32::from_int(-i).exp10();
        assert_eq!(format!("{v:e}"), format!("1e-{i}"));
        assert_eq!(format!("{v:E}"), format!("1E-{i}"));
    }
}

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::F32::NAN);
    check_round_int_exact(swfp::F32::INFINITY);
    check_round_int_exact(-swfp::F32::INFINITY);

    for s in [false, true] {
        for e in 23..=127 {
            for hi in 0..=255 {
                for lo in 0..=255 {
                    let m = (hi << (23 - 8)) | lo;
                    check_round_int_exact(mk_f32(s, e, m));
                }
            }
        }

        check_round_int_exact(mk_f32(s, 22, 0x3FFFFE));
        check_round_int_exact(mk_f32(s, 19, 0x3FFFF0));

        check_round_int_round(
            mk_f32(s, 19, 0x0000F8),
            mk_f32(s, 19, 0x0000F0),
            mk_f32(s, 19, 0x000100),
            Loss::HalfOdd,
        );
        check_round_int_round(
            mk_f32(s, 19, 0x0000E8),
            mk_f32(s, 19, 0x0000E0),
            mk_f32(s, 19, 0x0000F0),
            Loss::HalfEven,
        );
        check_round_int_round(
            mk_f32(s, 19, 0x0000F9),
            mk_f32(s, 19, 0x0000F0),
            mk_f32(s, 19, 0x000100),
            Loss::HalfUp,
        );
        check_round_int_round(
            mk_f32(s, 19, 0x0000F1),
            mk_f32(s, 19, 0x0000F0),
            mk_f32(s, 19, 0x000100),
            Loss::HalfDown,
        );

        let zero = mk_f32(s, -127, 0);
        let one = mk_f32(s, 0, 0);
        check_round_int_round(mk_f32(s, -127, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f32(s, -2, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f32(s, -1, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_f32(s, -1, 1 << 22), zero, one, Loss::HalfUp);
        check_round_int_round(mk_f32(s, -1, 0x7FFFFF), zero, one, Loss::HalfUp);
    }
}

#[test]
fn test_scalbn() {
    for exp in [i32::MIN, -1, 0, 1, i32::MAX] {
        check_scalbn_exact(swfp::F32::NAN, exp, swfp::F32::NAN);
        check_scalbn_exact(swfp::F32::INFINITY, exp, swfp::F32::INFINITY);
        check_scalbn_exact(-swfp::F32::INFINITY, exp, -swfp::F32::INFINITY);
        check_scalbn_exact(swfp::F32::ZERO, exp, swfp::F32::ZERO);
        check_scalbn_exact(-swfp::F32::ZERO, exp, -swfp::F32::ZERO);
    }

    for s in [false, true] {
        check_scalbn_exact(mk_f32(s, 0, 1), 0, mk_f32(s, 0, 1));
        check_scalbn_exact(mk_f32(s, 0, 1), 1, mk_f32(s, 1, 1));
        check_scalbn_exact(mk_f32(s, 0, 1), -1, mk_f32(s, -1, 1));
        check_scalbn_exact(mk_f32(s, 0, 1), 127, mk_f32(s, 127, 1));
        check_scalbn_exact(mk_f32(s, 0, 1), -126, mk_f32(s, -126, 1));
        check_scalbn_exact(mk_f32(s, 127, 1), -126 - 127, mk_f32(s, -126, 1));
        check_scalbn_exact(mk_f32(s, -126, 1), 126 + 127, mk_f32(s, 127, 1));
        check_scalbn_exact(mk_f32(s, 0, 0), -127, mk_f32(s, -127, 1 << 22));
        check_scalbn_exact(mk_f32(s, 0, 0), -149, mk_f32(s, -127, 1));
        check_scalbn_exact(mk_f32(s, 127, 0), -127 - 149, mk_f32(s, -127, 1));
        check_scalbn_exact(mk_f32(s, -127, 1), 127 + 149, mk_f32(s, 127, 0));

        check_scalbn_round(
            mk_f32(s, 0, 0),
            i32::MAX,
            mk_f32(s, 127, 0x7FFFFF),
            mk_f32(s, 128, 0),
            Loss::Overflow,
        );
        check_scalbn_round(
            mk_f32(s, 0, 0),
            i32::MIN,
            mk_f32(s, -127, 0),
            mk_f32(s, -127, 1),
            Loss::HalfDown,
        );

        check_scalbn_round(
            mk_f32(s, 0, 1),
            128,
            mk_f32(s, 127, 0x7FFFFF),
            mk_f32(s, 128, 0),
            Loss::Overflow,
        );
        check_scalbn_round(
            mk_f32(s, 127, 0),
            1,
            mk_f32(s, 127, 0x7FFFFF),
            mk_f32(s, 128, 0),
            Loss::Overflow,
        );

        check_scalbn_round(
            mk_f32(s, 0, 0x48),
            -130,
            mk_f32(s, -127, 0x80004),
            mk_f32(s, -127, 0x80005),
            Loss::HalfEven,
        );
        check_scalbn_round(
            mk_f32(s, 0, 0x58),
            -130,
            mk_f32(s, -127, 0x80005),
            mk_f32(s, -127, 0x80006),
            Loss::HalfOdd,
        );
        check_scalbn_round(
            mk_f32(s, 0, 0x41),
            -130,
            mk_f32(s, -127, 0x80004),
            mk_f32(s, -127, 0x80005),
            Loss::HalfDown,
        );
        check_scalbn_round(
            mk_f32(s, 0, 0x49),
            -130,
            mk_f32(s, -127, 0x80004),
            mk_f32(s, -127, 0x80005),
            Loss::HalfUp,
        );

        // 2^-126 * (1 - 2^-24), halfway between the largest subnormal number
        // and the smallest normal number
        check_scalbn_round(
            mk_f32(s, 0, 0x7FFFFF),
            -127,
            mk_f32(s, -127, 0x7FFFFF),
            mk_f32(s, -126, 0),
            Loss::HalfOdd,
        );
    }
}

#[test]
fn test_frexp() {
    check_frexp(swfp::F32::NAN, (swfp::F32::NAN, 0));
    check_frexp(swfp::F32::INFINITY, (swfp::F32::INFINITY, 0));
    check_frexp(-swfp::F32::INFINITY, (-swfp::F32::INFINITY, 0));
    check_frexp(swfp::F32::ZERO, (swfp::F32::ZERO, 0));
    check_frexp(-swfp::F32::ZERO, (-swfp::F32::ZERO, 0));

    for s in [false, true] {
        for e in -126..=127 {
            for hi in 0..=255 {
                for lo in 0..=255 {
                    let m = (hi << (23 - 8)) | lo;
                    check_frexp(mk_f32(s, e, m), (mk_f32(s, -1, m), (e + 1).into()));
                }
            }
        }

        for sub_e in 0..=22 {
            check_frexp(
                mk_f32(s, -127, 1 << sub_e),
                (mk_f32(s, -1, 0), -148 + sub_e),
            );
        }

        check_frexp(mk_f32(s, -127, 0b11), (mk_f32(s, -1, 1 << 22), -147));
    }
}

#[test]
fn test_add_sub() {
    for s in [false, true] {
        // same sign
        let inf = if s {
            -swfp::F32::INFINITY
        } else {
            swfp::F32::INFINITY
        };
        check_add_sub_exact(inf, inf, inf);

        check_add_sub_exact(mk_f32(s, -127, 0), mk_f32(s, -127, 0), mk_f32(s, -127, 0));
        check_add_sub_exact(mk_f32(s, 0, 0x7FFFFF), mk_f32(s, -23, 0), mk_f32(s, 1, 0));
        check_add_sub_exact(
            mk_f32(s, 0, 0x7FF0FF),
            mk_f32(s, -23, 0),
            mk_f32(s, 0, 0x7FF100),
        );
        check_add_sub_exact(
            mk_f32(s, -127, 0x001B11),
            mk_f32(s, -127, 0x1000FA),
            mk_f32(s, -127, 0x101C0B),
        );

        check_add_sub_round(
            mk_f32(s, 0, 0x000003),
            mk_f32(s, -24, 0),
            mk_f32(s, 0, 0x000003),
            mk_f32(s, 0, 0x000004),
            s,
            Loss::HalfOdd,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x000004),
            mk_f32(s, -24, 0),
            mk_f32(s, 0, 0x000004),
            mk_f32(s, 0, 0x000005),
            s,
            Loss::HalfEven,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x000003),
            mk_f32(s, -24, 1),
            mk_f32(s, 0, 0x000003),
            mk_f32(s, 0, 0x000004),
            s,
            Loss::HalfUp,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x000003),
            mk_f32(s, -25, 0),
            mk_f32(s, 0, 0x000003),
            mk_f32(s, 0, 0x000004),
            s,
            Loss::HalfDown,
        );

        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(s, -19, 0),
            mk_f32(s, 1, 0x000007),
            mk_f32(s, 1, 0x000008),
            s,
            Loss::HalfOdd,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(s, -19, 1),
            mk_f32(s, 1, 0x000007),
            mk_f32(s, 1, 0x000008),
            s,
            Loss::HalfUp,
        );

        // different sign
        for round in ALL_ROUND_MODES {
            let expected_r = mk_f32(round == Round::TowardNegative, -127, 0);

            let (r, status) = mk_f32(s, -127, 0).add_ex(mk_f32(!s, -127, 0), round);
            assert_eq!(status, FpStatus::OK);
            assert_eq!(r.to_bits(), expected_r.to_bits());

            let (r, status) = mk_f32(s, 0, 1).add_ex(mk_f32(!s, 0, 1), round);
            assert_eq!(status, FpStatus::OK);
            assert_eq!(r.to_bits(), expected_r.to_bits());
        }

        check_add_sub_exact(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, -23, 0),
            mk_f32(s, 0, 0x7FFFFE),
        );
        check_add_sub_exact(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, 0, 0),
            mk_f32(s, -1, 0x7FFFFE),
        );
        check_add_sub_exact(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, 0, 1 << 22),
            mk_f32(s, -2, 0x7FFFFC),
        );
        check_add_sub_exact(
            mk_f32(s, -127, 0x031B51),
            mk_f32(!s, -127, 0x031A01),
            mk_f32(s, -127, 0x000150),
        );
        check_add_sub_exact(
            mk_f32(s, -126, 0),
            mk_f32(!s, -127, 1),
            mk_f32(s, -127, 0x7FFFFF),
        );

        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, -24, 0),
            mk_f32(s, 0, 0x7FFFFE),
            mk_f32(s, 0, 0x7FFFFF),
            s,
            Loss::HalfEven,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFE),
            mk_f32(!s, -24, 0),
            mk_f32(s, 0, 0x7FFFFD),
            mk_f32(s, 0, 0x7FFFFE),
            s,
            Loss::HalfOdd,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, -24, 1 << 22),
            mk_f32(s, 0, 0x7FFFFE),
            mk_f32(s, 0, 0x7FFFFF),
            s,
            Loss::HalfDown,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, -25, 0),
            mk_f32(s, 0, 0x7FFFFE),
            mk_f32(s, 0, 0x7FFFFF),
            s,
            Loss::HalfUp,
        );
        check_add_sub_round(
            mk_f32(s, 0, 0x7FFFFF),
            mk_f32(!s, -10, 1),
            mk_f32(s, 0, 0x7FDFFE),
            mk_f32(s, 0, 0x7FDFFF),
            s,
            Loss::HalfUp,
        );
    }
}

#[test]
fn test_mul() {
    check_mul_exact(swfp::F32::ZERO, swfp::F32::ZERO, swfp::F32::ZERO);
    check_mul_exact(swfp::F32::ZERO, -swfp::F32::ZERO, -swfp::F32::ZERO);

    check_mul_exact(
        swfp::F32::INFINITY,
        swfp::F32::INFINITY,
        swfp::F32::INFINITY,
    );
    check_mul_exact(
        swfp::F32::INFINITY,
        -swfp::F32::INFINITY,
        -swfp::F32::INFINITY,
    );

    for a in 1..=2000 {
        for b in 1..=2000 {
            check_mul_exact(
                swfp::F32::from_int(i128::from(a)),
                swfp::F32::from_int(i128::from(b)),
                swfp::F32::from_int(i128::from(a * b)),
            );
            check_mul_exact(
                swfp::F32::from_int(i128::from(a)),
                swfp::F32::from_int(i128::from(-b)),
                swfp::F32::from_int(i128::from(-(a * b))),
            );
        }
    }

    for s in [false, true] {
        check_mul_round(
            mk_f32(s, 4, 0x4F0F0F),
            mk_f32(s, 10, 0x5AA10D),
            mk_f32(false, 15, 0x30D515),
            mk_f32(false, 15, 0x30D516),
            false,
            Loss::HalfUp,
        );
        check_mul_round(
            mk_f32(s, 4, 0x4F0F0F),
            mk_f32(!s, 10, 0x5AA10D),
            mk_f32(true, 15, 0x30D515),
            mk_f32(true, 15, 0x30D516),
            true,
            Loss::HalfUp,
        );

        let min_subnormal = mk_f32(s, -127, 1);
        let max_subnormal = mk_f32(s, -127, 0x7FFFFF);
        let min_normal = mk_f32(s, -126, 0);

        // Subnormal results
        check_mul_exact(mk_f32(s, -100, 0), mk_f32(false, -49, 0), min_subnormal);
        check_mul_exact(
            mk_f32(s, -100, 0x7FFFFE),
            mk_f32(false, -27, 0),
            max_subnormal,
        );
        check_mul_round(
            mk_f32(s, -100, 0x000001),
            mk_f32(false, -28, 0),
            mk_f32(s, -127, 0x200000),
            mk_f32(s, -127, 0x200001),
            s,
            Loss::HalfDown,
        );

        // Results between the largest subnormal number and the smallest
        // normal number are tiny when they are still less than the smallest
        // normal number after being rounded with an unbounded exponent range,
        // even if they are finally rounded to the smallest normal number.

        // 2^-126 * (1 - 2^-24 - 3 * 2^-47), less than halfway
        check_mul_round(
            mk_f32(s, -100, 0x7FFFFD),
            mk_f32(false, -27, 0x000001),
            max_subnormal,
            min_normal,
            s,
            Loss::HalfDown,
        );

        // 2^-126 * (1 - 2^-24), halfway
        check_mul_round(
            mk_f32(s, -100, 0x7FFFFF),
            mk_f32(false, -27, 0),
            max_subnormal,
            min_normal,
            s,
            Loss::HalfOdd,
        );

        // 2^-126 * (1 - 4500000 * 2^-47), between halfway and three quarters
        check_mul_round(
            mk_f32(s, -100, 0x7FF448),
            mk_f32(false, -27, 0x0005DC),
            max_subnormal,
            min_normal,
            s,
            Loss::HalfUp,
        );

        // 2^-126 * (1 - 2^-25), three quarters
        check_mul_round(
            mk_f32(s, -100, 0x118E00),
            mk_f32(false, -27, 0x612000),
            max_subnormal,
            min_normal,
            s,
            Loss::ThreeQuartersUp,
        );

        // 2^-126 * (1 - 2^-46), more than three quarters
        check_mul_round(
            mk_f32(s, -100, 0x7FFFFE),
            mk_f32(false, -27, 0x000001),
            max_subnormal,
            min_normal,
            s,
            Loss::ThreeQuartersUp,
        );
    }

    // The flags of the last case, without relying on helpers.
    let lhs = mk_f32(false, -100, 0x7FFFFE);
    let rhs = mk_f32(false, -27, 0x000001);
    let max_subnormal = mk_f32(false, -127, 0x7FFFFF);
    let min_normal = mk_f32(false, -126, 0);
    for (round, expected, expected_status) in [
        (Round::NearestTiesToEven, min_normal, FpStatus::INEXACT),
        (Round::NearestTiesToAway, min_normal, FpStatus::INEXACT),
        (Round::TowardPositive, min_normal, FpStatus::INEXACT),
        (
            Round::TowardNegative,
            max_subnormal,
            FpStatus::UNDERFLOW | FpStatus::INEXACT,
        ),
        (
            Round::TowardZero,
            max_subnormal,
            FpStatus::UNDERFLOW | FpStatus::INEXACT,
        ),
    ] {
        let (value, status) = lhs.mul_ex(rhs, round);
        assert_eq!(value.to_bits(), expected.to_bits());
        assert_eq!(status, expected_status);
    }
}

#[test]
fn test_div() {
    check_div_exact(swfp::F32::ZERO, swfp::F32::INFINITY, swfp::F32::ZERO);
    check_div_exact(swfp::F32::ZERO, -swfp::F32::INFINITY, -swfp::F32::ZERO);
    check_div_exact(swfp::F32::INFINITY, swfp::F32::ZERO, swfp::F32::INFINITY);
    check_div_exact(swfp::F32::INFINITY, -swfp::F32::ZERO, -swfp::F32::INFINITY);

    check_div_exact(
        swfp::F32::from_int(6),
        swfp::F32::from_int(3),
        swfp::F32::from_int(2),
    );
    check_div_exact(
        swfp::F32::from_int(6),
        swfp::F32::from_int(-3),
        swfp::F32::from_int(-2),
    );
    check_div_exact(
        swfp::F32::from_int(-6),
        swfp::F32::from_int(3),
        swfp::F32::from_int(-2),
    );

    for s in [false, true] {
        check_div_round(
            mk_f32(s, 4, 0x4F0F0F),
            mk_f32(s, 10, 0x5AA10D),
            mk_f32(false, -7, 0x7273B4),
            mk_f32(false, -7, 0x7273B5),
            false,
            Loss::HalfUp,
        );
        check_div_round(
            mk_f32(s, 4, 0x4F0F0F),
            mk_f32(!s, 10, 0x5AA10D),
            mk_f32(true, -7, 0x7273B4),
            mk_f32(true, -7, 0x7273B5),
            true,
            Loss::HalfUp,
        );
        check_div_round(
            mk_f32(!s, 4, 0x4F0F0F),
            mk_f32(s, 10, 0x5AA10D),
            mk_f32(true, -7, 0x7273B4),
            mk_f32(true, -7, 0x7273B5),
            true,
            Loss::HalfUp,
        );

        let min_subnormal = mk_f32(s, -127, 1);
        let max_subnormal = mk_f32(s, -127, 0x7FFFFF);
        let min_normal = mk_f32(s, -126, 0);

        // Subnormal results
        check_div_exact(mk_f32(s, -100, 0), mk_f32(false, 49, 0), min_subnormal);
        check_div_exact(
            mk_f32(s, -100, 0x7FFFFE),
            mk_f32(false, 27, 0),
            max_subnormal,
        );
        check_div_round(
            mk_f32(s, -100, 0x000001),
            mk_f32(false, 28, 0),
            mk_f32(s, -127, 0x200000),
            mk_f32(s, -127, 0x200001),
            s,
            Loss::HalfDown,
        );

        // 2^-126 * (1 - 2^-24), halfway between the largest subnormal number
        // and the smallest normal number
        check_div_round(
            mk_f32(s, -100, 0x7FFFFF),
            mk_f32(false, 27, 0),
            max_subnormal,
            min_normal,
            s,
            Loss::HalfOdd,
        );
    }
}

#[test]
fn test_rem() {
    for a in 1..=2000 {
        for b in 1..=2000 {
            check_rem(
                swfp::F32::from_int(a),
                swfp::F32::from_int(b),
                swfp::F32::from_int(a % b),
                FpStatus::OK,
            );
            check_rem(
                -swfp::F32::from_int(a),
                swfp::F32::from_int(b),
                -swfp::F32::from_int(a % b),
                FpStatus::OK,
            );
            check_rem(
                swfp::F32::from_int(a),
                -swfp::F32::from_int(b),
                swfp::F32::from_int(a % b),
                FpStatus::OK,
            );
            check_rem(
                -swfp::F32::from_int(a),
                -swfp::F32::from_int(b),
                -swfp::F32::from_int(a % b),
                FpStatus::OK,
            );
        }
    }
}
