use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_compare, check_from_str_exact,
    check_from_str_round, check_to_uint_exact, mk_f16, test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for bits in u16::MIN..=u16::MAX {
        let new_bits = swfp::F16::from_bits(bits).to_bits();
        if new_bits != bits {
            panic!("0x{new_bits:04X} != 0x{bits:04X}");
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F16>(true, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        check_category(mk_f16(s, -15, 0), FpCategory::Zero, s);

        for m in 1..(1 << 10) {
            check_category(mk_f16(s, -15, m), FpCategory::Subnormal, s);
        }

        for e in -14..=15 {
            for m in 0..(1 << 10) {
                check_category(mk_f16(s, e, m), FpCategory::Normal, s);
            }
        }

        check_category(mk_f16(s, 16, 0), FpCategory::Infinite, s);

        for m in 1..(1 << 10) {
            check_category(mk_f16(s, 16, m), FpCategory::Nan, s);
        }
    }
}

#[test]
fn test_compare() {
    for a_hi in 0..255 {
        for a_lo in 0..15 {
            let a_bits = (a_hi << 8) | a_lo;
            let a = swfp::F16::from_bits(a_bits);
            for b_hi in 0..255 {
                for b_lo in 0..15 {
                    let b_bits = (b_hi << 8) | b_lo;
                    let b = swfp::F16::from_bits(b_bits);

                    let a_nan = matches!(a_bits & 0x7FFF, 0x7C01..=0x7FFF);
                    let b_nan = matches!(b_bits & 0x7FFF, 0x7C01..=0x7FFF);
                    let expected_ord = if a_nan || b_nan {
                        // NaN
                        None
                    } else if (a_bits & 0x7FFF) == 0 && (b_bits & 0x7FFF) == 0 {
                        // both zero
                        Some(std::cmp::Ordering::Equal)
                    } else if a_bits >= 0x8000 {
                        if b_bits <= 0x7FFF {
                            Some(std::cmp::Ordering::Less)
                        } else {
                            Some(b_bits.cmp(&a_bits))
                        }
                    } else if b_bits >= 0x8000 {
                        Some(std::cmp::Ordering::Greater)
                    } else {
                        Some(a_bits.cmp(&b_bits))
                    };
                    check_compare(a, b, expected_ord);
                }
            }
        }
    }
}

#[test]
fn test_convert_to_self() {
    let round_modes = [
        Round::NearestTiesToEven,
        Round::NearestTiesToAway,
        Round::TowardPositive,
        Round::TowardNegative,
        Round::TowardZero,
    ];
    for bits in u16::MIN..=u16::MAX {
        let value = swfp::F16::from_bits(bits);
        for round in round_modes {
            let (new_value, status) = swfp::F16::convert_from_ex(value, round);
            let new_bits = new_value.to_bits();
            let expected_status;
            let expected_bits;
            if value.is_nan() && bits & 0x0200 == 0 {
                expected_status = FpStatus::Invalid;
                expected_bits = bits | 0x0200;
            } else {
                expected_status = FpStatus::Ok;
                expected_bits = bits;
            }
            assert_eq!(status, expected_status);
            if new_bits != expected_bits {
                panic!("0x{new_bits:04X} != 0x{expected_bits:04X}");
            }
        }
    }
}

#[test]
fn test_to_uint() {
    check_to_uint_exact(mk_f16(false, 10, 0), (Some(1024), FpStatus::Ok));
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::F16>();

    check_from_str_round(
        "1e5",
        mk_f16(false, 15, 0x3FF),
        swfp::F16::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        "111111",
        mk_f16(false, 15, 0x3FF),
        swfp::F16::INFINITY,
        Loss::Overflow,
    );

    check_from_str_round(
        "-1e5",
        mk_f16(true, 15, 0x3FF),
        -swfp::F16::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        "-111111",
        mk_f16(true, 15, 0x3FF),
        -swfp::F16::INFINITY,
        Loss::Overflow,
    );

    for e in -14..=15 {
        for m in 0..(1 << 10) {
            let value = mk_f16(false, e, m);
            let s = format!(
                "{:.30e}",
                rug::Float::with_val(11, m | (1 << 10)) << (i32::from(e) - 10),
            );
            check_from_str_exact(&s, value);
            check_from_str_exact(&format!("-{s}"), -value);
        }
    }

    for m in 0..(1 << 10) {
        let value = mk_f16(false, -15, m);
        let s = format!("{:.30e}", rug::Float::with_val(11, m) >> (14 + 10));
        check_from_str_exact(&s, value);
        check_from_str_exact(&format!("-{s}"), -value);
    }
}

#[test]
fn test_fmt_display() {
    let v = swfp::F16::from_int(1);
    assert_eq!(format!("{v}"), "1");

    let v = swfp::F16::from_bits(0x3C01); // 1.001
    assert_eq!(format!("{v}"), "1.001");
    assert_eq!(format!("{v:.6}"), "1.000977");

    let v = swfp::F16::from_bits(0x3C02); // 1.002
    assert_eq!(format!("{v}"), "1.002");

    let v = swfp::F16::from_bits(0x3C04); // 1.004
    assert_eq!(format!("{v}"), "1.004");
    assert_eq!(format!("{v:.2}"), "1.00");

    let v = swfp::F16::from_bits(0x3C06); // 1.006
    assert_eq!(format!("{v}"), "1.006");
    assert_eq!(format!("{v:.2}"), "1.01");

    let v = swfp::F16::from_bits(0x7BFF); // MAX
    assert_eq!(format!("{v}"), "65500");

    let v = swfp::F16::from_bits(0x0100); // MIN normal
    assert_eq!(format!("{v}"), "0.00001526");

    let v = swfp::F16::from_bits(0x0001); // MIN subnormal
    assert_eq!(format!("{v}"), "0.00000006");
}

#[test]
fn test_to_from_str_roundtrip() {
    for bits in u16::MIN..=u16::MAX {
        let value = swfp::F16::from_bits(bits);
        let strings = [
            format!("{value:?}"),
            format!("{value}"),
            format!("{value:.20}"),
            format!("{value:.100}"),
            format!("{value:e}"),
            format!("{value:.10e}"),
            format!("{value:.100e}"),
            format!("{value:E}"),
            format!("{value:.10E}"),
            format!("{value:.100E}"),
        ];

        for s in strings {
            let expected_value = if value.is_nan() {
                swfp::F16::NAN
            } else {
                value
            };
            let parsed = s.parse::<swfp::F16>().unwrap();
            assert_eq!(parsed.to_bits(), expected_value.to_bits());
        }
    }
}

fn result_with_f32((value, status): (swfp::F32, FpStatus), round: Round) -> (swfp::F16, FpStatus) {
    let (conv_value, conv_status) = swfp::F16::convert_from_ex(value, round);
    if conv_status == FpStatus::Ok {
        (conv_value, status)
    } else {
        (conv_value, conv_status)
    }
}

#[test]
fn test_mul_div() {
    for a_mant in 0..(1 << 10) {
        let a = mk_f16(false, 0, a_mant);
        let a32 = swfp::F32::convert_from(a);

        for b_mant in 0..(1 << 10) {
            let b = mk_f16(false, 0, b_mant);
            let b32 = swfp::F32::convert_from(b);

            for round in ALL_ROUND_MODES {
                let (expected_r, expected_status) = result_with_f32(a32.mul_ex(b32, round), round);
                let (r, status) = a.mul_ex(b, round);
                assert_eq!(status, expected_status);
                assert_eq!(r.to_bits(), expected_r.to_bits());

                let (expected_r, expected_status) = result_with_f32(a32.div_ex(b32, round), round);
                let (r, status) = a.div_ex(b, round);
                assert_eq!(status, expected_status);
                assert_eq!(r.to_bits(), expected_r.to_bits());
            }
        }
    }
}
