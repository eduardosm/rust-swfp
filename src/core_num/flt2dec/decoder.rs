//! Decodes a floating-point value into individual parts and error ranges.

/// Decoded unsigned finite value, such that:
///
/// - The original value equals to `mant * 2^exp`.
///
/// - Any number from `(mant - minus) * 2^exp` to `(mant + plus) * 2^exp` will
///   round to the original value. The range is inclusive only when
///   `inclusive` is `true`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Decoded {
    /// The scaled mantissa.
    pub mant: u128,
    /// The lower error range.
    pub minus: u64,
    /// The upper error range.
    pub plus: u64,
    /// The shared exponent in base 2.
    pub exp: i16,
    /// True when the error range is inclusive.
    ///
    /// In IEEE 754, this is true when the original mantissa was even.
    pub inclusive: bool,
}

/// Decoded unsigned value.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FullDecoded {
    /// Not-a-number.
    Nan,
    /// Infinities, either positive or negative.
    Infinite,
    /// Zero, either positive or negative.
    Zero,
    /// Finite numbers with further decoded fields.
    Finite(Decoded),
}

pub(crate) trait DecodableFloat: Copy {
    const IS_LARGE: bool;

    fn decode(self) -> (/*negative?*/ bool, FullDecoded);
}

/// Returns a sign (true when negative) and `FullDecoded` value
/// from given floating point number.
pub(super) fn decode<T: DecodableFloat>(v: T) -> (/*negative?*/ bool, FullDecoded) {
    v.decode()
}
