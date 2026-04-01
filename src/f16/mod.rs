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
pub(crate) struct F16Semantics;

impl ieee_float::Semantics for F16Semantics {
    type Bits = u16;
    type Mant = u16;
    type Exp = i8;

    const FORMAT: ieee_float::Format = ieee_float::Format::Ieee;
    const PREC_BITS: u32 = 11;
    const EXP_BITS: u32 = 5;
}

/// IEEE 754 half-precision (16-bit) floating point type.
#[derive(Copy, Clone)]
pub struct F16(pub(crate) ieee_float::IeeeFloat<F16Semantics>);

ieee_float::impl_float_traits! {
    impl F16 {
        type Bits = u16;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -15, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 16, 1 << 9));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 16, 0));
    }
}

impl crate::math::Sqrt for F16 {
    fn sqrt_ex(self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.sqrt(round);
        (Self(res_value), status)
    }
}

impl crate::math::Hypot for F16 {
    fn hypot_ex(self, other: Self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.hypot(other.0, round);
        (Self(res_value), status)
    }
}

impl crate::traits::FloatImpExt for F16 {
    type Exp = i8;

    const MANT_BITS: u32 = 10;

    const ONE: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        0,
        1 << 10,
    ));
    const TWO: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        1,
        1 << 10,
    ));
    const HALF: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        -1,
        1 << 10,
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
