#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]

use core::num::FpCategory;

use swfp::{Float, FpStatus, Round};

mod convert;
mod f128;
mod f16;
mod f32;
mod f64;
mod f8e4m3b8nnz;
mod f8e4m3nao;
mod f8e5m2;
mod x87f80;

const ALL_ROUND_MODES: [Round; 5] = [
    Round::NearestTiesToEven,
    Round::NearestTiesToAway,
    Round::TowardPositive,
    Round::TowardNegative,
    Round::TowardZero,
];

fn mk_f16(sign: bool, exp: i8, mant: u16) -> swfp::F16 {
    assert!(matches!(exp, -15..=16));
    assert!(mant <= 0x3FF);
    let bits = (u16::from(sign) << 15) | (((exp + 15) as u16) << 10) | mant;
    swfp::F16::from_bits(bits)
}

fn mk_f32(sign: bool, exp: i16, mant: u32) -> swfp::F32 {
    assert!(matches!(exp, -127..=128));
    assert!(mant < 1 << 23);
    let bits = (u32::from(sign) << 31) | (((exp + 127) as u32) << 23) | mant;
    swfp::F32::from_bits(bits)
}

fn mk_f64(sign: bool, exp: i16, mant: u64) -> swfp::F64 {
    assert!(matches!(exp, -1023..=1024));
    assert!(mant < 1 << 52);
    let bits = (u64::from(sign) << 63) | (((exp + 1023) as u64) << 52) | mant;
    swfp::F64::from_bits(bits)
}

fn mk_f128(sign: bool, exp: i16, mant: u128) -> swfp::F128 {
    assert!(matches!(exp, -16383..=16384));
    assert!(mant < 1 << 112);
    let bits = (u128::from(sign) << 127) | (((exp + 16383) as u128) << 112) | mant;
    swfp::F128::from_bits(bits)
}

fn mk_x87f80(sign: bool, exp: i16, int: bool, mant: u128) -> swfp::X87F80 {
    assert!(matches!(exp, -16383..=16384));
    assert!(mant < (1 << 63));
    let bits =
        (u128::from(sign) << 79) | (((exp + 16383) as u128) << 64) | (u128::from(int) << 63) | mant;
    swfp::X87F80::from_bits(bits)
}

fn mk_f8e5m2(sign: bool, exp: i8, mant: u8) -> swfp::F8E5M2 {
    assert!(matches!(exp, -15..=16));
    assert!(mant <= 0b11);
    let bits = (u8::from(sign) << 7) | (((exp + 15) as u8) << 2) | mant;
    swfp::F8E5M2::from_bits(bits)
}

fn mk_f8e4m3b8nnz(sign: bool, exp: i8, mant: u8) -> swfp::F8E4M3B8Nnz {
    assert!(matches!(exp, -8..=7));
    assert!(mant <= 0b111);
    let bits = (u8::from(sign) << 7) | (((exp + 8) as u8) << 3) | mant;
    swfp::F8E4M3B8Nnz::from_bits(bits)
}

fn mk_f8e4m3nao(sign: bool, exp: i8, mant: u8) -> swfp::F8E4M3Nao {
    assert!(matches!(exp, -7..=8));
    assert!(mant <= 0b111);
    let bits = (u8::from(sign) << 7) | (((exp + 7) as u8) << 3) | mant;
    swfp::F8E4M3Nao::from_bits(bits)
}

fn check_category<F: Float>(value: F, expected: FpCategory, sign: bool) {
    assert_eq!(value.classify(), expected);
    assert_eq!(value.is_sign_positive(), !sign);
    assert_eq!(value.is_sign_negative(), sign);

    match expected {
        FpCategory::Nan => {
            assert!(value.is_nan());
            assert!(!value.is_infinite());
            assert!(!value.is_subnormal());
            assert!(!value.is_normal());
            assert!(!value.is_finite());
        }
        FpCategory::Infinite => {
            assert!(!value.is_nan());
            assert!(value.is_infinite());
            assert!(!value.is_subnormal());
            assert!(!value.is_normal());
            assert!(!value.is_finite());
        }
        FpCategory::Zero => {
            assert!(!value.is_nan());
            assert!(!value.is_infinite());
            assert!(!value.is_subnormal());
            assert!(!value.is_normal());
            assert!(value.is_finite());
        }
        FpCategory::Subnormal => {
            assert!(!value.is_nan());
            assert!(!value.is_infinite());
            assert!(value.is_subnormal());
            assert!(!value.is_normal());
            assert!(value.is_finite());
        }
        FpCategory::Normal => {
            assert!(!value.is_nan());
            assert!(!value.is_infinite());
            assert!(!value.is_subnormal());
            assert!(value.is_normal());
            assert!(value.is_finite());
        }
    }
}

