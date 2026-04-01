use crate::ieee_float;
use crate::traits::{self, CastFrom as _, Int as _};
use crate::uint192::U192;
use crate::utils::RoundLoss;

pub(crate) type SfpM16E8 = Sfp<u16, i8>;
pub(crate) type SfpM32E16 = Sfp<u32, i16>;
pub(crate) type SfpM64E16 = Sfp<u64, i16>;
pub(crate) type SfpM128E16 = Sfp<u128, i16>;
pub(crate) type SfpM192E16 = Sfp<U192, i16>;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Sfp<M: traits::UInt, E: traits::SInt> {
    s: bool,
    e: E,
    m: M,
}

impl<M: traits::UInt, E: traits::SInt> Sfp<M, E> {
    pub(crate) const ZERO: Self = Self {
        s: false,
        e: E::MIN,
        m: M::ZERO,
    };

    pub(crate) const INFINITY: Self = Self {
        s: false,
        e: E::MAX,
        m: M::ZERO,
    };

    pub(crate) const NAN: Self = Self {
        s: false,
        e: E::MAX,
        m: M::ONE,
    };

    #[inline]
    pub(crate) fn one() -> Self {
        Self {
            s: false,
            e: E::ZERO,
            m: M::ONE << (M::BITS - 1),
        }
    }

    #[inline]
    pub(crate) fn two() -> Self {
        Self {
            s: false,
            e: E::ONE,
            m: M::ONE << (M::BITS - 1),
        }
    }

    #[inline]
    pub(crate) fn half() -> Self {
        Self {
            s: false,
            e: -E::ONE,
            m: M::ONE << (M::BITS - 1),
        }
    }

    #[inline]
    pub(crate) const fn new(s: bool, e: E, m: M) -> Self {
        Self { s, e, m }
    }

    #[inline]
    pub(crate) const fn newp(e: E, m: M) -> Self {
        Self::new(false, e, m)
    }

