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
    ///
    /// | `self`             | Result |
    /// |--------------------|--------|
    /// | -0                 | -0     |
    /// | +∞                 | +∞     |
    /// | < 0 (including -∞) | NaN    |
    /// | NaN                | NaN    |
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
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -∞     |
    /// | NaN    | NaN    |
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
    ///
    /// | `self`              | `other`             | Result |
    /// |---------------------|---------------------|--------|
    /// | ±∞                  | any (including NaN) | +∞     |
    /// | any (including NaN) | ±∞                  | +∞     |
    /// | NaN                 | not ±∞              | NaN    |
    /// | not ±∞              | NaN                 | NaN    |
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
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | +∞     | +∞     |
    /// | -∞     | +0     |
    /// | NaN    | NaN    |
    fn exp(self) -> Self;

    /// Calculates `exp(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -1     |
    /// | NaN    | NaN    |
    fn exp_m1(self) -> Self;

    /// Calculates 2 raised to `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | +∞     | +∞     |
    /// | -∞     | +0     |
    /// | NaN    | NaN    |
    fn exp2(self) -> Self;

    /// Calculates `exp2(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -1     |
    /// | NaN    | NaN    |
    fn exp2_m1(self) -> Self;

    /// Calculates 10 raised to `self`, rounding to nearest with ties to even.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | +∞     | +∞     |
    /// | -∞     | +0     |
    /// | NaN    | NaN    |
    fn exp10(self) -> Self;

    /// Calculates `exp10(self) - 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -1     |
    /// | NaN    | NaN    |
    fn exp10_m1(self) -> Self;
}

pub trait Log: crate::Float {
    /// Calculates the natural logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`             | Result |
    /// |--------------------|--------|
    /// | ±0                 | -∞     |
    /// | +∞                 | +∞     |
    /// | < 0 (including -∞) | NaN    |
    /// | NaN                | NaN    |
    fn ln(self) -> Self;

    /// Calculates the natural logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`              | Result |
    /// |---------------------|--------|
    /// | -0                  | -0     |
    /// | -1                  | -∞     |
    /// | +∞                  | +∞     |
    /// | < -1 (including -∞) | NaN    |
    /// | NaN                 | NaN    |
    fn ln_1p(self) -> Self;

    /// Calculates the base-2 logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`             | Result |
    /// |--------------------|--------|
    /// | ±0                 | -∞     |
    /// | +∞                 | +∞     |
    /// | < 0 (including -∞) | NaN    |
    /// | NaN                | NaN    |
    fn log2(self) -> Self;

    /// Calculates the base-2 logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`              | Result |
    /// |---------------------|--------|
    /// | -0                  | -0     |
    /// | -1                  | -∞     |
    /// | +∞                  | +∞     |
    /// | < -1 (including -∞) | NaN    |
    /// | NaN                 | NaN    |
    fn log2_1p(self) -> Self;

    /// Calculates the base-10 logarithm of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`             | Result |
    /// |--------------------|--------|
    /// | ±0                 | -∞     |
    /// | +∞                 | +∞     |
    /// | < 0 (including -∞) | NaN    |
    /// | NaN                | NaN    |
    fn log10(self) -> Self;

    /// Calculates the base-10 logarithm of `self + 1`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`              | Result |
    /// |---------------------|--------|
    /// | -0                  | -0     |
    /// | -1                  | -∞     |
    /// | +∞                  | +∞     |
    /// | < -1 (including -∞) | NaN    |
    /// | NaN                 | NaN    |
    fn log10_1p(self) -> Self;
}

pub trait Pow: crate::Float {
    /// Calculates `x` (`self`) raised to `y` (`other`), rounding to nearest
    /// with ties to even.
    ///
    /// Special cases:
    ///
    /// | `x`                 | `y`                     | Result |
    /// |---------------------|-------------------------|--------|
    /// | 1                   | any (including NaN)     | 1      |
    /// | any (including NaN) | ±0                      | 1      |
    /// | -1                  | ±∞                      | 1      |
    /// | NaN                 | not ±0                  | NaN    |
    /// | not 1               | NaN                     | NaN    |
    /// | +0                  | > 0                     | +0     |
    /// | +0                  | < 0                     | +∞     |
    /// | -0                  | > 0, odd integer        | -0     |
    /// | -0                  | > 0, not an odd integer | +0     |
    /// | -0                  | < 0, odd integer        | -∞     |
    /// | -0                  | < 0, not an odd integer | +∞     |
    /// | +∞                  | > 0                     | +∞     |
    /// | +∞                  | < 0                     | +0     |
    /// | -∞                  | > 0, odd integer        | -∞     |
    /// | -∞                  | > 0, not an odd integer | +∞     |
    /// | -∞                  | < 0, odd integer        | -0     |
    /// | -∞                  | < 0, not an odd integer | +0     |
    /// | > -1 and < 1        | +∞                      | +0     |
    /// | > -1 and < 1        | -∞                      | +∞     |
    /// | < -1 or > 1         | +∞                      | +∞     |
    /// | < -1 or > 1         | -∞                      | +0     |
    /// | < 0, finite         | finite, not an integer  | NaN    |
    fn pow(self, other: Self) -> Self;

