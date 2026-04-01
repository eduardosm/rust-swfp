use crate::ieee_float;

#[derive(Copy, Clone)]
pub(crate) struct F8E4M3B8NnzSemantics;

impl ieee_float::Semantics for F8E4M3B8NnzSemantics {
    type Bits = u8;
    type Mant = u8;
    type Exp = i8;

    const FORMAT: ieee_float::Format = ieee_float::Format::NoInfNanNegZero;
    const PREC_BITS: u32 = 4;
    const EXP_BITS: u32 = 4;

    #[inline]
    fn exp_bias() -> Self::Exp {
        8
    }
}

/// 8-bit floating point type with 4 exponent bits and 3 mantissa bits.
///
/// This format is described in <https://arxiv.org/abs/2206.02915>. The exponent
/// bias is 8. It does not have infinites or signed zeros, and NaN is encoded
/// as negative zero.
#[derive(Copy, Clone)]
pub struct F8E4M3B8Nnz(pub(crate) ieee_float::IeeeFloat<F8E4M3B8NnzSemantics>);

ieee_float::impl_float_traits! {
    impl F8E4M3B8Nnz {
        type Bits = u8;
        const ZERO: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Zero, false, -8, 0));
        const NAN: Self = Self(ieee_float::IeeeFloat::with_parts(core::num::FpCategory::Nan, true, -8, 0));
        const INFINITY: Self = Self::NAN;
    }
}
