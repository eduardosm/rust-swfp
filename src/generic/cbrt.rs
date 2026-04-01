use crate::traits::FloatImpExt;

pub(crate) trait Cbrt: FloatImpExt {
    fn cbrt_finite(x: Self) -> Self;
}

pub(crate) fn cbrt<F: Cbrt>(x: F) -> F {
    if x.is_nan() {
        // cbrt(NaN) = NaN
        F::NAN
    } else if x.is_infinite() || x.is_zero() {
        // cbrt(±inf) = ±inf
        // cbrt(±0) = ±0
        x
    } else {
        F::cbrt_finite(x)
    }
}
