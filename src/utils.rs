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

    /// Returns the loss of discarding a fractional part of `frac / 2^(2 * B)`,
    /// where `frac = frac_hi * 2^B + frac_lo` and `B` is `T::BITS`.
    ///
    /// `from_wide_shift(lo, hi, shift)` is equivalent to using the low
    /// `shift` bits of the value as the high bits of `frac`. This function
    /// is faster when `shift` is not a constant, and it is also able to
    /// handle a `shift` of `2 * B`.
    #[inline]
    pub(crate) fn from_wide_frac<T: UInt>(frac_lo: T, frac_hi: T) -> Self {
        let half_hi = T::ONE << (T::BITS - 1);
        if frac_hi < half_hi {
            if frac_hi == T::ZERO && frac_lo == T::ZERO {
                RoundLoss::Zero
            } else {
                RoundLoss::HalfDown
            }
        } else if frac_hi == half_hi && frac_lo == T::ZERO {
            RoundLoss::Halfway
        } else {
            RoundLoss::HalfUp
        }
    }
}

/// Number of decimal digits of the chunks used to convert between integers
/// and decimal digits, which is the largest `n` such that `10^n` fits in a
/// `u64`.
pub(crate) const DEC_CHUNK_LEN: usize = 19;

/// `10^DEC_CHUNK_LEN`
pub(crate) const DEC_CHUNK_POW10: u64 = 10u64.pow(DEC_CHUNK_LEN as u32);

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

#[cfg(test)]
mod tests {
    use super::{RoundLoss, wide_shift_left};

    /// Checks `RoundLoss::from_wide_shift` and `RoundLoss::from_wide_frac`
    /// with every 16-bit value split in two `u8`.
    #[test]
    fn test_wide_round_loss() {
        fn expected(frac: u32, half: u32) -> RoundLoss {
            if frac == 0 {
                RoundLoss::Zero
            } else {
                match frac.cmp(&half) {
                    core::cmp::Ordering::Less => RoundLoss::HalfDown,
                    core::cmp::Ordering::Equal => RoundLoss::Halfway,
                    core::cmp::Ordering::Greater => RoundLoss::HalfUp,
                }
            }
        }

        for value in 0..=u16::MAX {
            let [lo, hi] = value.to_le_bytes();
            assert_eq!(
                RoundLoss::from_wide_frac(lo, hi),
                expected(value.into(), 1 << 15),
                "value = {value}",
            );

            for shift in 1..16 {
                let lost = u32::from(value) & ((1 << shift) - 1);
                let expected = expected(lost, 1 << (shift - 1));
                assert_eq!(
                    RoundLoss::from_wide_shift(lo, hi, shift),
                    expected,
                    "value = {value}, shift = {shift}",
                );

                // The lost bits as the high bits of the fraction.
                let (frac_lo, frac_hi) = wide_shift_left(lo, hi, 16 - shift);
                assert_eq!(
                    RoundLoss::from_wide_frac(frac_lo, frac_hi),
                    expected,
                    "value = {value}, shift = {shift}",
                );
            }
        }
    }
}
