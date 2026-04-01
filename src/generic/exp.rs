use crate::traits::FloatImpExt;

pub(crate) trait Exp: FloatImpExt {
    fn exp_finite(x: Self) -> Self;

    fn exp_m1_finite(x: Self) -> Self;

    fn exp2_finite(x: Self) -> Self;

    fn exp2_m1_finite(x: Self) -> Self;

    fn exp10_finite(x: Self) -> Self;

    fn exp10_m1_finite(x: Self) -> Self;
}

pub(crate) fn exp<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp(-inf) = 0
            F::ZERO
        } else {
            // exp(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp(0) = 1
        F::ONE
    } else {
        F::exp_finite(x)
    }
}

pub(crate) fn exp_m1<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp_m1(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp_m1(-inf) = -1
            -F::ONE
        } else {
            // exp_m1(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp_m1(±0) = ±0
        x
    } else {
        F::exp_m1_finite(x)
    }
}

pub(crate) fn exp2<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp(-inf) = 0
            F::ZERO
        } else {
            // exp(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp(0) = 1
        F::ONE
    } else {
        F::exp2_finite(x)
    }
}

pub(crate) fn exp2_m1<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp2_m1(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp2_m1(-inf) = -1
            -F::ONE
        } else {
            // exp2_m1(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp2_m1(±0) = ±0
        x
    } else {
        F::exp2_m1_finite(x)
    }
}

pub(crate) fn exp10<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp(-inf) = 0
            F::ZERO
        } else {
            // exp(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp(0) = 1
        F::ONE
    } else {
        F::exp10_finite(x)
    }
}

pub(crate) fn exp10_m1<F: Exp>(x: F) -> F {
    if x.is_nan() {
        // exp10_m1(NaN) = NaN
        F::NAN
    } else if x.is_infinite() {
        if x.sign() {
            // exp10_m1(-inf) = -1
            -F::ONE
        } else {
            // exp10_m1(inf) = inf
            F::INFINITY
        }
    } else if x.is_zero() {
        // exp10_m1(±0) = ±0
        x
    } else {
        F::exp10_m1_finite(x)
    }
}