    /// Calculates `x` (`self`) raised to `n`, rounding to nearest with ties to
    /// even.
    ///
    /// Special cases:
    ///
    /// | `x`                 | `n`       | Result |
    /// |---------------------|-----------|--------|
    /// | 1                   | any       | 1      |
    /// | any (including NaN) | 0         | 1      |
    /// | NaN                 | not 0     | NaN    |
    /// | +0                  | > 0       | +0     |
    /// | +0                  | < 0       | +∞     |
    /// | -0                  | > 0, odd  | -0     |
    /// | -0                  | > 0, even | +0     |
    /// | -0                  | < 0, odd  | -∞     |
    /// | -0                  | < 0, even | +∞     |
    /// | +∞                  | > 0       | +∞     |
    /// | +∞                  | < 0       | +0     |
    /// | -∞                  | > 0, odd  | -∞     |
    /// | -∞                  | > 0, even | +∞     |
    /// | -∞                  | < 0, odd  | -0     |
    /// | -∞                  | < 0, even | +0     |
    fn powi(self, n: i32) -> Self;
}

pub trait Trigonometric: crate::Float {
    /// Calculates the sine of `self` radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
    fn sin(self) -> Self;

    /// Calculates the cosine of `self` radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
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
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
    fn tan(self) -> Self;

    /// Calculates the sine of `self` degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
    fn sind(self) -> Self;

    /// Calculates the cosine of `self` degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
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
    ///
    /// | `self`                 | Result |
    /// |------------------------|--------|
    /// | -0                     | -0     |
    /// | 90 + 360k (integer k)  | +∞     |
    /// | -90 + 360k (integer k) | -∞     |
    /// | ±∞                     | NaN    |
    /// | NaN                    | NaN    |
    fn tand(self) -> Self;

    /// Calculates the sine of `self` half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
    fn sinpi(self) -> Self;

    /// Calculates the cosine of `self` half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | ±∞     | NaN    |
    /// | NaN    | NaN    |
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
    ///
    /// | `self`                | Result |
    /// |-----------------------|--------|
    /// | -0                    | -0     |
    /// | 0.5 + 2k (integer k)  | +∞     |
    /// | -0.5 + 2k (integer k) | -∞     |
    /// | ±∞                    | NaN    |
    /// | NaN                   | NaN    |
    fn tanpi(self) -> Self;
}

pub trait InvTrigonometric: crate::Float {
    /// Calculates the arcsine of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | -0                         | -0     |
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn asin(self) -> Self;

    /// Calculates the arccosine of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn acos(self) -> Self;

