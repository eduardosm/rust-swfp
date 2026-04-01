//! Custom arbitrary-precision number (bignum) implementation.
//!
//! This is designed to avoid the heap allocation at expense of stack memory.
//! The most used bignum type, `Big32x40`, is limited by 32 × 40 = 1,280 bits
//! and will take at most 160 bytes of stack memory. This is more than enough
//! for round-tripping all possible finite `f64` values.
//!
//! In principle it is possible to have multiple bignum types for different
//! inputs, but we don't do so to avoid the code bloat. Each bignum is still
//! tracked for the actual usages, so it normally doesn't matter.

/// Arithmetic operations required by bignums.
pub(super) trait FullOps: Sized {
    /// Returns `(carry', v')` such that `carry' * 2^W + v' = self * other + other2 + carry`,
    /// where `W` is the number of bits in `Self`.
    fn full_mul_add(self, other: Self, other2: Self, carry: Self) -> (Self /* carry */, Self);

    /// Returns `(quo, rem)` such that `borrow * 2^W + self = quo * other + rem`
    /// and `0 <= rem < other`, where `W` is the number of bits in `Self`.
    fn full_div_rem(self, other: Self, borrow: Self)
    -> (Self /* quotient */, Self /* remainder */);
}

macro_rules! impl_full_ops {
    ($($ty:ty: mul/div($bigty:ident);)*) => (
        $(
            impl FullOps for $ty {
                fn full_mul_add(self, other: $ty, other2: $ty, carry: $ty) -> ($ty, $ty) {
                    // This cannot overflow;
                    // the output is between `0` and `2^nbits * (2^nbits - 1)`.
                    let (lo, hi) = self.carrying_mul_add(other, other2, carry);
                    (hi, lo)
                }

                fn full_div_rem(self, other: $ty, borrow: $ty) -> ($ty, $ty) {
                    debug_assert!(borrow < other);
                    // This cannot overflow; the output is between `0` and `other * (2^nbits - 1)`.
                    let lhs = ((borrow as $bigty) << <$ty>::BITS) | (self as $bigty);
                    let rhs = other as $bigty;
                    ((lhs / rhs) as $ty, (lhs % rhs) as $ty)
                }
            }
        )*
    )
}

impl_full_ops! {
    u8:  mul/div(u16);
    u16: mul/div(u32);
    u32: mul/div(u64);
    u64: mul/div(u128);
}

