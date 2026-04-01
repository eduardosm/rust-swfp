use crate::traits::{CastFrom, Int, UInt};

#[derive(Copy, Clone, Debug)]
pub(crate) struct U192 {
    lo: u128,
    hi: u64,
}

#[inline]
pub(crate) const fn u192(hi: u64, lo: u128) -> U192 {
    U192 { lo, hi }
}

impl U192 {
    #[inline]
    pub(crate) const fn hi(&self) -> u64 {
        self.hi
    }
}

impl Default for U192 {
    #[inline]
    fn default() -> Self {
        Self { lo: 0, hi: 0 }
    }
}

impl From<bool> for U192 {
    #[inline]
    fn from(value: bool) -> Self {
        Self {
            lo: value.into(),
            hi: 0,
        }
    }
}

impl CastFrom<U192> for U192 {
    #[inline]
    fn cast_from(value: U192) -> Self {
        value
    }
}

macro_rules! impl_cast_from_uint {
    ($from:ty) => {
        impl CastFrom<$from> for U192 {
            #[inline]
            fn cast_from(value: $from) -> Self {
                Self {
                    lo: value.into(),
                    hi: 0,
                }
            }
        }
    };
}

impl_cast_from_uint!(u8);
impl_cast_from_uint!(u16);
impl_cast_from_uint!(u32);
impl_cast_from_uint!(u64);
impl_cast_from_uint!(u128);

macro_rules! impl_cast_from_sint {
    ($from:ty) => {
        impl CastFrom<$from> for U192 {
            #[inline]
            fn cast_from(value: $from) -> Self {
                Self {
                    lo: i128::from(value) as u128,
                    hi: if value < 0 { u64::MAX } else { 0 },
                }
            }
        }
    };
}

impl_cast_from_sint!(i8);
impl_cast_from_sint!(i16);
impl_cast_from_sint!(i32);
impl_cast_from_sint!(i64);
impl_cast_from_sint!(i128);

macro_rules! impl_cast_into {
    ($into:ty) => {
        const _: () = assert!(<$into>::BITS <= 128);

        impl CastFrom<U192> for $into {
            #[allow(trivial_numeric_casts)]
            #[inline]
            fn cast_from(value: U192) -> Self {
                value.lo as Self
            }
        }
    };
}

impl_cast_into!(u8);
impl_cast_into!(u16);
impl_cast_into!(u32);
impl_cast_into!(u64);
impl_cast_into!(u128);

impl_cast_into!(i8);
impl_cast_into!(i16);
impl_cast_into!(i32);
impl_cast_into!(i64);
impl_cast_into!(i128);

macro_rules! impl_from_uint {
    ($from:ty) => {
        impl From<$from> for U192 {
            #[inline]
            fn from(value: $from) -> Self {
                Self {
                    lo: value.into(),
                    hi: 0,
                }
            }
        }
    };
}

impl_from_uint!(u8);
impl_from_uint!(u16);
impl_from_uint!(u32);
impl_from_uint!(u64);
impl_from_uint!(u128);

pub(crate) struct TryFromIntError;

macro_rules! impl_try_from_sint {
    ($from:ty) => {
        const _: () = assert!(<$from>::BITS <= 128);

        impl TryFrom<$from> for U192 {
            type Error = TryFromIntError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                u128::try_from(value)
                    .map(|lo| Self { lo, hi: 0 })
                    .map_err(|_| TryFromIntError)
            }
        }
    };
}

impl_try_from_sint!(i8);
impl_try_from_sint!(i16);
impl_try_from_sint!(i32);
impl_try_from_sint!(i64);
impl_try_from_sint!(i128);

macro_rules! impl_try_into {
    ($into:ty) => {
        const _: () = assert!(<$into>::BITS <= 128);

        impl TryFrom<U192> for $into {
            type Error = TryFromIntError;

            #[inline]
            fn try_from(value: U192) -> Result<Self, Self::Error> {
                if value.hi == 0 {
                    Self::try_from(value.lo).map_err(|_| TryFromIntError)
                } else {
                    Err(TryFromIntError)
                }
            }
        }
    };
}

impl_try_into!(u8);
impl_try_into!(u16);
impl_try_into!(u32);
impl_try_into!(u64);
impl_try_into!(u128);

