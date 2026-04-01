pub(crate) trait CastFrom<T> {
    fn cast_from(value: T) -> Self;
}

pub(crate) trait CastInto<T> {
    fn cast_into(self) -> T;
}

impl<T: CastFrom<U>, U> CastInto<T> for U {
    #[inline]
    fn cast_into(self) -> T {
        T::cast_from(self)
    }
}

pub(crate) trait Int:
    Copy
    + From<bool>
    + CastFrom<Self>
    + CastFrom<u8>
    + CastFrom<u16>
    + CastFrom<u32>
    + CastFrom<u64>
    + CastFrom<u128>
    + CastFrom<i8>
    + CastFrom<i16>
    + CastFrom<i32>
    + CastFrom<i64>
    + CastFrom<i128>
    + CastInto<Self>
    + CastInto<u8>
    + CastInto<u16>
    + CastInto<u32>
    + CastInto<u64>
    + CastInto<u128>
    + CastInto<i8>
    + CastInto<i16>
    + CastInto<i32>
    + CastInto<i64>
    + CastInto<i128>
    + TryFrom<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + TryFrom<u128>
    + TryFrom<i8>
    + TryFrom<i16>
    + TryFrom<i32>
    + TryFrom<i64>
    + TryFrom<i128>
    + TryInto<u8>
    + TryInto<u16>
    + TryInto<u32>
    + TryInto<u64>
    + TryInto<u128>
    + TryInto<i8>
    + TryInto<i16>
    + TryInto<i32>
    + TryInto<i64>
    + TryInto<i128>
    + Ord
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
    + core::ops::Not<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOr<Output = Self>
    + core::ops::BitXor<Output = Self>
    + core::ops::Shl<u32, Output = Self>
    + core::ops::Shr<u32, Output = Self>
    + core::ops::AddAssign
    + core::ops::SubAssign
    + core::ops::MulAssign
    + core::ops::BitAndAssign
    + core::ops::BitOrAssign
    + core::ops::ShlAssign<u32>
    + core::ops::ShrAssign<u32>
{
    const BITS: u32;

    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;

    const MIN: Self;
    const MAX: Self;

    fn leading_zeros(self) -> u32;

    fn wrapping_neg(self) -> Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;

    fn overflowing_add(self, rhs: Self) -> (Self, bool);

    fn saturating_add(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;

    fn checked_add(self, rhs: Self) -> Option<Self>;
}

pub(crate) trait UInt: Int + From<u8> {
    fn wide_mul(self, rhs: Self) -> (Self, Self);
}

pub(crate) trait SInt: Int + From<i8> + core::ops::Neg<Output = Self> {}

pub(crate) trait FloatImpExt: crate::Float<Bits: UInt> {
    type Exp: SInt;

    const MANT_BITS: u32;

    const ONE: Self;
    const TWO: Self;
    const HALF: Self;

    fn sign(self) -> bool;

    // Not named `exp` to avoid confussion with the exp function
    fn exponent(self) -> Self::Exp;

    fn mant(self) -> Self::Bits;

    fn bitwise_eq(self, rhs: Self) -> bool;

    #[inline]
    fn is_zero(self) -> bool {
        self.classify() == core::num::FpCategory::Zero
    }

    fn is_int(self) -> bool;

    fn is_odd_int(self) -> bool;

    fn set_sign(self, sign: bool) -> Self;
}