fn check_consts<F: Float>(has_inf: bool, nan_is_neg: bool) {
    let values = [
        (F::ZERO, FpCategory::Zero, false),
        (F::NAN, FpCategory::Nan, nan_is_neg),
        (
            F::INFINITY,
            if has_inf {
                FpCategory::Infinite
            } else {
                FpCategory::Nan
            },
            !has_inf && nan_is_neg,
        ),
    ];

    for (value, category, sign) in values {
        check_category(value, category, sign);

        let new_value = F::from_bits(value.to_bits());
        check_category(new_value, category, sign);
    }
}

fn check_compare<F: Float>(a: F, b: F, ord: Option<std::cmp::Ordering>) {
    assert_eq!(a.partial_cmp(&b), ord);

    assert_eq!(a == b, ord == Some(std::cmp::Ordering::Equal));
    assert_eq!(a != b, ord != Some(std::cmp::Ordering::Equal));
    assert_eq!(a < b, ord.is_some_and(|ord| ord.is_lt()));
    assert_eq!(a <= b, ord.is_some_and(|ord| ord.is_le()));
    assert_eq!(a > b, ord.is_some_and(|ord| ord.is_gt()));
    assert_eq!(a >= b, ord.is_some_and(|ord| ord.is_ge()));
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Loss {
    HalfUp,
    HalfDown,
    HalfEven,
    HalfOdd,
    Overflow,
}

fn check_round<T>(f: impl Fn(Round) -> T, expected_tz: T, expected_az: T, sign: bool, loss: Loss)
where
    T: PartialEq + core::fmt::Debug,
{
    let check_cases = |expected: T, round_modes: &[Round]| {
        for &round in round_modes {
            let actual = f(round);
            assert_eq!(actual, expected);
        }
    };

    match (loss, sign) {
        (Loss::HalfEven, false) => {
            check_cases(
                expected_tz,
                &[
                    Round::NearestTiesToEven,
                    Round::TowardNegative,
                    Round::TowardZero,
                ],
            );
            check_cases(
                expected_az,
                &[Round::NearestTiesToAway, Round::TowardPositive],
            );
        }
        (Loss::HalfEven, true) => {
            check_cases(
                expected_tz,
                &[
                    Round::NearestTiesToEven,
                    Round::TowardPositive,
                    Round::TowardZero,
                ],
            );
            check_cases(
                expected_az,
                &[Round::NearestTiesToAway, Round::TowardNegative],
            );
        }
        (Loss::HalfOdd, false) => {
            check_cases(expected_tz, &[Round::TowardNegative, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardPositive,
                ],
            );
        }
        (Loss::HalfOdd, true) => {
            check_cases(expected_tz, &[Round::TowardPositive, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardNegative,
                ],
            );
        }
        (Loss::HalfUp, false) => {
            check_cases(expected_tz, &[Round::TowardNegative, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardPositive,
                ],
            );
        }
        (Loss::HalfUp, true) => {
            check_cases(expected_tz, &[Round::TowardPositive, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardNegative,
                ],
            );
        }
        (Loss::HalfDown, false) => {
            check_cases(
                expected_tz,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardNegative,
                    Round::TowardZero,
                ],
            );
            check_cases(expected_az, &[Round::TowardPositive]);
        }
        (Loss::HalfDown, true) => {
            check_cases(
                expected_tz,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardPositive,
                    Round::TowardZero,
                ],
            );
            check_cases(expected_az, &[Round::TowardNegative]);
        }
        (Loss::Overflow, false) => {
            check_cases(expected_tz, &[Round::TowardNegative, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardPositive,
                ],
            );
        }
        (Loss::Overflow, true) => {
            check_cases(expected_tz, &[Round::TowardPositive, Round::TowardZero]);
            check_cases(
                expected_az,
                &[
                    Round::NearestTiesToEven,
                    Round::NearestTiesToAway,
                    Round::TowardNegative,
                ],
            );
        }
    }
}

