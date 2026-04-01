use crate::traits::FloatImpExt;

pub(crate) trait Log: FloatImpExt {
    fn ln_finite(x: Self) -> Self;

    fn ln_1p_finite(x: Self) -> Self;

    fn log2_finite(x: Self) -> Self;

    fn log2_1p_finite(x: Self) -> Self;

    fn log10_finite(x: Self) -> Self;

    fn log10_1p_finite(x: Self) -> Self;
}

pub(crate) fn ln<F: Log>(x: F) -> F {
    if x.is_nan() {
        // ln(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // ln(±0) = -inf
        -F::INFINITY
    } else if x.sign() {
        // ln(x < 0) = NaN
        F::NAN
    } else if x.is_infinite() {
        // ln(inf) = inf
        x
    } else {
        F::ln_finite(x)
    }
}

pub(crate) fn ln_1p<F: Log>(x: F) -> F {
    if x.is_nan() {
        // ln_1p(NaN) = NaN
        F::NAN
    } else if x.is_zero() || (x.is_infinite() && !x.sign()) {
        // ln_1p(±0) = ±0
        // ln_1p(inf) = inf
        x
    } else {
        match x.partial_cmp(&-F::ONE) {
            Some(core::cmp::Ordering::Equal) => {
                // ln_1p(-1) = -inf
                return -F::INFINITY;
            }
            Some(core::cmp::Ordering::Less) => {
                // ln_1p(x < -1) = NaN
                return F::NAN;
            }
            _ => {}
        }

        F::ln_1p_finite(x)
    }
}

pub(crate) fn log2<F: Log>(x: F) -> F {
    if x.is_nan() {
        // log2(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // log2(±0) = -inf
        -F::INFINITY
    } else if x.sign() {
        // log2(x < 0) = NaN
        F::NAN
    } else if x.is_infinite() {
        // log2(inf) = inf
        x
    } else {
        F::log2_finite(x)
    }
}

pub(crate) fn log2_1p<F: Log>(x: F) -> F {
    if x.is_nan() {
        // log2_1p(NaN) = NaN
        F::NAN
    } else if x.is_zero() || (x.is_infinite() && !x.sign()) {
        // log2_1p(±0) = ±0
        // log2_1p(inf) = inf
        x
    } else {
        match x.partial_cmp(&-F::ONE) {
            Some(core::cmp::Ordering::Equal) => {
                // log2_1p(-1) = -inf
                return -F::INFINITY;
            }
            Some(core::cmp::Ordering::Less) => {
                // log2_1p(x < -1) = NaN
                return F::NAN;
            }
            _ => {}
        }

        F::log2_1p_finite(x)
    }
}

pub(crate) fn log10<F: Log>(x: F) -> F {
    if x.is_nan() {
        // log10(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // log10(±0) = -inf
        -F::INFINITY
    } else if x.sign() {
        // log10(x < 0) = NaN
        F::NAN
    } else if x.is_infinite() {
        // log10(inf) = inf
        x
    } else {
        F::log10_finite(x)
    }
}

pub(crate) fn log10_1p<F: Log>(x: F) -> F {
    if x.is_nan() {
        // log10_1p(NaN) = NaN
        F::NAN
    } else if x.is_zero() || (x.is_infinite() && !x.sign()) {
        // log10_1p(±0) = ±0
        // log10_1p(inf) = inf
        x
    } else {
        match x.partial_cmp(&-F::ONE) {
            Some(core::cmp::Ordering::Equal) => {
                // log10_1p(-1) = -inf
                return -F::INFINITY;
            }
            Some(core::cmp::Ordering::Less) => {
                // log10_1p(x < -1) = NaN
                return F::NAN;
            }
            _ => {}
        }

        F::log10_1p_finite(x)
    }
}
