use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_from_str_exact, check_from_str_round, mk_f64,
    test_from_str_specials,
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
                let expected_status;
                let expected_bits;
                if value.is_nan() && bits & (1 << 51) == 0 {
                    expected_status = FpStatus::Invalid;
                    expected_bits = bits | (1 << 51);
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
fn test_from_str() {
    test_from_str_specials::<swfp::F64>();

    check_from_str_round(
        "1e309",
        mk_f64(false, 1023, 0xFFFFFFFFFFFFF),
        swfp::F64::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        &"1".repeat(310),
        mk_f64(false, 1023, 0xFFFFFFFFFFFFF),
        swfp::F64::INFINITY,
        Loss::Overflow,
    );

    check_from_str_round(
        "-1e309",
        mk_f64(true, 1023, 0xFFFFFFFFFFFFF),
        -swfp::F64::INFINITY,
        Loss::Overflow,
    );
    check_from_str_round(
        &format!("-{}", "1".repeat(310)),
        mk_f64(true, 1023, 0xFFFFFFFFFFFFF),
        -swfp::F64::INFINITY,
        Loss::Overflow,
    );

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F64::from_int(n));
    }

    for e in -1074..=1023 {
        let s = format!("{:.1000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F64::from_int(1).scalbn(e));
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
