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
                let expected_status;
                let expected_bits;
                if value.is_nan() && bits & (1 << 22) == 0 {
                    expected_status = FpStatus::Invalid;
                    expected_bits = bits | (1 << 22);
                } else {
                    expected_status = FpStatus::Ok;
                    expected_bits = bits;
                }
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
    check_to_uint_exact(swfp::F32::NAN, (None, FpStatus::Invalid));
    check_to_uint_exact(swfp::F32::INFINITY, (None, FpStatus::Overflow));
    check_to_uint_exact(-swfp::F32::INFINITY, (None, FpStatus::Overflow));

    check_to_uint_exact(swfp::F32::ZERO, (Some(0), FpStatus::Ok));
    check_to_uint_exact(-swfp::F32::ZERO, (Some(0), FpStatus::Inexact));

    for e in 0..=127 {
        check_to_uint_exact(mk_f32(false, e, 0), (Some(1 << e), FpStatus::Ok));
        check_to_uint_exact(mk_f32(true, e, 0), (None, FpStatus::Overflow));
    }

    for hi in 0..=255 {
        for lo in 0..=255 {
            let m = (hi << (23 - 8)) | lo;
            check_to_uint_exact(
                mk_f32(false, 23, m),
                (Some(u128::from(m | (1 << 23))), FpStatus::Ok),
            );
            check_to_uint_exact(mk_f32(true, 23, m), (None, FpStatus::Overflow));
        }
    }

    for e in 23..=127 {
        let m = 0x7FEFEF;
        check_to_uint_exact(
            mk_f32(false, e, m),
            (Some(u128::from(m | (1 << 23)) << (e - 23)), FpStatus::Ok),
        );
        check_to_uint_exact(mk_f32(true, e, m), (None, FpStatus::Overflow));
    }

    check_to_uint_round(
        mk_f32(false, -1, 0),
        (Some(0), FpStatus::Inexact),
        (Some(1), FpStatus::Inexact),
        Loss::HalfEven,
    );
    check_to_uint_round(
        mk_f32(false, 0, 1 << 22),
        (Some(1), FpStatus::Inexact),
        (Some(2), FpStatus::Inexact),
        Loss::HalfOdd,
    );
    check_to_uint_round(
        mk_f32(false, -1, 1 << 22),
        (Some(0), FpStatus::Inexact),
        (Some(1), FpStatus::Inexact),
        Loss::HalfUp,
    );
    for e in -126..=-2 {
        check_to_uint_round(
            mk_f32(false, e, 0),
            (Some(0), FpStatus::Inexact),
            (Some(1), FpStatus::Inexact),
            Loss::HalfDown,
        );
    }
}

