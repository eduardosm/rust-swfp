use crate::traits::{CastFrom as _, CastInto as _, FloatImpExt, Int as _};

pub(crate) trait Pow: FloatImpExt {
    fn pow_finite(x: Self, y: Self, sign: bool) -> Self;

    fn powi_finite(x: Self, y: i32) -> Self;
}

pub(crate) fn pow<F: Pow>(x: F, y: F) -> F {
    if y.is_zero() || x.bitwise_eq(F::ONE) {
        // pow(x, 0) = 1
        // pow(1, y) = 1
        F::ONE
    } else if x.is_nan() || y.is_nan() {
        // pow(NaN, y) = NaN when y != 0
        F::NAN
    } else if x.is_zero() {
        // x = ±0
        if y.is_odd_int() {
            // y is an odd integer
            if y.sign() {
                // pow(±0, y) = ±inf when y < 0
                F::INFINITY.set_sign(x.sign())
            } else {
                // pow(±0, y) = ±0 when y > 0
                x
            }
        } else {
            // y is not an odd integer
            if y.sign() {
                // pow(±0, y) = +inf when y < 0
                F::INFINITY
            } else {
                // pow(±0, y) = 0 when y > 0
                F::ZERO
            }
        }
    } else if x.is_infinite() {
        // x = ±inf
        if x.sign() {
            // x = -inf
            if y.is_odd_int() {
                // y is an odd integer
                if y.sign() {
                    // pow(-inf, y) = -0 when y < 0
                    -F::ZERO
                } else {
                    // pow(-inf, y) = -inf when y > 0
                    -F::INFINITY
                }
            } else {
                // y is not an odd integer
                if y.sign() {
                    // pow(-inf, y) = -0 when y < 0
                    F::ZERO
                } else {
                    // pow(-inf, y) = inf when y > 0
                    F::INFINITY
                }
            }
        } else {
            // x = +inf
            if y.sign() {
                // pow(+inf, y) = 0 when y < 0
                F::ZERO
            } else {
                // pow(+inf, y) = +inf when y > 0
                F::INFINITY
            }
        }
    } else if y.is_infinite() {
        // y = ±inf
        if x == -F::ONE {
            // pow(-1, ±inf) = 1
            F::ONE
        } else if y.sign() {
            // y = -inf
            if x.exponent() < F::Exp::ZERO {
                // pow(x, -inf) = inf when |x| < 1
                F::INFINITY
            } else {
                // pow(x, -inf) = 0 when |x| > 1
                F::ZERO
            }
        } else {
            // y = +inf
            if x.exponent() < F::Exp::ZERO {
                // pow(x, +inf) = 0 when |x| < 1
                F::ZERO
            } else {
                // pow(x, +inf) = inf when |x| > 1
                F::INFINITY
            }
        }
    } else if x.sign() && !y.is_int() {
        // pow(x, y) = NaN when x < 0 and y is finite and not integer
        F::NAN
    } else {
        F::pow_finite(x, y, x.sign() && int_is_odd(y))
    }
}

pub(crate) fn powi<F: Pow>(x: F, y: i32) -> F {
    if y == 0 || x.bitwise_eq(F::ONE) {
        // pow(x, 0) = 1
        // pow(1, y) = 1
        F::ONE
    } else if x.is_nan() {
        // pow(NaN, y) = NaN when y != 0
        F::NAN
    } else if x.is_zero() {
        // x = ±0
        if (y & 1) != 0 {
            // y is odd
            if y < 0 {
                // pow(±0, y) = ±inf when y < 0
                F::INFINITY.set_sign(x.sign())
            } else {
                // pow(±0, y) = ±0 when y > 0
                x
            }
        } else {
            // y is even
            if y < 0 {
                // pow(±0, y) = +inf when y < 0
                F::INFINITY
            } else {
                // pow(±0, y) = 0 when y > 0
                F::ZERO
            }
        }
    } else if x.is_infinite() {
        // x = ±inf
        if x.sign() {
            // x = -inf
            if (y & 1) != 0 {
                // y is odd
                if y < 0 {
                    // pow(-inf, y) = -0 when y < 0
                    -F::ZERO
                } else {
                    // pow(-inf, y) = -inf when y > 0
                    -F::INFINITY
                }
            } else {
                // y is even
                if y < 0 {
                    // pow(-inf, y) = -0 when y < 0
                    F::ZERO
                } else {
                    // pow(-inf, y) = inf when y > 0
                    F::INFINITY
                }
            }
        } else {
            // x = +inf
            if y < 0 {
                // pow(+inf, y) = 0 when y < 0
                F::ZERO
            } else {
                // pow(+inf, y) = +inf when y > 0
                F::INFINITY
            }
        }
    } else {
        F::powi_finite(x, y)
    }
}

// assumes `x` is a non-zero integer
fn int_is_odd<F: FloatImpExt>(x: F) -> bool {
    let e = x.exponent();
    if e > F::Exp::cast_from(F::MANT_BITS) {
        false
    } else {
        let frac_shift = F::Exp::cast_from(F::MANT_BITS) - e;
        let frac_shift: u32 = frac_shift.cast_into();
        ((x.mant() >> frac_shift) & F::Bits::ONE) == F::Bits::ONE
    }
}