fn check_float_round<F>(
    f: impl Fn(Round) -> (F, FpStatus),
    expected_tz: F,
    expected_az: F,
    sign: bool,
    loss: Loss,
) where
    F: Float<Bits: Eq + std::fmt::Debug>,
{
    let get_expected_status = |expected_value: F| {
        if loss == Loss::Overflow {
            FpStatus::Overflow
        } else {
            match expected_value.classify() {
                // Handle NaNs for formats without infinities.
                FpCategory::Nan | FpCategory::Infinite => FpStatus::Overflow,
                FpCategory::Zero | FpCategory::Subnormal => FpStatus::Underflow,
                FpCategory::Normal => FpStatus::Inexact,
            }
        }
    };

    check_round(
        |round| {
            let (value, status) = f(round);
            (value.to_bits(), status)
        },
        (expected_tz.to_bits(), get_expected_status(expected_tz)),
        (expected_az.to_bits(), get_expected_status(expected_az)),
        sign,
        loss,
    );
}

fn check_from_uint_exact<F>(value: u128, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = F::from_uint_ex(value, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!(F::from_uint(value).to_bits(), expected_res.to_bits());
}

fn check_from_uint_round<F>(value: u128, expected_res_tz: F, expected_res_az: F, loss: Loss)
where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| F::from_uint_ex(value, round),
        expected_res_tz,
        expected_res_az,
        false,
        loss,
    );

    assert_eq!(
        F::from_uint(value).to_bits(),
        F::from_uint_ex(value, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_from_int_exact<F>(value: i128, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = F::from_int_ex(value, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!(F::from_int(value).to_bits(), expected_res.to_bits());
}

fn check_from_int_round<F>(value: i128, expected_res_tz: F, expected_res_az: F, loss: Loss)
where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| F::from_int_ex(value, round),
        expected_res_tz,
        expected_res_az,
        false,
        loss,
    );

    assert_eq!(
        F::from_int(value).to_bits(),
        F::from_int_ex(value, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_to_uint_exact<F>(value: F, expected: (Option<u128>, FpStatus))
where
    F: Float,
{
    let bits = expected
        .0
        .map(|v| 128 - v.leading_zeros())
        .unwrap_or(1)
        .max(1);

    for n in bits..=128 {
        for round in ALL_ROUND_MODES {
            assert_eq!(value.to_uint_ex(n, round), expected);
            assert_eq!(value.round_int_ex(round).0.to_uint_ex(n, round), expected);
        }
        assert_eq!(value.to_uint(n), expected.0);
    }
    for n in 1..bits {
        for round in ALL_ROUND_MODES {
            assert_eq!(value.to_uint_ex(n, round), (None, FpStatus::Overflow));
            assert_eq!(
                value.round_int_ex(round).0.to_uint_ex(n, round),
                (None, FpStatus::Overflow),
            );
        }
        assert_eq!(value.to_uint(n), None);
    }
}

fn check_to_uint_round<F>(
    value: F,
    expected_tz: (Option<u128>, FpStatus),
    expected_az: (Option<u128>, FpStatus),
    loss: Loss,
) where
    F: Float,
{
    let bits_tz = expected_tz
        .0
        .map(|v| 128 - v.leading_zeros())
        .unwrap_or(1)
        .max(1);
    let bits_az = expected_az
        .0
        .map(|v| 128 - v.leading_zeros())
        .unwrap_or(1)
        .max(1);

    for n in 1..=128 {
        check_round(
            |round| value.to_uint_ex(n, round),
            if n >= bits_tz {
                expected_tz
            } else {
                (None, FpStatus::Overflow)
            },
            if n >= bits_az {
                expected_az
            } else {
                (None, FpStatus::Overflow)
            },
            value.is_sign_negative(),
            loss,
        );
        assert_eq!(value.to_uint(n), value.to_uint_ex(n, Round::TowardZero).0);
    }
}

fn check_to_int_exact<F>(value: F, expected: (Option<i128>, FpStatus))
where
    F: Float,
{
    let bits = expected
        .0
        .map(|v| {
            if v >= 0 {
                129 - v.leading_zeros()
            } else {
                let v = v.unsigned_abs();
                if v.is_power_of_two() {
                    128 - v.leading_zeros()
                } else {
                    129 - v.leading_zeros()
                }
            }
        })
        .unwrap_or(1)
        .max(1);

    for n in bits..=128 {
        for round in ALL_ROUND_MODES {
            assert_eq!(value.to_int_ex(n, round), expected);
            assert_eq!(value.round_int_ex(round).0.to_int_ex(n, round), expected);
        }
        assert_eq!(value.to_int(n), expected.0);
    }
    for n in 1..bits {
        for round in ALL_ROUND_MODES {
            assert_eq!(value.to_int_ex(n, round), (None, FpStatus::Overflow));
            assert_eq!(
                value.round_int_ex(round).0.to_int_ex(n, round),
                (None, FpStatus::Overflow),
            );
        }
        assert_eq!(value.to_int(n), None);
    }
}

fn check_to_int_round<F>(
    value: F,
    expected_tz: (Option<i128>, FpStatus),
    expected_az: (Option<i128>, FpStatus),
    loss: Loss,
) where
    F: Float,
{
    let bits_tz = expected_tz
        .0
        .map(|v| {
            if v >= 0 {
                129 - v.leading_zeros()
            } else {
                let v = v.unsigned_abs();
                if v.is_power_of_two() {
                    128 - v.leading_zeros()
                } else {
                    129 - v.leading_zeros()
                }
            }
        })
        .unwrap_or(1)
        .max(1);
    let bits_az = expected_az
        .0
        .map(|v| {
            if v >= 0 {
                129 - v.leading_zeros()
            } else {
                let v = v.unsigned_abs();
                if v.is_power_of_two() {
                    128 - v.leading_zeros()
                } else {
                    129 - v.leading_zeros()
                }
            }
        })
        .unwrap_or(1)
        .max(1);

    for n in 1..=128 {
        check_round(
            |round| value.to_int_ex(n, round),
            if n >= bits_tz {
                expected_tz
            } else {
                (None, FpStatus::Overflow)
            },
            if n >= bits_az {
                expected_az
            } else {
                (None, FpStatus::Overflow)
            },
            value.is_sign_negative(),
            loss,
        );
        assert_eq!(value.to_int(n), value.to_int_ex(n, Round::TowardZero).0);
    }
}

fn check_from_str_exact<F>(s: &str, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = F::from_str_ex(s, round).unwrap();
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!(
        s.parse::<F>().ok().unwrap().to_bits(),
        expected_res.to_bits(),
    );
}

fn test_from_str_specials<F>()
where
    F: Float<Bits: std::fmt::Debug>,
{
    let exact = [
        ("nan", F::NAN),
        ("NaN", F::NAN),
        ("NAN", F::NAN),
        ("Nan", F::NAN),
        ("nAn", F::NAN),
        ("inf", F::INFINITY),
        ("Inf", F::INFINITY),
        ("INF", F::INFINITY),
        ("InF", F::INFINITY),
        ("iNf", F::INFINITY),
        ("infinity", F::INFINITY),
        ("Infinity", F::INFINITY),
        ("INFINITY", F::INFINITY),
        ("inFiNIty", F::INFINITY),
        ("0", F::ZERO),
        ("1", F::from_int(1)),
        ("0.5", F::from_int(1).scalbn(-1)),
        ("1.5", F::from_int(3).scalbn(-1)),
    ];

    for &(s, value) in exact.iter() {
        check_from_str_exact(s, value);
        check_from_str_exact(&format!("+{s}"), value);
        check_from_str_exact(&format!("-{s}"), -value);
    }
}

fn check_from_str_round<F>(s: &str, expected_tz: F, expected_az: F, loss: Loss)
where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| F::from_str_ex(s, round).unwrap(),
        expected_tz,
        expected_az,
        s.starts_with('-'),
        loss,
    );

    assert_eq!(
        s.parse::<F>().ok().unwrap().to_bits(),
        F::from_str_ex(s, Round::NearestTiesToEven)
            .unwrap()
            .0
            .to_bits(),
    );
}

fn check_round_int_exact<F>(value: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = value.round_int_ex(round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), value.to_bits());
    }

    assert_eq!(value.round_ties_even().to_bits(), value.to_bits());
    assert_eq!(value.round_ties_away().to_bits(), value.to_bits());
    assert_eq!(value.floor().to_bits(), value.to_bits());
    assert_eq!(value.ceil().to_bits(), value.to_bits());
    assert_eq!(value.trunc().to_bits(), value.to_bits());
}

