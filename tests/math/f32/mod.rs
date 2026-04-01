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

fn mk_normal(m: u32, e: i16, s: bool) -> swfp::F32 {
    assert!(m < (1 << 23));
    assert!(matches!(e, -126..=127));
    let e = u32::from((e + 127) as u16) << 23;
    let s = u32::from(s) << 31;
    swfp::F32::from_bits(m | e | s)
}

fn mk_subnormal(m: u32, s: bool) -> swfp::F32 {
    assert!(m < (1 << 23));
    let s = u32::from(s) << 31;
    swfp::F32::from_bits(m | s)
}

const PREC: u32 = 24;

fn to_rug(value: swfp::F32) -> rug::Float {
    to_rug_prec(value, PREC)
}

fn to_rug_prec(value: swfp::F32, prec: u32) -> rug::Float {
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

        let m = bits & ((1 << 23) - 1);
        let e = (bits >> 23) & 0xFF;
        let s = (bits >> 31) != 0;
        let f = if e == 0 {
            rug::Float::with_val(prec, m) >> 149u32
        } else {
            let m = m | (1 << 23);
            let e = (e as i32) - 127;
            rug::Float::with_val(prec, m) << (e - 23)
        };
        if s { -f } else { f }
    }
}

fn from_rug(
    mut value: rug::Float,
    round_cmp: std::cmp::Ordering,
    round: rug::float::Round,
) -> (swfp::F32, swfp::FpStatus) {
    assert_eq!(value.prec(), PREC);

    let round_cmp = value.subnormalize_round(-126 + 1, round_cmp, round);
    if value.is_nan() {
        (swfp::F32::NAN, swfp::FpStatus::Invalid)
    } else if value.is_infinite() {
        let status = swfp::FpStatus::Ok;
        if value.is_sign_negative() {
            (-swfp::F32::INFINITY, status)
        } else {
            (swfp::F32::INFINITY, status)
        }
    } else if value.is_zero() {
        let status = if round_cmp.is_eq() {
            swfp::FpStatus::Ok
        } else {
            swfp::FpStatus::Underflow
        };
        if value.is_sign_negative() {
            (-swfp::F32::ZERO, status)
        } else {
            (swfp::F32::ZERO, status)
        }
    } else {
        let e = value.get_exp().unwrap() - 1;
        if e > 127 {
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
                    -swfp::F32::INFINITY
                } else {
                    swfp::F32::INFINITY
                }
            } else {
                mk_normal((1 << 23) - 1, 127, value.is_sign_negative())
            };
            (res, status)
        } else if e < -126 {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Underflow
            };
            let s = value.is_sign_negative();
            let m = (value.abs() << (126i32 + 23)).to_u32_saturating().unwrap();
            (mk_subnormal(m, s), status)
        } else {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Inexact
            };
            let s = value.is_sign_negative();
            let m = (value.abs() >> (e - 23)).to_u32_saturating().unwrap();
            let m = m & ((1 << 23) - 1);
            let e = i16::try_from(e).unwrap();
            (mk_normal(m, e, s), status)
        }
    }
}

#[track_caller]
fn check_exact(
    input: impl std::fmt::Debug,
    actual: swfp::F32,
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

fn gen_mantissa(rng: &mut impl rand::RngExt) -> u32 {
    rng.random::<u32>() >> (32 - 23)
}

static NORMAL_MANTISSAS: std::sync::LazyLock<BTreeSet<u32>> = std::sync::LazyLock::new(|| {
    let mut mantissas = BTreeSet::new();

    mantissas.insert(0);

    for i in 1..=18 {
        let w = (1 << i) - 1;
        for i in 0..(23 - i) {
            mantissas.insert(w << i);
        }
    }

    for i in 0x1..=0xF {
        mantissas.insert((i * 0x0011_1111) >> 1);
        mantissas.insert((i * 0x0010_1010) >> 1);
        mantissas.insert((i * 0x0001_0101) >> 1);
        mantissas.insert((i * 0x0011_0011) >> 1);
        mantissas.insert((i * 0x0001_1000) >> 1);
    }

    for i in 0x1..=0xF {
        for j in 0x1..=0xF {
            if i != j {
                mantissas.insert(((i * 0x0010_1010) | (j * 0x0001_0101)) >> 1);
                mantissas.insert(((i * 0x0011_0011) | (j * 0x0000_1100)) >> 1);
            }
        }
    }

    mantissas
});
