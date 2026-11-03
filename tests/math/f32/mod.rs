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
mod values;

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

    // Tininess is detected after rounding with an unbounded exponent range,
    // which is how `value` has been rounded so far.
    let is_tiny = value.get_exp().is_some_and(|e| e - 1 < -126);

    let round_cmp = value.subnormalize_round(-126 + 1, round_cmp, round);
    if value.is_nan() {
        (swfp::F32::NAN, swfp::FpStatus::INVALID)
    } else if value.is_infinite() {
        let status = swfp::FpStatus::OK;
        if value.is_sign_negative() {
            (-swfp::F32::INFINITY, status)
        } else {
            (swfp::F32::INFINITY, status)
        }
    } else if value.is_zero() {
        let status = if round_cmp.is_eq() {
            swfp::FpStatus::OK
        } else {
            swfp::FpStatus::UNDERFLOW | swfp::FpStatus::INEXACT
        };
        if value.is_sign_negative() {
            (-swfp::F32::ZERO, status)
        } else {
            (swfp::F32::ZERO, status)
        }
    } else {
        let e = value.get_exp().unwrap() - 1;
        if e > 127 {
            let status = swfp::FpStatus::OVERFLOW | swfp::FpStatus::INEXACT;
            let inf = match round {
                rug::float::Round::Nearest => true,
                rug::float::Round::Up => value.is_sign_positive(),
                rug::float::Round::Down => value.is_sign_negative(),
                rug::float::Round::Zero => false,
                rug::float::Round::AwayZero => true,
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
                swfp::FpStatus::OK
            } else {
                swfp::FpStatus::UNDERFLOW | swfp::FpStatus::INEXACT
            };
            let s = value.is_sign_negative();
            let m = (value.abs() << (126i32 + 23)).to_u32_saturating().unwrap();
            (mk_subnormal(m, s), status)
        } else {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::OK
            } else if is_tiny {
                swfp::FpStatus::UNDERFLOW | swfp::FpStatus::INEXACT
            } else {
                swfp::FpStatus::INEXACT
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