macro_rules! define_bignum {
    ($name:ident: type=$ty:ty, n=$n:expr) => {
        /// Stack-allocated arbitrary-precision (up to certain limit) integer.
        ///
        /// This is backed by a fixed-size array of given type ("digit").
        /// While the array is not very large (normally some hundred bytes),
        /// copying it recklessly may result in the performance hit.
        /// Thus this is intentionally not `Copy`.
        ///
        /// All operations available to bignums panic in the case of overflows.
        /// The caller is responsible to use large enough bignum types.
        pub(super) struct $name {
            /// One plus the offset to the maximum "digit" in use.
            /// This does not decrease, so be aware of the computation order.
            /// `base[size..]` should be zero.
            size: usize,
            /// Digits. `[a, b, c, ...]` represents `a + b*2^W + c*2^(2W) + ...`
            /// where `W` is the number of bits in the digit type.
            base: [$ty; $n],
        }

        impl $name {
            /// Makes a bignum from one digit.
            pub(super) fn from_small(v: $ty) -> $name {
                let mut base = [0; $n];
                base[0] = v;
                $name { size: 1, base }
            }

            /// Makes a bignum from `u64` value.
            pub(super) fn from_u64(mut v: u64) -> $name {
                let mut base = [0; $n];
                let mut sz = 0;
                while v > 0 {
                    base[sz] = v as $ty;
                    v >>= <$ty>::BITS;
                    sz += 1;
                }
                $name { size: sz, base }
            }

            /// Makes a bignum from `u128` value.
            #[allow(dead_code)] // not used in all versions
            pub(super) fn from_u128(mut v: u128) -> $name {
                let mut base = [0; $n];
                let mut sz = 0;
                while v > 0 {
                    base[sz] = v as $ty;
                    v >>= <$ty>::BITS;
                    sz += 1;
                }
                $name { size: sz, base }
            }

            /// Returns the internal digits as a slice `[a, b, c, ...]` such that the numeric
            /// value is `a + b * 2^W + c * 2^(2W) + ...` where `W` is the number of bits in
            /// the digit type.
            pub(super) fn digits(&self) -> &[$ty] {
                &self.base[..self.size]
            }

            /// Returns `true` if the bignum is zero.
            pub(super) fn is_zero(&self) -> bool {
                self.digits().iter().all(|&v| v == 0)
            }

            /// Adds `other` to itself and returns its own mutable reference.
            pub(super) fn add<'a>(&'a mut self, other: &$name) -> &'a mut $name {
                use core::{cmp, iter};

                let mut sz = cmp::max(self.size, other.size);
                let mut carry = false;
                for (a, b) in iter::zip(&mut self.base[..sz], &other.base[..sz]) {
                    let (v, c) = (*a).carrying_add(*b, carry);
                    *a = v;
                    carry = c;
                }
                if carry {
                    self.base[sz] = 1;
                    sz += 1;
                }
                self.size = sz;
                self
            }

            /// Subtracts `other` from itself and returns its own mutable reference.
            pub(super) fn sub<'a>(&'a mut self, other: &$name) -> &'a mut $name {
                use core::{cmp, iter};

                let sz = cmp::max(self.size, other.size);
                let mut noborrow = true;
                for (a, b) in iter::zip(&mut self.base[..sz], &other.base[..sz]) {
                    let (v, c) = (*a).carrying_add(!*b, noborrow);
                    *a = v;
                    noborrow = c;
                }
                assert!(noborrow);
                self.size = sz;
                self
            }

            /// Multiplies itself by a digit-sized `other` and returns its own
            /// mutable reference.
            pub(super) fn mul_small(&mut self, other: $ty) -> &mut $name {
                let mut sz = self.size;
                let mut carry = 0;
                for a in &mut self.base[..sz] {
                    let (v, c) = (*a).carrying_mul(other, carry);
                    *a = v;
                    carry = c;
                }
                if carry > 0 {
                    self.base[sz] = carry;
                    sz += 1;
                }
                self.size = sz;
                self
            }

            /// Multiplies itself by `2^bits` and returns its own mutable reference.
            pub(super) fn mul_pow2(&mut self, bits: usize) -> &mut $name {
                let digitbits = <$ty>::BITS as usize;
                let digits = bits / digitbits;
                let bits = bits % digitbits;

                assert!(digits < $n);
                debug_assert!(self.base[$n - digits..].iter().all(|&v| v == 0));
                debug_assert!(bits == 0 || (self.base[$n - digits - 1] >> (digitbits - bits)) == 0);

                // shift by `digits * digitbits` bits
                for i in (0..self.size).rev() {
                    self.base[i + digits] = self.base[i];
                }
                for i in 0..digits {
                    self.base[i] = 0;
                }

                // shift by `bits` bits
                let mut sz = self.size + digits;
                if bits > 0 {
                    let last = sz;
                    let overflow = self.base[last - 1] >> (digitbits - bits);
                    if overflow > 0 {
                        self.base[last] = overflow;
                        sz += 1;
                    }
                    for i in (digits + 1..last).rev() {
                        self.base[i] =
                            (self.base[i] << bits) | (self.base[i - 1] >> (digitbits - bits));
                    }
                    self.base[digits] <<= bits;
                    // self.base[..digits] is zero, no need to shift
                }

                self.size = sz;
                self
            }

            /// Multiplies itself by a number described by `other[0] + other[1] * 2^W +
            /// other[2] * 2^(2W) + ...` (where `W` is the number of bits in the digit type)
            /// and returns its own mutable reference.
            pub(super) fn mul_digits<'a>(&'a mut self, other: &[$ty]) -> &'a mut $name {
                // the internal routine. works best when aa.len() <= bb.len().
                fn mul_inner(ret: &mut [$ty; $n], aa: &[$ty], bb: &[$ty]) -> usize {
                    use crate::core_num::bignum::FullOps;

                    let mut retsz = 0;
                    for (i, &a) in aa.iter().enumerate() {
                        if a == 0 {
                            continue;
                        }
                        let mut sz = bb.len();
                        let mut carry = 0;
                        for (j, &b) in bb.iter().enumerate() {
                            let (c, v) = a.full_mul_add(b, ret[i + j], carry);
                            ret[i + j] = v;
                            carry = c;
                        }
                        if carry > 0 {
                            ret[i + sz] = carry;
                            sz += 1;
                        }
                        if retsz < i + sz {
                            retsz = i + sz;
                        }
                    }
                    retsz
                }

                let mut ret = [0; $n];
                let retsz = if self.size < other.len() {
                    mul_inner(&mut ret, &self.digits(), other)
                } else {
                    mul_inner(&mut ret, other, &self.digits())
                };
                self.base = ret;
                self.size = retsz;
                self
            }

            /// Divides itself by a digit-sized `other` and returns its own
            /// mutable reference *and* the remainder.
            pub(super) fn div_rem_small(&mut self, other: $ty) -> (&mut $name, $ty) {
                use crate::core_num::bignum::FullOps;

                assert!(other > 0);

                let sz = self.size;
                let mut borrow = 0;
                for a in self.base[..sz].iter_mut().rev() {
                    let (q, r) = (*a).full_div_rem(other, borrow);
                    *a = q;
                    borrow = r;
                }
                (self, borrow)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                self.base[..] == other.base[..]
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &$name) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &$name) -> core::cmp::Ordering {
                use core::cmp::max;
                let sz = max(self.size, other.size);
                let lhs = self.base[..sz].iter().cloned().rev();
                let rhs = other.base[..sz].iter().cloned().rev();
                lhs.cmp(rhs)
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                Self {
                    size: self.size,
                    base: self.base,
                }
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let sz = if self.size < 1 { 1 } else { self.size };
                let digitlen = <$ty>::BITS as usize / 4;

                write!(f, "{:#x}", self.base[sz - 1])?;
                for &v in self.base[..sz - 1].iter().rev() {
                    write!(f, "_{:01$x}", v, digitlen)?;
                }
                Ok(())
            }
        }
    };
}

