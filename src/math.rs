//! Traits for mathematical operations on floating-point numbers.
//!
//! All operations produce results rounded to nearest with ties to even.
//! Additionally, [`Sqrt`] and [`Hypot`] also provide methods that allow
//! rounding to other directions.

use crate::{FpStatus, Round};

pub trait Sqrt: crate::Float {
    /// Calculates the square root of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns a quiet NaN if `self` is NaN or negative non-zero (including
    ///   infinity)
    fn sqrt(self) -> Self {
        self.sqrt_ex(Round::NearestTiesToEven).0
    }

    /// Calculates the square root of `self`, rounding according to `round`.
    fn sqrt_ex(self, round: Round) -> (Self, FpStatus);
}

pub trait Cbrt: crate::Float {
    /// Calculates the cube root of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns negative infinity if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn cbrt(self) -> Self;
}

pub trait Hypot: crate::Float {
    /// Calculates the Pythagorean addition of `self` and `other`, rounding to
    /// nearest with ties to even.
    ///
    /// The Pythagorean addition of `x` and `y` is equal to the length of the
    /// hypotenuse of a triangle with sides of length `x` and `y`.
    ///
    /// Special cases:
    /// * Returns positive infinity if `self` or `other` is infinity
    /// * Returns NaN if `self` or `other` is NaN and neither is infinity
    fn hypot(self, other: Self) -> Self {
        self.hypot_ex(other, Round::NearestTiesToEven).0
    }

    /// Calculates the Pythagorean addition of `self` and `other`, rounding
    /// according to `round`.
    fn hypot_ex(self, other: Self, round: Round) -> (Self, FpStatus);
}

pub trait Exp: crate::Float {
    /// Calculates Euler's number raised to `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns zero if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp(self) -> Self;

    /// Calculates `exp(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns minus one if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp_m1(self) -> Self;

    /// Calculates 2 raised to `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns zero if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp2(self) -> Self;

    /// Calculates `exp2(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns minus one if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp2_m1(self) -> Self;

    /// Calculates 10 raised to `self`, rounding to nearest with ties to even.
    ///
    /// Special cases:
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns zero if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp10(self) -> Self;

    /// Calculates `exp10(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns minus one if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn exp10_m1(self) -> Self;
}

pub trait Log: crate::Float {
    /// Calculates the natural logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative infinity if `self` is positive or negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or negative non-zero (including infinity)
    fn ln(self) -> Self;

