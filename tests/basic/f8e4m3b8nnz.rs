use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_compare, check_from_str_exact,
    check_from_str_round, check_round_int_exact, check_round_int_round, mk_f8e4m3b8nnz,
    test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for bits in u8::MIN..=u8::MAX {
        let new_bits = swfp::F8E4M3B8Nnz::from_bits(bits).to_bits();
        if new_bits != bits {
            panic!("0x{new_bits:04X} != 0x{bits:04X}");
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F8E4M3B8Nnz>(false, true);
}

#[test]
fn test_categories() {
    check_category(mk_f8e4m3b8nnz(false, -8, 0), FpCategory::Zero, false);
    check_category(mk_f8e4m3b8nnz(true, -8, 0), FpCategory::Nan, true);

    for s in [false, true] {
        for m in 1..=0b111 {
            check_category(mk_f8e4m3b8nnz(s, -8, m), FpCategory::Subnormal, s);
        }

        for e in -7..=7 {
            for m in 0..=0b111 {
                check_category(mk_f8e4m3b8nnz(s, e, m), FpCategory::Normal, s);
            }
        }
    }
}

#[test]
fn test_compare() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E4M3B8Nnz::from_bits(a_bits);
        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E4M3B8Nnz::from_bits(b_bits);
            let expected_ord = if a_bits == 0x80 || b_bits == 0x80 {
                // NaN
                None
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
        let value = swfp::F8E4M3B8Nnz::from_bits(bits);
        for round in round_modes {
            let (new_value, status) = swfp::F8E4M3B8Nnz::convert_from_ex(value, round);
            let new_bits = new_value.to_bits();
            assert_eq!(status, FpStatus::Ok);
            if new_bits != bits {
                panic!("0x{new_bits:04X} != 0x{bits:04X}");
            }
        }
    }
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::F8E4M3B8Nnz>();

    let exact = [
        ("240", swfp::F8E4M3B8Nnz::from_bits(0x7F)),
        ("2.4e2", swfp::F8E4M3B8Nnz::from_bits(0x7F)),
        ("0.0009765625", swfp::F8E4M3B8Nnz::from_bits(0x01)),
        ("6.8359375e-3", swfp::F8E4M3B8Nnz::from_bits(0x07)),
        ("0.0078125", swfp::F8E4M3B8Nnz::from_bits(0x08)),
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
            "247.999999999999999999999999999999999999999999999",
            0x7F,
            0x80,
            Loss::HalfDown,
        ),
        ("248", 0x7F, 0x80, Loss::HalfOdd),
        (
            "248.0000000000000000000000000000000000000000000001",
            0x7F,
            0x80,
            Loss::HalfUp,
        ),
        (
            "255.999999999999999999999999999999999999999999999",
            0x7F,
            0x80,
            Loss::HalfUp,
        ),
        ("256", 0x7F, 0x80, Loss::Overflow),
        ("1e99999999999999999999", 0x7F, 0x80, Loss::Overflow),
        ("1e9223372036854775807", 0x7F, 0x80, Loss::Overflow),
        // Around the underflow threshold
        (
            "0.00048828124999999999999999999999999999999999999999999999",
            0x00,
            0x01,
            Loss::HalfDown,
        ),
        ("0.00048828125", 0x00, 0x01, Loss::HalfEven),
        (
            "0.000488281250000000000000000000000000000000000000000000001",
            0x00,
            0x01,
            Loss::HalfUp,
        ),
        ("0.00146484375", 0x01, 0x02, Loss::HalfOdd),
        (
            "7.32421874999999999999999999999999999999999999999999999e-3",
            0x07,
            0x08,
            Loss::HalfDown,
        ),
        ("7.32421875e-3", 0x07, 0x08, Loss::HalfOdd),
        (
            "7.324218750000000000000000000000000000000000000000000001e-3",
            0x07,
            0x08,
            Loss::HalfUp,
        ),
        ("1e-99999999999999999999", 0x00, 0x01, Loss::HalfDown),
        ("1e-9223372036854775808", 0x00, 0x01, Loss::HalfDown),
        // Ties in the normal range
        ("17", 0x60, 0x61, Loss::HalfEven),
        ("19", 0x61, 0x62, Loss::HalfOdd),
        ("68", 0x70, 0x71, Loss::HalfEven),
        ("76", 0x71, 0x72, Loss::HalfOdd),
        (
            "1.0624999999999999999999999999999999999999999999999",
            0x40,
            0x41,
            Loss::HalfDown,
        ),
        ("1.0625", 0x40, 0x41, Loss::HalfEven),
        (
            "1.06250000000000000000000000000000000000000000000001",
            0x40,
            0x41,
            Loss::HalfUp,
        ),
        ("1.1875", 0x41, 0x42, Loss::HalfOdd),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3F,
            0x40,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x40,
            0x41,
            Loss::HalfDown,
        ),
        // Other values
        ("0.1", 0x24, 0x25, Loss::HalfUp),
        ("3.3", 0x4D, 0x4E, Loss::HalfDown),
        ("100", 0x74, 0x75, Loss::HalfEven),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::F8E4M3B8Nnz::from_bits(tz);
        let az = swfp::F8E4M3B8Nnz::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }
}

