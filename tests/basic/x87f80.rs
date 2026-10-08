use core::num::FpCategory;

use swfp::Float as _;

use crate::{
    Loss, check_category, check_compare, check_from_str_exact, check_from_str_round,
    check_round_int_exact, check_round_int_round, mk_x87f80, test_from_str_specials,
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
        (
            "18446744073709551615",
            swfp::X87F80::from_bits(0x403EFFFFFFFFFFFFFFFF),
        ),
        (
            "18446744073709551616",
            swfp::X87F80::from_bits(0x403F8000000000000000),
        ),
        (
            "9223372036854775807.5",
            swfp::X87F80::from_bits(0x403DFFFFFFFFFFFFFFFF),
        ),
        (
            "1.000000000000000000108420217248550443400745280086994171142578125",
            swfp::X87F80::from_bits(0x3FFF8000000000000001),
        ),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }

    for n in -1_000_000..=1_000_000 {
        check_from_str_exact(&n.to_string(), swfp::X87F80::from_int(n));
    }

    for e in (-16445..=-16380).chain(16380..=16383) {
        let s = format!("{:.12000e}", rug::Float::with_val(1, 1) << e);
        check_from_str_exact(&s, swfp::X87F80::from_int(1).scalbn(e));
    }

    // (input, rounded toward zero, rounded away from zero, loss)
    let inexact = [
        // Around the overflow threshold
        (
            "1.189731495357231765053511589829488667966e4932",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::HalfDown,
        ),
        (
            "1.189731495357231765053511589829488667967e4932",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.189731495357231765085759326628007130763e4932",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.189731495357231765085759326628007130764e4932",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::Overflow,
        ),
        (
            "1.2e4932",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::Overflow,
        ),
        (
            "1e4933",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::Overflow,
        ),
        (
            "1e99999999999999999999",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::Overflow,
        ),
        (
            "1e9223372036854775807",
            0x7FFEFFFFFFFFFFFFFFFF,
            0x7FFF8000000000000000,
            Loss::Overflow,
        ),
        // Around the underflow threshold
        (
            "1.822599765941237301264202966809709908199e-4951",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfDown,
        ),
        (
            "1.822599765941237301264202966809709908200e-4951",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfUp,
        ),
        (
            "5.467799297823711903792608900429129724598e-4951",
            0x00000000000000000001,
            0x00000000000000000002,
            Loss::HalfDown,
        ),
        (
            "5.467799297823711903792608900429129724599e-4951",
            0x00000000000000000001,
            0x00000000000000000002,
            Loss::HalfUp,
        ),
        (
            "3.362103143112093506080417840727628872471e-4932",
            0x00007FFFFFFFFFFFFFFF,
            0x00018000000000000000,
            Loss::HalfDown,
        ),
        (
            "3.362103143112093506080417840727628872472e-4932",
            0x00007FFFFFFFFFFFFFFF,
            0x00018000000000000000,
            Loss::HalfUp,
        ),
        (
            "1e-4951",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfDown,
        ),
        (
            "3e-4951",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfUp,
        ),
        (
            "1e-99999999999999999999",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfDown,
        ),
        (
            "1e-9223372036854775808",
            0x00000000000000000000,
            0x00000000000000000001,
            Loss::HalfDown,
        ),
        // Ties in the normal range
        (
            "10734942146176220434.5",
            0x403E94FA2D017B33FD12,
            0x403E94FA2D017B33FD13,
            Loss::HalfEven,
        ),
        (
            "10734942146176220433.5",
            0x403E94FA2D017B33FD11,
            0x403E94FA2D017B33FD12,
            Loss::HalfOdd,
        ),
        (
            "18446744073709551616.999999999999999999999999999999999999999999999",
            0x403F8000000000000000,
            0x403F8000000000000001,
            Loss::HalfDown,
        ),
        (
            "18446744073709551617",
            0x403F8000000000000000,
            0x403F8000000000000001,
            Loss::HalfEven,
        ),
        (
            "18446744073709551617.0000000000000000000000000000000000000000000001",
            0x403F8000000000000000,
            0x403F8000000000000001,
            Loss::HalfUp,
        ),
        (
            "18446744073709551619",
            0x403F8000000000000001,
            0x403F8000000000000002,
            Loss::HalfOdd,
        ),
        (
            "1.00000000000000000005421010862427522170037264004349708557128906249999999999\
             99999999999999999999999999999999999",
            0x3FFF8000000000000000,
            0x3FFF8000000000000001,
            Loss::HalfDown,
        ),
        (
            "1.0000000000000000000542101086242752217003726400434970855712890625",
            0x3FFF8000000000000000,
            0x3FFF8000000000000001,
            Loss::HalfEven,
        ),
        (
            "1.00000000000000000005421010862427522170037264004349708557128906250000000000\
             000000000000000000000000000000000001",
            0x3FFF8000000000000000,
            0x3FFF8000000000000001,
            Loss::HalfUp,
        ),
        (
            "1.0000000000000000001626303258728256651011179201304912567138671875",
            0x3FFF8000000000000001,
            0x3FFF8000000000000002,
            Loss::HalfOdd,
        ),
        // Very close to representable values
        (
            "0.99999999999999999999999999999999999999999999999999",
            0x3FFEFFFFFFFFFFFFFFFF,
            0x3FFF8000000000000000,
            Loss::HalfUp,
        ),
        (
            "1.00000000000000000000000000000000000000000000000001",
            0x3FFF8000000000000000,
            0x3FFF8000000000000001,
            Loss::HalfDown,
        ),
        // Other values
        (
            "0.1",
            0x3FFBCCCCCCCCCCCCCCCC,
            0x3FFBCCCCCCCCCCCCCCCD,
            Loss::HalfUp,
        ),
        (
            "0.7",
            0x3FFEB333333333333333,
            0x3FFEB333333333333334,
            Loss::HalfDown,
        ),
        (
            "0.33333333333333333333333333333333333333333333333333333333333333333333333333",
            0x3FFDAAAAAAAAAAAAAAAA,
            0x3FFDAAAAAAAAAAAAAAAB,
            Loss::HalfUp,
        ),
        (
            "1.50000000000000000000000000000000000000000000000000000000000000000001",
            0x3FFFC000000000000000,
            0x3FFFC000000000000001,
            Loss::HalfDown,
        ),
        (
            "3.14159265358979323846264338327950288419716939937510",
            0x4000C90FDAA22168C234,
            0x4000C90FDAA22168C235,
            Loss::HalfUp,
        ),
        (
            "2.71828182845904523536028747135266249775724709369995",
            0x4000ADF85458A2BB4A9A,
            0x4000ADF85458A2BB4A9B,
            Loss::HalfUp,
        ),
        (
            "1e4000",
            0x73E6D1BA8323FE558C61,
            0x73E6D1BA8323FE558C62,
            Loss::HalfDown,
        ),
        (
            "1e-4000",
            0x0C179C3D73864F3805C0,
            0x0C179C3D73864F3805C1,
            Loss::HalfDown,
        ),
    ];

    for &(s, tz, az, loss) in inexact.iter() {
        let tz = swfp::X87F80::from_bits(tz);
        let az = swfp::X87F80::from_bits(az);
        check_from_str_round(s, tz, az, loss);
        check_from_str_round(&format!("+{s}"), tz, az, loss);
        check_from_str_round(&format!("-{s}"), -tz, -az, loss);
    }

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
    assert_eq!(format!("{v:.10}"), "0.0000000000");

    let v = swfp::X87F80::from_bits(0x7FFE8FFFFFFFFFFFFFFF); // MAX
    assert_eq!(
        format!("{v}"),
        format!("6692239661384428678{}", "0".repeat(4913))
    );

    let v = swfp::X87F80::from_bits(0xC179C3D73864F3805C0); // 1e-4000
    assert_eq!(format!("{v}"), format!("0.{}1", "0".repeat(3999)));
    assert_eq!(format!("{v:.10}"), "0.0000000000");
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

    let v = swfp::X87F80::from_bits(0xC179C3D73864F3805C0); // 1e-4000
    assert_eq!(format!("{v:e}"), "1e-4000");
    assert_eq!(format!("{v:.10e}"), "1.0000000000e-4000");
    let s = format!("{v:.10000e}");
    assert!(
        s.starts_with(
            "9.9999999999999999998725766037771403366859164981113486605022635033705520196",
        ),
        "{s}",
    );
    assert!(s.ends_with("e-4001"));
}

