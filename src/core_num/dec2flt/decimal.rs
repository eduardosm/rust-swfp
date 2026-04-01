//! Representation of a float as the significant digits and exponent.

/// A floating point number with up to 64 bits of mantissa and an `i64` exponent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Decimal {
    pub(super) exponent: i64,
    pub(super) mantissa: u64,
    pub(super) negative: bool,
    pub(super) many_digits: bool,
}