impl_try_into!(i8);
impl_try_into!(i16);
impl_try_into!(i32);
impl_try_into!(i64);
impl_try_into!(i128);

impl PartialEq for U192 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.lo == other.lo && self.hi == other.hi
    }
}

impl Eq for U192 {}

impl PartialOrd for U192 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U192 {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.hi.cmp(&other.hi).then_with(|| self.lo.cmp(&other.lo))
    }
}

impl core::ops::Add for U192 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let (lo, carry) = self.lo.overflowing_add(rhs.lo);
        let hi = self.hi + rhs.hi + u64::from(carry);
        Self { lo, hi }
    }
}

impl core::ops::Sub for U192 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let (lo, borrow) = self.lo.overflowing_sub(rhs.lo);
        let hi = self.hi - rhs.hi - u64::from(borrow);
        Self { lo, hi }
    }
}

impl core::ops::Mul for U192 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let (lo, hi) = self.wide_mul(rhs);
        if cfg!(debug_assertions) && hi != u192(0, 0) {
            panic!("attempt to multiply with overflow");
        }
        lo
    }
}

impl core::ops::Not for U192 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        Self {
            lo: !self.lo,
            hi: !self.hi,
        }
    }
}

impl core::ops::BitAnd for U192 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo & rhs.lo,
            hi: self.hi & rhs.hi,
        }
    }
}

impl core::ops::BitOr for U192 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo | rhs.lo,
            hi: self.hi | rhs.hi,
        }
    }
}

impl core::ops::BitXor for U192 {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            lo: self.lo ^ rhs.lo,
            hi: self.hi ^ rhs.hi,
        }
    }
}

impl core::ops::Shl<u32> for U192 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        if rhs >= 192 {
            if cfg!(debug_assertions) {
                panic!("attempt to shift left with overflow");
            } else {
                Self::default()
            }
        } else if rhs >= 128 {
            Self {
                lo: 0,
                hi: (self.lo << (rhs - 128)) as u64,
            }
        } else if rhs >= 64 {
            Self {
                lo: self.lo << rhs,
                hi: (self.lo >> (128 - rhs)) as u64,
            }
        } else if rhs != 0 {
            Self {
                lo: self.lo << rhs,
                hi: (self.hi << rhs) | ((self.lo >> (128 - rhs)) as u64),
            }
        } else {
            self
        }
    }
}

impl core::ops::Shr<u32> for U192 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        if rhs >= 192 {
            if cfg!(debug_assertions) {
                panic!("attempt to shift right with overflow");
            } else {
                Self::default()
            }
        } else if rhs >= 128 {
            Self {
                lo: (self.hi >> (rhs - 128)) as u128,
                hi: 0,
            }
        } else if rhs >= 64 {
            Self {
                lo: (self.lo >> rhs) | (u128::from(self.hi) << (128 - rhs)),
                hi: 0,
            }
        } else if rhs != 0 {
            Self {
                lo: (self.lo >> rhs) | (u128::from(self.hi) << (128 - rhs)),
                hi: self.hi >> rhs,
            }
        } else {
            self
        }
    }
}

impl core::ops::AddAssign for U192 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl core::ops::SubAssign for U192 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl core::ops::MulAssign for U192 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl core::ops::BitAndAssign for U192 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl core::ops::BitOrAssign for U192 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl core::ops::ShlAssign<u32> for U192 {
    #[inline]
    fn shl_assign(&mut self, rhs: u32) {
        *self = *self << rhs;
    }
}

impl core::ops::ShrAssign<u32> for U192 {
    #[inline]
    fn shr_assign(&mut self, rhs: u32) {
        *self = *self >> rhs;
    }
}

impl Int for U192 {
    const BITS: u32 = 192;

    const ZERO: Self = u192(0, 0);
    const ONE: Self = u192(0, 1);
    const TWO: Self = u192(0, 2);

    const MIN: Self = u192(0, 0);
    const MAX: Self = u192(u64::MAX, u128::MAX);

    #[inline]
    fn leading_zeros(self) -> u32 {
        if self.hi == 0 {
            64 + self.lo.leading_zeros()
        } else {
            self.hi.leading_zeros()
        }
    }