#[test]
fn test_round_int() {
    check_round_int_exact(swfp::X87F80::NAN);
    check_round_int_exact(swfp::X87F80::INFINITY);
    check_round_int_exact(-swfp::X87F80::INFINITY);
    check_round_int_exact(swfp::X87F80::ZERO);
    check_round_int_exact(-swfp::X87F80::ZERO);

    for s in [false, true] {
        check_round_int_exact(mk_x87f80(s, 0, true, 0));
        check_round_int_exact(mk_x87f80(s, 2, true, 1 << 62));
        check_round_int_exact(mk_x87f80(s, 63, true, (1 << 63) - 1));

        let zero = mk_x87f80(s, -16383, false, 0);
        let one = mk_x87f80(s, 0, true, 0);
        check_round_int_round(mk_x87f80(s, -16383, false, 1), zero, one, Loss::HalfDown);
        check_round_int_round(mk_x87f80(s, -2, true, 0), zero, one, Loss::HalfDown);
        check_round_int_round(mk_x87f80(s, -1, true, 0), zero, one, Loss::HalfEven);
        check_round_int_round(mk_x87f80(s, -1, true, 1 << 62), zero, one, Loss::HalfUp);
        check_round_int_round(
            mk_x87f80(s, -1, true, (1 << 63) - 1),
            zero,
            one,
            Loss::HalfUp,
        );
    }
}
