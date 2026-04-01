use std::collections::BTreeSet;

use swfp::Float as _;

mod cbrt;
mod exp;
mod gamma;
mod hyperbolic;
mod hypot;
mod inv_hyperbolic;
mod inv_trigonometric;
mod log;
mod pow;
mod sqrt;
mod trigonometric;

fn mk_normal(m: u64, e: i16, s: bool) -> swfp::F64 {
    assert!(m < (1 << 52));
    assert!(matches!(e, -1022..=1023));
    let e = u64::from((e + 1023) as u16) << 52;
    let s = u64::from(s) << 63;
    swfp::F64::from_bits(m | e | s)
}

fn mk_subnormal(m: u64, s: bool) -> swfp::F64 {
    assert!(m < (1 << 52));
    let s = u64::from(s) << 63;
    swfp::F64::from_bits(m | s)
}

const PREC: u32 = 53;

fn to_rug(value: swfp::F64) -> rug::Float {
    to_rug_prec(value, PREC)
}

fn to_rug_prec(value: swfp::F64, prec: u32) -> rug::Float {
    if value.is_nan() {
        rug::Float::with_val(prec, rug::float::Special::Nan)
    } else if value.is_infinite() {
        if value.is_sign_negative() {
            rug::Float::with_val(prec, rug::float::Special::NegInfinity)
        } else {
            rug::Float::with_val(prec, rug::float::Special::Infinity)
        }
    } else {
        let bits = value.to_bits();

        let m = bits & ((1 << 52) - 1);
        let e = (bits >> 52) & 0x7FF;
        let s = (bits >> 63) != 0;
        let f = if e == 0 {
            rug::Float::with_val(prec, m) >> 1074u32
        } else {
            let m = m | (1 << 52);
            let e = (e as i32) - 1023;
            rug::Float::with_val(prec, m) << (e - 52)
        };
        if s { -f } else { f }
    }
}

fn from_rug(
    mut value: rug::Float,
    round_cmp: std::cmp::Ordering,
    round: rug::float::Round,
) -> (swfp::F64, swfp::FpStatus) {
    assert_eq!(value.prec(), PREC);

    let round_cmp = value.subnormalize_round(-1022 + 1, round_cmp, round);
    if value.is_nan() {
        (swfp::F64::NAN, swfp::FpStatus::Invalid)
    } else if value.is_infinite() {
        let status = swfp::FpStatus::Ok;
        if value.is_sign_negative() {
            (-swfp::F64::INFINITY, status)
        } else {
            (swfp::F64::INFINITY, status)
        }
    } else if value.is_zero() {
        let status = if round_cmp.is_eq() {
            swfp::FpStatus::Ok
        } else {
            swfp::FpStatus::Underflow
        };
        if value.is_sign_negative() {
            (-swfp::F64::ZERO, status)
        } else {
            (swfp::F64::ZERO, status)
        }
    } else {
        let e = value.get_exp().unwrap() - 1;
        if e > 1023 {
            let status = swfp::FpStatus::Overflow;
            let inf = match round {
                rug::float::Round::Nearest => true,
                rug::float::Round::Up => value.is_sign_positive(),
                rug::float::Round::Down => value.is_sign_negative(),
                rug::float::Round::Zero => false,
                _ => unreachable!(),
            };
            let res = if inf {
                if value.is_sign_negative() {
                    -swfp::F64::INFINITY
                } else {
                    swfp::F64::INFINITY
                }
            } else {
                mk_normal((1 << 52) - 1, 1023, value.is_sign_negative())
            };
            (res, status)
        } else if e < -1022 {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Underflow
            };
            let s = value.is_sign_negative();
            let m = (value.abs() << (1022i32 + 52))
                .to_integer()
                .unwrap()
                .to_u64_wrapping();
            (mk_subnormal(m, s), status)
        } else {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Inexact
            };
            let s = value.is_sign_negative();
            let m = (value.abs() >> (e - 52))
                .to_integer()
                .unwrap()
                .to_u64_wrapping();
            let m = m & ((1 << 52) - 1);
            let e = i16::try_from(e).unwrap();
            (mk_normal(m, e, s), status)
        }
    }
}

