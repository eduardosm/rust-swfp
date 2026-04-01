use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_from_str_exact, check_from_str_round, mk_f128,
    test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    for hi in 0..=0xFFFFF {
        for lo in 0..=0x7 {
            let bits = (hi << (128 - 20)) | lo;
            let new_bits = swfp::F128::from_bits(bits).to_bits();
            if new_bits != bits {
                panic!("0x{new_bits:04X} != 0x{bits:04X}");
            }
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::F128>(true, false);
}

#[test]
fn test_categories() {
    for s in [false, true] {
        for hi in 0..=127 {
            for lo in 0..=127 {
                let m = (hi << (112 - 7)) | lo;
                let category = if m == 0 {
                    FpCategory::Zero
                } else {
                    FpCategory::Subnormal
                };
                check_category(mk_f128(s, -16383, m), category, s);
            }
        }

        for e in -16382..=16383 {
            for hi in 0..=127 {
                for lo in 0..=127 {
                    let m = (hi << (112 - 7)) | lo;
                    check_category(mk_f128(s, e, m), FpCategory::Normal, s);
                }
            }
        }

        for hi in 0..=127 {
            for lo in 0..=127 {
                let m = (hi << (112 - 7)) | lo;
                let category = if m == 0 {
                    FpCategory::Infinite
                } else {
                    FpCategory::Nan
                };
                check_category(mk_f128(s, 16384, m), category, s);
            }
        }
    }
}

#[test]
fn test_convert_to_self() {
    for hi in 0..=0xFFFFF {
        for lo in 0..=0xF {
            let bits = (hi << (128 - 20)) | lo;
            let value = swfp::F128::from_bits(bits);
            for round in ALL_ROUND_MODES {
                let (new_value, status) = swfp::F128::convert_from_ex(value, round);
                let new_bits = new_value.to_bits();
                let expected_status;
                let expected_bits;
                if value.is_nan() && bits & (1 << 111) == 0 {
                    expected_status = FpStatus::Invalid;
                    expected_bits = bits | (1 << 111);
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
    test_from_str_specials::<swfp::F128>();

    let exact = [
        (
            "5.42101086242752217003726400434970855712890625e-20",
            swfp::F128::from_bits(0x3FBF0000000000000000000000000000),
        ),
        (
            "3.40282366920938463463374607431768211456e38",
            swfp::F128::from_bits(0x407F0000000000000000000000000000),
        ),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F128::from_int(n));
    }

    for e in [-16494, -16383, -16382, 16382, 16383] {
        let s = format!("{:.12000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F128::from_int(1).scalbn(e));
    }

    check_from_str_round(
        "0.1",
        swfp::F128::from_bits(0x3FFB9999999999999999999999999999),
        swfp::F128::from_bits(0x3FFB999999999999999999999999999A),
        Loss::HalfUp,
    );
    check_from_str_round(
        "0.7",
        swfp::F128::from_bits(0x3FFE6666666666666666666666666666),
        swfp::F128::from_bits(0x3FFE6666666666666666666666666667),
        Loss::HalfDown,
    );
    check_from_str_round(
        "9870910079278514663430221538513944.5",
        swfp::F128::from_bits(0x406FE6AC66A5D55D6B8C57203797D418),
        swfp::F128::from_bits(0x406FE6AC66A5D55D6B8C57203797D419),
        Loss::HalfEven,
    );
    check_from_str_round(
        "9870910079278514663430221538513945.5",
        swfp::F128::from_bits(0x406FE6AC66A5D55D6B8C57203797D419),
        swfp::F128::from_bits(0x406FE6AC66A5D55D6B8C57203797D41A),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "-0.1",
        swfp::F128::from_bits(0xBFFB9999999999999999999999999999),
        swfp::F128::from_bits(0xBFFB999999999999999999999999999A),
        Loss::HalfUp,
    );
    check_from_str_round(
        "-0.7",
        swfp::F128::from_bits(0xBFFE6666666666666666666666666666),
        swfp::F128::from_bits(0xBFFE6666666666666666666666666667),
        Loss::HalfDown,
    );
    check_from_str_round(
        "-9870910079278514663430221538513944.5",
        swfp::F128::from_bits(0xC06FE6AC66A5D55D6B8C57203797D418),
        swfp::F128::from_bits(0xC06FE6AC66A5D55D6B8C57203797D419),
        Loss::HalfEven,
    );
    check_from_str_round(
        "-9870910079278514663430221538513945.5",
        swfp::F128::from_bits(0xC06FE6AC66A5D55D6B8C57203797D419),
        swfp::F128::from_bits(0xC06FE6AC66A5D55D6B8C57203797D41A),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
        swfp::F128::from_bits(0x3FFD5555555555555555555555555555),
        swfp::F128::from_bits(0x3FFD5555555555555555555555555556),
        Loss::HalfDown,
    );
    check_from_str_round(
        "1.50000000000000000000000000000000000000000000000000000000000000000001",
        swfp::F128::from_bits(0x3FFF8000000000000000000000000000),
        swfp::F128::from_bits(0x3FFF8000000000000000000000000001),
        Loss::HalfDown,
    );
    check_from_str_round(
        &format!("1.50{}01", "0".repeat(100_000)),
        swfp::F128::from_bits(0x3FFF8000000000000000000000000000),
        swfp::F128::from_bits(0x3FFF8000000000000000000000000001),
        Loss::HalfDown,
    );
}

#[test]
fn test_fmt_display() {
    let v = swfp::F128::from_int(1);
    assert_eq!(format!("{v}"), "1");

    let v = swfp::F128::from_bits(0x3FFF0000000000000000000000000001);
    assert_eq!(format!("{v}"), "1.0000000000000000000000000000000002");

    let v = swfp::F128::from_bits(1); // min subnormal
    assert_eq!(format!("{v}"), format!("0.{}6", "0".repeat(4965)));

    let v = swfp::F128::from_bits(0x00010000000000000000000000000000); // MIN_POSITIVE
    assert_eq!(
        format!("{v}"),
        format!("0.{}33621031431120935062626778173217526", "0".repeat(4931))
    );

    let v = swfp::F128::from_bits(0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF); // MAX
    assert_eq!(
        format!("{v}"),
        format!("1189731495357231765085759326628007{}", "0".repeat(4899))
    );
}

#[test]
fn test_fmt_exp() {
    let v = swfp::F128::from_bits(1); // min subnormal
    assert_eq!(format!("{v:e}"), "6e-4966");

    let v = swfp::F128::from_bits(2);
    assert_eq!(format!("{v:e}"), "1e-4965");

    let v = swfp::F128::from_bits(3);
    assert_eq!(format!("{v:e}"), "2e-4965");
    assert_eq!(format!("{v:.0e}"), "2e-4965");

    let v = swfp::F128::from_bits(0x00010000000000000000000000000000); // MIN_POSITIVE
    assert_eq!(
        format!("{v:e}"),
        "3.3621031431120935062626778173217526e-4932"
    );
    assert_eq!(format!("{v:.10e}"), "3.3621031431e-4932");

    let v = swfp::F128::from_bits(0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF); // MAX
    assert_eq!(format!("{v:e}"), "1.189731495357231765085759326628007e4932");
    assert_eq!(format!("{v:.10e}"), "1.1897314954e4932");

    let v = swfp::F128::from_bits(0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFE);
    assert_eq!(
        format!("{v:e}"),
        "1.1897314953572317650857593266280069e4932"
    );
    assert_eq!(format!("{v:.10e}"), "1.1897314954e4932");
}
