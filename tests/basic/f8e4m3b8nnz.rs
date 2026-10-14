use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus, Round};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_compare, check_fmt_specials, check_fmt_ties,
    check_from_str_exact, check_from_str_round, check_round_int_exact, check_round_int_round,
    mk_f8e4m3b8nnz, test_from_str_specials,
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
            assert_eq!(status, FpStatus::OK);
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
fn test_fmt_specials() {
    check_fmt_specials::<swfp::F8E4M3B8Nnz>(false, false);
    check_fmt_ties::<swfp::F8E4M3B8Nnz>(4);
}

#[test]
fn test_fmt_debug() {
    let v = swfp::F8E4M3B8Nnz::from_int(1);
    assert_eq!(format!("{v:?}"), "1.0");
    assert_eq!(format!("{v:+?}"), "+1.0");
    assert_eq!(format!("{v:.3?}"), "1.000");

    let v = swfp::F8E4M3B8Nnz::from_int(-1);
    assert_eq!(format!("{v:?}"), "-1.0");
    assert_eq!(format!("{v:+?}"), "-1.0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x41); // 1.125
    assert_eq!(format!("{v:?}"), "1.1");
    assert_eq!(format!("{v:+?}"), "+1.1");
    assert_eq!(format!("{v:.1?}"), "1.1");
    assert_eq!(format!("{v:.3?}"), "1.125");
    assert_eq!(format!("{v:07?}"), "00001.1");
    assert_eq!(format!("{v:+07?}"), "+0001.1");
    assert_eq!(format!("{v:7?}"), "    1.1");
    assert_eq!(format!("{v:+7?}"), "   +1.1");
    assert_eq!(format!("{v:<7?}"), "1.1    ");
    assert_eq!(format!("{v:<+7?}"), "+1.1   ");
    assert_eq!(format!("{v:^7?}"), "  1.1  ");
    assert_eq!(format!("{v:07.3?}"), "001.125");

    let v = swfp::F8E4M3B8Nnz::from_bits(0xC1); // -1.125
    assert_eq!(format!("{v:?}"), "-1.1");
    assert_eq!(format!("{v:+?}"), "-1.1");
    assert_eq!(format!("{v:.3?}"), "-1.125");
    assert_eq!(format!("{v:07?}"), "-0001.1");
    assert_eq!(format!("{v:7?}"), "   -1.1");
    assert_eq!(format!("{v:<7?}"), "-1.1   ");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x2D); // 0.203125
    assert_eq!(format!("{v:?}"), "0.2");
    assert_eq!(format!("{v:.4?}"), "0.2031");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7B); // 176
    assert_eq!(format!("{v:?}"), "180.0");
    assert_eq!(format!("{v:.1?}"), "176.0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7F); // MAX
    assert_eq!(format!("{v:?}"), "240.0");
    assert_eq!(format!("{v:.1?}"), "240.0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x08); // MIN normal
    assert_eq!(format!("{v:?}"), "0.008");
    assert_eq!(format!("{v:.4?}"), "0.0078");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x07); // MAX subnormal
    assert_eq!(format!("{v:?}"), "0.007");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x01); // MIN subnormal
    assert_eq!(format!("{v:?}"), "0.001");
    assert_eq!(format!("{v:.10?}"), "0.0009765625");
}