#[track_caller]
fn check_exact(
    input: impl std::fmt::Debug,
    actual: swfp::F64,
    actual_status: Option<swfp::FpStatus>,
    expected: rug::Float,
    round_cmp: std::cmp::Ordering,
    round: rug::float::Round,
) {
    let (expected, expected_status) = from_rug(expected, round_cmp, round);
    assert_total_eq!(actual, expected, "input = {input:?}");
    if let Some(actual_status) = actual_status {
        assert_eq!(actual_status, expected_status, "input = {input:?}");
    }
}

static NORMAL_MANTISSAS: std::sync::LazyLock<BTreeSet<u64>> = std::sync::LazyLock::new(|| {
    let mut mantissas = BTreeSet::new();

    mantissas.insert(0);

    for i in 1..=32 {
        let w = (1 << i) - 1;
        for i in 0..(52 - i) {
            mantissas.insert(w << i);
        }
    }

    for i in 0x1..=0xF {
        mantissas.insert(i * 0x0001_1111_1111_1111);
        mantissas.insert(i * 0x0000_1010_1010_1010);
        mantissas.insert(i * 0x0001_0101_0101_0101);
        mantissas.insert(i * 0x0000_1100_1100_1100);
        mantissas.insert(i * 0x0001_0011_0011_0011);
        mantissas.insert(i * 0x0000_1111_0000_1111);
        mantissas.insert(i * 0x0001_0000_1111_0000);
        mantissas.insert(i * 0x0000_1111_1111_0000);
        mantissas.insert(i * 0x0001_0000_0000_1111);
        mantissas.insert(i * 0x0000_1111_1111_1111);
    }

    for i in 0x1..=0xF {
        for j in 0x1..=0xF {
            if i != j {
                mantissas.insert((i * 0x0000_1010_1010_1010) | (j * 0x0001_0101_0101_0101));
                mantissas.insert((i * 0x0000_1100_1100_1100) | (j * 0x0001_0011_0011_0011));
                mantissas.insert((i * 0x0000_1111_0000_1111) | (j * 0x0001_0000_1111_0000));
                mantissas.insert((i * 0x0000_1111_1111_0000) | (j * 0x0001_0000_0000_1111));
                mantissas.insert((i * 0x0000_1111_1111_1111) | (j * 0x0001_0000_0000_0000));
            }
        }
    }

    mantissas
});

fn gen_mantissa(rng: &mut impl rand::RngExt) -> u64 {
    rng.random::<u64>() >> (64 - 52)
}

static SUBNORMAL_MANTISSAS: std::sync::LazyLock<BTreeSet<u64>> = std::sync::LazyLock::new(|| {
    let mut mantissas = BTreeSet::new();

    for i in 1..=32 {
        let w = (1 << i) - 1;
        for i in 0..(52 - i) {
            mantissas.insert(w << i);
        }
    }

    for i in 0x1..=0xF {
        mantissas.insert(i * 0x0000_0000_0000_0001);
        mantissas.insert(i * 0x0000_0000_0000_0011);
        mantissas.insert(i * 0x0000_0000_0000_0111);
        mantissas.insert(i * 0x0000_0000_0000_1111);
        mantissas.insert(i * 0x0000_0000_0001_1111);
        mantissas.insert(i * 0x0000_0000_0011_1111);
        mantissas.insert(i * 0x0000_0000_0111_1111);
        mantissas.insert(i * 0x0000_0000_1111_1111);
        mantissas.insert(i * 0x0000_0001_1111_1111);
        mantissas.insert(i * 0x0000_0011_1111_1111);
        mantissas.insert(i * 0x0000_0111_1111_1111);
        mantissas.insert(i * 0x0000_1111_1111_1111);
        mantissas.insert(i * 0x0001_1111_1111_1111);
    }

    mantissas
});
