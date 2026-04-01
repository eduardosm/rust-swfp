use crate::traits::FloatImpExt;

pub(crate) trait Hyperbolic: FloatImpExt {
    fn sinh_finite(x: Self) -> Self;

    fn cosh_finite(x: Self) -> Self;

    fn sinh_cosh_finite(x: Self) -> (Self, Self);

    fn tanh_finite(x: Self) -> Self;
}

pub(crate) fn sinh<F: Hyperbolic>(x: F) -> F {
    if x.is_nan() {
        // sinh(NaN) = NaN
        F::NAN
    } else if x.is_infinite() || x.is_zero() {
        // sinh(±inf) = ±inf
        // sinh(±0) = ±0
        x
    } else {
        F::sinh_finite(x)
    }
}

pub(crate) fn cosh<F: Hyperbolic>(x: F) -> F {
    if x.is_nan() {
        // cosh(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        // cosh(±inf) = +inf
        F::INFINITY
    } else {
        F::cosh_finite(x)
    }
}

pub(crate) fn sinh_cosh<F: Hyperbolic>(x: F) -> (F, F) {
    if x.is_nan() {
        // sinh(NaN) = NaN
        // cosh(NaN) = NaN
        (F::NAN, F::NAN)
    } else if x.is_infinite() {
        // sinh(±inf) = ±inf
        // cosh(±inf) = +inf
        (x, F::INFINITY)
    } else if x.is_zero() {
        // sinh(±0) = ±0
        // cosh(±0) = 1
        (x, F::ONE)
    } else {
        F::sinh_cosh_finite(x)
    }
}

pub(crate) fn tanh<F: Hyperbolic>(x: F) -> F {
    if x.is_nan() {
        // tanh(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        // tanh(±inf) = 1
        F::ONE.set_sign(x.sign())
    } else if x.is_zero() {
        // tanh(±0) = ±0
        // cosh(±0) = 1
        x
    } else {
        F::tanh_finite(x)
    }
}
