use crate::traits::FloatImpExt;

pub(crate) trait InvTrigonometric: FloatImpExt {
    fn pi() -> Self;

    fn frac_pi_2() -> Self;

    fn asin_finite(x: Self) -> Self;

    fn acos_finite(x: Self) -> Self;

    fn atan_finite(x: Self) -> Self;

    fn atan2_finite(y: Self, x: Self) -> Self;

    fn asind_finite(x: Self) -> Self;

    fn acosd_finite(x: Self) -> Self;

    fn atand_finite(x: Self) -> Self;

    fn atan2d_finite(y: Self, x: Self) -> Self;

    fn asinpi_finite(x: Self) -> Self;

    fn acospi_finite(x: Self) -> Self;

    fn atanpi_finite(x: Self) -> Self;

    fn atan2pi_finite(y: Self, x: Self) -> Self;
}

pub(crate) fn asin<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // asin(NaN) = NaN
        // asin(±inf) = NaN
        // asin(|x| > 1) = NaN
        F::NAN
    } else if x.is_zero() {
        // asin(±0) = ±0
        x
    } else {
        F::asin_finite(x)
    }
}

pub(crate) fn acos<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // asin(NaN) = NaN
        // asin(±inf) = NaN
        // asin(|x| > 1) = NaN
        F::NAN
    } else {
        F::acos_finite(x)
    }
}

pub(crate) fn atan<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() {
        // atan(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // atan(±0) = ±0
        x
    } else {
        F::atan_finite(x)
    }
}

pub(crate) fn atan2<F: InvTrigonometric>(y: F, x: F) -> F {
    if x.is_nan() || y.is_nan() {
        // atan2(NaN, x) = NaN
        // atan2(y, NaN) = NaN
        return F::NAN;
    }

    let (y, x) = atan2_de_inf(y, x);
    if y.is_zero() {
        if x.sign() {
            // atan2(±0, -0) = ±π
            // atan2(±0, x < 0) = ±π
            F::pi().set_sign(y.sign())
        } else {
            // atan2(±0, +0) = ±π
            // atan2(±0, x > 0) = ±0
            y
        }
    } else if x.is_zero() {
        // atan2(y < 0, ±0) = -π/2
        // atan2(y > 0, ±0) = +π/2
        F::frac_pi_2().set_sign(y.sign())
    } else {
        F::atan2_finite(y, x)
    }
}

pub(crate) fn asind<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // asind(NaN) = NaN
        // asind(±inf) = NaN
        // asind(|x| > 1) = NaN
        F::NAN
    } else if x.is_zero() {
        // asind(±0) = ±0
        x
    } else {
        F::asind_finite(x)
    }
}

pub(crate) fn acosd<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // asind(NaN) = NaN
        // asind(±inf) = NaN
        // asind(|x| > 1) = NaN
        F::NAN
    } else {
        F::acosd_finite(x)
    }
}

pub(crate) fn atand<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() {
        // atand(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // atand(±0) = ±0
        x
    } else {
        F::atand_finite(x)
    }
}

pub(crate) fn atan2d<F: InvTrigonometric>(y: F, x: F) -> F {
    if x.is_nan() || y.is_nan() {
        // atan2d(NaN, x) = NaN
        // atan2d(y, NaN) = NaN
        return F::NAN;
    }

    let (y, x) = atan2_de_inf(y, x);
    if y.is_zero() {
        if x.sign() {
            // atan2d(±0, -0) = ±180
            // atan2d(±0, x < 0) = ±180
            F::from_uint(180).set_sign(y.sign())
        } else {
            // atan2(±0, +0) = ±180
            // atan2(±0, x > 0) = ±0
            y
        }
    } else if x.is_zero() {
        // atan2(y < 0, ±0) = -90
        // atan2(y > 0, ±0) = +90
        F::from_uint(90).set_sign(y.sign())
    } else {
        F::atan2d_finite(y, x)
    }
}

pub(crate) fn asinpi<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // asinpi(NaN) = NaN
        // asinpi(±inf) = NaN
        // asinpi(|x| > 1) = NaN
        F::NAN
    } else if x.is_zero() {
        // asinpi(±0) = ±0
        x
    } else {
        F::asinpi_finite(x)
    }
}

pub(crate) fn acospi<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() || x.abs() > F::ONE {
        // acospi(NaN) = NaN
        // acospi(±∞) = NaN
        // acospi(|x| > 1) = NaN
        F::NAN
    } else {
        F::acospi_finite(x)
    }
}

pub(crate) fn atanpi<F: InvTrigonometric>(x: F) -> F {
    if x.is_nan() {
        // atanpi(NaN) = NaN
        F::NAN
    } else if x.is_zero() {
        // atanpi(±0) = ±0
        x
    } else {
        F::atanpi_finite(x)
    }
}

pub(crate) fn atan2pi<F: InvTrigonometric>(y: F, x: F) -> F {
    if x.is_nan() || y.is_nan() {
        // atan2pi(NaN, x) = NaN
        // atan2pi(y, NaN) = NaN
        return F::NAN;
    }

    let (y, x) = atan2_de_inf(y, x);
    if y.is_zero() {
        if x.sign() {
            // atan2pi(±0, -0) = ±1
            // atan2pi(±0, x < 0) = ±1
            F::ONE.set_sign(y.sign())
        } else {
            // atan2pi(±0, +0) = ±1
            // atan2pi(±0, x > 0) = ±0
            y
        }
    } else if x.is_zero() {
        // atan2pi(y < 0, ±0) = -0.5
        // atan2pi(y > 0, ±0) = +0.5
        F::HALF.set_sign(y.sign())
    } else {
        F::atan2pi_finite(y, x)
    }
}

#[inline]
fn atan2_de_inf<F: FloatImpExt>(y: F, x: F) -> (F, F) {
    let y_inf = y.is_infinite();
    let x_inf = x.is_infinite();
    if y_inf || x_inf {
        let y = (if y_inf { F::ONE } else { F::ZERO }).set_sign(y.sign());
        let x = (if x_inf { F::ONE } else { F::ZERO }).set_sign(x.sign());
        (y, x)
    } else {
        (y, x)
    }
}
