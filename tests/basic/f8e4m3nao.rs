use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{ALL_ROUND_MODES, check_category, check_compare, mk_f8e4m3nao, test_from_str_specials};

#[test]
fn from_to_bits_lossless() {
    for bits in u8::MIN..=u8::MAX {
        let new_bits = swfp::F8E4M3Nao::from_bits(bits).to_bits();
        if new_bits != bits {
            panic!("0x{new_bits:04X} != 0x{bits:04X}");
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F8E4M3Nao>(false, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        check_category(mk_f8e4m3nao(s, -7, 0), FpCategory::Zero, s);

        for m in 1..=0b111 {
            check_category(mk_f8e4m3nao(s, -7, m), FpCategory::Subnormal, s);
        }

        for e in -6..=8 {
            for m in 0..=0b111 {
                let category = if e == 8 && m == 0b111 {
                    FpCategory::Nan
                } else {
                    FpCategory::Normal
                };
                check_category(mk_f8e4m3nao(s, e, m), category, s);
            }
        }
    }
}

#[test]
fn test_compare() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E4M3Nao::from_bits(a_bits);
        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E4M3Nao::from_bits(b_bits);
            let expected_ord = if (a_bits & 0x7F) == 0x7F || (b_bits & 0x7F) == 0x7F {
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
        let value = swfp::F8E4M3Nao::from_bits(bits);
        for round in round_modes {
            let (new_value, status) = swfp::F8E4M3Nao::convert_from_ex(value, round);
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
    test_from_str_specials::<swfp::F8E4M3Nao>();
}

#[test]
fn test_to_from_str_roundtrip() {
    for bits in u8::MIN..=u8::MAX {
        let value = swfp::F8E4M3Nao::from_bits(bits);
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
                swfp::F8E4M3Nao::NAN
            } else {
                value
            };
            let parsed = s.parse::<swfp::F8E4M3Nao>().unwrap();
            assert_eq!(parsed.to_bits(), expected_value.to_bits());
        }
    }
}

fn result_with_f32(
    (value, status): (swfp::F32, FpStatus),
    round: Round,
) -> (swfp::F8E4M3Nao, FpStatus) {
    let (conv_value, conv_status) = swfp::F8E4M3Nao::convert_from_ex(value, round);
    if conv_status == FpStatus::Ok {
        (conv_value, status)
    } else if status == FpStatus::DivByZero {
        (conv_value, FpStatus::DivByZero)
    } else {
        (conv_value, conv_status)
    }
}

#[test]
fn test_binary_op_exhaustive() {
    for a_bits in u8::MIN..=u8::MAX {
        let a = swfp::F8E4M3Nao::from_bits(a_bits);
        for b_bits in u8::MIN..=u8::MAX {
            let b = swfp::F8E4M3Nao::from_bits(b_bits);

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