    /// Calculates the arctangent of `self` in radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | π/2    |
    /// | -∞     | -π/2   |
    /// | NaN    | NaN    |
    fn atan(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in radians, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `y`         | `x`                      | Result |
    /// |-------------|--------------------------|--------|
    /// | NaN         | any                      | NaN    |
    /// | any         | NaN                      | NaN    |
    /// | +0          | +0 or > 0 (including +∞) | +0     |
    /// | -0          | +0 or > 0 (including +∞) | -0     |
    /// | +0          | -0 or < 0 (including -∞) | π      |
    /// | -0          | -0 or < 0 (including -∞) | -π     |
    /// | > 0, finite | +∞                       | +0     |
    /// | < 0, finite | +∞                       | -0     |
    /// | > 0, finite | -∞                       | π      |
    /// | < 0, finite | -∞                       | -π     |
    /// | > 0, finite | ±0                       | π/2    |
    /// | < 0, finite | ±0                       | -π/2   |
    /// | +∞          | finite (including ±0)    | π/2    |
    /// | -∞          | finite (including ±0)    | -π/2   |
    /// | +∞          | +∞                       | π/4    |
    /// | -∞          | +∞                       | -π/4   |
    /// | +∞          | -∞                       | 3π/4   |
    /// | -∞          | -∞                       | -3π/4  |
    fn atan2(self, other: Self) -> Self;

    /// Calculates the arcsine of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | -0                         | -0     |
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn asind(self) -> Self;

    /// Calculates the arccosine of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn acosd(self) -> Self;

    /// Calculates the arctangent of `self` in degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | 90     |
    /// | -∞     | -90    |
    /// | NaN    | NaN    |
    fn atand(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in degrees, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `y`         | `x`                      | Result |
    /// |-------------|--------------------------|--------|
    /// | NaN         | any                      | NaN    |
    /// | any         | NaN                      | NaN    |
    /// | +0          | +0 or > 0 (including +∞) | +0     |
    /// | -0          | +0 or > 0 (including +∞) | -0     |
    /// | +0          | -0 or < 0 (including -∞) | 180    |
    /// | -0          | -0 or < 0 (including -∞) | -180   |
    /// | > 0, finite | +∞                       | +0     |
    /// | < 0, finite | +∞                       | -0     |
    /// | > 0, finite | -∞                       | 180    |
    /// | < 0, finite | -∞                       | -180   |
    /// | > 0, finite | ±0                       | 90     |
    /// | < 0, finite | ±0                       | -90    |
    /// | +∞          | finite (including ±0)    | 90     |
    /// | -∞          | finite (including ±0)    | -90    |
    /// | +∞          | +∞                       | 45     |
    /// | -∞          | +∞                       | -45    |
    /// | +∞          | -∞                       | 135    |
    /// | -∞          | -∞                       | -135   |
    fn atan2d(self, other: Self) -> Self;

    /// Calculates the arcsine of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | -0                         | -0     |
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn asinpi(self) -> Self;

    /// Calculates the arccosine of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn acospi(self) -> Self;

    /// Calculates the arctangent of `self` in half-revolutions, rounding to
    /// nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | 0.5    |
    /// | -∞     | -0.5   |
    /// | NaN    | NaN    |
    fn atanpi(self) -> Self;

    /// Calculates the 2-argument arctangent of `y` (`self`) and `x` (`other`)
    /// in half-revolutions, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `y`         | `x`                      | Result |
    /// |-------------|--------------------------|--------|
    /// | NaN         | any                      | NaN    |
    /// | any         | NaN                      | NaN    |
    /// | +0          | +0 or > 0 (including +∞) | +0     |
    /// | -0          | +0 or > 0 (including +∞) | -0     |
    /// | +0          | -0 or < 0 (including -∞) | 1      |
    /// | -0          | -0 or < 0 (including -∞) | -1     |
    /// | > 0, finite | +∞                       | +0     |
    /// | < 0, finite | +∞                       | -0     |
    /// | > 0, finite | -∞                       | 1      |
    /// | < 0, finite | -∞                       | -1     |
    /// | > 0, finite | ±0                       | 0.5    |
    /// | < 0, finite | ±0                       | -0.5   |
    /// | +∞          | finite (including ±0)    | 0.5    |
    /// | -∞          | finite (including ±0)    | -0.5   |
    /// | +∞          | +∞                       | 0.25   |
    /// | -∞          | +∞                       | -0.25  |
    /// | +∞          | -∞                       | 0.75   |
    /// | -∞          | -∞                       | -0.75  |
    fn atan2pi(self, other: Self) -> Self;
}

pub trait Hyperbolic: crate::Float {
    /// Calculates the hyperbolic sine of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -∞     |
    /// | NaN    | NaN    |
    fn sinh(self) -> Self;

    /// Calculates the hyperbolic cosine of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | ±∞     | +∞     |
    /// | NaN    | NaN    |
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
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | 1      |
    /// | -∞     | -1     |
    /// | NaN    | NaN    |
    fn tanh(self) -> Self;
}

pub trait InvHyperbolic: crate::Float {
    /// Calculates the hyperbolic arcsine of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self` | Result |
    /// |--------|--------|
    /// | -0     | -0     |
    /// | +∞     | +∞     |
    /// | -∞     | -∞     |
    /// | NaN    | NaN    |
    fn asinh(self) -> Self;

    /// Calculates the hyperbolic arccosine of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`             | Result |
    /// |--------------------|--------|
    /// | +∞                 | +∞     |
    /// | < 1 (including -∞) | NaN    |
    /// | NaN                | NaN    |
    fn acosh(self) -> Self;

    /// Calculates the hyperbolic arctangent of `self`, rounding to nearest.
    ///
    /// Special cases:
    ///
    /// | `self`                     | Result |
    /// |----------------------------|--------|
    /// | -0                         | -0     |
    /// | 1                          | +∞     |
    /// | -1                         | -∞     |
    /// | < -1 or > 1 (including ±∞) | NaN    |
    /// | NaN                        | NaN    |
    fn atanh(self) -> Self;
}

pub trait Gamma: crate::Float {
    /// Calculates the gamma function of `self`, rounding to nearest with ties
    /// to even.
    ///
    /// When `self` is negative and not an integer, the sign of the gamma
    /// function alternates between consecutive integers, and results that
    /// underflow to zero keep that sign. For example, `gamma(-180.5)` returns
    /// negative zero and `gamma(-181.5)` returns positive zero.
    ///
    /// Special cases:
    ///
    /// | `self`       | Result |
    /// |--------------|--------|
    /// | +0           | +∞     |
    /// | -0           | -∞     |
    /// | +∞           | +∞     |
    /// | -∞           | NaN    |
    /// | < 0, integer | NaN    |
    /// | NaN          | NaN    |
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
    ///
    /// | `self`       | Result | Sign |
    /// |--------------|--------|------|
    /// | +0           | +∞     | 1    |
    /// | -0           | +∞     | -1   |
    /// | +∞           | +∞     | 1    |
    /// | -∞           | NaN    | 0    |
    /// | < 0, integer | +∞     | 0    |
    /// | NaN          | NaN    | 0    |
    fn ln_gamma(self) -> (Self, i8);
}
