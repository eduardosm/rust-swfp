use crate::ieee_float;

#[derive(Copy, Clone)]
pub(crate) struct F128Semantics;

impl ieee_float::Semantics for F128Semantics {
    type Bits = u128;
    type Mant = u128;
    type Exp = i32;

    const FORMAT: ieee_float::Format = ieee_float::Format::Ieee;
    const PREC_BITS: u32 = 113;
    const EXP_BITS: u32 = 15;
}

/// IEEE 754 quadruple-precision (128-bit) floating point type.
#[derive(Copy, Clone)]
pub struct F128(pub(crate) ieee_float::IeeeFloat<F128Semantics>);

ieee_float::impl_float_traits! {
    impl F128 {
        type Bits = u128;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -16383, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 16384, 1 << 111));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 16384, 0));
    }
}

impl crate::math::Sqrt for F128 {
    fn sqrt_ex(self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.sqrt(round);
        (Self(res_value), status)
    }
}