#[test]
fn test_fmt_display() {
    let v = swfp::F8E4M3B8Nnz::from_int(1);
    assert_eq!(format!("{v}"), "1");
    assert_eq!(format!("{v:+}"), "+1");
    assert_eq!(format!("{v:.3}"), "1.000");

    let v = swfp::F8E4M3B8Nnz::from_int(-1);
    assert_eq!(format!("{v}"), "-1");
    assert_eq!(format!("{v:+}"), "-1");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x41); // 1.125
    assert_eq!(format!("{v}"), "1.1");
    assert_eq!(format!("{v:+}"), "+1.1");
    assert_eq!(format!("{v:.0}"), "1");
    assert_eq!(format!("{v:.1}"), "1.1");
    assert_eq!(format!("{v:.2}"), "1.12");
    assert_eq!(format!("{v:.3}"), "1.125");
    assert_eq!(format!("{v:.5}"), "1.12500");
    assert_eq!(format!("{v:07}"), "00001.1");
    assert_eq!(format!("{v:+07}"), "+0001.1");
    assert_eq!(format!("{v:7}"), "    1.1");
    assert_eq!(format!("{v:+7}"), "   +1.1");
    assert_eq!(format!("{v:<7}"), "1.1    ");
    assert_eq!(format!("{v:<+7}"), "+1.1   ");
    assert_eq!(format!("{v:^7}"), "  1.1  ");
    assert_eq!(format!("{v:*>7.1}"), "****1.1");
    assert_eq!(format!("{v:+07.2}"), "+001.12");

    let v = swfp::F8E4M3B8Nnz::from_bits(0xC1); // -1.125
    assert_eq!(format!("{v}"), "-1.1");
    assert_eq!(format!("{v:+}"), "-1.1");
    assert_eq!(format!("{v:.1}"), "-1.1");
    assert_eq!(format!("{v:07}"), "-0001.1");
    assert_eq!(format!("{v:7}"), "   -1.1");
    assert_eq!(format!("{v:<7}"), "-1.1   ");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x47); // 1.875
    assert_eq!(format!("{v}"), "1.9");
    assert_eq!(format!("{v:.0}"), "2");
    assert_eq!(format!("{v:.1}"), "1.9");
    assert_eq!(format!("{v:.2}"), "1.88");
    assert_eq!(format!("{v:.3}"), "1.875");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x2D); // 0.203125
    assert_eq!(format!("{v}"), "0.2");
    assert_eq!(format!("{v:.1}"), "0.2");
    assert_eq!(format!("{v:.2}"), "0.20");
    assert_eq!(format!("{v:.3}"), "0.203");
    assert_eq!(format!("{v:.4}"), "0.2031");
    assert_eq!(format!("{v:.8}"), "0.20312500");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x5C); // 12
    assert_eq!(format!("{v}"), "12");
    assert_eq!(format!("{v:.1}"), "12.0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7B); // 176
    assert_eq!(format!("{v}"), "180");
    assert_eq!(format!("{v:.0}"), "176");
    assert_eq!(format!("{v:.2}"), "176.00");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7F); // MAX
    assert_eq!(format!("{v}"), "240");
    assert_eq!(format!("{v:.0}"), "240");
    assert_eq!(format!("{v:.2}"), "240.00");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x08); // MIN normal
    assert_eq!(format!("{v}"), "0.008");
    assert_eq!(format!("{v:.2}"), "0.01");
    assert_eq!(format!("{v:.4}"), "0.0078");
    assert_eq!(format!("{v:.10}"), "0.0078125000");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x07); // MAX subnormal
    assert_eq!(format!("{v}"), "0.007");
    assert_eq!(format!("{v:.10}"), "0.0068359375");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x01); // MIN subnormal
    assert_eq!(format!("{v}"), "0.001");
    assert_eq!(format!("{v:.2}"), "0.00");
    assert_eq!(format!("{v:.4}"), "0.0010");
    assert_eq!(format!("{v:.10}"), "0.0009765625");
    assert_eq!(format!("{v:.14}"), "0.00097656250000");
}

