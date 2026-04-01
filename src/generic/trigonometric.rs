use crate::math_aux::reduce_half_revs;
use crate::traits::FloatImpExt;

pub(crate) trait Trigonometric: FloatImpExt {
    fn sin_finite(x: Self) -> Self;

    fn cos_finite(x: Self) -> Self;

    fn sin_cos_finite(x: Self) -> (Self, Self);

    fn tan_finite(x: Self) -> Self;

    /// Reduces the angle argument `x` (in degrees), returning `(n, y)` such as:
    /// * `|y| <= 45`
    /// * `0 <= n <= 3`
    /// * `x = 360*M + 90*n + y`
    /// * `M` is an integer
    fn reduce_90_deg(x: Self) -> (u8, Self);

    fn sind_reduced(x: Self) -> Self;

    fn cosd_reduced(x: Self) -> Self;

    fn sind_cosd_reduced(x: Self) -> (Self, Self);

    fn tand_reduced(x: Self, inv: bool) -> Self;

    fn sinpi_reduced(x: Self) -> Self;

    fn cospi_reduced(x: Self) -> Self;

    fn sinpi_cospi_reduced(x: Self) -> (Self, Self);

    fn tanpi_reduced(x: Self, inv: bool) -> Self;
}

pub(crate) fn sin<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // sin(NaN) = NaN
        // sin(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // sin(±0) = ±0
        x
    } else {
        F::sin_finite(x)
    }
}

pub(crate) fn cos<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // cos(NaN) = NaN
        // cos(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // cos(±0) = 1
        F::ONE
    } else {
        F::cos_finite(x)
    }
}

pub(crate) fn sin_cos<F: Trigonometric>(x: F) -> (F, F) {
    if x.is_nan() || x.is_infinite() {
        // sin(NaN) = NaN
        // cos(NaN) = NaN
        // sin(±inf) = NaN
        // cos(±inf) = NaN
        (F::NAN, F::NAN)
    } else if x.is_zero() {
        // sin(±0) = ±0
        // cos(±0) = 1
        (x, F::ONE)
    } else {
        F::sin_cos_finite(x)
    }
}

pub(crate) fn tan<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // tan(NaN) = NaN
        // tan(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // tan(±0) = ±0
        x
    } else {
        F::tan_finite(x)
    }
}