    #[inline]
    pub(crate) const fn newn(e: E, m: M) -> Self {
        Self::new(true, e, m)
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn largest() -> Self {
        Self {
            s: false,
            e: E::MAX - E::ONE,
            m: M::MAX,
        }
    }

    #[inline]
    pub(crate) fn sign(self) -> bool {
        self.s
    }

    #[inline]
    pub(crate) fn exponent(self) -> E {
        self.e
    }

    #[inline]
    pub(crate) fn mant(self) -> M {
        self.m
    }

    #[inline]
    pub(crate) fn set_sign(self, s: bool) -> Self {
        Self {
            s,
            e: self.e,
            m: self.m,
        }
    }

    #[inline]
    pub(crate) fn set_exp(self, e: E) -> Self {
        debug_assert!(e != E::MIN && e != E::MAX);
        Self {
            s: self.s,
            e,
            m: self.m,
        }
    }

    #[inline]
    pub(crate) fn from_sfp<Ms, Es>(value: Sfp<Ms, Es>) -> Self
    where
        Ms: traits::UInt,
        Es: traits::SInt,
        E: From<Es>,
        M: From<Ms>,
    {
        Self {
            s: value.s,
            e: if value.e == Es::MIN {
                E::MIN
            } else if value.e == Es::MAX {
                E::MAX
            } else {
                value.e.into()
            },
            m: M::from(value.m) << (M::BITS - Ms::BITS),
        }
    }

    #[inline]
    pub(crate) fn from_int<T: traits::Int + traits::CastInto<M>>(value: T) -> Self {
        if value == T::ZERO {
            Self::ZERO
        } else {
            let (s, abs) = if value < T::ZERO {
                (true, value.wrapping_neg())
            } else {
                (false, value)
            };
            let lz = abs.leading_zeros();
            let e = ((T::BITS - 1) as i16) - (lz as i16);
            if let Ok(e) = E::try_from(e) {
                let m = if T::BITS > M::BITS {
                    ((abs << lz) >> (T::BITS - M::BITS)).cast_into()
                } else {
                    let v: M = abs.cast_into();
                    v << (lz + (M::BITS - T::BITS))
                };
                Self { s, e, m }
            } else {
                Self::INFINITY
            }
        }
    }

    #[inline]
    pub(crate) fn to_int_round<T: traits::Int>(self) -> Option<T>
    where
        M: TryInto<T>,
    {
        if self.e == E::MAX {
            // infinity or NaN
            return None;
        } else if self.is_zero() {
            return Some(T::ZERO);
        } else if T::MIN == T::ZERO && self.s {
            // `T` is unsigned
            return None;
        }

        let m_bits_m1 = E::try_from(M::BITS - 1).ok().unwrap();
        if self.e < m_bits_m1 {
            let shift: u32 = ((m_bits_m1 - E::ONE) - self.e).cast_into();
            if shift >= M::BITS {
                return Some(T::ZERO);
            }
            let v = ((self.m >> shift) + M::ONE) >> 1;
            let v: T = v.try_into().ok()?;
            if self.s {
                Some(v.wrapping_neg())
            } else {
                Some(v)
            }
        } else {
            let m: T = self.m.try_into().ok()?;
            let shift: u32 = (self.e - m_bits_m1).cast_into();
            if shift >= T::BITS {
                return None;
            }
            let v = m << shift;
            if (v >> shift) != m {
                return None;
            }
            if self.s {
                Some(v.wrapping_neg())
            } else {
                Some(v)
            }
        }
    }

    pub(crate) fn from_ieee_float<S: ieee_float::Semantics>(v: ieee_float::IeeeFloat<S>) -> Self
    where
        M: From<S::Mant>,
        E: TryFrom<S::Exp>,
    {
        let (category, sign, exp, mant) = v.to_parts();
        match category {
            core::num::FpCategory::Nan => Self {
                s: sign,
                e: E::MAX,
                m: M::ONE,
            },
            core::num::FpCategory::Infinite => Self {
                s: sign,
                e: E::MAX,
                m: M::ZERO,
            },
            core::num::FpCategory::Zero => Self {
                s: sign,
                e: E::MIN,
                m: M::ZERO,
            },
            core::num::FpCategory::Subnormal | core::num::FpCategory::Normal => {
                let m = M::from(mant) << (M::BITS - S::PREC_BITS);
                if let Ok(e) = E::try_from(exp) {
                    Self { s: sign, e, m }
                } else if exp < S::Exp::ZERO {
                    Self {
                        s: sign,
                        e: E::MIN,
                        m: M::ZERO,
                    }
                } else {
                    Self {
                        s: sign,
                        e: E::MAX,
                        m: M::ZERO,
                    }
                }
            }
        }
    }

    pub(crate) fn to_ieee_float<S: ieee_float::Semantics>(self) -> ieee_float::IeeeFloat<S>
    where
        S::Exp: TryFrom<E>,
        S::Mant: traits::CastFrom<M>,
    {
        if self.e == E::MAX {
            if self.m == M::ZERO {
                return ieee_float::IeeeFloat::make_inf(self.s);
            } else {
                return ieee_float::IeeeFloat::make_default_qnan();
            }
        } else if self.e == E::MIN {
            return ieee_float::IeeeFloat::make_zero(self.s);
        }

        // `IeeeFloat::round_and_classify` needs the exponent to
        // be less than `MAX`.
        let e = S::Exp::try_from(self.e).ok().filter(|&e| e < S::Exp::MAX);
        let Some(e) = e else {
            if self.e > E::ZERO {
                return ieee_float::IeeeFloat::make_inf(self.s);
            } else {
                return ieee_float::IeeeFloat::make_zero(self.s);
            }
        };

        let mant;
        let loss;
        if S::PREC_BITS >= M::BITS {
            mant = S::Mant::cast_from(self.m) << (S::PREC_BITS - M::BITS);
            loss = RoundLoss::Zero;
        } else {
            let shift = M::BITS - S::PREC_BITS;
            mant = S::Mant::cast_from(self.m >> shift);
            loss = RoundLoss::from_shift(self.m, shift);
        };
        ieee_float::IeeeFloat::round_and_classify(
            self.s,
            e,
            mant,
            loss,
            crate::Round::NearestTiesToEven,
        )
        .0
    }

    #[inline]
    pub(crate) fn round_prec(self, prec: u32) -> Self {
        assert!(prec > 0 && prec < M::BITS);
        if self.e == E::MIN || self.e == E::MAX {
            self
        } else {
            let add = M::ONE << ((M::BITS - 1) - prec);
            if let Some(new_m) = self.m.checked_add(add) {
                Self {
                    s: self.s,
                    e: self.e,
                    m: new_m & !(M::MAX >> prec),
                }
            } else {
                let new_e = self.e + E::ONE;
                Self {
                    s: self.s,
                    e: new_e,
                    m: if new_e == E::MAX {
                        M::ZERO
                    } else {
                        M::ONE << (M::BITS - 1)
                    },
                }
            }
        }
    }

    #[inline]
    pub(crate) fn is_zero(self) -> bool {
        self.e == E::MIN
    }

    #[inline]
    pub(crate) fn is_infinite(self) -> bool {
        self.e == E::MAX && self.m == M::ZERO
    }

    #[inline]
    pub(crate) fn is_nan(self) -> bool {
        self.e == E::MAX && self.m != M::ZERO
    }

    #[inline]
    pub(crate) fn abs(self) -> Self {
        Self {
            s: false,
            e: self.e,
            m: self.m,
        }
    }

    #[inline]
    pub(crate) fn scalbn(self, e: E) -> Self {
        if self.e == E::MIN || self.e == E::MAX {
            return self;
        }

        let new_e = self.e.saturating_add(e);
        if new_e == E::MIN || new_e == E::MAX {
            Self {
                s: self.s,
                e: new_e,
                m: M::ZERO,
            }
        } else {
            Self {
                s: self.s,
                e: new_e,
                m: self.m,
            }
        }
    }

    #[allow(dead_code)] // FIXME: unused?
    #[inline]
    pub(crate) fn next_mant_up(self) -> Self {
        if self.is_nan() || self.is_infinite() {
            self
        } else if self.is_zero() {
            Self {
                s: self.s,
                e: E::MIN + E::ONE,
                m: M::ONE << (M::BITS - 1),
            }
        } else {
            if let Some(m) = self.m.checked_add(M::ONE) {
                Self {
                    s: self.s,
                    e: self.e,
                    m,
                }
            } else {
                let e = self.e + E::ONE;
                if e == E::MAX {
                    // Infinity
                    Self {
                        s: self.s,
                        e: E::MAX,
                        m: M::ZERO,
                    }
                } else {
                    Self {
                        s: self.s,
                        e,
                        m: M::ONE << (M::BITS - 1),
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn next_mant_down(self) -> Self {
        if self.is_nan() || self.is_infinite() {
            self
        } else if self.is_zero() {
            Self {
                s: !self.s,
                e: E::MIN + E::ONE,
                m: M::ONE << (M::BITS - 1),
            }
        } else {
            let m = self.m - M::ONE;
            if m < (M::ONE << (M::BITS - 1)) {
                let e = self.e - E::ONE;
                if e == E::MIN {
                    // Underflow to zero
                    Self {
                        s: self.s,
                        e: E::MIN,
                        m: M::ZERO,
                    }
                } else {
                    Self {
                        s: self.s,
                        e,
                        m: m << 1,
                    }
                }
            } else {
                Self {
                    s: self.s,
                    e: self.e,
                    m,
                }
            }
        }
    }

    #[inline]
    fn add_or_sub<const SUB: bool>(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::NAN;
        }

        if self.s == (rhs.s ^ SUB) {
            let (xe, xm, ye, ym) = if self.e >= rhs.e {
                (self.e, self.m, rhs.e, rhs.m)
            } else {
                (rhs.e, rhs.m, self.e, self.m)
            };
            let rs = self.s;
            let (re, rm) = Self::add_inner(xe, xm, ye, ym);
            Self {
                s: rs,
                e: re,
                m: rm,
            }
        } else {
            let (xe, xm, ye, ym, rs) = if self.e > rhs.e || (self.e == rhs.e && self.m >= rhs.m) {
                (self.e, self.m, rhs.e, rhs.m, self.s)
            } else {
                (rhs.e, rhs.m, self.e, self.m, rhs.s ^ SUB)
            };
            let (re, rm) = Self::sub_inner(xe, xm, ye, ym);
            Self {
                s: rs,
                e: re,
                m: rm,
            }
        }
    }

    #[inline]
    pub(crate) fn add_inner(xe: E, xm: M, ye: E, ym: M) -> (E, M) {
        if xe == E::MAX || ye == E::MAX {
            // inf + inf = inf
            return (E::MAX, M::ZERO);
        }

        let edelta: u32 = xe.saturating_sub(ye).cast_into();
        if edelta >= M::BITS {
            // y is much smaller than x, return x
            return (xe, xm);
        }

        let (rm, ov) = xm.overflowing_add(ym >> edelta);
        if ov {
            if xe == E::MAX - E::ONE {
                // overflow into infinity
                (E::MAX, M::ZERO)
            } else {
                // handle overflow by increasing exponent
                (xe + E::ONE, (M::ONE << (M::BITS - 1)) | (rm >> 1))
            }
        } else {
            (xe, rm)
        }
    }

    #[inline]
    pub(crate) fn sub_inner(xe: E, xm: M, ye: E, ym: M) -> (E, M) {
        if xe == E::MAX && ye == E::MAX {
            // inf - inf = nan
            return (E::MAX, M::ONE);
        } else if xe == E::MAX {
            // inf - finite = inf
            return (E::MAX, M::ZERO);
        }

        let edelta: u32 = xe.saturating_sub(ye).cast_into();
        if edelta >= M::BITS {
            // y is much smaller than x, return x
            return (xe, xm);
        }

        let rm = xm - (ym >> edelta);
        if rm == M::ZERO {
            // x == y, result is zero
            (E::MIN, M::ZERO)
        } else {
            let eoff = rm.leading_zeros();
            let re = xe.saturating_sub(E::cast_from(eoff));
            if re == E::MIN {
                // Underflow
                (E::MIN, M::ZERO)
            } else {
                (re, rm << eoff)
            }
        }
    }

    #[inline]
    pub(crate) fn square(self) -> Self {
        if self.is_nan() {
            return Self::NAN;
        }

        let xe = self.e;
        let xm = self.m;

        if xe == E::MAX {
            // inf^2 = inf
            return Self::INFINITY;
        } else if self.is_zero() {
            // 0^2 = 0
            return Self::ZERO;
        }

        let (rm_lo, rm_hi) = xm.wide_mul(xm);
        let (eoff, rm) = if (rm_hi >> (M::BITS - 1)) == M::ZERO {
            (E::ZERO, (rm_hi << 1) | (rm_lo >> (M::BITS - 1)))
        } else {
            (E::ONE, rm_hi)
        };

        let re = (xe + eoff).saturating_add(xe);
        if re == E::MIN || re == E::MAX {
            Self {
                s: false,
                e: re,
                m: M::ZERO,
            }
        } else {
            Self {
                s: false,
                e: re,
                m: rm,
            }
        }
    }

    #[inline]
    pub(crate) fn recip(self) -> Self {
        Self::one() / self
    }

    #[inline]
    pub(crate) fn sqrt(self) -> Self {
        if self.is_nan() || self.is_zero() {
            return self;
        } else if self.s {
            return Self::NAN;
        } else if self.e == E::MAX {
            return self;
        }

        let (k, m) = if (self.e & E::ONE) == E::ZERO {
            // exponent is even
            (self.e, self.m >> 2)
        } else {
            // exponent is odd
            (self.e - E::ONE, self.m >> 1)
        };

        // Calculate sqrt(m) bit-by-bit
        let mut rem = m;
        let mut res = M::ZERO;
        let mut s = M::ZERO;
        let mut d = M::ONE << (M::BITS - 3);

        while d != M::ZERO {
            let t = s + d;
            if rem >= t {
                s += d + d;
                rem -= t;
                res += d;
            }
            rem <<= 1;
            d >>= 1;
        }

        // Build result with k/2 as exponent and res as mantissa
        Self {
            s: false,
            e: k >> 1,
            m: res << 2,
        }
    }
}

impl<M: traits::UInt, E: traits::SInt> PartialEq for Sfp<M, E> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        if self.is_nan() || rhs.is_nan() {
            false
        } else if self.is_zero() && rhs.is_zero() {
            true
        } else {
            self.s == rhs.s && self.e == rhs.e && self.m == rhs.m
        }
    }
}

impl<M: traits::UInt, E: traits::SInt> PartialOrd for Sfp<M, E> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.is_nan() || other.is_nan() {
            None
        } else if self.is_zero() && other.is_zero() {
            Some(core::cmp::Ordering::Equal)
        } else {
            match (self.s, other.s) {
                (true, false) => Some(core::cmp::Ordering::Less),
                (false, true) => Some(core::cmp::Ordering::Greater),
                (false, false) => Some(self.e.cmp(&other.e).then_with(|| self.m.cmp(&other.m))),
                (true, true) => Some(other.e.cmp(&self.e).then_with(|| other.m.cmp(&self.m))),
            }
        }
    }
}

impl<M: traits::UInt, E: traits::SInt> core::ops::Neg for Sfp<M, E> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self {
            s: !self.s,
            e: self.e,
            m: self.m,
        }
    }
}

