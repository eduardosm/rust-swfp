use crate::ieee_float;

#[derive(Copy, Clone)]
pub(crate) struct F8E4M3NaoSemantics;

impl ieee_float::Semantics for F8E4M3NaoSemantics {
    type Bits = u8;
    type Mant = u8;
    type Exp = i8;

    const FORMAT: ieee_float::Format = ieee_float::Format::NoInfNanAllOnes;
    const PREC_BITS: u32 = 4;
    const EXP_BITS: u32 = 4;
}

/// 8-bit floating point type with 4 exponent bits and 3 mantissa bits.
///
/// This format is described in <https://arxiv.org/abs/2209.05433>. It does not
/// have infinites and NaN is encoded as all-ones.
#[derive(Copy, Clone)]
pub struct F8E4M3Nao(pub(crate) ieee_float::IeeeFloat<F8E4M3NaoSemantics>);

ieee_float::impl_float_traits! {
    impl F8E4M3Nao {
        type Bits = u8;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -7, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, false, 8, 7));
        const INFINITY: Self = Self::NAN;
    }
}
