use crate::traits::{FloatImpExt, Int as _};

pub(crate) trait InvHyperbolic: FloatImpExt {
    fn asinh_finite(x: Self) -> Self;

    fn acosh_finite(x: Self) -> Self;

    fn atanh_finite(x: Self) -> Self;
}

pub(crate) fn asinh<F: InvHyperbolic>(x: F) -> F {
    if x.is_nan() {
        // asinh(NaN) = NaN
        F::NAN
    } else if x.is_infinite() || x.is_zero() {
        // asinh(±inf) = ±inf
        // asinh(±0) = ±0
        // asinh(x) = ~x for very small x
        x
    } else {
        F::asinh_finite(x)
    }
}

pub(crate) fn acosh<F: InvHyperbolic>(x: F) -> F {
    if x.is_nan() {
        // acosh(NaN) = NaN
        F::NAN
    } else if x.sign() || x.exponent() < F::Exp::ZERO {
        // acosh(x < 1) = NaN
        F::NAN
    } else if x.is_infinite() {
        // acosh(inf) = inf
        x
    } else {
        F::acosh_finite(x)
    }
}

pub(crate) fn atanh<F: InvHyperbolic>(x: F) -> F {
    if x.is_nan() {
        // atanh(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // atanh(±0) = ±0
        x
    } else {
        match x.abs().partial_cmp(&F::ONE) {
            Some(core::cmp::Ordering::Equal) => {
                // atanh(±1) = ±inf
                F::INFINITY.set_sign(x.sign())
            }
            Some(core::cmp::Ordering::Greater) => {
                // atanh(|x| > 1) = NaN
                F::NAN
            }
            _ => F::atanh_finite(x),
        }
    }
}