#[test]
fn test_to_from_str_roundtrip() {
    for bits in u8::MIN..=u8::MAX {
        let value = swfp::F8E4M3B8Nnz::from_bits(bits);
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
                swfp::F8E4M3B8Nnz::NAN
            } else {
                value
            };
            let parsed = s.parse::<swfp::F8E4M3B8Nnz>().unwrap();
            assert_eq!(parsed.to_bits(), expected_value.to_bits());
        }
    }
}

fn result_with_f32(
    (value, status): (swfp::F32, FpStatus),
    round: Round,
) -> (swfp::F8E4M3B8Nnz, FpStatus) {
    let (conv_value, conv_status) = swfp::F8E4M3B8Nnz::convert_from_ex(value, round);
    if conv_status == FpStatus::Ok {
        (conv_value, status)
    } else if status == FpStatus::DivByZero {
        (conv_value, FpStatus::DivByZero)
    } else if value != swfp::F32::ZERO {
        (conv_value, conv_status)
    } else {
        (conv_value, FpStatus::Ok)
    }
}

#[test]
fn test_binary_op_exhaustive() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E4M3B8Nnz::from_bits(a_bits);
        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E4M3B8Nnz::from_bits(b_bits);

            for round in ALL_ROUND_MODES {
                let a32 = swfp::F32::convert_from(a);
                let b32 = swfp::F32::convert_from(b);

                let (expected_r, expected_status) = result_with_f32(a32.add_ex(b32, round), round);
                let (r, status) = a.add_ex(b, round);
                assert_eq!(status, expected_status);
                assert_eq!(r.to_bits(), expected_r.to_bits());

                let (expected_r, expected_status) = result_with_f32(a32.sub_ex(b32, round), round);
                let (r, status) = a.sub_ex(b, round);
                assert_eq!(status, expected_status);
                assert_eq!(r.to_bits(), expected_r.to_bits());

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

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::F8E4M3B8Nnz::NAN);
    check_round_int_exact(swfp::F8E4M3B8Nnz::INFINITY);
    check_round_int_exact(-swfp::F8E4M3B8Nnz::INFINITY);
    check_round_int_exact(swfp::F8E4M3B8Nnz::ZERO);
    check_round_int_exact(-swfp::F8E4M3B8Nnz::ZERO);

    for s in [false, true] {
        check_round_int_exact(mk_f8e4m3b8nnz(s, 0, 0));
        check_round_int_exact(mk_f8e4m3b8nnz(s, 2, 0b100));
        check_round_int_exact(mk_f8e4m3b8nnz(s, 3, 0b111));

        let zero = mk_f8e4m3b8nnz(false, -8, 0);
        let one = mk_f8e4m3b8nnz(s, 0, 0);
        check_round_int_round(mk_f8e4m3b8nnz(s, -8, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f8e4m3b8nnz(s, -2, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f8e4m3b8nnz(s, -1, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_f8e4m3b8nnz(s, -1, 0b100), zero, one, Loss::HalfUp);
        check_round_int_round(mk_f8e4m3b8nnz(s, -1, 0b111), zero, one, Loss::HalfUp);
    }
}