/// The digit type for `Big32x40` / `Big32x540`.
pub(super) type Digit32 = u32;

define_bignum!(Big32x40: type=Digit32, n=40);
define_bignum!(Big32x540: type=Digit32, n=540);

#[cfg(test)]
mod tests {
    use std::format;

    define_bignum!(Big8x3: type=u8, n=3);

    use Big8x3 as Big;

    #[test]
    #[should_panic]
    fn test_from_u64_overflow() {
        Big::from_u64(0x1000000);
    }

    #[test]
    #[should_panic]
    fn test_from_u128_overflow() {
        Big::from_u128(0x1000000);
    }

    #[test]
    fn test_add() {
        assert_eq!(
            *Big::from_small(3).add(&Big::from_small(4)),
            Big::from_small(7)
        );
        assert_eq!(
            *Big::from_small(3).add(&Big::from_small(0)),
            Big::from_small(3)
        );
        assert_eq!(
            *Big::from_small(0).add(&Big::from_small(3)),
            Big::from_small(3)
        );
        assert_eq!(
            *Big::from_small(3).add(&Big::from_u64(0xfffe)),
            Big::from_u64(0x10001)
        );
        assert_eq!(
            *Big::from_u64(0xfedc).add(&Big::from_u64(0x789)),
            Big::from_u64(0x10665)
        );
        assert_eq!(
            *Big::from_u64(0x789).add(&Big::from_u64(0xfedc)),
            Big::from_u64(0x10665)
        );
    }

    #[test]
    #[should_panic]
    fn test_add_overflow_1() {
        Big::from_small(1).add(&Big::from_u64(0xffffff));
    }

