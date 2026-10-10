use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_compare, check_from_str_exact,
    check_from_str_round, check_round_int_exact, check_round_int_round, mk_f8e5m2,
    test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for bits in u8::MIN..=u8::MAX {
        let new_bits = swfp::F8E5M2::from_bits(bits).to_bits();
        if new_bits != bits {
            panic!("0x{new_bits:04X} != 0x{bits:04X}");
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F8E5M2>(true, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        check_category(mk_f8e5m2(s, -15, 0), FpCategory::Zero, s);

        for m in 0b01..=0b11 {
            check_category(mk_f8e5m2(s, -15, m), FpCategory::Subnormal, s);
        }

        for e in -14..=15 {
            for m in 0..=0b11 {
                check_category(mk_f8e5m2(s, e, m), FpCategory::Normal, s);
            }
        }

        check_category(mk_f8e5m2(s, 16, 0), FpCategory::Infinite, s);

        for m in 1..=0b11 {
            check_category(mk_f8e5m2(s, 16, m), FpCategory::Nan, s);
        }
    }
}

#[test]
fn test_compare() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E5M2::from_bits(a_bits);
        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E5M2::from_bits(b_bits);

            let a_nan = matches!(a_bits & 0x7F, 0x7D..=0x7F);
            let b_nan = matches!(b_bits & 0x7F, 0x7D..=0x7F);
            let expected_ord = if a_nan || b_nan {
                // NaN
                None
            } else if (a_bits & 0x7F) == 0 && (b_bits & 0x7F) == 0 {
                // both zero
                Some(std::cmp::Ordering::Equal)
            } else if a_bits >= 0x80 {
                if b_bits <= 0x7F {
                    Some(std::cmp::Ordering::Less)
                } else {
                    Some(b_bits.cmp(&a_bits))
                }
            } else if b_bits >= 0x80 {
                Some(std::cmp::Ordering::Greater)
            } else {
                Some(a_bits.cmp(&b_bits))
            };
            check_compare(a, b, expected_ord);
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
    for bits in u8::MIN..=u8::MAX {
        let value = swfp::F8E5M2::from_bits(bits);
        for round in round_modes {
            let (new_value, status) = swfp::F8E5M2::convert_from_ex(value, round);
            let new_bits = new_value.to_bits();

            let (expected_status, expected_bits) = if value.is_nan() && bits & 0b10 == 0 {
                (FpStatus::INVALID, bits | 0b10)
            } else {
                (FpStatus::OK, bits)
            };
            assert_eq!(status, expected_status);
            if new_bits != expected_bits {
                panic!("0x{new_bits:04X} != 0x{expected_bits:04X}");
            }
        }
    }
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::F8E5M2>();

    let exact = [
        ("57344", swfp::F8E5M2::from_bits(0x7B)),
        ("5.7344e4", swfp::F8E5M2::from_bits(0x7B)),
        ("0.0000152587890625", swfp::F8E5M2::from_bits(0x01)),
        ("4.57763671875e-5", swfp::F8E5M2::from_bits(0x03)),
        ("0.00006103515625", swfp::F8E5M2::from_bits(0x04)),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    // (input, rounded toward zero, rounded away from zero, loss)
    let inexact = [
        // Around the overflow threshold
        (
            "61439.999999999999999999999999999999999999999999999",
            0x7B,
            0x7C,
            Loss::HalfDown,
        ),
        ("61440", 0x7B, 0x7C, Loss::HalfOdd),
        (
            "61440.0000000000000000000000000000000000000000000001",
            0x7B,
            0x7C,
            Loss::HalfUp,
        ),
        (
            "65535.999999999999999999999999999999999999999999999",
            0x7B,
            0x7C,
            Loss::HalfUp,
        ),
        ("65536", 0x7B, 0x7C, Loss::Overflow),
        ("1e99999999999999999999", 0x7B, 0x7C, Loss::Overflow),
        ("1e9223372036854775807", 0x7B, 0x7C, Loss::Overflow),
        // Around the underflow threshold
        (
            "0.00000762939453124999999999999999999999999999999999999999999999",
            0x00,
            0x01,
            Loss::HalfDown,
        ),
        ("0.00000762939453125", 0x00, 0x01, Loss::HalfEven),
        (
            "0.000007629394531250000000000000000000000000000000000000000000001",
            0x00,
            0x01,
            Loss::HalfUp,
        ),
        ("0.00002288818359375", 0x01, 0x02, Loss::HalfOdd),
        (
            "5.340576171874999999999999999999999999999999999999999999999e-5",
            0x03,
            0x04,
            Loss::HalfDown,
        ),
        ("5.340576171875e-5", 0x03, 0x04, Loss::HalfOdd),
        (
            "5.3405761718750000000000000000000000000000000000000000000001e-5",
            0x03,
            0x04,
            Loss::HalfUp,
        ),
        ("1e-99999999999999999999", 0x00, 0x01, Loss::HalfDown),
        ("1e-9223372036854775808", 0x00, 0x01, Loss::HalfDown),
        // Ties in the normal range
        ("9", 0x48, 0x49, Loss::HalfEven),
        ("11", 0x49, 0x4A, Loss::HalfOdd),
        ("72", 0x54, 0x55, Loss::HalfEven),
        ("88", 0x55, 0x56, Loss::HalfOdd),
        (
            "1.124999999999999999999999999999999999999999999999",
            0x3C,
            0x3D,
            Loss::HalfDown,
        ),
        ("1.125", 0x3C, 0x3D, Loss::HalfEven),
        (
            "1.1250000000000000000000000000000000000000000000001",
            0x3C,
            0x3D,
            Loss::HalfUp,
        ),
        ("1.375", 0x3D, 0x3E, Loss::HalfOdd),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3B,
            0x3C,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x3C,
            0x3D,
            Loss::HalfDown,
        ),
        // Other values
        ("0.1", 0x2E, 0x2F, Loss::HalfDown),
        ("3.3", 0x42, 0x43, Loss::HalfUp),
        ("1e4", 0x70, 0x71, Loss::HalfUp),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::F8E5M2::from_bits(tz);
        let az = swfp::F8E5M2::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }
}