fn check_round_int_round<F>(value: F, expected_tz: F, expected_az: F, loss: Loss)
where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| value.round_int_ex(round),
        expected_tz,
        expected_az,
        value.is_sign_negative(),
        loss,
    );

    assert_eq!(
        value.round_ties_even().to_bits(),
        value.round_int_ex(Round::NearestTiesToEven).0.to_bits(),
    );
    assert_eq!(
        value.round_ties_away().to_bits(),
        value.round_int_ex(Round::NearestTiesToAway).0.to_bits(),
    );
    assert_eq!(
        value.floor().to_bits(),
        value.round_int_ex(Round::TowardNegative).0.to_bits(),
    );
    assert_eq!(
        value.ceil().to_bits(),
        value.round_int_ex(Round::TowardPositive).0.to_bits(),
    );
    assert_eq!(
        value.trunc().to_bits(),
        value.round_int_ex(Round::TowardZero).0.to_bits(),
    );
}

fn check_scalbn_exact<F>(value: F, exp: i32, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = value.scalbn_ex(exp, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), value.to_bits());
    }

    assert_eq!(value.scalbn(exp).to_bits(), expected_res.to_bits());
}

fn check_scalbn_round<F>(value: F, exp: i32, expected_tz: F, expected_az: F, loss: Loss)
where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| value.scalbn_ex(exp, round),
        expected_tz,
        expected_az,
        value.is_sign_negative(),
        loss,
    );

    assert_eq!(
        value.scalbn(exp).to_bits(),
        value.scalbn_ex(exp, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_frexp<F>(value: F, expected: (F, i32))
where
    F: Float<Bits: std::fmt::Debug>,
{
    let (mant, exp) = value.frexp();
    assert_eq!((mant.to_bits(), exp), (expected.0.to_bits(), expected.1));
}

fn check_add_sub_exact<F>(lhs: F, rhs: F, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = lhs.add_ex(rhs, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());

        let (value, status) = rhs.add_ex(lhs, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!((lhs + rhs).to_bits(), expected_res.to_bits());
    assert_eq!((rhs + lhs).to_bits(), expected_res.to_bits());
}

fn check_add_sub_round<F>(
    lhs: F,
    rhs: F,
    expected_res_tz: F,
    expected_res_az: F,
    sign: bool,
    loss: Loss,
) where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| lhs.add_ex(rhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );
    check_float_round(
        |round| rhs.add_ex(lhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );

    assert_eq!(
        (lhs + rhs).to_bits(),
        lhs.add_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
    assert_eq!(
        (rhs + lhs).to_bits(),
        lhs.add_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );

    check_float_round(
        |round| lhs.sub_ex(-rhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );
    check_float_round(
        |round| rhs.sub_ex(-lhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );

    assert_eq!(
        (lhs - (-rhs)).to_bits(),
        lhs.add_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
    assert_eq!(
        (rhs - (-lhs)).to_bits(),
        lhs.add_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_mul_exact<F>(lhs: F, rhs: F, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = lhs.mul_ex(rhs, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());

        let (value, status) = rhs.mul_ex(lhs, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!((lhs * rhs).to_bits(), expected_res.to_bits());
    assert_eq!((rhs * lhs).to_bits(), expected_res.to_bits());
}

fn check_mul_round<F>(
    lhs: F,
    rhs: F,
    expected_res_tz: F,
    expected_res_az: F,
    sign: bool,
    loss: Loss,
) where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| lhs.mul_ex(rhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );
    check_float_round(
        |round| rhs.mul_ex(lhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );

    assert_eq!(
        (lhs * rhs).to_bits(),
        lhs.mul_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
    assert_eq!(
        (rhs * lhs).to_bits(),
        lhs.mul_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_div_exact<F>(lhs: F, rhs: F, expected_res: F)
where
    F: Float<Bits: std::fmt::Debug>,
{
    for round in ALL_ROUND_MODES {
        let (value, status) = lhs.div_ex(rhs, round);
        assert_eq!(status, FpStatus::Ok);
        assert_eq!(value.to_bits(), expected_res.to_bits());
    }

    assert_eq!((lhs / rhs).to_bits(), expected_res.to_bits());
}

fn check_div_round<F>(
    lhs: F,
    rhs: F,
    expected_res_tz: F,
    expected_res_az: F,
    sign: bool,
    loss: Loss,
) where
    F: Float<Bits: std::fmt::Debug>,
{
    check_float_round(
        |round| lhs.div_ex(rhs, round),
        expected_res_tz,
        expected_res_az,
        sign,
        loss,
    );

    assert_eq!(
        (lhs / rhs).to_bits(),
        lhs.div_ex(rhs, Round::NearestTiesToEven).0.to_bits(),
    );
}

fn check_rem<F>(lhs: F, rhs: F, expected_res: F, expected_status: FpStatus)
where
    F: Float<Bits: std::fmt::Debug>,
{
    let (value, status) = lhs.rem_ex(rhs);
    assert_eq!(status, expected_status);
    assert_eq!(value.to_bits(), expected_res.to_bits());

    assert_eq!((lhs % rhs).to_bits(), expected_res.to_bits());
}
