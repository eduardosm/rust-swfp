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
pub(crate) struct F64Semantics;

impl ieee_float::Semantics for F64Semantics {
    type Bits = u64;
    type Mant = u64;
    type Exp = i16;

    const FORMAT: ieee_float::Format = ieee_float::Format::Ieee;
    const PREC_BITS: u32 = 53;
    const EXP_BITS: u32 = 11;
}

/// IEEE 754 double-precision (64-bit) floating point type.
#[derive(Copy, Clone)]
pub struct F64(pub(crate) ieee_float::IeeeFloat<F64Semantics>);

ieee_float::impl_float_traits! {
    impl F64 {
        type Bits = u64;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -1023, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 1024, 1 << 51));
        const INFINITY: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Infinite, false, 1024, 0));
    }
}

impl F64 {
    #[inline]
    pub fn from_host(value: f64) -> Self {
        crate::Float::from_bits(value.to_bits())
    }

    #[inline]
    pub fn to_host(self) -> f64 {
        f64::from_bits(crate::Float::to_bits(self))
    }
}

impl crate::math::Sqrt for F64 {
    fn sqrt_ex(self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.sqrt(round);
        (Self(res_value), status)
    }
}

impl crate::math::Hypot for F64 {
    fn hypot_ex(self, other: Self, round: crate::Round) -> (Self, crate::FpStatus) {
        let (res_value, status) = self.0.hypot(other.0, round);
        (Self(res_value), status)
    }
}

impl crate::traits::FloatImpExt for F64 {
    type Exp = i16;

    const MANT_BITS: u32 = 52;

    const ONE: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        0,
        1 << 52,
    ));
    const TWO: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        1,
        1 << 52,
    ));
    const HALF: Self = Self(ieee_float::IeeeFloat::with_parts(
        core::num::FpCategory::Normal,
        false,
        -1,
        1 << 52,
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
