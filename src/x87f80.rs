use crate::ieee_float;

#[derive(Copy, Clone)]
pub(crate) struct X87F80Semantics;

impl ieee_float::Semantics for X87F80Semantics {
    type Bits = u128;
    type Mant = u128;
    type Exp = i32;

    const FORMAT: ieee_float::Format = ieee_float::Format::X87;
    const PREC_BITS: u32 = 64;
    const EXP_BITS: u32 = 15;
}

/// X87 80-bit extended precision floating point type.
#[derive(Copy, Clone)]
pub struct X87F80(pub(crate) ieee_float::IeeeFloat<X87F80Semantics>);

ieee_float::impl_float_traits! {
    impl X87F80 {
        type Bits = u128;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -16383, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 16384, 0b11 << 62));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 16384, 0b10 << 62));
    }
}

impl crate::math::Sqrt for X87F80 {
    fn sqrt_ex(self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.sqrt(round);
        (Self(res_value), status)
    }
}