impl<M: traits::UInt, E: traits::SInt> core::ops::Add for Sfp<M, E> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::add_or_sub::<false>(self, rhs)
    }
}

impl<M: traits::UInt, E: traits::SInt> core::ops::Sub for Sfp<M, E> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::add_or_sub::<true>(self, rhs)
    }
}

impl<M: traits::UInt, E: traits::SInt> core::ops::Mul for Sfp<M, E> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::NAN;
        }

        let xs = self.s;
        let xe = self.e;
        let xm = self.m;
        let ys = rhs.s;
        let ye = rhs.e;
        let ym = rhs.m;

        let rs = xs ^ ys;

        if (xe == E::MAX && ye == E::MIN) || (xe == E::MIN && ye == E::MAX) {
            return Self::NAN; // inf * 0 = nan
        } else if xe == E::MAX || ye == E::MAX {
            // inf * inf = inf
            return Self::INFINITY.set_sign(rs);
        } else if xe == E::MIN || ye == E::MIN {
            // 0 * x = 0
            return Self::ZERO.set_sign(rs);
        }

        let (rm_lo, rm_hi) = xm.wide_mul(ym);
        let (eoff, rm) = if (rm_hi >> (M::BITS - 1)) == M::ZERO {
            (E::ZERO, (rm_hi << 1) | (rm_lo >> (M::BITS - 1)))
        } else {
            (E::ONE, rm_hi)
        };

        let re = (xe + eoff).saturating_add(ye);
        if re == E::MIN || re == E::MAX {
            Self {
                s: rs,
                e: re,
                m: M::ZERO,
            }
        } else {
            Self {
                s: rs,
                e: re,
                m: rm,
            }
        }
    }
}

