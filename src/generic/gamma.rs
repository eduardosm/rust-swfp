use crate::traits::FloatImpExt;

pub(crate) trait Gamma: FloatImpExt {
    fn gamma_finite(x: Self) -> Self;

    fn ln_gamma_finite(x: Self) -> (Self, i8);
}

pub(crate) fn gamma<F: Gamma>(x: F) -> F {
    if x.is_nan() {
        // gamma(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // gamma(-inf) = NaN
            F::NAN
        } else {
            // gamma(+inf) = +inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // gamma(±0) = ±inf
        F::INFINITY.set_sign(x.sign())
    } else if x.sign() && x.is_int() {
        // gamma(neg integer) = NaN
        F::NAN
    } else {
        F::gamma_finite(x)
    }
}

pub(crate) fn ln_gamma<F: Gamma>(x: F) -> (F, i8) {
    if x.is_nan() {
        // ln_gamma(NaN) = NaN
        (F::NAN, 0)
    } else if x.is_infinite() {
        if x.sign() {
            // ln_gamma(-inf) = NaN
            (F::NAN, 0)
        } else {
            // ln_gamma(+inf) = +inf
            (F::INFINITY, 1)
        }
    } else if x.is_zero() {
        // ln_gamma(0) = +inf
        (F::INFINITY, if x.sign() { -1 } else { 1 })
    } else if x.sign() && x.is_int() {
        // ln_gamma(neg integer) = +inf
        (F::INFINITY, 0)
    } else if x.bitwise_eq(F::ONE) || x.bitwise_eq(F::TWO) {
        // ln_gamma(1 or 2) = 0
        // ensure positive zero
        (F::ZERO, 1)
    } else {
        F::ln_gamma_finite(x)
    }
}
