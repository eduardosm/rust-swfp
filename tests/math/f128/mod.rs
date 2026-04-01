use swfp::Float as _;

mod sqrt;

fn mk_normal(m: u128, e: i16, s: bool) -> swfp::F128 {
    assert!(m < (1 << 112));
    assert!(matches!(e, -16382..=16383));
    let e = u128::from((e + 16383) as u16) << 112;
    let s = u128::from(s) << 127;
    swfp::F128::from_bits(m | e | s)
}

fn mk_subnormal(m: u128, s: bool) -> swfp::F128 {
    assert!(m < (1 << 112));
    let s = u128::from(s) << 127;
    swfp::F128::from_bits(m | s)
}

const PREC: u32 = 113;

fn to_rug(value: swfp::F128) -> rug::Float {
    to_rug_prec(value, PREC)
}

fn to_rug_prec(value: swfp::F128, prec: u32) -> rug::Float {
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

        let m = bits & ((1 << 112) - 1);
        let e = (bits >> 112) & 0x7FFF;
        let s = (bits >> 127) != 0;
        let f = if e == 0 {
            rug::Float::with_val(prec, m) >> 16494u32
        } else {
            let m = m | (1 << 112);
            let e = (e as i32) - 16383;
            rug::Float::with_val(prec, m) << (e - 112)
        };
        if s { -f } else { f }
    }
}

fn from_rug(
    mut value: rug::Float,
    round_cmp: std::cmp::Ordering,
    round: rug::float::Round,
) -> (swfp::F128, swfp::FpStatus) {
    assert_eq!(value.prec(), PREC);

    let round_cmp = value.subnormalize_round(-16382 + 1, round_cmp, round);
    if value.is_nan() {
        (swfp::F128::NAN, swfp::FpStatus::Invalid)
    } else if value.is_infinite() {
        let status = swfp::FpStatus::Ok;
        if value.is_sign_negative() {
            (-swfp::F128::INFINITY, status)
        } else {
            (swfp::F128::INFINITY, status)
        }
    } else if value.is_zero() {
        let status = if round_cmp.is_eq() {
            swfp::FpStatus::Ok
        } else {
            swfp::FpStatus::Underflow
        };
        if value.is_sign_negative() {
            (-swfp::F128::ZERO, status)
        } else {
            (swfp::F128::ZERO, status)
        }
    } else {
        let e = value.get_exp().unwrap() - 1;
        if e > 16383 {
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
                    -swfp::F128::INFINITY
                } else {
                    swfp::F128::INFINITY
                }
            } else {
                mk_normal((1 << 112) - 1, 16383, value.is_sign_negative())
            };
            (res, status)
        } else if e < -16382 {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Underflow
            };
            let s = value.is_sign_negative();
            let m = (value.abs() << (16382i32 + 112))
                .to_integer()
                .unwrap()
                .to_u128_wrapping();
            (mk_subnormal(m, s), status)
        } else {
            let status = if round_cmp.is_eq() {
                swfp::FpStatus::Ok
            } else {
                swfp::FpStatus::Inexact
            };
            let s = value.is_sign_negative();
            let m = (value.abs() >> (e - 112))
                .to_integer()
                .unwrap()
                .to_u128_wrapping();
            let m = m & ((1 << 112) - 1);
            let e = i16::try_from(e).unwrap();
            (mk_normal(m, e, s), status)
        }
    }
}

#[track_caller]
fn check_exact(
    input: impl std::fmt::Debug,
    actual: swfp::F128,
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

fn gen_mantissa(rng: &mut impl rand::RngExt) -> u128 {
    rng.random::<u128>() >> (128 - 112)
}
