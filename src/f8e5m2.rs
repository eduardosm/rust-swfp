use crate::ieee_float;

#[derive(Copy, Clone)]
pub(crate) struct F8E5M2Semantics;

impl ieee_float::Semantics for F8E5M2Semantics {
    type Bits = u8;
    type Mant = u8;
    type Exp = i8;

    const FORMAT: ieee_float::Format = ieee_float::Format::Ieee;
    const PREC_BITS: u32 = 3;
    const EXP_BITS: u32 = 5;
}

/// 8-bit floating point type with 5 exponent bits and 2 mantissa bits.
///
/// It is not a standard format, but it follows all semantics of IEEE 754.
#[derive(Copy, Clone)]
pub struct F8E5M2(pub(crate) ieee_float::IeeeFloat<F8E5M2Semantics>);

ieee_float::impl_float_traits! {
    impl F8E5M2 {
        type Bits = u8;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -15, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 16, 0b10));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 16, 0));
    }
}