#[test]
fn test_fmt_exp() {
    let v = swfp::F8E4M3B8Nnz::from_int(1);
    assert_eq!(format!("{v:e}"), "1e0");
    assert_eq!(format!("{v:E}"), "1E0");
    assert_eq!(format!("{v:+e}"), "+1e0");
    assert_eq!(format!("{v:.2e}"), "1.00e0");
    assert_eq!(format!("{v:.2E}"), "1.00E0");

    let v = swfp::F8E4M3B8Nnz::from_int(-1);
    assert_eq!(format!("{v:e}"), "-1e0");
    assert_eq!(format!("{v:+e}"), "-1e0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x41); // 1.125
    assert_eq!(format!("{v:e}"), "1.1e0");
    assert_eq!(format!("{v:E}"), "1.1E0");
    assert_eq!(format!("{v:+e}"), "+1.1e0");
    assert_eq!(format!("{v:.0e}"), "1e0");
    assert_eq!(format!("{v:.1e}"), "1.1e0");
    assert_eq!(format!("{v:.2e}"), "1.12e0");
    assert_eq!(format!("{v:.3e}"), "1.125e0");
    assert_eq!(format!("{v:.5E}"), "1.12500E0");
    assert_eq!(format!("{v:09e}"), "00001.1e0");
    assert_eq!(format!("{v:+09e}"), "+0001.1e0");
    assert_eq!(format!("{v:9e}"), "    1.1e0");
    assert_eq!(format!("{v:+9e}"), "   +1.1e0");
    assert_eq!(format!("{v:<9e}"), "1.1e0    ");
    assert_eq!(format!("{v:<+9e}"), "+1.1e0   ");
    assert_eq!(format!("{v:^9e}"), "  1.1e0  ");
    assert_eq!(format!("{v:+010.2e}"), "+0001.12e0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0xC1); // -1.125
    assert_eq!(format!("{v:e}"), "-1.1e0");
    assert_eq!(format!("{v:+e}"), "-1.1e0");
    assert_eq!(format!("{v:.1e}"), "-1.1e0");
    assert_eq!(format!("{v:09e}"), "-0001.1e0");
    assert_eq!(format!("{v:9e}"), "   -1.1e0");
    assert_eq!(format!("{v:<9e}"), "-1.1e0   ");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x47); // 1.875
    assert_eq!(format!("{v:e}"), "1.9e0");
    assert_eq!(format!("{v:.0e}"), "2e0");
    assert_eq!(format!("{v:.1e}"), "1.9e0");
    assert_eq!(format!("{v:.2e}"), "1.88e0");
    assert_eq!(format!("{v:.3e}"), "1.875e0");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x2D); // 0.203125
    assert_eq!(format!("{v:e}"), "2e-1");
    assert_eq!(format!("{v:.0e}"), "2e-1");
    assert_eq!(format!("{v:.1e}"), "2.0e-1");
    assert_eq!(format!("{v:.2e}"), "2.03e-1");
    assert_eq!(format!("{v:.6e}"), "2.031250e-1");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x5C); // 12
    assert_eq!(format!("{v:e}"), "1.2e1");
    assert_eq!(format!("{v:.0e}"), "1e1");
    assert_eq!(format!("{v:.2e}"), "1.20e1");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7B); // 176
    assert_eq!(format!("{v:e}"), "1.8e2");
    assert_eq!(format!("{v:E}"), "1.8E2");
    assert_eq!(format!("{v:.0e}"), "2e2");
    assert_eq!(format!("{v:.1e}"), "1.8e2");
    assert_eq!(format!("{v:.3e}"), "1.760e2");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x7F); // MAX
    assert_eq!(format!("{v:e}"), "2.4e2");
    assert_eq!(format!("{v:E}"), "2.4E2");
    assert_eq!(format!("{v:.0e}"), "2e2");
    assert_eq!(format!("{v:.1e}"), "2.4e2");
    assert_eq!(format!("{v:.4e}"), "2.4000e2");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x08); // MIN normal
    assert_eq!(format!("{v:e}"), "8e-3");
    assert_eq!(format!("{v:.0e}"), "8e-3");
    assert_eq!(format!("{v:.2e}"), "7.81e-3");
    assert_eq!(format!("{v:.5e}"), "7.81250e-3");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x07); // MAX subnormal
    assert_eq!(format!("{v:e}"), "7e-3");
    assert_eq!(format!("{v:.2e}"), "6.84e-3");
    assert_eq!(format!("{v:.5e}"), "6.83594e-3");

    let v = swfp::F8E4M3B8Nnz::from_bits(0x01); // MIN subnormal
    assert_eq!(format!("{v:e}"), "1e-3");
    assert_eq!(format!("{v:E}"), "1E-3");
    assert_eq!(format!("{v:.0e}"), "1e-3");
    assert_eq!(format!("{v:.2e}"), "9.77e-4");
    assert_eq!(format!("{v:.5e}"), "9.76562e-4");
    assert_eq!(format!("{v:.10E}"), "9.7656250000E-4");
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
    if conv_status == FpStatus::OK {
        (conv_value, status)
    } else if status == FpStatus::DIV_BY_ZERO {
        (conv_value, FpStatus::DIV_BY_ZERO)
    } else if value != swfp::F32::ZERO {
        (conv_value, conv_status)
    } else {
        (conv_value, FpStatus::OK)
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
