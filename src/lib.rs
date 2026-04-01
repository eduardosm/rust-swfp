//! A library for soft-float arithmetic and math.
//!
//! This crate is `no_std`.
//!
//! The following floating-point types are provided:
//!
//! - [`F16`] (IEEE 754 half precision)
//! - [`F32`] (IEEE 754 single precision)
//! - [`F64`] (IEEE 754 double precision)
//! - [`F128`] (IEEE 754 quadruple precision)
//! - [`X87F80`] (X87 80-bit extended precision)
//! - [`F8E5M2`]
//! - [`F8E4M3B8Nnz`]
//! - [`F8E4M3Nao`]
//!
//! Arithmetic operations are provided for all the above types, with support for
//! different rounding modes and status flags. Operations are provided through
//! the [`Float`] trait, as well as standard operators (`+`, `-`, `*`, `/`, `%`)
//! that round to nearest with ties to even.
//!
//! Math operations are provided for [`F16`], [`F32`] and [`F64`] through the
//! traits in the [`math`] module.

#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications,
    clippy::float_arithmetic
)]
#![forbid(unsafe_code)]
#![no_std]

#[cfg(test)]
extern crate std;

macro_rules! horner {
    ($outer_x:ident, $inner_x:ident, [$coef:expr]) => {
        $outer_x * $coef
    };
    ($outer_x:ident, $inner_x:ident, [$coef0:expr, $($coefs:expr),+]) => {
        $outer_x * ($coef0 + horner!($inner_x, $inner_x, [$($coefs),+]))
    };
}

mod core_num;
mod f128;
mod f16;
mod f32;
mod f64;
mod f8e4m3b8nnz;
mod f8e4m3nao;
mod f8e5m2;
mod generic;
mod ieee_float;
mod int;
pub mod math;
mod math_aux;
mod simple_fp;
mod traits;
mod uint192;
mod utils;
mod x87f80;

pub use f8e4m3b8nnz::F8E4M3B8Nnz;
pub use f8e4m3nao::F8E4M3Nao;
pub use f8e5m2::F8E5M2;
pub use f16::F16;
pub use f32::F32;
pub use f64::F64;
pub use f128::F128;
pub use x87f80::X87F80;

/// Specifies the rounding direction for floating-point operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Round {
    /// Round to nearest, with ties rounding to the even number.
    NearestTiesToEven,
    /// Round to nearest, with ties rounding away from zero.
    NearestTiesToAway,
    /// Round towards positive infinity.
    TowardPositive,
    /// Round towards negative infinity.
    TowardNegative,
    /// Round towards zero.
    TowardZero,
}

/// Indicates the status of a floating-point operation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FpStatus {
    /// The operation is valid and the result is exact.
    Ok,
    /// The operation is not valid or an operand is a signaling NaN.
    Invalid,
    /// The result is larger than the maximum representable value.
    Overflow,
    /// The result result is not exact and has been rounded into zero or a
    /// subnormal value.
    Underflow,
    /// The result is not exact and has been rounded into a normal value.
    Inexact,
    /// Attempted to use a zero divisor.
    DivByZero,
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for crate::F16 {}
    impl Sealed for crate::F32 {}
    impl Sealed for crate::F64 {}
    impl Sealed for crate::F128 {}
    impl Sealed for crate::X87F80 {}
    impl Sealed for crate::F8E5M2 {}
    impl Sealed for crate::F8E4M3B8Nnz {}
    impl Sealed for crate::F8E4M3Nao {}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseFloatError {
    kind: ParseFloatErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ParseFloatErrorKind {
    Empty,
    Invalid,
}

impl ParseFloatError {
    #[inline]
    fn mk_empty() -> Self {
        Self {
            kind: ParseFloatErrorKind::Empty,
        }
    }

    #[inline]
    fn mk_invalid() -> Self {
        Self {
            kind: ParseFloatErrorKind::Invalid,
        }
    }
}

impl core::error::Error for ParseFloatError {}

impl core::fmt::Display for ParseFloatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ParseFloatErrorKind::Empty => write!(f, "cannot parse float from empty string"),
            ParseFloatErrorKind::Invalid => write!(f, "invalid float literal"),
        }
    }
}

