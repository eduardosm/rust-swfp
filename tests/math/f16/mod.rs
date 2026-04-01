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

fn mk_normal(m: u16, e: i8, s: bool) -> swfp::F16 {
    assert!(m < (1 << 10));
    assert!(matches!(e, -14..=15));
    let e = u16::from((e + 15) as u8) << 10;
    let s = u16::from(s) << 15;
    swfp::F16::from_bits(m | e | s)
}

fn mk_subnormal(m: u16, s: bool) -> swfp::F16 {
    assert!(m < (1 << 10));
    let s = u16::from(s) << 15;
    swfp::F16::from_bits(m | s)
}

const PREC: u32 = 11;

fn to_rug(value: swfp::F16) -> rug::Float {
    to_rug_prec(value, PREC)
}

fn to_rug_prec(value: swfp::F16, prec: u32) -> rug::Float {
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

        let m = bits & ((1 << 10) - 1);
        let e = (bits >> 10) & 0x1F;
        let s = (bits >> 15) != 0;
        let f = if e == 0 {
            rug::Float::with_val(prec, m) >> 24u32
        } else {
            let m = m | (1 << 10);
            let e = (e as i32) - 15;
            rug::Float::with_val(prec, m) << (e - 10)
        };
        if s { -f } else { f }
    }
}

fn from_rug(
    mut value: rug::Float,
    round_cmp: std::cmp::Ordering,
    round: rug::float::Round,
) -> (swfp::F16, swfp::FpStatus) {
    assert_eq!(value.prec(), PREC);

    let round_cmp = value.subnormalize_round(-14 + 1, round_cmp, round);
    if value.is_nan() {
        (swfp::F16::NAN, swfp::FpStatus::Invalid)
    } else if value.is_infinite() {
        let status = swfp::FpStatus::Ok;
        if value.is_sign_negative() {
            (-swfp::F16::INFINITY, status)
        } else {
            (swfp::F16::INFINITY, status)
        }
    } else if value.is_zero() {
        let status = if round_cmp.is_eq() {
            swfp::FpStatus::Ok
        } else {
            swfp::FpStatus::Underflow
        };
        if value.is_sign_negative() {
            (-swfp::F16::ZERO, status)
        } else {
            (swfp::F16::ZERO, status)
        }
    } else {
        let e = value.get_exp().unwrap() - 1;
        if e > 15 {
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
                    -swfp::F16::INFINITY
                } else {
                    swfp::F16::INFINITY
                }
            } else {
                mk_normal((1 << 10) - 1, 15, value.is_sign_negative())
            };
            (res, status)
        } else if e < -14 {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Underflow
            };
            let s = value.is_sign_negative();
            let m = (value.abs() << (14i32 + 10)).to_u32_saturating().unwrap();
            let m = m as u16;
            (mk_subnormal(m, s), status)
        } else {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Inexact
            };
            let s = value.is_sign_negative();
            let m = (value.abs() >> (e - 10)).to_u32_saturating().unwrap();
            let m = (m & ((1 << 10) - 1)) as u16;
            let e = i8::try_from(e).unwrap();
            (mk_normal(m, e, s), status)
        }
    }
}

#[track_caller]
fn check_exact(
    input: impl std::fmt::Debug,
    actual: swfp::F16,
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

fn gen_mantissa(rng: &mut impl rand::RngExt) -> u16 {
    rng.random::<u16>() >> (16 - 10)
}