#[test]
fn test_to_int() {
    check_to_int_exact(swfp::F32::NAN, (None, FpStatus::Invalid));
    check_to_int_exact(swfp::F32::INFINITY, (None, FpStatus::Overflow));
    check_to_int_exact(-swfp::F32::INFINITY, (None, FpStatus::Overflow));

    check_to_int_exact(swfp::F32::ZERO, (Some(0), FpStatus::Ok));
    check_to_int_exact(-swfp::F32::ZERO, (Some(0), FpStatus::Inexact));

    for e in 0..=127 {
        check_to_int_exact(
            mk_f32(false, e, 0),
            if e == 127 {
                (None, FpStatus::Overflow)
            } else {
                (Some(1 << e), FpStatus::Ok)
            },
        );
        check_to_int_exact(mk_f32(true, e, 0), (Some(-1 << e), FpStatus::Ok));
    }

    for hi in 0..=255 {
        for lo in 0..=255 {
            let m = (hi << (23 - 8)) | lo;
            check_to_int_exact(
                mk_f32(false, 23, m),
                (Some(i128::from(m | (1 << 23))), FpStatus::Ok),
            );
            check_to_int_exact(
                mk_f32(true, 23, m),
                (Some(-i128::from(m | (1 << 23))), FpStatus::Ok),
            );
        }
    }

    for e in 23..=126 {
        let m = 0x7FEFEF;
        check_to_int_exact(
            mk_f32(false, e, m),
            (Some(i128::from(m | (1 << 23)) << (e - 23)), FpStatus::Ok),
        );
        check_to_int_exact(
            mk_f32(true, e, m),
            (Some(-i128::from(m | (1 << 23)) << (e - 23)), FpStatus::Ok),
        );
    }

    check_to_int_round(
        mk_f32(false, -1, 0),
        (Some(0), FpStatus::Inexact),
        (Some(1), FpStatus::Inexact),
        Loss::HalfEven,
    );
    check_to_int_round(
        mk_f32(true, -1, 0),
        (Some(0), FpStatus::Inexact),
        (Some(-1), FpStatus::Inexact),
        Loss::HalfEven,
    );
    check_to_int_round(
        mk_f32(false, 0, 1 << 22),
        (Some(1), FpStatus::Inexact),
        (Some(2), FpStatus::Inexact),
        Loss::HalfOdd,
    );
    check_to_int_round(
        mk_f32(true, 0, 1 << 22),
        (Some(-1), FpStatus::Inexact),
        (Some(-2), FpStatus::Inexact),
        Loss::HalfOdd,
    );
    check_to_int_round(
        mk_f32(false, -1, 1 << 22),
        (Some(0), FpStatus::Inexact),
        (Some(1), FpStatus::Inexact),
        Loss::HalfUp,
    );
    check_to_int_round(
        mk_f32(true, -1, 1 << 22),
        (Some(0), FpStatus::Inexact),
        (Some(-1), FpStatus::Inexact),
        Loss::HalfUp,
    );
    for e in -126..=-2 {
        check_to_int_round(
            mk_f32(false, e, 0),
            (Some(0), FpStatus::Inexact),
            (Some(1), FpStatus::Inexact),
            Loss::HalfDown,
        );
        check_to_int_round(
            mk_f32(true, e, 0),
            (Some(0), FpStatus::Inexact),
            (Some(-1), FpStatus::Inexact),
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
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    check_from_str_round(
        "1e39",
        mk_f32(false, 127, 0x7FFFFF),
        swfp::F32::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        &"1".repeat(40),
        mk_f32(false, 127, 0x7FFFFF),
        swfp::F32::INFINITY,
        Loss::Overflow,
    );

    check_from_str_round(
        "-1e39",
        mk_f32(true, 127, 0x7FFFFF),
        -swfp::F32::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        &format!("-{}", "1".repeat(40)),
        mk_f32(true, 127, 0x7FFFFF),
        -swfp::F32::INFINITY,
        Loss::Overflow,
    );

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F32::from_int(n));
    }

    for e in -149..=127 {
        let s = format!("{:.1000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F32::from_int(1).scalbn(e));
    }

    check_from_str_round(
        "0.1",
        swfp::F32::from_bits(0x3DCCCCCC),
        swfp::F32::from_bits(0x3DCCCCCD),
        Loss::HalfUp,
    );
    check_from_str_round(
        "0.7",
        swfp::F32::from_bits(0x3F333333),
        swfp::F32::from_bits(0x3F333334),
        Loss::HalfDown,
    );
    check_from_str_round(
        "10528446.5",
        swfp::F32::from_bits(0x4B20A6BE),
        swfp::F32::from_bits(0x4B20A6BF),
        Loss::HalfEven,
    );
    check_from_str_round(
        "10528445.5",
        swfp::F32::from_bits(0x4B20A6BD),
        swfp::F32::from_bits(0x4B20A6BE),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "-0.1",
        swfp::F32::from_bits(0xBDCCCCCC),
        swfp::F32::from_bits(0xBDCCCCCD),
        Loss::HalfUp,
    );
    check_from_str_round(
        "-0.7",
        swfp::F32::from_bits(0xBF333333),
        swfp::F32::from_bits(0xBF333334),
        Loss::HalfDown,
    );
    check_from_str_round(
        "-10528446.5",
        swfp::F32::from_bits(0xCB20A6BE),
        swfp::F32::from_bits(0xCB20A6BF),
        Loss::HalfEven,
    );
    check_from_str_round(
        "-10528445.5",
        swfp::F32::from_bits(0xCB20A6BD),
        swfp::F32::from_bits(0xCB20A6BE),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
        swfp::F32::from_bits(0x3EAAAAAA),
        swfp::F32::from_bits(0x3EAAAAAB),
        Loss::HalfUp,
    );
    check_from_str_round(
        "1.50000000000000000000000000000000000000000000000000000000000000000001",
        swfp::F32::from_bits(0x3FC00000),
        swfp::F32::from_bits(0x3FC00001),
        Loss::HalfDown,
    );
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
            assert_eq!(status, FpStatus::Ok);
            assert_eq!(r.to_bits(), expected_r.to_bits());

            let (r, status) = mk_f32(s, 0, 1).add_ex(mk_f32(!s, 0, 1), round);
            assert_eq!(status, FpStatus::Ok);
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
                FpStatus::Ok,
            );
            check_rem(
                -swfp::F32::from_int(a),
                swfp::F32::from_int(b),
                -swfp::F32::from_int(a % b),
                FpStatus::Ok,
            );
            check_rem(
                swfp::F32::from_int(a),
                -swfp::F32::from_int(b),
                swfp::F32::from_int(a % b),
                FpStatus::Ok,
            );
            check_rem(
                -swfp::F32::from_int(a),
                -swfp::F32::from_int(b),
                -swfp::F32::from_int(a % b),
                FpStatus::Ok,
            );
        }
    }
}