/// Trait that represents provides operations on soft-floating-point of this
/// crate.
///
/// This trait is sealed, so it cannot be implemented for types outside this
/// crate.
pub trait Float:
    sealed::Sealed
    + Copy
    + PartialOrd
    + Default
    + core::fmt::Debug
    + core::str::FromStr<Err: core::fmt::Debug + core::fmt::Display>
    + core::ops::Neg<Output = Self>
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + core::ops::Rem<Output = Self>
{
    /// Type of the raw bit representation of the floating-point number.
    type Bits: Copy + Ord;

    /// Zero value with positive sign.
    const ZERO: Self;

    /// NaN value.
    const NAN: Self;

    /// Infinity value with positive sign.
    const INFINITY: Self;

    // Creates a floating-point number from its binary representation.
    fn from_bits(bits: Self::Bits) -> Self;

    /// Returns the raw bit representation of the floating-point number.
    fn to_bits(self) -> Self::Bits;

    /// Converts an unsigned integer to a floating-point number rounding to
    /// nearest with ties to even.
    #[inline]
    fn from_uint(value: u128) -> Self {
        Self::from_uint_ex(value, Round::NearestTiesToEven).0
    }

    /// Converts an unsigned integer to a floating-point number rounding
    /// according to `round`.
    fn from_uint_ex(value: u128, round: Round) -> (Self, FpStatus);

    /// Converts a signed integer to a floating-point number rounding to nearest
    /// with ties to even.
    #[inline]
    fn from_int(value: i128) -> Self {
        Self::from_int_ex(value, Round::NearestTiesToEven).0
    }

    /// Converts a signed integer to a floating-point number rounding according
    /// to `round`.
    fn from_int_ex(value: i128, round: Round) -> (Self, FpStatus);

    /// Converts `self` to a `bits`-bit unsigned integer rounding towards zero.
    ///
    /// Returns `None` if `self` is NaN, infinity or out of range.
    #[inline]
    fn to_uint(self, bits: u32) -> Option<u128> {
        self.to_uint_ex(bits, Round::TowardZero).0
    }

    /// Converts `self` to a `bits`-bit unsigned integer rounding according to
    /// `round`.
    ///
    /// Returns `None` if `self` is NaN or out of range.
    ///
    /// A status of [`FpStatus::Invalid`] is also returned if `self` is a quiet
    /// NaN.
    fn to_uint_ex(self, bits: u32, round: Round) -> (Option<u128>, FpStatus);

    /// Converts `self` to a `bits`-bit signed integer rounding towards zero.
    ///
    /// Returns `None` if `self` is NaN or out of range.
    #[inline]
    fn to_int(self, bits: u32) -> Option<i128> {
        self.to_int_ex(bits, Round::TowardZero).0
    }

    /// Converts `self` to a `bits`-bit signed integer rounding according to
    /// `round`.
    ///
    /// Returns `None` if `self` is NaN or out of range.
    ///
    /// A status of [`FpStatus::Invalid`] is also returned if `self` is a quiet
    /// NaN.
    fn to_int_ex(self, bits: u32, round: Round) -> (Option<i128>, FpStatus);

    /// Parses a floating-point number from a base-10 string, rounding according
    /// to `round`.
    ///
    /// The following EBNF grammar is accepted (case insensitive):
    ///
    /// ```text
    /// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
    /// Number ::= ( Digit+ |
    ///     Digit+ '.' Digit* |
    ///     Digit* '.' Digit+ ) Exp?
    /// Exp    ::= 'e' Sign? Digit+
    /// Sign   ::= [+-]
    /// Digit  ::= [0-9]
    /// ```
    fn from_str_ex(s: &str, round: Round) -> Result<(Self, FpStatus), ParseFloatError>;

    /// Returns whether `self` is NaN.
    fn is_nan(self) -> bool;

    /// Returns whether `self` is infinite.
    fn is_infinite(self) -> bool;

    /// Returns whether `self` is finite.
    fn is_finite(self) -> bool;

    /// Returns true if `self` is subnormal.
    fn is_subnormal(self) -> bool;

    /// Returns `true` if `self` is neither zero, infinite, subnormal, or NaN.
    fn is_normal(self) -> bool;

    /// Returns the category of this floating-point number.
    fn classify(self) -> core::num::FpCategory;

    /// Returns `true` if `self` has a positive sign, including `+0.0`, NaNs
    /// with positive sign bit and positive infinity.
    fn is_sign_positive(self) -> bool;

    /// Returns `true` if `self` has a negative sign, including `-0.0`, NaNs
    /// with negative sign bit and negative infinity.
    fn is_sign_negative(self) -> bool;

    /// Calculates the absolute value of `self`.
    fn abs(self) -> Self;

    /// Returns a value with the magnitude of `self` and the sign of `sign`.
    fn copysign(self, sign: Self) -> Self;

    /// Rounds `self` to to an integer, according to `round`.
    fn round_int_ex(self, round: Round) -> (Self, FpStatus);

    /// Rounds `self` to the nearest integer, ties round to even.
    #[inline]
    fn round_ties_even(self) -> Self {
        self.round_int_ex(Round::NearestTiesToEven).0
    }

    /// Rounds `self` to the nearest integer, ties round away from zero.
    #[inline]
    fn round_ties_away(self) -> Self {
        self.round_int_ex(Round::NearestTiesToAway).0
    }

    /// Rounds `self` to the nearest integer that is not greater than `self`.
    #[inline]
    fn floor(self) -> Self {
        self.round_int_ex(Round::TowardNegative).0
    }

    /// Rounds `self` to the nearest integer that is not less than `self`.
    #[inline]
    fn ceil(self) -> Self {
        self.round_int_ex(Round::TowardPositive).0
    }

    /// Rounds `self` to the nearest integer that is not greater in magnitude
    /// than `self`.
    #[inline]
    fn trunc(self) -> Self {
        self.round_int_ex(Round::TowardZero).0
    }

    /// Calculates `self * 2^exp`.
    fn scalbn(self, exp: i32) -> Self {
        self.scalbn_ex(exp, Round::NearestTiesToEven).0
    }

    /// Calculates `self * 2^exp`, rounding according to `round`.
    fn scalbn_ex(self, exp: i32, round: Round) -> (Self, FpStatus);

    /// Splits `self` into mantissa and exponent.
    ///
    /// Returns `(m, e)` such as:
    /// * `0.5 <= m < 1.0`
    /// * `self = m * 2^e`
    ///
    /// When `self` is zero, infinity or NaN, returns `self` as mantissa and
    /// zero as exponent.
    fn frexp(self) -> (Self, i32);

    /// Adds `self` and `rhs`, rounding according to `round`.
    fn add_ex(self, rhs: Self, round: Round) -> (Self, FpStatus);

    /// Subtracts `rhs` from `self`, rounding according to `round`.
    fn sub_ex(self, rhs: Self, round: Round) -> (Self, FpStatus);

    /// Multiplies `self` and `rhs`, rounding according to `round`.
    fn mul_ex(self, rhs: Self, round: Round) -> (Self, FpStatus);

    /// Divides `self` by `rhs`, rounding according to `round`.
    fn div_ex(self, rhs: Self, round: Round) -> (Self, FpStatus);

    /// Calculates the remainder of `self` divided by `rhs`.
    fn rem_ex(self, rhs: Self) -> (Self, FpStatus);
}

pub trait FloatConvertFrom<T: Float>: Float {
    /// Converts `value`, of type `T`, to `Self`, rounding to nearest with ties
    /// to even.
    #[inline]
    fn convert_from(value: T) -> Self {
        Self::convert_from_ex(value, Round::NearestTiesToEven).0
    }

    /// Converts `value`, of type `T`, to `Self`, rounding according to `round`.
    fn convert_from_ex(value: T, round: Round) -> (Self, FpStatus);
}

pub trait FloatConvertTo<T: Float>: Float {
    /// Converts `self` to `T`, rounding to nearest with ties to even.
    #[inline]
    fn convert_to(self) -> T {
        self.convert_to_ex(Round::NearestTiesToEven).0
    }

    /// Converts `self` to `T`, rounding according to `round`.
    fn convert_to_ex(self, round: Round) -> (T, FpStatus);
}

impl<F1: Float, F2: Float + FloatConvertFrom<F1>> FloatConvertTo<F2> for F1 {
    #[inline]
    fn convert_to_ex(self, round: Round) -> (F2, FpStatus) {
        F2::convert_from_ex(self, round)
    }
}
