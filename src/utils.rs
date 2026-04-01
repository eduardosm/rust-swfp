use crate::traits::UInt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum RoundLoss {
    Zero,
    Halfway,
    HalfUp,
    HalfDown,
}

impl RoundLoss {
    #[inline]
    pub(crate) fn from_shift<T: UInt>(value: T, shift: u32) -> Self {
        let loss_half = T::ONE << (shift - 1);
        let loss_bits = value & !(T::MAX << shift);
        if loss_bits == T::ZERO {
            RoundLoss::Zero
        } else if loss_bits < loss_half {
            RoundLoss::HalfDown
        } else if loss_bits > loss_half {
            RoundLoss::HalfUp
        } else {
            RoundLoss::Halfway
        }
    }

    #[inline]
    pub(crate) fn from_shift_and<T: UInt>(value: T, shift: u32, prev_loss: Self) -> Self {
        let loss_half = T::ONE << (shift - 1);
        let loss_bits = value & !(T::MAX << shift);
        if loss_bits == T::ZERO {
            if prev_loss == Self::Zero {
                Self::Zero
            } else {
                Self::HalfDown
            }
        } else if loss_bits < loss_half {
            Self::HalfDown
        } else if loss_bits > loss_half {
            Self::HalfUp
        } else if prev_loss == Self::Zero {
            Self::Halfway
        } else {
            Self::HalfUp
        }
    }

    #[inline]
    pub(crate) fn from_wide_shift<T: UInt>(value_lo: T, value_hi: T, shift: u32) -> Self {
        let (loss_half_lo, loss_half_hi) = wide_shift_left(T::ONE, T::ZERO, shift - 1);
        let (n_mask_lo, n_mask_hi) = wide_shift_left(T::MAX, T::MAX, shift);
        let loss_bits_lo = value_lo & !n_mask_lo;
        let loss_bits_hi = value_hi & !n_mask_hi;
        if (loss_bits_hi, loss_bits_lo) == (T::ZERO, T::ZERO) {
            RoundLoss::Zero
        } else if (loss_bits_hi, loss_bits_lo) < (loss_half_hi, loss_half_lo) {
            RoundLoss::HalfDown
        } else if (loss_bits_hi, loss_bits_lo) > (loss_half_hi, loss_half_lo) {
            RoundLoss::HalfUp
        } else {
            RoundLoss::Halfway
        }
    }
}

#[inline]
pub(crate) fn wide_shift_left<T: UInt>(lo: T, hi: T, shift: u32) -> (T, T) {
    if shift >= T::BITS * 2 {
        panic!("attempt to shift left with overflow");
    } else if shift >= T::BITS {
        (T::ZERO, lo << (shift - T::BITS))
    } else if shift != 0 {
        (lo << shift, (hi << shift) | (lo >> (T::BITS - shift)))
    } else {
        (lo, hi)
    }
}

#[inline]
pub(crate) fn wide_shift_right<T: UInt>(lo: T, hi: T, shift: u32) -> (T, T) {
    if shift >= T::BITS * 2 {
        panic!("attempt to shift right with overflow");
    } else if shift >= T::BITS {
        (hi >> (shift - T::BITS), T::ZERO)
    } else if shift != 0 {
        ((hi << (T::BITS - shift)) | (lo >> shift), hi >> shift)
    } else {
        (lo, hi)
    }
}