pub(crate) fn sind<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // sind(NaN) = NaN
        // sind(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // sind(±0) = ±0
        x
    } else {
        let (n, y) = F::reduce_90_deg(x);
        if y.is_zero() {
            match n {
                0 => F::ZERO.set_sign(x.sign()),
                1 => F::ONE,
                2 => F::ZERO.set_sign(x.sign()),
                3 => -F::ONE,
                _ => unreachable!(),
            }
        } else {
            match n {
                0 => F::sind_reduced(y),
                1 => F::cosd_reduced(y),
                2 => -F::sind_reduced(y),
                3 => -F::cosd_reduced(y),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) fn cosd<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // cosd(NaN) = NaN
        // cosd(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // cosd(±0) = 1
        F::ONE
    } else {
        let (n, y) = F::reduce_90_deg(x);
        if y.is_zero() {
            match n {
                0 => F::ONE,
                1 => F::ZERO,
                2 => -F::ONE,
                3 => F::ZERO,
                _ => unreachable!(),
            }
        } else {
            match n {
                0 => F::cosd_reduced(y),
                1 => -F::sind_reduced(y),
                2 => -F::cosd_reduced(y),
                3 => F::sind_reduced(y),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) fn sind_cosd<F: Trigonometric>(x: F) -> (F, F) {
    if x.is_nan() || x.is_infinite() {
        // sind(NaN) = NaN
        // cosd(NaN) = NaN
        // sind(±inf) = NaN
        // cosd(±inf) = NaN
        (F::NAN, F::NAN)
    } else if x.is_zero() {
        // sind(±0) = ±0
        // cosd(±0) = 1
        (x, F::ONE)
    } else {
        let (n, y) = F::reduce_90_deg(x);
        if y.is_zero() {
            let sym_zero = F::ZERO.set_sign(x.sign());
            match n {
                0 => (sym_zero, F::ONE),
                1 => (F::ONE, F::ZERO),
                2 => (sym_zero, -F::ONE),
                3 => (-F::ONE, F::ZERO),
                _ => unreachable!(),
            }
        } else {
            let (sin, cos) = F::sind_cosd_reduced(y);
            let (sin, cos) = match n {
                0 => (sin, cos),
                1 => (cos, -sin),
                2 => (-sin, -cos),
                3 => (-cos, sin),
                _ => unreachable!(),
            };
            (sin, cos)
        }
    }
}

pub(crate) fn tand<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // tand(NaN) = NaN
        // tand(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // tand(±0) = ±0
        x
    } else {
        let (n, y) = F::reduce_90_deg(x);
        let inv = (n & 1) != 0;
        if y.is_zero() {
            if inv {
                F::INFINITY.set_sign(x.sign())
            } else {
                F::ZERO.set_sign(x.sign())
            }
        } else {
            F::tand_reduced(y, inv)
        }
    }
}

pub(crate) fn sinpi<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // sind(NaN) = NaN
        // sind(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // sind(±0) = ±0
        x
    } else {
        let (n, y) = reduce_half_revs(x);
        if y.is_zero() {
            match n {
                0 => F::ZERO.set_sign(x.sign()),
                1 => F::ONE,
                2 => F::ZERO.set_sign(x.sign()),
                3 => -F::ONE,
                _ => unreachable!(),
            }
        } else {
            match n {
                0 => F::sinpi_reduced(y),
                1 => F::cospi_reduced(y),
                2 => -F::sinpi_reduced(y),
                3 => -F::cospi_reduced(y),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) fn cospi<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // cosd(NaN) = NaN
        // cosd(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // cosd(±0) = 1
        F::ONE
    } else {
        let (n, y) = reduce_half_revs(x);
        if y.is_zero() {
            match n {
                0 => F::ONE,
                1 => F::ZERO,
                2 => -F::ONE,
                3 => F::ZERO,
                _ => unreachable!(),
            }
        } else {
            match n {
                0 => F::cospi_reduced(y),
                1 => -F::sinpi_reduced(y),
                2 => -F::cospi_reduced(y),
                3 => F::sinpi_reduced(y),
                _ => unreachable!(),
            }
        }
    }
}

pub(crate) fn sinpi_cospi<F: Trigonometric>(x: F) -> (F, F) {
    if x.is_nan() || x.is_infinite() {
        // sind(NaN) = NaN
        // cosd(NaN) = NaN
        // sind(±inf) = NaN
        // cosd(±inf) = NaN
        (F::NAN, F::NAN)
    } else if x.is_zero() {
        // sind(±0) = ±0
        // cosd(±0) = 1
        (x, F::ONE)
    } else {
        let (n, y) = reduce_half_revs(x);
        if y.is_zero() {
            let sym_zero = F::ZERO.set_sign(x.sign());
            match n {
                0 => (sym_zero, F::ONE),
                1 => (F::ONE, F::ZERO),
                2 => (sym_zero, -F::ONE),
                3 => (-F::ONE, F::ZERO),
                _ => unreachable!(),
            }
        } else {
            let (sin, cos) = F::sinpi_cospi_reduced(y);
            let (sin, cos) = match n {
                0 => (sin, cos),
                1 => (cos, -sin),
                2 => (-sin, -cos),
                3 => (-cos, sin),
                _ => unreachable!(),
            };
            (sin, cos)
        }
    }
}

pub(crate) fn tanpi<F: Trigonometric>(x: F) -> F {
    if x.is_nan() || x.is_infinite() {
        // tand(NaN) = NaN
        // tand(±inf) = NaN
        F::NAN
    } else if x.is_zero() {
        // tand(±0) = ±0
        x
    } else {
        let (n, y) = reduce_half_revs(x);
        let inv = (n & 1) != 0;
        if y.is_zero() {
            if inv {
                F::INFINITY.set_sign(x.sign())
            } else {
                F::ZERO.set_sign(x.sign())
            }
        } else {
            F::tanpi_reduced(y, inv)
        }
    }
}
