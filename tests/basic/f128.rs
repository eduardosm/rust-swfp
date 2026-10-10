use std::num::FpCategory;

use swfp::{Float as _, FloatConvertFrom, FpStatus};

use crate::{
    ALL_ROUND_MODES, Loss, check_category, check_from_str_exact, check_from_str_round,
    check_round_int_exact, check_round_int_round, mk_f128, test_from_str_specials,
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

                let (expected_status, expected_bits) = if value.is_nan() && bits & (1 << 111) == 0 {
                    (FpStatus::INVALID, bits | (1 << 111))
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
        (
            "10384593717069655257060992658440191",
            swfp::F128::from_bits(0x406FFFFFFFFFFFFFFFFFFFFFFFFFFFFF),
        ),
        (
            "10384593717069655257060992658440192",
            swfp::F128::from_bits(0x40700000000000000000000000000000),
        ),
        (
            "5192296858534827628530496329220095.5",
            swfp::F128::from_bits(0x406EFFFFFFFFFFFFFFFFFFFFFFFFFFFF),
        ),
        (
            "1.00000000000000000000000000000000019259299443872358530559779425849273185381\
             01648215388195239938795566558837890625",
            swfp::F128::from_bits(0x3FFF0000000000000000000000000001),
        ),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..=1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::F128::from_int(n));
    }

    for e in (-16494..=-16380).chain(16380..=16383) {
        let s = format!("{:.12000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::F128::from_int(1).scalbn(e));
    }

    // (input, rounded toward zero, rounded away from zero, loss)
    let inexact = [
        // Around the overflow threshold
        (
            "1.189731495357231765085759326628007073479e4932",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::HalfDown,
        ),
        (
            "1.189731495357231765085759326628007073480e4932",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.189731495357231765085759326628007130763e4932",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.189731495357231765085759326628007130764e4932",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::Overflow,
        ),
        (
            "1.2e4932",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::Overflow,
        ),
        (
            "1e4933",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::Overflow,
        ),
        (
            "1e99999999999999999999",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::Overflow,
        ),
        (
            "1e9223372036854775807",
            0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x7FFF0000000000000000000000000000,
            Loss::Overflow,
        ),
        // Around the underflow threshold
        (
            "3.237587559719012555462219479113823276249e-4966",
            0x00000000000000000000000000000000,
            0x00000000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "3.237587559719012555462219479113823276250e-4966",
            0x00000000000000000000000000000000,
            0x00000000000000000000000000000001,
            Loss::HalfUp,
        ),
        (
            "9.712762679157037666386658437341469828749e-4966",
            0x00000000000000000000000000000001,
            0x00000000000000000000000000000002,
            Loss::HalfDown,
        ),
        (
            "9.712762679157037666386658437341469828750e-4966",
            0x00000000000000000000000000000001,
            0x00000000000000000000000000000002,
            Loss::HalfUp,
        ),
        (
            "3.362103143112093506262677817321752278839e-4932",
            0x0000FFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x00010000000000000000000000000000,
            Loss::HalfDown,
        ),
        (
            "3.362103143112093506262677817321752278840e-4932",
            0x0000FFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x00010000000000000000000000000000,
            Loss::HalfUp,
        ),
        (
            "1e-4966",
            0x00000000000000000000000000000000,
            0x00000000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "6.5e-4966",
            0x00000000000000000000000000000001,
            0x00000000000000000000000000000002,
            Loss::HalfDown,
        ),
        (
            "1e-99999999999999999999",
            0x00000000000000000000000000000000,
            0x00000000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "1e-9223372036854775808",
            0x00000000000000000000000000000000,
            0x00000000000000000000000000000001,
            Loss::HalfDown,
        ),
        // Ties in the normal range
        (
            "9870910079278514663430221538513944.5",
            0x406FE6AC66A5D55D6B8C57203797D418,
            0x406FE6AC66A5D55D6B8C57203797D419,
            Loss::HalfEven,
        ),
        (
            "9870910079278514663430221538513945.5",
            0x406FE6AC66A5D55D6B8C57203797D419,
            0x406FE6AC66A5D55D6B8C57203797D41A,
            Loss::HalfOdd,
        ),
        (
            "10384593717069655257060992658440192.999999999999999999999999999999999999999999999",
            0x40700000000000000000000000000000,
            0x40700000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "10384593717069655257060992658440193",
            0x40700000000000000000000000000000,
            0x40700000000000000000000000000001,
            Loss::HalfEven,
        ),
        (
            "10384593717069655257060992658440193.0000000000000000000000000000000000000000000001",
            0x40700000000000000000000000000000,
            0x40700000000000000000000000000001,
            Loss::HalfUp,
        ),
        (
            "10384593717069655257060992658440195",
            0x40700000000000000000000000000001,
            0x40700000000000000000000000000002,
            Loss::HalfOdd,
        ),
        (
            "1.00000000000000000000000000000000009629649721936179265279889712924636592690\
             50824107694097619969397783279418945312499999999999999999999",
            0x3FFF0000000000000000000000000000,
            0x3FFF0000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "1.00000000000000000000000000000000009629649721936179265279889712924636592690\
             508241076940976199693977832794189453125",
            0x3FFF0000000000000000000000000000,
            0x3FFF0000000000000000000000000001,
            Loss::HalfEven,
        ),
        (
            "1.00000000000000000000000000000000009629649721936179265279889712924636592690\
             508241076940976199693977832794189453125000000000000000000001",
            0x3FFF0000000000000000000000000000,
            0x3FFF0000000000000000000000000001,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000028888949165808537795839669138773909778071\
             524723230822928599081933498382568359375",
            0x3FFF0000000000000000000000000001,
            0x3FFF0000000000000000000000000002,
            Loss::HalfOdd,
        ),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            0x3FFF0000000000000000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x3FFF0000000000000000000000000000,
            0x3FFF0000000000000000000000000001,
            Loss::HalfDown,
        ),
        // Other values
        (
            "0.1",
            0x3FFB9999999999999999999999999999,
            0x3FFB999999999999999999999999999A,
            Loss::HalfUp,
        ),
        (
            "0.7",
            0x3FFE6666666666666666666666666666,
            0x3FFE6666666666666666666666666667,
            Loss::HalfDown,
        ),
        (
            "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
            0x3FFD5555555555555555555555555555,
            0x3FFD5555555555555555555555555556,
            Loss::HalfDown,
        ),
        (
            "1.50000000000000000000000000000000000000000000000000000000000000000001",
            0x3FFF8000000000000000000000000000,
            0x3FFF8000000000000000000000000001,
            Loss::HalfDown,
        ),
        (
            "3.14159265358979323846264338327950288419716939937510",
            0x4000921FB54442D18469898CC51701B8,
            0x4000921FB54442D18469898CC51701B9,
            Loss::HalfDown,
        ),
        (
            "2.71828182845904523536028747135266249775724709369995",
            0x40005BF0A8B1457695355FB8AC404E7A,
            0x40005BF0A8B1457695355FB8AC404E7B,
            Loss::HalfDown,
        ),
        (
            "1e4000",
            0x73E6A3750647FCAB18C21AB905450CC2,
            0x73E6A3750647FCAB18C21AB905450CC3,
            Loss::HalfUp,
        ),
        (
            "1e-4000",
            0x0C17387AE70C9E700B8049732D11A23C,
            0x0C17387AE70C9E700B8049732D11A23D,
            Loss::HalfUp,
        ),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::F128::from_bits(tz);
        let az = swfp::F128::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }

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
    assert_eq!(format!("{v:.10}"), "0.0000000000");

    let v = swfp::F128::from_bits(0x7FFEFFFFFFFFFFFFFFFFFFFFFFFFFFFF); // MAX
    assert_eq!(
        format!("{v}"),
        format!("1189731495357231765085759326628007{}", "0".repeat(4899))
    );

    let v = swfp::F128::from_bits(0xC17387AE70C9E700B8049732D11A23D); // 1e-4000
    assert_eq!(format!("{v}"), format!("0.{}1", "0".repeat(3999)));
    assert_eq!(format!("{v:.10}"), "0.0000000000");

    let v = swfp::F128::from_bits(0x40640000000000000000000000000800); // 2^101 + 1
    assert_eq!(format!("{v:.0}"), "2535301200456458802993406410753");
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

    let v = swfp::F128::from_bits(0xC17387AE70C9E700B8049732D11A23D); // 1e-4000
    assert_eq!(format!("{v:e}"), "1e-4000");
    assert_eq!(format!("{v:.10e}"), "1.0000000000e-4000");
    let s = format!("{v:.10000e}");
    assert!(s.starts_with(
        "1.0000000000000000000000000000000000767485085088860217165223132611805143194",
    ));
    assert!(s.ends_with("e-4000"));
}

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::F128::NAN);
    check_round_int_exact(swfp::F128::INFINITY);
    check_round_int_exact(-swfp::F128::INFINITY);
    check_round_int_exact(swfp::F128::ZERO);
    check_round_int_exact(-swfp::F128::ZERO);

    for s in [false, true] {
        check_round_int_exact(mk_f128(s, 0, 0));
        check_round_int_exact(mk_f128(s, 2, 1 << 111));
        check_round_int_exact(mk_f128(s, 112, (1 << 112) - 1));

        let zero = mk_f128(s, -16383, 0);
        let one = mk_f128(s, 0, 0);
        check_round_int_round(mk_f128(s, -16383, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f128(s, -2, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_f128(s, -1, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_f128(s, -1, 1 << 111), zero, one, Loss::HalfUp);
        check_round_int_round(mk_f128(s, -1, (1 << 112) - 1), zero, one, Loss::HalfUp);
    }
}