impl<M: traits::UInt, E: traits::SInt> core::ops::Div for Sfp<M, E> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::NAN;
        }

        let rs = self.s ^ rhs.s;

        match (self.is_zero(), rhs.is_zero()) {
            (false, false) => {}
            (true, false) => {
                return Self::ZERO.set_sign(rs);
            }
            (false, true) => {
                return Self::INFINITY.set_sign(rs);
            }
            (true, true) => {
                return Self::NAN;
            }
        }

        match (self.e == E::MAX, rhs.e == E::MAX) {
            (false, false) => {}
            (true, false) => {
                // inf / x = inf
                return Self::INFINITY.set_sign(rs);
            }
            (false, true) => {
                // x / inf = 0
                return Self::ZERO.set_sign(rs);
            }
            (true, true) => {
                // inf / inf = nan
                return Self::NAN;
            }
        }

        let mut q = M::ZERO;
        let mut r = self.m;
        let d = rhs.m;

        let mut extra = false;
        let mut off_one = false;
        if r < d {
            extra = true;
            off_one = true;
            r <<= 1;
        }

        for bit in (0..M::BITS).rev() {
            if extra {
                q |= M::ONE << bit;
                r = r.wrapping_sub(d);
                extra = (r >> (M::BITS - 1)) != M::ZERO;
            } else if r >= d {
                q |= M::ONE << bit;
                r -= d;
            } else {
                extra = (r >> (M::BITS - 1)) != M::ZERO;
            }
            r <<= 1;
        }

        let e = self.e.saturating_sub(rhs.e + E::from(off_one));
        if e == E::MIN || e == E::MAX {
            Self {
                s: rs,
                e,
                m: M::ZERO,
            }
        } else {
            Self { s: rs, e, m: q }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sfp;
    use crate::{F32, Float as _, f16::F16Semantics};

    type SfpM8E16 = Sfp<u8, i16>;
    type SfpM32E16 = Sfp<u32, i16>;
    type SfpM64E16 = Sfp<u64, i16>;

    #[test]
    fn test_from_int() {
        let value = SfpM32E16::from_int(0u32);
        assert_eq!(value, SfpM32E16::ZERO);

        let value = SfpM32E16::from_int(0x1ABC_u16);
        assert_eq!(
            value,
            SfpM32E16 {
                s: false,
                e: 12,
                m: 0x1ABC << (16 + 3),
            }
        );

        let value = SfpM32E16::from_int(-0x1ABC_i16);
        assert_eq!(
            value,
            SfpM32E16 {
                s: true,
                e: 12,
                m: 0x1ABC << (16 + 3),
            }
        );

        let value = SfpM32E16::from_int(0x1ABCD_u32);
        assert_eq!(
            value,
            SfpM32E16 {
                s: false,
                e: 16,
                m: 0x1ABCD << 15,
            }
        );

        let value = SfpM32E16::from_int(-0x1ABCD_i32);
        assert_eq!(
            value,
            SfpM32E16 {
                s: true,
                e: 16,
                m: 0x1ABCD << 15,
            }
        );

        let value = SfpM32E16::from_int(0x1_ABCD_FEFF_u64);
        assert_eq!(
            value,
            SfpM32E16 {
                s: false,
                e: 32,
                m: (0x1_ABCD_FEFE_u64 >> 1) as u32,
            }
        );

        let value = SfpM32E16::from_int(-0x1_ABCD_FEFF_i64);
        assert_eq!(
            value,
            SfpM32E16 {
                s: true,
                e: 32,
                m: (0x1_ABCD_FEFE_u64 >> 1) as u32,
            }
        );
    }

    #[test]
    fn test_to_int_round() {
        let value = SfpM32E16 {
            s: false,
            e: 2,
            m: 0xFFFF_0000,
        };
        assert_eq!(value.to_int_round::<u8>(), Some(1 << 3));

        let value = SfpM32E16 {
            s: false,
            e: 48,
            m: 0xFEEE_0000,
        };
        assert_eq!(value.to_int_round::<u64>(), Some(0xFEEE << (48 - 15)));
    }

    #[test]
    fn test_from_ieee_float() {
        let value = SfpM32E16::from_ieee_float(F32::from_bits(0x3FC00000).0);
        assert_eq!(
            value,
            SfpM32E16 {
                s: false,
                e: 0,
                m: 0b11 << (32 - 2),
            }
        );

        let value = SfpM64E16::from_ieee_float(F32::from_bits(0x3FC00000).0);
        assert_eq!(
            value,
            SfpM64E16 {
                s: false,
                e: 0,
                m: 0b11 << (64 - 2),
            }
        );

        let value = SfpM32E16::from_ieee_float(F32::from_bits(0b101).0);
        assert_eq!(
            value,
            SfpM32E16 {
                s: false,
                e: -147,
                m: 0b101 << (32 - 3),
            }
        );

        let value = SfpM64E16::from_ieee_float(F32::from_bits(0b101).0);
        assert_eq!(
            value,
            SfpM64E16 {
                s: false,
                e: -147,
                m: 0b101 << (64 - 3),
            }
        );
    }

    #[test]
    fn test_to_ieee_float() {
        let value = SfpM32E16 {
            s: false,
            e: -14,
            m: 0x8000_0000,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0400);

        let value = SfpM32E16 {
            s: false,
            e: -15,
            m: 0xFFFF_0000,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0400);

        let value = SfpM8E16 {
            s: false,
            e: -15,
            m: 0xFF,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x03FC);

        let value = SfpM8E16 {
            s: false,
            e: -24,
            m: 0x80,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0001);

        let value = SfpM8E16 {
            s: false,
            e: -24,
            m: 0xC0,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0002);

        let value = SfpM8E16 {
            s: false,
            e: -25,
            m: 0x80,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0000);

        let value = SfpM8E16 {
            s: false,
            e: -25,
            m: 0xC0,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0001);

        let value = SfpM8E16 {
            s: false,
            e: -26,
            m: 0x80,
        };
        assert_eq!(value.to_ieee_float::<F16Semantics>().to_bits(), 0x0000);
    }

    #[test]
    fn test_add() {
        let inf = SfpM32E16::INFINITY;
        assert_eq!(inf + inf, inf);
        assert!((inf + (-inf)).is_nan());
        assert!(((-inf) + inf).is_nan());

        let a = SfpM32E16::from_int(0x1000_u32);
        let b = SfpM32E16::from_int(0x0200_u32);
        let r = SfpM32E16::from_int(0x1200_u32);
        assert_eq!(a + b, r);
        assert_eq!(b + a, r);

        let a = SfpM32E16::from_int(u32::MAX);
        let b = SfpM32E16::one();
        let r = SfpM32E16::from_int(1u64 << 32);
        assert_eq!(a + b, r);
        assert_eq!(b + a, r);

        let a = SfpM32E16::from_int(0x1_0000_2000_u64);
        let b = SfpM32E16::from_int(0x0_00C0_00AF_u32);
        let r = SfpM32E16::from_int(0x1_00C0_20AE_u64);
        assert_eq!(a + b, r);
        assert_eq!(b + a, r);

        let a = SfpM32E16::from_int(0x1400_u32);
        let b = SfpM32E16::from_int(0x1000_u32);
        let r = SfpM32E16::from_int(0x0400_u32);
        assert_eq!(a + (-b), r);
        assert_eq!((-b) + a, r);
        assert_eq!((-a) + b, -r);
        assert_eq!(b + (-a), -r);

        let a = SfpM32E16::from_int(0x1000_u32);
        assert_eq!(a + (-a), SfpM32E16::ZERO);
        assert_eq!((-a) + a, -SfpM32E16::ZERO);
    }

    #[test]
    fn test_sub() {
        let inf = SfpM32E16::INFINITY;
        assert!((inf - inf).is_nan());
        assert_eq!(inf - (-inf), inf);
        assert_eq!((-inf) - inf, -inf);

        let a = SfpM32E16::from_int(0x1400_u32);
        let b = SfpM32E16::from_int(0x1000_u32);
        let r = SfpM32E16::from_int(0x0400_u32);
        assert_eq!(a - b, r);
        assert_eq!(b - a, -r);

        let a = SfpM32E16::from_int(0x1000_u32);
        let b = SfpM32E16::from_int(0x0200_u32);
        let r = SfpM32E16::from_int(0x1200_u32);
        assert_eq!(a - (-b), r);
        assert_eq!((-b) - a, -r);
        assert_eq!((-a) - b, -r);
        assert_eq!(b - (-a), r);

        let a = SfpM32E16::from_int(0x1000_u32);
        assert_eq!(a - a, SfpM32E16::ZERO);
    }

    #[test]
    fn test_mul() {
        let zero = SfpM32E16::ZERO;
        let inf = SfpM32E16::INFINITY;
        assert!((zero * inf).is_nan());
        assert!((inf * zero).is_nan());

        let a = SfpM32E16::from_int(0xA_u32);
        let b = SfpM32E16::from_int(0xB_u32);
        let r = SfpM32E16::from_int(0x6E_u32);
        assert_eq!(a * b, r);
        assert_eq!(b * a, r);

        let a = SfpM32E16::largest();
        let b = SfpM32E16::from_int(2u8);
        assert_eq!(a * b, inf);
        assert_eq!(b * a, inf);
    }

    #[test]
    fn test_div() {
        let zero = SfpM32E16::ZERO;
        let inf = SfpM32E16::INFINITY;
        assert!((zero / zero).is_nan());
        assert!((inf / inf).is_nan());

        let a = SfpM32E16::from_int(15_u32);
        let b = SfpM32E16::from_int(3_u32);
        let c = SfpM32E16::from_int(5_u32);
        assert_eq!(a / b, c);
        assert_eq!(a / c, b);

        let a = SfpM32E16::from_int(10_u32);
        let b = SfpM32E16::from_int(3_u32);
        let c = SfpM32E16 {
            s: false,
            e: 1,
            m: 0xD555_5555,
        };
        assert_eq!(a / b, c);
        assert_eq!(a / c, b);

        let a = SfpM32E16::from_int(28_u32);
        let b = SfpM32E16::from_int(15_u32);
        let c = SfpM32E16 {
            s: false,
            e: 0,
            m: 0xEEEE_EEEE,
        };
        assert_eq!(a / b, c);
        assert_eq!(a / c, b);
    }

    #[test]
    fn test_sqrt() {
        let a = SfpM32E16::from_int(2u8);
        let b = SfpM32E16 {
            s: false,
            e: 0,
            m: 0xB504_F330,
        };
        assert_eq!(a.sqrt(), b);

        let a = SfpM32E16::from_int(3u8);
        let b = SfpM32E16 {
            s: false,
            e: 0,
            m: 0xDDB3_D740,
        };
        assert_eq!(a.sqrt(), b);

        let a = SfpM32E16::from_int(6u8);
        let b = SfpM32E16 {
            s: false,
            e: 1,
            m: 0x9CC4_70A0,
        };
        assert_eq!(a.sqrt(), b);
    }
}