    /// Calculates the natural logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns negative infinity if `self` is minus one
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or less than minus one (including
    ///   negative infinity)
    fn ln_1p(self) -> Self;

    /// Calculates the base-2 logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative infinity if `self` is positive or negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or negative non-zero (including infinity)
    fn log2(self) -> Self;

    /// Calculates the base-2 logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns negative infinity if `self` is minus one
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or less than minus one (including
    ///   negative infinity)
    fn log2_1p(self) -> Self;

    /// Calculates the base-10 logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative infinity if `self` is positive or negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or negative non-zero (including infinity)
    fn log10(self) -> Self;

    /// Calculates the base-10 logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns negative infinity if `self` is minus one
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns NaN if `self` is NaN or less than minus one (including
    ///   negative infinity)
    fn log10_1p(self) -> Self;
}

pub trait Pow: crate::Float {
    /// Calculates `x` (`self`) raised to `y` (`other`), rounding to nearest
    /// with ties to even.
    ///
    /// Special cases:
    /// * Returns 1 when `x` is 1 or `y` is zero
    /// * Returns 1 when `x` is -1 and `y` is positive or negative infinity
    /// * Returns NaN when `x` is NaN and `y` is not zero
    /// * Returns NaN when `y` is NaN and `z` is not 1
    /// * Returns positive zero when `x` is positive zero and `y` is positive
    /// * Returns positive zero when `x` is positive infinity and `y` is
    ///   negative
    /// * Returns positive infinity when `x` is positive infinity and `y` is
    ///   positive
    /// * Returns positive infinity when `x` is positive zero and `y` is
    ///   negative
    /// * Returns positive zero when `x` is negative zero and `y` is positive
    ///   and not an odd integer
    /// * Returns positive zero when `x` is negative infinity and `y` is
    ///   negative and not an odd integer
    /// * Returns positive infinity when `x` is negative infinity and `y` is
    ///   positive and not an odd integer
    /// * Returns positive infinity when `x` is negative zero and `y` is
    ///   negative and not an odd integer
    /// * Returns negative zero when `x` is negative zero and `y` is positive
    ///   and an odd integer
    /// * Returns negative zero when `x` is negative infinity and `y` is
    ///   negative and an odd integer
    /// * Returns negative infinity when `x` is negative infinity and `y` is
    ///   positive and an odd integer
    /// * Returns negative infinity when `x` is negative zero and `y` is
    ///   negative and an odd integer
    /// * Returns positive zero when the absolute value of `x` is less than one
    ///   and `y` is positive infinity
    /// * Returns positive zero when the absolute value of `x` is greater than
    ///   one and `y` is negative infinity
    /// * Returns positive infinity when the absolute value of `x` is less than
    ///   one and `y` is negative infinity
    /// * Returns positive infinity when the absolute value of `x` is greater
    ///   than one and `y` is positive infinity
    /// * Returns NaN when `x` is negative and `y` is finite and not integer
    fn pow(self, other: Self) -> Self;

    /// Calculates `x` (`self`) raised to `n`, rounding to nearest with ties to
    /// even.
    ///
    /// Special cases:
    /// * Returns 1 when `x` is 1 or `n` is zero
    /// * Returns NaN when `x` is NaN and `n` is not zero
    /// * Returns NaN when `n` is NaN and `z` is not 1
    /// * Returns positive zero when `x` is positive zero and `n` is positive
    /// * Returns positive zero when `x` is positive infinity and `n` is
    ///   negative
    /// * Returns positive infinity when `x` is positive infinity and `n` is
    ///   positive
    /// * Returns positive infinity when `x` is positive zero and `n` is
    ///   negative
    /// * Returns positive zero when `x` is negative zero and `n` is positive
    ///   and even
    /// * Returns positive zero when `x` is negative infinity and `n` is
    ///   negative and even
    /// * Returns positive infinity when `x` is negative infinity and `n` is
    ///   positive and even
    /// * Returns positive infinity when `x` is negative zero and `n` is
    ///   negative and even
    /// * Returns negative zero when `x` is negative zero and `n` is positive
    ///   and odd
    /// * Returns negative zero when `x` is negative infinity and `n` is
    ///   negative and odd
    /// * Returns negative infinity when `x` is negative infinity and `n` is
    ///   positive and odd
    /// * Returns negative infinity when `x` is negative zero and `n` is
    ///   negative and odd
    fn powi(self, n: i32) -> Self;
}

pub trait Trigonometric: crate::Float {
    /// Calculates the sine of `self` radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn sin(self) -> Self;

    /// Calculates the cosine of `self` radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is infinity or NaN
    fn cos(self) -> Self;

    /// Calculates the sine and the cosine of `self` radians, rounding to
    /// nearest.
    ///
    /// The same special cases of [`Trigonometric::sin`] and
    /// [`Trigonometric::cos`] also apply to this function. Using this function
    /// can be faster than using [`Trigonometric::sin`] and
    /// [`Trigonometric::cos`] separately.
    fn sin_cos(self) -> (Self, Self);

    /// Calculates the tangent of `self` radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn tan(self) -> Self;

    /// Calculates the sine of `self` degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn sind(self) -> Self;

    /// Calculates the cosine of `self` degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is infinity or NaN
    fn cosd(self) -> Self;

    /// Calculates the sine and the cosine of `self` degrees, rounding to
    /// nearest.
    ///
    /// The same special cases of [`Trigonometric::sind`] and
    /// [`Trigonometric::cosd`] also apply to this function. Using this function
    /// can be faster than using [`Trigonometric::sind`] and
    /// [`Trigonometric::cosd`] separately.
    fn sind_cosd(self) -> (Self, Self);

    /// Calculates the tangent of `self` degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn tand(self) -> Self;

    /// Calculates the sine of `self` half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn sinpi(self) -> Self;

    /// Calculates the cosine of `self` half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is infinity or NaN
    fn cospi(self) -> Self;

    /// Calculates the sine and the cosine of `self` half-revolutions, rounding
    /// to nearest.
    ///
    /// The same special cases of [`Trigonometric::sinpi`] and
    /// [`Trigonometric::cospi`] also apply to this function. Using this
    /// function can be faster than using [`Trigonometric::sinpi`] and
    /// [`Trigonometric::cospi`] separately.
    fn sinpi_cospi(self) -> (Self, Self);

    /// Calculates the tangent of `self` half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is infinity or NaN
    fn tanpi(self) -> Self;
}

pub trait InvTrigonometric: crate::Float {
    /// Calculates the arcsine of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn asin(self) -> Self;

    /// Calculates the arccosine of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn acos(self) -> Self;

    /// Calculates the arctangent of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN
    /// * Returns π/2 if `self` is positive infinity
    /// * Returns -π/2 if `self` is negative infinity
    fn atan(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in radians, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `x` is NaN or `y` is NaN
    /// * Returns positive zero if `y` is positive zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns negative zero if `y` is negative zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns π if `y` is positive zero `x` is positive (zero, finite or
    ///   infinity)
    /// * Returns -π if `y` is negative zero `x` is negative (zero, finite or
    ///   infinity)
    /// * Returns π/2 if `y` is positive infinity and `x` is zero or finite
    /// * Returns -π/2 if `y` is negative infinity and `x` is zero or finite
    /// * Returns positive zero if `x` is positive infinity and `y` is positive
    ///   (zero or finite)
    /// * Returns negative zero if `x` is positive infinity and `y` is negative
    ///   (zero or finite)
    /// * Returns π if `x` is negative infinity and `y` is positive (zero or
    ///   finite)
    /// * Returns -π if `x` is negative infinity and `y` is negative (zero or
    ///   finite)
    /// * Returns π/4 if `x` is positive infinity and `y` is positive infinity
    /// * Returns -π/4 if `x` is positive infinity and `y` is negative infinity
    /// * Returns 3π/4 if `x` is negative infinity and `y` is positive infinity
    /// * Returns -3π/4 if `x` is negative infinity and `y` is negative infinity
    fn atan2(self, other: Self) -> Self;

    /// Calculates the arcsine of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn asind(self) -> Self;

    /// Calculates the arccosine of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn acosd(self) -> Self;

    /// Calculates the arctangent of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN
    /// * Returns 90 if `self` is positive infinity
    /// * Returns -90 if `self` is negative infinity
    fn atand(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in degrees, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `x` is NaN or `y` is NaN
    /// * Returns positive zero if `y` is positive zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns negative zero if `y` is negative zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns 180 if `y` is positive zero `x` is positive (zero, finite or
    ///   infinity)
    /// * Returns -180 if `y` is negative zero `x` is negative (zero, finite or
    ///   infinity)
    /// * Returns 90 if `y` is positive infinity and `x` is zero or finite
    /// * Returns -90 if `y` is negative infinity and `x` is zero or finite
    /// * Returns positive zero if `x` is positive infinity and `y` is positive
    ///   (zero or finite)
    /// * Returns negative zero if `x` is positive infinity and `y` is negative
    ///   (zero or finite)
    /// * Returns 180 if `x` is negative infinity and `y` is positive (zero or
    ///   finite)
    /// * Returns -180 if `x` is negative infinity and `y` is negative (zero or
    ///   finite)
    /// * Returns 45 if `x` is positive infinity and `y` is positive infinity
    /// * Returns -45 if `x` is positive infinity and `y` is negative infinity
    /// * Returns 135 if `x` is negative infinity and `y` is positive infinity
    /// * Returns -135 if `x` is negative infinity and `y` is negative infinity
    fn atan2d(self, other: Self) -> Self;

    /// Calculates the arcsine of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn asinpi(self) -> Self;

    /// Calculates the arccosine of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or greater than one in magnitude
    ///   (including infinity)
    fn acospi(self) -> Self;

    /// Calculates the arctangent of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns NaN if `self` is NaN
    /// * Returns 0.5 if `self` is positive infinity
    /// * Returns -0.5 if `self` is negative infinity
    fn atanpi(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `x` is NaN or `y` is NaN
    /// * Returns positive zero if `y` is positive zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns negative zero if `y` is negative zero `x` is positive (zero,
    ///   finite or infinity)
    /// * Returns 1 if `y` is positive zero `x` is positive (zero, finite or
    ///   infinity)
    /// * Returns -1 if `y` is negative zero `x` is negative (zero, finite or
    ///   infinity)
    /// * Returns 0.5 if `y` is positive infinity and `x` is zero or finite
    /// * Returns -0.5 if `y` is negative infinity and `x` is zero or finite
    /// * Returns positive zero if `x` is positive infinity and `y` is positive
    ///   (zero or finite)
    /// * Returns negative zero if `x` is positive infinity and `y` is negative
    ///   (zero or finite)
    /// * Returns 1 if `x` is negative infinity and `y` is positive (zero or
    ///   finite)
    /// * Returns -1 if `x` is negative infinity and `y` is negative (zero or
    ///   finite)
    /// * Returns 0.25 if `x` is positive infinity and `y` is positive infinity
    /// * Returns -0.25 if `x` is positive infinity and `y` is negative infinity
    /// * Returns 0.75 if `x` is negative infinity and `y` is positive infinity
    /// * Returns -0.75 if `x` is negative infinity and `y` is negative infinity
    fn atan2pi(self, other: Self) -> Self;
}

pub trait Hyperbolic: crate::Float {
    /// Calculates the hyperbolic sine of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns negative infinity if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn sinh(self) -> Self;

    /// Calculates the hyperbolic cosine of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns positive infinity if `self` is positive or negative infinity
    /// * Returns NaN if `self` is NaN
    fn cosh(self) -> Self;

    /// Calculates the hyperbolic sine and hyperbolic cosine of `self`, rounding
    /// to nearest.
    ///
    /// The special cases of [`Hyperbolic::sinh`] and [`Hyperbolic::cosh`] also
    /// apply to this function. Using this function can be faster than using
    /// [`Hyperbolic::sinh`] and [`Hyperbolic::cosh`] separately.
    fn sinh_cosh(self) -> (Self, Self);

    /// Calculates the hyperbolic tangent of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns negative zero if `self` is negative zero
    /// * Returns one if `self` is positive infinity
    /// * Returns minus one if `self` is negative infinity
    /// * Returns NaN if `self` is NaN
    fn tanh(self) -> Self;
}

pub trait InvHyperbolic: crate::Float {
    /// Calculates the hyperbolic arcsine of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns negative infinity if `self` is negative infinity
    fn asinh(self) -> Self;

    /// Calculates the hyperbolic arccosine of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or less than one (including negative
    ///   infinity)
    /// * Returns positive infinity if `self` is positive infinity
    fn acosh(self) -> Self;

    /// Calculates the hyperbolic arctangent of `self`, rounding to nearest.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or greater than 1 in magnitude
    /// * Returns negative zero if `self` is negative zero
    /// * Returns positive infinity if `self` is 1
    /// * Returns negative infinity if `self` is -1
    fn atanh(self) -> Self;
}

pub trait Gamma: crate::Float {
    /// Calculates the gamma function of `self`, rounding to nearest with ties
    /// to even.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN, negative infinity or a negative integer
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns positive infinity if `self` is positive zero
    /// * Returns negative infinity if `self` is negative zero
    fn gamma(self) -> Self;

    /// Calculates the logarithm of the absolute value of the gamma function of
    /// `self`, rounding to nearest with ties to even.
    ///
    /// The integer field of the returned tuple is `1` when the gamma function
    /// of `self` is positive, `-1` when the gamma function of `self` is
    /// negative, and `0` when the sign of the gamma function of `self` is not
    /// defined.
    ///
    /// Special cases:
    /// * Returns NaN if `self` is NaN or negative infinity
    /// * Returns positive infinity if `self` is positive infinity
    /// * Returns positive infinity if `self` is zero or a negative integer
    ///
    /// The sign is considered undefined when `self` is NaN, a negative integer
    /// or negative infinity.
    fn ln_gamma(self) -> (Self, i8);
}