    #[inline]
    fn wrapping_neg(self) -> Self {
        (!self).wrapping_add(Self::ONE)
    }

    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        let (lo, c) = self.lo.overflowing_add(rhs.lo);
        let hi = self.hi.wrapping_add(rhs.hi).wrapping_add(u64::from(c));
        Self { lo, hi }
    }

    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        let (lo, b) = self.lo.overflowing_sub(rhs.lo);
        let hi = self.hi.wrapping_sub(rhs.hi).wrapping_sub(u64::from(b));
        Self { lo, hi }
    }

    #[inline]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (lo, c) = self.lo.overflowing_add(rhs.lo);
        let (hi, c) = self.hi.carrying_add(rhs.hi, c);
        (Self { lo, hi }, c)
    }

    #[inline]
    fn saturating_add(self, rhs: Self) -> Self {
        let (r, ov) = self.overflowing_add(rhs);
        if ov { Self::MAX } else { r }
    }

    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        let (lo, b) = self.lo.overflowing_sub(rhs.lo);
        let (hi, b) = self.hi.borrowing_sub(rhs.hi, b);
        if b { Self::MIN } else { Self { lo, hi } }
    }

    #[inline]
    fn checked_add(self, rhs: Self) -> Option<Self> {
        let (r, ov) = self.overflowing_add(rhs);
        (!ov).then_some(r)
    }
}

impl UInt for U192 {
    fn wide_mul(self, rhs: Self) -> (Self, Self) {
        let (lo_lo, lo_hi1) = self.lo.carrying_mul(rhs.lo, 0);
        let hi_lo1 = lo_hi1 >> 64;
        let lo_hi1 = lo_hi1 as u64;

        let (lo_hi2, hi_lo2) = u128::from(self.hi).carrying_mul(rhs.lo, 0);
        let hi_lo2 = (hi_lo2 << 64) | (lo_hi2 >> 64);
        let lo_hi2 = lo_hi2 as u64;

        let (lo_hi3, hi_lo3) = u128::from(rhs.hi).carrying_mul(self.lo, 0);
        let hi_lo3 = (hi_lo3 << 64) | (lo_hi3 >> 64);
        let lo_hi3 = lo_hi3 as u64;

        let (hi_lo4, hi_hi) = self.hi.carrying_mul(rhs.hi, 0);
        let hi_lo4 = u128::from(hi_lo4) << 64;

        let (lo_hi, c11) = lo_hi1.overflowing_add(lo_hi2);
        let (lo_hi, c12) = lo_hi.overflowing_add(lo_hi3);

        let (hi_lo, c21) = hi_lo1.overflowing_add(hi_lo2);
        let (hi_lo, c22) = hi_lo.overflowing_add(hi_lo3);
        let (hi_lo, c23) = hi_lo.overflowing_add(hi_lo4);
        let (hi_lo, c24) = hi_lo.overflowing_add(u128::from(c11) + u128::from(c12));

        let hi_hi = hi_hi + u64::from(c21) + u64::from(c22) + u64::from(c23) + u64::from(c24);

        (
            Self {
                lo: lo_lo,
                hi: lo_hi,
            },
            Self {
                lo: hi_lo,
                hi: hi_hi,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{U192, u192};
    use crate::traits::{Int as _, UInt as _};

    #[test]
    fn test_wide_mul() {
        let a = u192(u64::MAX, u128::MAX);
        let b = u192(u64::MAX, u128::MAX);
        let (lo, hi) = a.wide_mul(b);
        assert_eq!(lo, U192::ONE);
        assert_eq!(hi, U192::MAX - U192::ONE);

        let a = u192(
            0x8B43_439A_273C_7DB8,
            0xE9CD_F054_A171_2C7B_9A1B_9F01_F6DD_B1F5,
        );
        let b = u192(
            0xEF46_5820_9818_9BDA,
            0x8E26_8A95_74EB_6CDB_F7AA_30E7_7791_26EE,
        );
        let (lo, hi) = a.wide_mul(b);
        assert_eq!(
            lo,
            u192(
                0x872C_7A41_0891_E750,
                0xF00C_EB2E_5A53_1BEA_3C41_83D6_194A_CFC6,
            ),
        );
        assert_eq!(
            hi,
            u192(
                0x822A_1072_412A_F7E6,
                0x31E8_3F62_76D9_85C3_D4F9_C538_C828_E395,
            ),
        );
    }
}
