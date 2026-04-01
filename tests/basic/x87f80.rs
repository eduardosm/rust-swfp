use core::num::FpCategory;

use swfp::Float as _;

use crate::{
    Loss, check_category, check_compare, check_from_str_exact, check_from_str_round, mk_x87f80,
    test_from_str_specials,
};

#[test]
fn from_to_bits_lossless() {
    // Iterating through all combinations of the top 20 bits ensures coverage of all
    // possible exponents, sign, and integer bit combinations, along with three bits
    // of the mantissa. This includes special values (NaN, infinity) as well as
    // invalid bit patterns (pseudo-NaN, pseudo-infinity, unnormal).
    for hi in 0u128..(1 << 20) {
        for lo in 0..=127 {
            let bits = (hi << (80 - 20)) | lo;
            let new_bits = swfp::X87F80::from_bits(bits).to_bits();
            if new_bits != bits {
                panic!("0x{new_bits:04X} != 0x{bits:04X}");
            }
        }
    }
}

#[test]
fn test_consts() {
    super::check_consts::<swfp::X87F80>(true, false);
}

#[test]
fn test_categories() {
    check_category(mk_x87f80(false, -16383, false, 0), FpCategory::Zero, false);
    check_category(mk_x87f80(true, -16383, false, 0), FpCategory::Zero, true);

    for hi in 0..=255 {
        for lo in 0..=255 {
            let mant = (hi << (63 - 8)) | lo;
            let category = if mant == 0 {
                FpCategory::Zero
            } else {
                FpCategory::Subnormal
            };
            check_category(mk_x87f80(false, -16383, false, mant), category, false);
            check_category(mk_x87f80(true, -16383, false, mant), category, true);

            // "Pseudo Subnormal"
            check_category(
                mk_x87f80(false, -16383, true, mant),
                FpCategory::Subnormal,
                false,
            );
            check_category(
                mk_x87f80(true, -16383, true, mant),
                FpCategory::Subnormal,
                true,
            );
        }
    }

    for e in -16382..=16383 {
        for hi in 0..=15 {
            for lo in 0..=15 {
                let mant = (hi << (63 - 4)) | lo;
                check_category(mk_x87f80(false, e, true, mant), FpCategory::Normal, false);
                check_category(mk_x87f80(true, e, true, mant), FpCategory::Normal, true);

                // "Unnormal"
                check_category(mk_x87f80(false, e, false, mant), FpCategory::Nan, false);
                check_category(mk_x87f80(true, e, false, mant), FpCategory::Nan, true);
            }
        }
    }

    for hi in 0..=255 {
        for lo in 0..=255 {
            let mant = (hi << (63 - 8)) | lo;
            let category = if mant == 0 {
                FpCategory::Infinite
            } else {
                FpCategory::Nan
            };
            check_category(mk_x87f80(false, 16384, true, mant), category, false);
            check_category(mk_x87f80(true, 16384, true, mant), category, true);

            // "Pseudo-infinity" / "Pseudo-Nan"
            check_category(mk_x87f80(false, 16384, false, mant), FpCategory::Nan, false);
            check_category(mk_x87f80(true, 16384, false, mant), FpCategory::Nan, true);
        }
    }
}

#[test]
fn test_compare() {
    // "Pseudo Subnormal" values should compare equal to their normal counterparts.
    for hi in 0..=255 {
        for lo in 0..=255 {
            let mant = (hi << (63 - 8)) | lo;
            let a = mk_x87f80(false, -16383, true, mant);
            let b = mk_x87f80(false, -16382, true, mant);
            check_compare(a, b, Some(std::cmp::Ordering::Equal));
        }
    }
}

