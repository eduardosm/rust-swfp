macro_rules! impl_cast_from {
    ($from:ty as $into:ty) => {
        impl $crate::traits::CastFrom<$from> for $into {
            #[allow(trivial_numeric_casts)]
            #[inline(always)]
            fn cast_from(value: $from) -> Self {
                value as Self
            }
        }
    };
}

macro_rules! impl_all_cast_from {
    ($into:ty) => {
        impl_cast_from!(u8 as $into);
        impl_cast_from!(u16 as $into);
        impl_cast_from!(u32 as $into);
        impl_cast_from!(u64 as $into);
        impl_cast_from!(u128 as $into);
        impl_cast_from!(i8 as $into);
        impl_cast_from!(i16 as $into);
        impl_cast_from!(i32 as $into);
        impl_cast_from!(i64 as $into);
        impl_cast_from!(i128 as $into);
    };
}

impl_all_cast_from!(u8);
impl_all_cast_from!(u16);
impl_all_cast_from!(u32);
impl_all_cast_from!(u64);
impl_all_cast_from!(u128);
impl_all_cast_from!(i8);
impl_all_cast_from!(i16);
impl_all_cast_from!(i32);
impl_all_cast_from!(i64);
impl_all_cast_from!(i128);

macro_rules! impl_int {
    ($t:ty) => {
        impl $crate::traits::Int for $t {
            const BITS: u32 = Self::BITS;

            const ZERO: Self = 0;
            const ONE: Self = 1;
            const TWO: Self = 2;

            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;

            #[inline]
            fn leading_zeros(self) -> u32 {
                self.leading_zeros()
            }

            #[inline]
            fn wrapping_neg(self) -> Self {
                self.wrapping_neg()
            }

            #[inline]
            fn wrapping_add(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }

            #[inline]
            fn wrapping_sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }

            #[inline]
            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }

            #[inline]
            fn saturating_add(self, rhs: Self) -> Self {
                self.saturating_add(rhs)
            }

            #[inline]
            fn saturating_sub(self, rhs: Self) -> Self {
                self.saturating_sub(rhs)
            }

            #[inline]
            fn checked_add(self, rhs: Self) -> Option<Self> {
                self.checked_add(rhs)
            }
        }
    };
}

macro_rules! impl_uint {
    ($t:ty) => {
        impl_int!($t);
        impl $crate::traits::UInt for $t {
            #[inline]
            fn wide_mul(self, rhs: Self) -> (Self, Self) {
                self.carrying_mul(rhs, 0)
            }
        }
    };
}

impl_uint!(u8);
impl_uint!(u16);
impl_uint!(u32);
impl_uint!(u64);
impl_uint!(u128);

macro_rules! impl_sint {
    ($t:ty) => {
        impl_int!($t);
        impl $crate::traits::SInt for $t {}
    };
}

impl_sint!(i8);
impl_sint!(i16);
impl_sint!(i32);
impl_sint!(i64);
impl_sint!(i128);