    #[test]
    #[should_panic]
    fn test_add_overflow_2() {
        Big::from_u64(0xffffff).add(&Big::from_small(1));
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            *Big::from_small(7).sub(&Big::from_small(4)),
            Big::from_small(3)
        );
        assert_eq!(
            *Big::from_u64(0x10665).sub(&Big::from_u64(0x789)),
            Big::from_u64(0xfedc)
        );
        assert_eq!(
            *Big::from_u64(0x10665).sub(&Big::from_u64(0xfedc)),
            Big::from_u64(0x789)
        );
        assert_eq!(
            *Big::from_u64(0x10665).sub(&Big::from_u64(0x10664)),
            Big::from_small(1)
        );
        assert_eq!(
            *Big::from_u64(0x10665).sub(&Big::from_u64(0x10665)),
            Big::from_small(0)
        );
    }

    #[test]
    #[should_panic]
    fn test_sub_underflow_1() {
        Big::from_u64(0x10665).sub(&Big::from_u64(0x10666));
    }

    #[test]
    #[should_panic]
    fn test_sub_underflow_2() {
        Big::from_small(0).sub(&Big::from_u64(0x123456));
    }

    #[test]
    fn test_mul_small() {
        assert_eq!(*Big::from_small(7).mul_small(5), Big::from_small(35));
        assert_eq!(
            *Big::from_small(0xff).mul_small(0xff),
            Big::from_u64(0xfe01)
        );
        assert_eq!(
            *Big::from_u64(0xffffff / 13).mul_small(13),
            Big::from_u64(0xffffff)
        );
    }

    #[test]
    #[should_panic]
    fn test_mul_small_overflow() {
        Big::from_u64(0x800000).mul_small(2);
    }

    #[test]
    fn test_mul_pow2() {
        assert_eq!(*Big::from_small(0x7).mul_pow2(4), Big::from_small(0x70));
        assert_eq!(*Big::from_small(0xff).mul_pow2(1), Big::from_u64(0x1fe));
        assert_eq!(*Big::from_small(0xff).mul_pow2(12), Big::from_u64(0xff000));
        assert_eq!(*Big::from_small(0x1).mul_pow2(23), Big::from_u64(0x800000));
        assert_eq!(*Big::from_u64(0x123).mul_pow2(0), Big::from_u64(0x123));
        assert_eq!(*Big::from_u64(0x123).mul_pow2(7), Big::from_u64(0x9180));
        assert_eq!(*Big::from_u64(0x123).mul_pow2(15), Big::from_u64(0x918000));
        assert_eq!(*Big::from_small(0).mul_pow2(23), Big::from_small(0));
    }

    #[test]
    #[should_panic]
    fn test_mul_pow2_overflow_1() {
        Big::from_u64(0x1).mul_pow2(24);
    }

    #[test]
    #[should_panic]
    fn test_mul_pow2_overflow_2() {
        Big::from_u64(0x123).mul_pow2(16);
    }

    #[test]
    fn test_mul_digits() {
        assert_eq!(*Big::from_small(3).mul_digits(&[5]), Big::from_small(15));
        assert_eq!(
            *Big::from_small(0xff).mul_digits(&[0xff]),
            Big::from_u64(0xfe01)
        );
        assert_eq!(
            *Big::from_u64(0x123).mul_digits(&[0x56, 0x4]),
            Big::from_u64(0x4edc2)
        );
        assert_eq!(
            *Big::from_u64(0x12345).mul_digits(&[0x67]),
            Big::from_u64(0x7530c3)
        );
        assert_eq!(
            *Big::from_small(0x12).mul_digits(&[0x67, 0x45, 0x3]),
            Big::from_u64(0x3ae13e)
        );
        assert_eq!(
            *Big::from_u64(0xffffff / 13).mul_digits(&[13]),
            Big::from_u64(0xffffff)
        );
        assert_eq!(
            *Big::from_small(13).mul_digits(&[0x3b, 0xb1, 0x13]),
            Big::from_u64(0xffffff)
        );
    }

    #[test]
    #[should_panic]
    fn test_mul_digits_overflow_1() {
        Big::from_u64(0x800000).mul_digits(&[2]);
    }

    #[test]
    #[should_panic]
    fn test_mul_digits_overflow_2() {
        Big::from_u64(0x1000).mul_digits(&[0, 0x10]);
    }

    #[test]
    fn test_div_rem_small() {
        let as_val = |(q, r): (&mut Big, u8)| (q.clone(), r);
        assert_eq!(
            as_val(Big::from_small(0xff).div_rem_small(15)),
            (Big::from_small(17), 0)
        );
        assert_eq!(
            as_val(Big::from_small(0xff).div_rem_small(16)),
            (Big::from_small(15), 15)
        );
        assert_eq!(
            as_val(Big::from_small(3).div_rem_small(40)),
            (Big::from_small(0), 3)
        );
        assert_eq!(
            as_val(Big::from_u64(0xffffff).div_rem_small(123)),
            (Big::from_u64(0xffffff / 123), (0xffffffu64 % 123) as u8)
        );
        assert_eq!(
            as_val(Big::from_u64(0x10000).div_rem_small(123)),
            (Big::from_u64(0x10000 / 123), (0x10000u64 % 123) as u8)
        );
    }

    #[test]
    fn test_is_zero() {
        assert!(Big::from_small(0).is_zero());
        assert!(!Big::from_small(3).is_zero());
        assert!(!Big::from_u64(0x123).is_zero());
        assert!(
            !Big::from_u64(0xffffff)
                .sub(&Big::from_u64(0xfffffe))
                .is_zero()
        );
        assert!(
            Big::from_u64(0xffffff)
                .sub(&Big::from_u64(0xffffff))
                .is_zero()
        );
    }

    #[test]
    fn test_ord() {
        assert!(Big::from_u64(0) < Big::from_u64(0xffffff));
        assert!(Big::from_u64(0x102) < Big::from_u64(0x201));
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{:?}", Big::from_u64(0)), "0x0");
        assert_eq!(format!("{:?}", Big::from_u64(0x1)), "0x1");
        assert_eq!(format!("{:?}", Big::from_u64(0x12)), "0x12");
        assert_eq!(format!("{:?}", Big::from_u64(0x123)), "0x1_23");
        assert_eq!(format!("{:?}", Big::from_u64(0x1234)), "0x12_34");
        assert_eq!(format!("{:?}", Big::from_u64(0x12345)), "0x1_23_45");
        assert_eq!(format!("{:?}", Big::from_u64(0x123456)), "0x12_34_56");
    }
}