#[test]
fn test_from_str() {
    test_from_str_specials::<swfp::X87F80>();

    let exact = [
        (
            "5.42101086242752217003726400434970855712890625e-20",
            swfp::X87F80::from_bits(0x3FBF8000000000000000),
        ),
        (
            "3.40282366920938463463374607431768211456e38",
            swfp::X87F80::from_bits(0x407F8000000000000000),
        ),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::X87F80::from_int(n));
    }

    for e in [-16445, -16383, -16382, 16382, 16383] {
        let s = format!("{:.12000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::X87F80::from_int(1).scalbn(e));
    }

    check_from_str_round(
        "0.1",
        swfp::X87F80::from_bits(0x3FFBCCCCCCCCCCCCCCCC),
        swfp::X87F80::from_bits(0x3FFBCCCCCCCCCCCCCCCD),
        Loss::HalfUp,
    );
    check_from_str_round(
        "0.7",
        swfp::X87F80::from_bits(0x3FFEB333333333333333),
        swfp::X87F80::from_bits(0x3FFEB333333333333334),
        Loss::HalfDown,
    );
    check_from_str_round(
        "10734942146176220434.5",
        swfp::X87F80::from_bits(0x403E94FA2D017B33FD12),
        swfp::X87F80::from_bits(0x403E94FA2D017B33FD13),
        Loss::HalfEven,
    );
    check_from_str_round(
        "10734942146176220433.5",
        swfp::X87F80::from_bits(0x403E94FA2D017B33FD11),
        swfp::X87F80::from_bits(0x403E94FA2D017B33FD12),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "-0.1",
        swfp::X87F80::from_bits(0xBFFBCCCCCCCCCCCCCCCC),
        swfp::X87F80::from_bits(0xBFFBCCCCCCCCCCCCCCCD),
        Loss::HalfUp,
    );
    check_from_str_round(
        "-0.7",
        swfp::X87F80::from_bits(0xBFFEB333333333333333),
        swfp::X87F80::from_bits(0xBFFEB333333333333334),
        Loss::HalfDown,
    );
    check_from_str_round(
        "-10734942146176220434.5",
        swfp::X87F80::from_bits(0xC03E94FA2D017B33FD12),
        swfp::X87F80::from_bits(0xC03E94FA2D017B33FD13),
        Loss::HalfEven,
    );
    check_from_str_round(
        "-10734942146176220433.5",
        swfp::X87F80::from_bits(0xC03E94FA2D017B33FD11),
        swfp::X87F80::from_bits(0xC03E94FA2D017B33FD12),
        Loss::HalfOdd,
    );

    check_from_str_round(
        "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
        swfp::X87F80::from_bits(0x3FFDAAAAAAAAAAAAAAAA),
        swfp::X87F80::from_bits(0x3FFDAAAAAAAAAAAAAAAB),
        Loss::HalfUp,
    );
    check_from_str_round(
        "1.50000000000000000000000000000000000000000000000000000000000000000001",
        swfp::X87F80::from_bits(0x3FFFC000000000000000),
        swfp::X87F80::from_bits(0x3FFFC000000000000001),
        Loss::HalfDown,
    );
    check_from_str_round(
        &format!("1.50{}01", "0".repeat(100_000)),
        swfp::X87F80::from_bits(0x3FFFC000000000000000),
        swfp::X87F80::from_bits(0x3FFFC000000000000001),
        Loss::HalfDown,
    );
}

#[test]
fn test_fmt_display() {
    let v = swfp::X87F80::from_int(1);
    assert_eq!(format!("{v}"), "1");

    let v = swfp::X87F80::from_bits(0x3FFF8000000000000001);
    assert_eq!(format!("{v}"), "1.0000000000000000001");

    let v = swfp::X87F80::from_bits(1); // min subnormal
    assert_eq!(format!("{v}"), format!("0.{}4", "0".repeat(4950)));

    let v = swfp::X87F80::from_bits(0x00018000000000000000); // MIN_POSITIVE
    assert_eq!(
        format!("{v}"),
        format!("0.{}33621031431120935063", "0".repeat(4931))
    );

    let v = swfp::X87F80::from_bits(0x7FFE8FFFFFFFFFFFFFFF); // MAX
    assert_eq!(
        format!("{v}"),
        format!("6692239661384428678{}", "0".repeat(4913))
    );
}

#[test]
fn test_fmt_exp() {
    let v = swfp::X87F80::from_bits(1); // min subnormal
    assert_eq!(format!("{v:e}"), "4e-4951");

    let v = swfp::X87F80::from_bits(2);
    assert_eq!(format!("{v:e}"), "7e-4951");

    let v = swfp::X87F80::from_bits(3);
    assert_eq!(format!("{v:e}"), "1e-4950");
    assert_eq!(format!("{v:.0e}"), "1e-4950");

    let v = swfp::X87F80::from_bits(0x00018000000000000000); // MIN_POSITIVE
    assert_eq!(format!("{v:e}"), "3.3621031431120935063e-4932");
    assert_eq!(format!("{v:.10e}"), "3.3621031431e-4932");

    let v = swfp::X87F80::from_bits(0x7FFE8FFFFFFFFFFFFFFF); // MAX
    assert_eq!(format!("{v:e}"), "6.692239661384428678e4931");
    assert_eq!(format!("{v:.10e}"), "6.6922396614e4931");

    let v = swfp::X87F80::from_bits(0x7FFE8FFFFFFFFFFFFFFE);
    assert_eq!(format!("{v:e}"), "6.692239661384428677e4931");
    assert_eq!(format!("{v:.10e}"), "6.6922396614e4931");
}