#[test]
fn test_to_from_str_roundtrip() {
    for bits in u8::MIN..=u8::MAX {
        let value = swfp::F8E5M2::from_bits(bits);
        let strings = [
            format!("{value:?}"),
            format!("{value}"),
            format!("{value:.10}"),
            format!("{value:.50}"),
            format!("{value:e}"),
            format!("{value:.5e}"),
            format!("{value:.50e}"),
            format!("{value:E}"),
            format!("{value:.5E}"),
            format!("{value:.50E}"),
        ];

        for s in strings {
            let expected_value = if value.is_nan() {
                swfp::F8E5M2::NAN
            } else {
                value
            };
            let parsed = s.parse::<swfp::F8E5M2>().unwrap();
            assert_eq!(parsed.to_bits(), expected_value.to_bits());
        }
    }
}

fn result_with_f32(
    (value, status): (swfp::F32, FpStatus),
    round: Round,
) -> (swfp::F8E5M2, FpStatus) {
    let (conv_value, conv_status) = swfp::F8E5M2::convert_from_ex(value, round);
    if conv_status == FpStatus::OK {
        (conv_value, status)
    } else {
        (conv_value, conv_status)
    }
}

#[test]
fn test_binary_op_exhaustive() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E5M2::from_bits(a_bits);
        let a32 = swfp::F32::convert_from(a);

        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E5M2::from_bits(b_bits);
            let b32 = swfp::F32::convert_from(b);

            for round in ALL_ROUND_MODES {
                if (a.is_nan() && a_bits & 0b10 == 0) || (b.is_nan() && b_bits & 0b10 == 0) {
                    let (r, status) = a.add_ex(b, round);
                    assert_eq!(status, FpStatus::INVALID);
                    assert!(r.is_nan());

                    let (r, status) = a.sub_ex(b, round);
                    assert_eq!(status, FpStatus::INVALID);
                    assert!(r.is_nan());

                    let (r, status) = a.mul_ex(b, round);
                    assert_eq!(status, FpStatus::INVALID);
                    assert!(r.is_nan());

                    let (r, status) = a.div_ex(b, round);
                    assert_eq!(status, FpStatus::INVALID);
                    assert!(r.is_nan());
                } else {
                    let (expected_r, expected_status) =
                        result_with_f32(a32.add_ex(b32, round), round);
                    let (r, status) = a.add_ex(b, round);
                    assert_eq!(status, expected_status);
                    assert_eq!(r.to_bits(), expected_r.to_bits());

                    let (expected_r, expected_status) =
                        result_with_f32(a32.sub_ex(b32, round), round);
                    let (r, status) = a.sub_ex(b, round);
                    assert_eq!(status, expected_status);
                    assert_eq!(r.to_bits(), expected_r.to_bits());

                    let (expected_r, expected_status) =
                        result_with_f32(a32.mul_ex(b32, round), round);
                    let (r, status) = a.mul_ex(b, round);
                    assert_eq!(status, expected_status);
                    assert_eq!(r.to_bits(), expected_r.to_bits());

                    let (expected_r, expected_status) =
                        result_with_f32(a32.div_ex(b32, round), round);
                    let (r, status) = a.div_ex(b, round);
                    assert_eq!(status, expected_status);
                    assert_eq!(r.to_bits(), expected_r.to_bits());
                }
            }
        }
    }
}

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::F8E5M2::NAN);
    check_round_int_exact(swfp::F8E5M2::INFINITY);
    check_round_int_exact(-swfp::F8E5M2::INFINITY);
    check_round_int_exact(swfp::F8E5M2::ZERO);
    check_round_int_exact(-swfp::F8E5M2::ZERO);

    for s in [false, true] {
        check_round_int_exact(mk_f8e5m2(s, 0, 0));
        check_round_int_exact(mk_f8e5m2(s, 2, 0b10));
        check_round_int_exact(mk_f8e5m2(s, 2, 0b11));

        let zero = mk_f8e5m2(s, -15, 0);
        let one = mk_f8e5m2(s, 0, 0);
        check_round_int_round(mk_f8e5m2(s, -15, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f8e5m2(s, -2, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f8e5m2(s, -1, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_f8e5m2(s, -1, 0b10), zero, one, Loss::HalfUp);
        check_round_int_round(mk_f8e5m2(s, -1, 0b11), zero, one, Loss::HalfUp);
    }
}
