use crate::ieee_float;

mod cbrt;
mod exp;
mod gamma;
mod hyperbolic;
mod inv_hyperbolic;
mod inv_trigonometric;
mod log;
mod pow;
mod trigonometric;

#[derive(Copy, Clone)]
pub(crate) struct F32Semantics;

impl ieee_float::Semantics for F32Semantics {
    type Bits = u32;
    type Mant = u32;
    type Exp = i16;

    const FORMAT: ieee_float::Format = ieee_float::Format::Ieee;
    const PREC_BITS: u32 = 24;
    const EXP_BITS: u32 = 8;
}

/// IEEE 754 single-precision (32-bit) floating point type.
#[derive(Copy, Clone)]
pub struct F32(pub(crate) ieee_float::IeeeFloat<F32Semantics>);

ieee_float::impl_float_traits! {
    impl F32 {
        type Bits = u32;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -127, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 128, 1 << 22));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 128, 0));
    }
}

impl F32 {
    #[inline]
    pub fn from_host(value: f32) -> Self {
        crate::Float::from_bits(value.to_bits())
    }

    #[inline]
    pub fn to_host(self) -> f32 {
        f32::from_bits(crate::Float::to_bits(self))
    }
}

impl crate::math::Sqrt for F32 {
    fn sqrt_ex(self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.sqrt(round);
        (Self(res_value), status)
    }
}

impl crate::math::Hypot for F32 {
    fn hypot_ex(self, other: Self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.hypot(other.0, round);
        (Self(res_value), status)
    }
}

impl crate::traits::FloatImpExt for F32 {
    type Exp = i16;

    const MANT_BITS: u32 = 23;

    const ONE: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        0,
        1 << 23,
    ));
    const TWO: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        1,
        1 << 23,
    ));
    const HALF: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        -1,
        1 << 23,
    ));

    #[inline]
    fn sign(self) -> bool {
        self.0.sign()
    }

    #[inline]
    fn exponent(self) -> Self::Exp {
        self.0.to_parts().2
    }

    #[inline]
    fn mant(self) -> Self::Bits {
        self.0.to_parts().3
    }

    #[inline]
    fn bitwise_eq(self, rhs: Self) -> bool {
        self.0.bitwise_eq(rhs.0)
    }

    #[inline]
    fn is_int(self) -> bool {
        self.0.is_int()
    }

    #[inline]
    fn is_odd_int(self) -> bool {
        self.0.is_odd_int()
    }

    #[inline]
    fn set_sign(self, sign: bool) -> Self {
        Self(self.0.set_sign(sign))
    }
}
