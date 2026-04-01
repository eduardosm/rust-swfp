use crate::traits::{SInt, UInt};
use crate::utils::RoundLoss;
use crate::{FpStatus, Round};

pub(crate) trait Float: Sized + core::ops::Neg<Output = Self> {
    const SIG_BITS: u32;

    type Mant: UInt;
    type Exp: SInt;

    fn zero(negative: bool) -> Self;

    fn inf(negative: bool) -> Self;

    fn nan() -> Self;

    fn overflow_value(negative: bool, round: Round) -> Self;

    fn underflow_value(negative: bool, round: Round) -> Self;

    fn build_value(
        negative: bool,
        exp: Self::Exp,
        mant: Self::Mant,
        loss: RoundLoss,
        round: Round,
    ) -> (Self, FpStatus);
}
