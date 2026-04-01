use core::num::FpCategory;

use crate::traits::{CastFrom, CastInto as _, Int, SInt, UInt};
use crate::utils::{self, RoundLoss};
use crate::{Float as _, FpStatus, Round, core_num};

pub(crate) trait Semantics: Copy {
    type Bits: UInt + From<Self::Mant> + CastFrom<Self::Exp>;
    type Mant: UInt + CastFrom<Self::Bits> + Into<u128> + core::fmt::Debug;
    type Exp: SInt
        + CastFrom<Self::Bits>
        + Into<i32>
        + core::fmt::Debug
        + core::ops::Div<Output = Self::Exp>;

    const FORMAT: Format;

    /// Includes the integer bit.
    const PREC_BITS: u32;

    /// Does not include the integer bit.
    const MANT_FRAC_BITS: u32 = Self::PREC_BITS - 1;

    const MANT_REPR_BITS: u32 = match Self::FORMAT {
        Format::Ieee => Self::MANT_FRAC_BITS,
        Format::X87 => Self::PREC_BITS,
        Format::NoInfNanAllOnes => Self::MANT_FRAC_BITS,
        Format::NoInfNanNegZero => Self::MANT_FRAC_BITS,
    };

    const EXP_BITS: u32;

    const EXP_SHIFT: u32 = Self::MANT_REPR_BITS;

    const SIGN_SHIFT: u32 = Self::EXP_SHIFT + Self::EXP_BITS;

    #[inline]
    fn exp_mask() -> Self::Bits {
        let num_bits = Self::EXP_BITS;
        (Self::Bits::ONE << num_bits) - Self::Bits::ONE
    }

    #[inline]
    fn exp_bias() -> Self::Exp {
        (Self::Exp::ONE << (Self::EXP_BITS - 1)) - Self::Exp::ONE
    }

    #[inline]
    fn mant_mask() -> Self::Bits {
        (Self::Bits::ONE << Self::MANT_REPR_BITS) - Self::Bits::ONE
    }

    #[inline]
    fn min_normal_exp() -> Self::Exp {
        Self::Exp::ONE - Self::exp_bias()
    }

    #[inline]
    fn min_subnormal_exp() -> Self::Exp {
        Self::min_normal_exp() - Self::Exp::cast_from(Self::MANT_FRAC_BITS)
    }

    #[inline]
    fn max_normal_exp() -> Self::Exp {
        let max_raw_exp = (Self::Exp::ONE << Self::EXP_BITS) - Self::Exp::ONE;
        match Self::FORMAT {
            Format::Ieee | Format::X87 => max_raw_exp - Self::Exp::ONE - Self::exp_bias(),
            Format::NoInfNanAllOnes | Format::NoInfNanNegZero => max_raw_exp - Self::exp_bias(),
        }
    }

    #[inline]
    fn mant_all_ones() -> Self::Mant {
        (Self::Mant::ONE << Self::MANT_REPR_BITS) - Self::Mant::ONE
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Format {
    Ieee,
    /// X87 format, where the integer bit is explicit.
    X87,
    /// Matches the format of FP8 E4M3 described in
    /// https://arxiv.org/abs/2209.05433.
    NoInfNanAllOnes,
    /// NaN is represented as negative zero. Matches the format of floating
    /// point types described in https://arxiv.org/abs/2206.02915.
    NoInfNanNegZero,
}

#[derive(Copy, Clone)]
pub(crate) struct IeeeFloat<S: Semantics> {
    category: FpCategory,
    sign: bool,
    exp: S::Exp,
    mant: S::Mant,
}

impl<S: Semantics> IeeeFloat<S> {
    #[inline]
    pub(crate) const fn with_parts(
        category: FpCategory,
        sign: bool,
        exp: S::Exp,
        mant: S::Mant,
    ) -> Self {
        Self {
            category,
            sign,
            exp,
            mant,
        }
    }

    #[inline]
    pub(crate) fn to_parts(self) -> (FpCategory, bool, S::Exp, S::Mant) {
        (self.category, self.sign, self.exp, self.mant)
    }

    #[inline]
    pub(crate) fn category(self) -> FpCategory {
        self.category
    }

    #[inline]
    pub(crate) fn sign(self) -> bool {
        self.sign
    }

    #[inline]
    pub(crate) fn is_int(self) -> bool {
        match self.category {
            FpCategory::Nan => false,
            FpCategory::Infinite => false,
            FpCategory::Zero => true,
            FpCategory::Subnormal | FpCategory::Normal => {
                if self.exp >= S::Exp::cast_from(S::MANT_FRAC_BITS) {
                    true
                } else if self.exp < S::Exp::ZERO {
                    false
                } else {
                    let frac_shift: u32 =
                        (S::Exp::cast_from(S::MANT_FRAC_BITS) - self.exp).cast_into();
                    (self.mant & !(S::Mant::MAX << frac_shift)) == S::Mant::ZERO
                }
            }
        }
    }

    #[inline]
    pub(crate) fn is_odd_int(self) -> bool {
        match self.category {
            FpCategory::Nan => false,
            FpCategory::Infinite => false,
            FpCategory::Zero => false,
            FpCategory::Subnormal | FpCategory::Normal => {
                if self.exp > S::Exp::cast_from(S::MANT_FRAC_BITS) || self.exp < S::Exp::ZERO {
                    false
                } else {
                    let frac_shift: u32 =
                        (S::Exp::cast_from(S::MANT_FRAC_BITS) - self.exp).cast_into();
                    if (self.mant & !(S::Mant::MAX << frac_shift)) != S::Mant::ZERO {
                        // not an integer
                        false
                    } else {
                        ((self.mant >> frac_shift) & S::Mant::ONE) == S::Mant::ONE
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn from_bits(bits: S::Bits) -> Self {
        let mut mant = S::Mant::cast_from(bits & S::mant_mask());
        let mut exp = S::Exp::cast_from((bits >> S::EXP_SHIFT) & S::exp_mask()) - S::exp_bias();
        let sign = ((bits >> S::SIGN_SHIFT) & S::Bits::ONE) != S::Bits::ZERO;

        let category = match S::FORMAT {
            Format::Ieee => {
                if exp == (S::min_normal_exp() - S::Exp::ONE) {
                    if mant == S::Mant::ZERO {
                        FpCategory::Zero
                    } else {
                        FpCategory::Subnormal
                    }
                } else if exp == (S::max_normal_exp() + S::Exp::ONE) {
                    if mant == S::Mant::ZERO {
                        FpCategory::Infinite
                    } else {
                        FpCategory::Nan
                    }
                } else {
                    mant |= S::Mant::ONE << S::MANT_FRAC_BITS;
                    FpCategory::Normal
                }
            }
            Format::X87 => {
                // Patterns with a non-all-1 exponent and a 0 integer bit are invalid and
                // interpreted as NaN.

                if exp == (S::min_normal_exp() - S::Exp::ONE) {
                    if mant == S::Mant::ZERO {
                        FpCategory::Zero
                    } else {
                        FpCategory::Subnormal
                    }
                } else if exp == (S::max_normal_exp() + S::Exp::ONE) {
                    if mant == (S::Mant::ONE << S::MANT_FRAC_BITS) {
                        FpCategory::Infinite
                    } else {
                        FpCategory::Nan
                    }
                } else if (mant >> S::MANT_FRAC_BITS) == S::Mant::ZERO {
                    FpCategory::Nan
                } else {
                    FpCategory::Normal
                }
            }
            Format::NoInfNanAllOnes => {
                if exp == (S::min_normal_exp() - S::Exp::ONE) {
                    if mant == S::Mant::ZERO {
                        FpCategory::Zero
                    } else {
                        FpCategory::Subnormal
                    }
                } else if exp == S::max_normal_exp() && mant == S::mant_all_ones() {
                    FpCategory::Nan
                } else {
                    mant |= S::Mant::ONE << S::MANT_FRAC_BITS;
                    FpCategory::Normal
                }
            }
            Format::NoInfNanNegZero => {
                if exp == (S::min_normal_exp() - S::Exp::ONE) {
                    if mant == S::Mant::ZERO {
                        if sign {
                            FpCategory::Nan
                        } else {
                            FpCategory::Zero
                        }
                    } else {
                        FpCategory::Subnormal
                    }
                } else {
                    mant |= S::Mant::ONE << S::MANT_FRAC_BITS;
                    FpCategory::Normal
                }
            }
        };

        if category == FpCategory::Subnormal {
            let shift = mant.leading_zeros() - (S::Mant::BITS - S::PREC_BITS);
            exp = S::min_normal_exp() - S::Exp::cast_from(shift);
            mant <<= shift;
        }

        Self {
            category,
            sign,
            exp,
            mant,
        }
    }

    #[inline]
    pub(crate) fn to_bits(self) -> S::Bits {
        if cfg!(debug_assertions) {
            self.check_consistency();
        }

        let raw_sign = S::Bits::from(self.sign);
        let raw_mant;
        let raw_exp;
        if self.category == FpCategory::Subnormal {
            let shift: u32 = (S::min_normal_exp() - self.exp).cast_into();
            raw_exp = S::Bits::cast_from(S::min_normal_exp() - S::Exp::ONE + S::exp_bias());
            raw_mant = S::Bits::from(self.mant >> shift);
        } else {
            raw_exp = S::Bits::cast_from(self.exp + S::exp_bias());
            raw_mant = S::Bits::from(self.mant) & S::mant_mask();
        };
        (raw_sign << S::SIGN_SHIFT) | (raw_exp << S::EXP_SHIFT) | raw_mant
    }

    #[inline]
    pub(crate) fn bitwise_eq(self, rhs: Self) -> bool {
        self.category == rhs.category
            && self.sign == rhs.sign
            && self.exp == rhs.exp
            && self.mant == rhs.mant
    }

    #[inline]
    pub(crate) fn abs(self) -> Self {
        if S::FORMAT == Format::NoInfNanNegZero && matches!(self.category, FpCategory::Nan) {
            // Changing the sign would mutate the NaN into a zero.
            self
        } else {
            Self {
                sign: false,
                ..self
            }
        }
    }

    #[inline]
    pub(crate) fn set_sign(self, sign: bool) -> Self {
        if S::FORMAT == Format::NoInfNanNegZero
            && matches!(self.category, FpCategory::Nan | FpCategory::Zero)
        {
            // Changing the sign could toggle between zero and NaN.
            self
        } else {
            Self { sign, ..self }
        }
    }

    pub(crate) fn convert_from<S2: Semantics>(
        value: IeeeFloat<S2>,
        round: Round,
    ) -> (Self, FpStatus)
    where
        S::Exp: TryFrom<S2::Exp>,
        S::Mant: CastFrom<S2::Mant>,
    {
        match value.category {
            FpCategory::Nan => {
                let (payload, signaling) = value.get_nan_info();
                (
                    Self::make_qnan::<false>(value.sign, S::Mant::cast_from(payload)),
                    if signaling {
                        FpStatus::Invalid
                    } else {
                        FpStatus::Ok
                    },
                )
            }
            FpCategory::Infinite => {
                let res_value = Self::make_inf(value.sign);
                if res_value.category == FpCategory::Infinite {
                    (res_value, FpStatus::Ok)
                } else {
                    (res_value, FpStatus::Inexact)
                }
            }
            FpCategory::Zero => {
                let res_value = Self::make_zero(value.sign);
                if res_value.sign == value.sign {
                    (res_value, FpStatus::Ok)
                } else {
                    (res_value, FpStatus::Inexact)
                }
            }
            FpCategory::Subnormal | FpCategory::Normal => match S::Exp::try_from(value.exp) {
                Ok(exp) => {
                    let (mant, loss) = if S::PREC_BITS >= S2::PREC_BITS {
                        let shift = S::PREC_BITS - S2::PREC_BITS;
                        (S::Mant::cast_from(value.mant) << shift, RoundLoss::Zero)
                    } else {
                        let shift = S2::PREC_BITS - S::PREC_BITS;
                        let mant = S::Mant::cast_from(value.mant >> shift);
                        let loss = RoundLoss::from_shift(value.mant, shift);
                        (mant, loss)
                    };
                    Self::round_and_classify(value.sign, exp, mant, loss, round)
                }
                Err(_) => {
                    if value.exp > S2::Exp::ZERO {
                        (
                            Self::make_overflow_value(value.sign, round),
                            FpStatus::Overflow,
                        )
                    } else {
                        (
                            Self::make_underflow_value(value.sign, round),
                            FpStatus::Underflow,
                        )
                    }
                }
            },
        }
    }

    pub(crate) fn from_uint(value: u128, round: Round) -> (Self, FpStatus) {
        if value == 0 {
            return (Self::make_zero(false), FpStatus::Ok);
        }

        let lz = value.leading_zeros();
        let Ok(exp) = S::Exp::try_from(127 - lz) else {
            return (Self::make_overflow_value(false, round), FpStatus::Overflow);
        };

        const {
            assert!(S::PREC_BITS < 128);
        }
        let wanted_lz = 128 - S::PREC_BITS;
        let (mant, loss) = if lz < wanted_lz {
            let shift = wanted_lz - lz;
            let mant = S::Mant::cast_from(value >> shift);
            let loss = RoundLoss::from_shift(value, shift);
            (mant, loss)
        } else {
            let shift = lz - wanted_lz;
            let mant = S::Mant::cast_from(value) << shift;
            (mant, RoundLoss::Zero)
        };

        Self::round_and_classify(false, exp, mant, loss, round)
    }

    pub(crate) fn from_int(value: i128, round: Round) -> (Self, FpStatus) {
        let (res_value, status) = Self::from_uint(value.unsigned_abs(), round);
        if value < 0 {
            (-res_value, status)
        } else {
            (res_value, status)
        }
    }

    pub(crate) fn to_uint(self, bits: u32, round: Round) -> (Option<u128>, FpStatus) {
        assert!(matches!(bits, 1..=128), "invalid number of bits: {bits}");
        const {
            assert!(S::PREC_BITS < 128);
        }
        match self.category {
            FpCategory::Nan => (None, FpStatus::Invalid),
            FpCategory::Infinite => (None, FpStatus::Overflow),
            FpCategory::Zero => (
                Some(0),
                if self.sign {
                    FpStatus::Inexact
                } else {
                    FpStatus::Ok
                },
            ),
            FpCategory::Subnormal | FpCategory::Normal => {
                if let Some((v, exact)) =
                    Self::to_int128_inner(self.sign, self.exp, self.mant, round)
                {
                    if (self.sign && v != 0) || v > u128::MAX >> (128 - bits) {
                        (None, FpStatus::Overflow)
                    } else if exact {
                        (Some(v), FpStatus::Ok)
                    } else {
                        (Some(v), FpStatus::Inexact)
                    }
                } else {
                    (None, FpStatus::Overflow)
                }
            }
        }
    }

    pub(crate) fn to_int(self, bits: u32, round: Round) -> (Option<i128>, FpStatus) {
        assert!(matches!(bits, 1..=128), "invalid number of bits: {bits}");
        const {
            assert!(S::PREC_BITS < 128);
        }
        match self.category {
            FpCategory::Nan => (None, FpStatus::Invalid),
            FpCategory::Infinite => (None, FpStatus::Overflow),
            FpCategory::Zero => (
                Some(0),
                if self.sign {
                    FpStatus::Inexact
                } else {
                    FpStatus::Ok
                },
            ),
            FpCategory::Subnormal | FpCategory::Normal => {
                if let Some((v, exact)) =
                    Self::to_int128_inner(self.sign, self.exp, self.mant, round)
                {
                    if self.sign {
                        if v == 0 {
                            (Some(0), FpStatus::Inexact)
                        } else if v > 1 << (bits - 1) {
                            (None, FpStatus::Overflow)
                        } else {
                            let v = v.wrapping_neg() as i128;
                            if exact {
                                (Some(v), FpStatus::Ok)
                            } else {
                                (Some(v), FpStatus::Inexact)
                            }
                        }
                    } else if v > (u128::MAX >> 1) >> (128 - bits) {
                        (None, FpStatus::Overflow)
                    } else if exact {
                        (Some(v as i128), FpStatus::Ok)
                    } else {
                        (Some(v as i128), FpStatus::Inexact)
                    }
                } else {
                    (None, FpStatus::Overflow)
                }
            }
        }
    }

    #[inline]
    fn to_int128_inner(
        sign: bool,
        exp: S::Exp,
        mant: S::Mant,
        round: Round,
    ) -> Option<(u128, bool)> {
        let exp: i32 = exp.into();
        if exp <= -2 {
            let v = Self::apply_round(sign, 0, RoundLoss::HalfDown, round);
            Some((v, false))
        } else if exp < S::MANT_FRAC_BITS as i32 {
            let shift = (S::MANT_FRAC_BITS as i32 - exp) as u32;
            let loss = RoundLoss::from_shift(mant, shift);
            let v = Self::apply_round(sign, mant >> shift, loss, round).into();
            Some((v, loss == RoundLoss::Zero))
        } else if exp < 128 {
            let shift = exp as u32 - S::MANT_FRAC_BITS;
            let mant: u128 = mant.into();
            Some((mant << shift, true))
        } else {
            None
        }
    }

    pub(crate) fn from_str(
        s: &str,
        round: Round,
    ) -> Result<(Self, FpStatus), crate::ParseFloatError> {
        core_num::dec2flt::dec2flt(s, round)
    }

    pub(crate) fn fmt_debug(self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core_num::fmt_float::float_to_general_debug(fmt, &self)
    }

    pub(crate) fn fmt_display(self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core_num::fmt_float::float_to_decimal_display(fmt, &self)
    }

    pub(crate) fn fmt_exp(
        self,
        fmt: &mut core::fmt::Formatter<'_>,
        upper: bool,
    ) -> core::fmt::Result {
        core_num::fmt_float::float_to_exponential_common(fmt, &self, upper)
    }

    pub(crate) fn round_int(self, round: Round) -> (Self, FpStatus) {
        match self.category {
            FpCategory::Nan => {
                let (res_value, is_signaling) = self.propagate_nan();
                if is_signaling {
                    (res_value, FpStatus::Invalid)
                } else {
                    (res_value, FpStatus::Ok)
                }
            }
            FpCategory::Infinite => (self, FpStatus::Ok),
            FpCategory::Zero => (self, FpStatus::Ok),
            FpCategory::Subnormal | FpCategory::Normal => {
                if self.exp <= S::Exp::from(-2i8) {
                    let res_value = match (round, self.sign) {
                        (Round::NearestTiesToEven, _)
                        | (Round::NearestTiesToAway, _)
                        | (Round::TowardPositive, true)
                        | (Round::TowardNegative, false)
                        | (Round::TowardZero, _) => Self::make_zero(self.sign),
                        (Round::TowardNegative, true) | (Round::TowardPositive, false) => Self {
                            category: FpCategory::Normal,
                            sign: self.sign,
                            exp: S::Exp::ZERO,
                            mant: S::Mant::ONE << S::MANT_FRAC_BITS,
                        },
                    };
                    (res_value, FpStatus::Inexact)
                } else if self.exp < S::Exp::cast_from(S::MANT_FRAC_BITS) {
                    let shift: u32 = (S::Exp::cast_from(S::MANT_FRAC_BITS) - self.exp).cast_into();
                    let loss = RoundLoss::from_shift(self.mant, shift);
                    let mant = Self::apply_round(self.sign, self.mant >> shift, loss, round);
                    let mut exp = self.exp;
                    let mut mant = mant << shift;
                    if mant >= S::Mant::ONE << S::PREC_BITS {
                        exp += S::Exp::ONE;
                        mant >>= 1;
                    }
                    if exp > S::max_normal_exp() {
                        (Self::make_inf(self.sign), FpStatus::Overflow)
                    } else {
                        (
                            Self {
                                category: FpCategory::Normal,
                                sign: self.sign,
                                exp,
                                mant,
                            },
                            if loss == RoundLoss::Zero {
                                FpStatus::Ok
                            } else {
                                FpStatus::Inexact
                            },
                        )
                    }
                } else {
                    (self, FpStatus::Ok)
                }
            }
        }
    }

    pub(crate) fn scalbn(self, exp: i32, round: Round) -> (Self, FpStatus) {
        match self.category {
            FpCategory::Nan => {
                let (res_value, is_signaling) = self.propagate_nan();
                if is_signaling {
                    (res_value, FpStatus::Invalid)
                } else {
                    (res_value, FpStatus::Ok)
                }
            }
            FpCategory::Infinite => (self, FpStatus::Ok),
            FpCategory::Zero => (self, FpStatus::Ok),
            FpCategory::Subnormal | FpCategory::Normal => {
                match S::Exp::try_from(exp)
                    .ok()
                    .and_then(|exp| self.exp.checked_add(exp))
                {
                    None => {
                        if exp > 0 {
                            (
                                Self::make_overflow_value(self.sign, round),
                                FpStatus::Overflow,
                            )
                        } else {
                            (
                                Self::make_underflow_value(self.sign, round),
                                FpStatus::Underflow,
                            )
                        }
                    }
                    Some(exp) => {
                        Self::round_and_classify(self.sign, exp, self.mant, RoundLoss::Zero, round)
                    }
                }
            }
        }
    }

    pub(crate) fn frexp(self) -> (Self, i32) {
        match self.category {
            FpCategory::Nan => (self, 0),
            FpCategory::Infinite => (self, 0),
            FpCategory::Zero => (self, 0),
            FpCategory::Subnormal | FpCategory::Normal => {
                let exp: i32 = self.exp.into();
                (
                    Self {
                        category: FpCategory::Normal,
                        sign: self.sign,
                        exp: -S::Exp::ONE,
                        mant: self.mant,
                    },
                    exp + 1,
                )
            }
        }
    }

    pub(crate) fn add(self, rhs: Self, round: Round) -> (Self, FpStatus) {
        if let Some(r) = self.handle_binary_op_nan(rhs) {
            return r;
        }

        match (self.category, rhs.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => {
                unreachable!();
            }
            (FpCategory::Infinite, FpCategory::Infinite) => {
                if self.sign == rhs.sign {
                    (self, FpStatus::Ok)
                } else {
                    (Self::make_default_qnan(), FpStatus::Invalid)
                }
            }
            (FpCategory::Infinite, _) => (self, FpStatus::Ok),
            (_, FpCategory::Infinite) => (rhs, FpStatus::Ok),
            (FpCategory::Zero, FpCategory::Zero) => {
                if self.sign == rhs.sign {
                    (self, FpStatus::Ok)
                } else {
                    (Self::make_zero_sum(round), FpStatus::Ok)
                }
            }
            (FpCategory::Zero, FpCategory::Normal | FpCategory::Subnormal) => (rhs, FpStatus::Ok),
            (FpCategory::Normal | FpCategory::Subnormal, FpCategory::Zero) => (self, FpStatus::Ok),
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => {
                if self.sign == rhs.sign {
                    match (self.exp, self.mant).cmp(&(rhs.exp, rhs.mant)) {
                        core::cmp::Ordering::Less | core::cmp::Ordering::Equal => Self::add_inner(
                            self.sign, rhs.exp, rhs.mant, self.exp, self.mant, round,
                        ),
                        core::cmp::Ordering::Greater => Self::add_inner(
                            self.sign, self.exp, self.mant, rhs.exp, rhs.mant, round,
                        ),
                    }
                } else {
                    match (self.exp, self.mant).cmp(&(rhs.exp, rhs.mant)) {
                        core::cmp::Ordering::Less => {
                            Self::sub_inner(rhs.sign, rhs.exp, rhs.mant, self.exp, self.mant, round)
                        }
                        core::cmp::Ordering::Equal => (Self::make_zero_sum(round), FpStatus::Ok),
                        core::cmp::Ordering::Greater => Self::sub_inner(
                            self.sign, self.exp, self.mant, rhs.exp, rhs.mant, round,
                        ),
                    }
                }
            }
        }
    }

    pub(crate) fn sub(self, rhs: Self, round: Round) -> (Self, FpStatus) {
        if let Some(r) = self.handle_binary_op_nan(rhs) {
            return r;
        }

        match (self.category, rhs.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => {
                unreachable!();
            }
            (FpCategory::Infinite, FpCategory::Infinite) => {
                if self.sign == rhs.sign {
                    (Self::make_default_qnan(), FpStatus::Invalid)
                } else {
                    (self, FpStatus::Invalid)
                }
            }
            (FpCategory::Infinite, _) => (self, FpStatus::Ok),
            (_, FpCategory::Infinite) => (-rhs, FpStatus::Ok),
            (FpCategory::Zero, FpCategory::Zero) => {
                if self.sign == rhs.sign {
                    (Self::make_zero_sum(round), FpStatus::Ok)
                } else {
                    (self, FpStatus::Ok)
                }
            }
            (FpCategory::Zero, FpCategory::Normal | FpCategory::Subnormal) => (-rhs, FpStatus::Ok),
            (FpCategory::Normal | FpCategory::Subnormal, FpCategory::Zero) => (self, FpStatus::Ok),
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => {
                if self.sign == rhs.sign {
                    match (self.exp, self.mant).cmp(&(rhs.exp, rhs.mant)) {
                        core::cmp::Ordering::Less => Self::sub_inner(
                            !self.sign, rhs.exp, rhs.mant, self.exp, self.mant, round,
                        ),
                        core::cmp::Ordering::Equal => (Self::make_zero_sum(round), FpStatus::Ok),
                        core::cmp::Ordering::Greater => Self::sub_inner(
                            self.sign, self.exp, self.mant, rhs.exp, rhs.mant, round,
                        ),
                    }
                } else {
                    match (self.exp, self.mant).cmp(&(rhs.exp, rhs.mant)) {
                        core::cmp::Ordering::Less | core::cmp::Ordering::Equal => Self::add_inner(
                            self.sign, rhs.exp, rhs.mant, self.exp, self.mant, round,
                        ),
                        core::cmp::Ordering::Greater => Self::add_inner(
                            self.sign, self.exp, self.mant, rhs.exp, rhs.mant, round,
                        ),
                    }
                }
            }
        }
    }

    fn add_inner(
        sign: bool,
        lhs_exp: S::Exp,
        lhs_mant: S::Mant,
        rhs_exp: S::Exp,
        rhs_mant: S::Mant,
        round: Round,
    ) -> (Self, FpStatus) {
        debug_assert!(lhs_exp >= rhs_exp);

        let shift: u32 = (lhs_exp - rhs_exp).cast_into();
        let (mut mant, mut loss) = if shift > S::PREC_BITS + 1 {
            (lhs_mant, RoundLoss::HalfDown)
        } else if shift == 0 {
            (lhs_mant + rhs_mant, RoundLoss::Zero)
        } else {
            let mant = lhs_mant + (rhs_mant >> shift);
            let loss = RoundLoss::from_shift(rhs_mant, shift);
            (mant, loss)
        };

        let mut exp = lhs_exp;
        if mant >= S::Mant::ONE << S::PREC_BITS {
            loss = RoundLoss::from_shift_and(mant, 1, loss);
            mant >>= 1;
            exp += S::Exp::ONE;
        }

        Self::round_and_classify(sign, exp, mant, loss, round)
    }

    fn sub_inner(
        sign: bool,
        lhs_exp: S::Exp,
        lhs_mant: S::Mant,
        rhs_exp: S::Exp,
        rhs_mant: S::Mant,
        round: Round,
    ) -> (Self, FpStatus) {
        debug_assert!(lhs_exp >= rhs_exp);

        let mut exp = lhs_exp;
        let shift: u32 = (lhs_exp - rhs_exp).cast_into();
        let (mant, loss) = if shift > S::PREC_BITS + 2 {
            let mut mant = lhs_mant - S::Mant::ONE;
            if mant < S::Mant::ONE << S::MANT_FRAC_BITS {
                mant = (S::Mant::ONE << S::PREC_BITS) - S::Mant::ONE;
                exp -= S::Exp::ONE;
            }
            (mant, RoundLoss::HalfUp)
        } else {
            let rhs_neg = rhs_mant.wrapping_neg();
            // double-not to simulate arithmetic shift of negative number
            let rhs_neg_shift = !(!rhs_neg >> shift);

            let mut loss_shift = shift;
            let mut mant = lhs_mant.wrapping_add(rhs_neg_shift);
            if mant < S::Mant::ONE << S::MANT_FRAC_BITS {
                let initial_loss_bits = rhs_neg & !(S::Mant::MAX << loss_shift);
                let lshift = mant.leading_zeros() - (S::Mant::BITS - S::PREC_BITS);
                if lshift > loss_shift {
                    mant = (mant << lshift) + (initial_loss_bits << (lshift - loss_shift));
                    loss_shift = 0;
                } else {
                    loss_shift -= lshift;
                    mant = (mant << lshift) + (initial_loss_bits >> loss_shift);
                }
                exp -= S::Exp::cast_from(lshift);
            }
            let loss = if loss_shift == 0 {
                RoundLoss::Zero
            } else {
                RoundLoss::from_shift(rhs_neg, loss_shift)
            };
            (mant, loss)
        };

        Self::round_and_classify(sign, exp, mant, loss, round)
    }

    pub(crate) fn mul(self, rhs: Self, round: Round) -> (Self, FpStatus) {
        if let Some(r) = self.handle_binary_op_nan(rhs) {
            return r;
        }

        match (self.category, rhs.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => {
                unreachable!();
            }
            (FpCategory::Infinite, FpCategory::Zero) | (FpCategory::Zero, FpCategory::Infinite) => {
                (Self::make_default_qnan(), FpStatus::Invalid)
            }
            (FpCategory::Infinite, _) | (_, FpCategory::Infinite) => {
                (Self::make_inf(self.sign ^ rhs.sign), FpStatus::Ok)
            }
            (FpCategory::Zero, _) | (_, FpCategory::Zero) => {
                (Self::make_zero(self.sign ^ rhs.sign), FpStatus::Ok)
            }
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => {
                let mut exp = self.exp + rhs.exp;
                let (p_lo, p_hi) = self.mant.wide_mul(rhs.mant);
                let (cmp_lo, cmp_hi) =
                    utils::wide_shift_left(S::Mant::ONE, S::Mant::ZERO, S::MANT_FRAC_BITS * 2 + 1);
                let (mant, loss) = if (p_hi, p_lo) >= (cmp_hi, cmp_lo) {
                    exp += S::Exp::ONE;
                    let shift = S::MANT_FRAC_BITS + 1;
                    let loss = RoundLoss::from_wide_shift(p_lo, p_hi, shift);
                    (utils::wide_shift_right(p_lo, p_hi, shift).0, loss)
                } else {
                    let shift = S::MANT_FRAC_BITS;
                    let loss = RoundLoss::from_wide_shift(p_lo, p_hi, shift);
                    (utils::wide_shift_right(p_lo, p_hi, shift).0, loss)
                };
                Self::round_and_classify(self.sign ^ rhs.sign, exp, mant, loss, round)
            }
        }
    }

    pub(crate) fn div(self, rhs: Self, round: Round) -> (Self, FpStatus) {
        if let Some(r) = self.handle_binary_op_nan(rhs) {
            return r;
        }

        match (self.category, rhs.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => {
                unreachable!();
            }
            (FpCategory::Infinite, FpCategory::Infinite) | (FpCategory::Zero, FpCategory::Zero) => {
                (Self::make_default_qnan(), FpStatus::Invalid)
            }
            (FpCategory::Infinite | FpCategory::Zero, _) => {
                (self.set_sign(self.sign ^ rhs.sign), FpStatus::Ok)
            }
            (_, FpCategory::Infinite) => (Self::make_zero(self.sign ^ rhs.sign), FpStatus::Ok),
            (_, FpCategory::Zero) => (Self::make_inf(self.sign ^ rhs.sign), FpStatus::DivByZero),
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => {
                let mut exp = self.exp - rhs.exp;

                let mut q = S::Mant::ZERO;
                let mut r = self.mant;
                let d = rhs.mant;

                if r < d {
                    r <<= 1;
                    exp -= S::Exp::ONE;
                }

                for bit in (0..S::PREC_BITS).rev() {
                    if r >= d {
                        q |= S::Mant::ONE << bit;
                        r -= d;
                    }
                    r <<= 1;
                }

                let loss = if r == S::Mant::ZERO {
                    RoundLoss::Zero
                } else {
                    match r.cmp(&d) {
                        core::cmp::Ordering::Less => RoundLoss::HalfDown,
                        core::cmp::Ordering::Equal => RoundLoss::Halfway,
                        core::cmp::Ordering::Greater => RoundLoss::HalfUp,
                    }
                };

                Self::round_and_classify(self.sign ^ rhs.sign, exp, q, loss, round)
            }
        }
    }

    pub(crate) fn rem(self, rhs: Self) -> (Self, FpStatus) {
        if let Some(r) = self.handle_binary_op_nan(rhs) {
            return r;
        }

        match (self.category, rhs.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => {
                unreachable!();
            }
            (FpCategory::Infinite, _) | (_, FpCategory::Zero) => {
                (Self::make_default_qnan(), FpStatus::Invalid)
            }
            (FpCategory::Zero, _) | (_, FpCategory::Infinite) => (self, FpStatus::Ok),
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => {
                let mut exp = self.exp;
                let mut mant = self.mant;
                while exp > rhs.exp || (exp == rhs.exp && mant >= rhs.mant) {
                    match mant.cmp(&rhs.mant) {
                        core::cmp::Ordering::Less => {
                            exp -= S::Exp::ONE;
                            mant <<= 1;
                        }
                        core::cmp::Ordering::Equal => {
                            return (Self::make_zero(self.sign), FpStatus::Ok);
                        }
                        core::cmp::Ordering::Greater => {}
                    }
                    mant -= rhs.mant;
                    let shift = mant.leading_zeros() - (S::Mant::BITS - S::PREC_BITS);
                    exp -= S::Exp::cast_from(shift);
                    mant <<= shift;
                }
                (
                    Self {
                        category: if exp >= S::min_normal_exp() {
                            FpCategory::Normal
                        } else {
                            FpCategory::Subnormal
                        },
                        sign: self.sign,
                        exp,
                        mant,
                    },
                    FpStatus::Ok,
                )
            }
        }
    }

    pub(crate) fn sqrt(self, round: Round) -> (Self, FpStatus) {
        match self.category {
            FpCategory::Nan => {
                let (res_value, is_signaling) = self.propagate_nan();
                if is_signaling {
                    (res_value, FpStatus::Invalid)
                } else {
                    (res_value, FpStatus::Ok)
                }
            }
            FpCategory::Infinite => {
                if self.sign {
                    (Self::make_default_qnan(), FpStatus::Invalid)
                } else {
                    (self, FpStatus::Ok)
                }
            }
            FpCategory::Zero => (self, FpStatus::Ok),
            FpCategory::Subnormal | FpCategory::Normal => {
                if self.sign {
                    return (Self::make_default_qnan(), FpStatus::Invalid);
                }

                let (m, e) = if self.exp & S::Exp::ONE == S::Exp::ZERO {
                    (self.mant, self.exp)
                } else {
                    (self.mant << 1, self.exp - S::Exp::ONE)
                };

                // Calculate sqrt(m) bit-by-bit with one extra bit
                let mut rem = m << 1; // extra bit
                let mut res = S::Mant::ZERO;
                let mut s = S::Mant::ZERO;
                let mut d = S::Mant::ONE << (S::MANT_FRAC_BITS + 1);

                while d != S::Mant::ZERO {
                    let t = s + d;
                    if rem >= t {
                        s += d + d;
                        rem -= t;
                        res += d;
                    }
                    rem <<= 1;
                    d >>= 1;
                }

                // Remove extra bit
                let extra = (res & S::Mant::ONE) != S::Mant::ZERO;
                res >>= 1;

                // Calculate loss from extra bit and remainder
                let loss = match (extra, rem != S::Mant::ZERO) {
                    (false, false) => RoundLoss::Zero,
                    (true, false) => RoundLoss::Halfway,
                    (false, true) => RoundLoss::HalfDown,
                    (true, true) => RoundLoss::HalfUp,
                };

                // Build result with e/2 as exponent and res as mantissa.
                Self::round_and_classify(false, e / S::Exp::TWO, res, loss, round)
            }
        }
    }

    pub(crate) fn hypot(self, other: Self, round: Round) -> (Self, FpStatus) {
        if self.category == FpCategory::Infinite {
            if other.category == FpCategory::Nan && !other.nan_is_quiet() {
                return (other.propagate_nan().0, FpStatus::Invalid);
            } else {
                return (self.abs(), FpStatus::Ok);
            }
        } else if other.category == FpCategory::Infinite {
            if self.category == FpCategory::Nan && !self.nan_is_quiet() {
                return (self.propagate_nan().0, FpStatus::Invalid);
            } else {
                return (other.abs(), FpStatus::Ok);
            }
        } else if let Some(r) = self.handle_binary_op_nan(other) {
            return r;
        } else if self.category == FpCategory::Zero {
            return (other.abs(), FpStatus::Ok);
        } else if other.category == FpCategory::Zero {
            return (self.abs(), FpStatus::Ok);
        }

        let mut a_exp = self.exp * S::Exp::TWO;
        let (mut a_lo, mut a_hi) = self.mant.wide_mul(self.mant);

        let mut b_exp = other.exp * S::Exp::TWO;
        let (mut b_lo, mut b_hi) = other.mant.wide_mul(other.mant);

        if b_exp > a_exp {
            core::mem::swap(&mut a_exp, &mut b_exp);
            core::mem::swap(&mut a_lo, &mut b_lo);
            core::mem::swap(&mut a_hi, &mut b_hi);
        }

        // Shift to have MANT_FRAC_BITS or MANT_FRAC_BITS+1 bits in a_hi.
        let shift = S::Mant::BITS - S::MANT_FRAC_BITS;
        let (a_lo, a_hi) = utils::wide_shift_left(a_lo, a_hi, shift);
        let (b_lo, b_hi) = utils::wide_shift_left(b_lo, b_hi, shift);

        let mut extra_loss;
        let mut sum_lo;
        let mut sum_hi;
        if a_exp == b_exp {
            extra_loss = false;
            let carry;
            (sum_lo, carry) = a_lo.overflowing_add(b_lo);
            sum_hi = a_hi + b_hi + S::Mant::from(carry);
        } else if a_exp - b_exp < S::Exp::cast_from(S::Mant::BITS + S::PREC_BITS + 1) {
            let shift: u32 = (a_exp - b_exp).cast_into();
            extra_loss = RoundLoss::from_wide_shift(b_lo, b_hi, shift) != RoundLoss::Zero;
            let (b_shift_lo, b_shift_hi) = utils::wide_shift_right(b_lo, b_hi, shift);
            let carry;
            (sum_lo, carry) = a_lo.overflowing_add(b_shift_lo);
            sum_hi = a_hi + b_shift_hi + S::Mant::from(carry);
        } else {
            extra_loss = true;
            sum_lo = a_lo;
            sum_hi = a_hi;
        }

        let mut sum_exp = a_exp;

        if sum_hi >= S::Mant::ONE << S::PREC_BITS {
            extra_loss |= sum_lo & S::Mant::ONE != S::Mant::ZERO;
            (sum_lo, sum_hi) = utils::wide_shift_right(sum_lo, sum_hi, 1);
            sum_exp += S::Exp::ONE;

            if sum_hi >= S::Mant::ONE << S::PREC_BITS {
                extra_loss |= sum_lo & S::Mant::ONE != S::Mant::ZERO;
                (sum_lo, sum_hi) = utils::wide_shift_right(sum_lo, sum_hi, 1);
                sum_exp += S::Exp::ONE;
            }
        }

        if sum_exp & S::Exp::ONE != S::Exp::ZERO {
            (sum_lo, sum_hi) = utils::wide_shift_left(sum_lo, sum_hi, 1);
            sum_exp -= S::Exp::ONE;
        }

        // Calculate sqrt(sum) bit-by-bit with one extra bit
        let (mut rem_lo, mut rem_hi) = utils::wide_shift_left(sum_lo, sum_hi, 1); // extra bit
        let mut res = S::Mant::ZERO;
        let mut s = S::Mant::ZERO;
        let mut d = S::Mant::ONE << (S::MANT_FRAC_BITS + 1);

        while d != S::Mant::ZERO {
            let t = s + d;
            if rem_hi >= t {
                s += d + d;
                rem_hi -= t;
                res += d;
            }
            (rem_lo, rem_hi) = utils::wide_shift_left(rem_lo, rem_hi, 1);
            d >>= 1;
        }

        // Remove extra bit
        let extra_bit = (res & S::Mant::ONE) != S::Mant::ZERO;
        res >>= 1;

        // Calculate loss from extra bit and remainder
        let rem_is_non_zero = extra_loss || rem_hi != S::Mant::ZERO || rem_lo != S::Mant::ZERO;
        let loss = match (extra_bit, rem_is_non_zero) {
            (false, false) => RoundLoss::Zero,
            (true, false) => RoundLoss::Halfway,
            (false, true) => RoundLoss::HalfDown,
            (true, true) => RoundLoss::HalfUp,
        };

        // Build result with e/2 as exponent and res as mantissa.
        Self::round_and_classify(false, sum_exp / S::Exp::TWO, res, loss, round)
    }

    #[inline]
    fn apply_round<T: UInt>(sign: bool, value: T, loss: RoundLoss, round: Round) -> T {
        let bump = match round {
            Round::NearestTiesToEven => match loss {
                RoundLoss::Zero => false,
                RoundLoss::Halfway => value & T::ONE != T::ZERO,
                RoundLoss::HalfUp => true,
                RoundLoss::HalfDown => false,
            },
            Round::NearestTiesToAway => match loss {
                RoundLoss::Zero => false,
                RoundLoss::Halfway => true,
                RoundLoss::HalfUp => true,
                RoundLoss::HalfDown => false,
            },
            Round::TowardPositive => match (loss, sign) {
                (RoundLoss::Zero, _) => false,
                (_, false) => true,
                (_, true) => false,
            },
            Round::TowardNegative => match (loss, sign) {
                (RoundLoss::Zero, _) => false,
                (_, false) => false,
                (_, true) => true,
            },
            Round::TowardZero => false,
        };
        value + T::from(bump)
    }

    pub(crate) fn round_and_classify(
        sign: bool,
        mut exp: S::Exp,
        mut mant: S::Mant,
        mut loss: RoundLoss,
        round: Round,
    ) -> (Self, FpStatus) {
        debug_assert!(exp < S::Exp::MAX);
        debug_assert!(mant >= S::Mant::ONE << S::MANT_FRAC_BITS);
        debug_assert!(mant < S::Mant::ONE << S::PREC_BITS);

        if exp < S::min_subnormal_exp() - S::Exp::ONE {
            return (Self::make_underflow_value(sign, round), FpStatus::Underflow);
        }

        let mut shift = 0u32;
        if exp < S::min_normal_exp() {
            shift = (S::min_normal_exp() - exp).cast_into();
            loss = RoundLoss::from_shift_and(mant, shift, loss);
            mant >>= shift;
        }

        mant = Self::apply_round(sign, mant, loss, round);
        mant <<= shift;
        if mant > (S::Mant::ONE << S::PREC_BITS) - S::Mant::ONE {
            debug_assert_eq!(mant & S::Mant::ONE, S::Mant::ZERO);
            mant >>= 1;
            exp += S::Exp::ONE;
        }

        let is_overflow = if S::FORMAT == Format::NoInfNanAllOnes {
            exp > S::max_normal_exp()
                || (exp == S::max_normal_exp()
                    && mant == (S::Mant::ONE << S::PREC_BITS) - S::Mant::ONE)
        } else {
            exp > S::max_normal_exp()
        };

        if is_overflow {
            (Self::make_overflow_value(sign, round), FpStatus::Overflow)
        } else if exp < S::min_subnormal_exp() {
            debug_assert_eq!(exp, S::min_subnormal_exp() - S::Exp::ONE);
            (Self::make_zero(sign), FpStatus::Underflow)
        } else if exp < S::min_normal_exp() {
            (
                Self {
                    category: FpCategory::Subnormal,
                    sign,
                    exp,
                    mant,
                },
                if loss == RoundLoss::Zero {
                    FpStatus::Ok
                } else {
                    FpStatus::Underflow
                },
            )
        } else {
            (
                Self {
                    category: FpCategory::Normal,
                    sign,
                    exp,
                    mant,
                },
                if loss == RoundLoss::Zero {
                    FpStatus::Ok
                } else {
                    FpStatus::Inexact
                },
            )
        }
    }

    #[inline]
    fn handle_binary_op_nan(self, rhs: Self) -> Option<(Self, FpStatus)> {
        match (self.category, rhs.category) {
            (FpCategory::Nan, FpCategory::Nan) => {
                let (res_value, lhs_is_signaling) = self.propagate_nan();
                if lhs_is_signaling || !rhs.nan_is_quiet() {
                    Some((res_value, FpStatus::Invalid))
                } else {
                    Some((res_value, FpStatus::Ok))
                }
            }
            (FpCategory::Nan, _) => {
                let (res_value, signaling) = self.propagate_nan();
                if signaling {
                    Some((res_value, FpStatus::Invalid))
                } else {
                    Some((res_value, FpStatus::Ok))
                }
            }
            (_, FpCategory::Nan) => {
                let (res_value, signaling) = rhs.propagate_nan();
                if signaling {
                    Some((res_value, FpStatus::Invalid))
                } else {
                    Some((res_value, FpStatus::Ok))
                }
            }
            _ => None,
        }
    }

    #[inline]
    fn make_zero_sum(round: Round) -> Self {
        // When adding two values with opposite signs and equal magnitudes, the result
        // is +0 except when the rounding mode is TowardNegative, in which case the
        // result is -0.
        Self::make_zero(round == Round::TowardNegative)
    }

    #[inline]
    fn make_overflow_value(sign: bool, round: Round) -> Self {
        match (round, sign) {
            (Round::NearestTiesToEven, _)
            | (Round::NearestTiesToAway, _)
            | (Round::TowardPositive, false)
            | (Round::TowardNegative, true) => Self::make_inf(sign),
            (Round::TowardPositive, true)
            | (Round::TowardNegative, false)
            | (Round::TowardZero, _) => Self::make_largest_finite(sign),
        }
    }

    #[inline]
    fn make_underflow_value(sign: bool, round: Round) -> Self {
        match (round, sign) {
            (Round::NearestTiesToEven, _)
            | (Round::NearestTiesToAway, _)
            | (Round::TowardPositive, true)
            | (Round::TowardNegative, false)
            | (Round::TowardZero, _) => Self::make_zero(sign),
            (Round::TowardPositive, false) | (Round::TowardNegative, true) => {
                Self::make_smallest_subnormal(sign)
            }
        }
    }

    #[inline]
    pub(crate) fn make_inf(sign: bool) -> Self {
        match S::FORMAT {
            Format::Ieee => Self {
                category: FpCategory::Infinite,
                sign,
                exp: S::max_normal_exp() + S::Exp::ONE,
                mant: S::Mant::ZERO,
            },
            Format::X87 => Self {
                category: FpCategory::Infinite,
                sign,
                exp: S::max_normal_exp() + S::Exp::ONE,
                mant: S::Mant::ONE << S::MANT_FRAC_BITS,
            },
            // Formats without infinity, try to preserve the sign if possible.
            Format::NoInfNanAllOnes | Format::NoInfNanNegZero => {
                Self::make_qnan::<true>(sign, S::Mant::ZERO)
            }
        }
    }

    #[inline]
    pub(crate) fn make_zero(sign: bool) -> Self {
        if S::FORMAT == Format::NoInfNanNegZero {
            Self {
                category: FpCategory::Zero,
                sign: false, // format does not have negative zero
                exp: S::min_normal_exp() - S::Exp::ONE,
                mant: S::Mant::ZERO,
            }
        } else {
            Self {
                category: FpCategory::Zero,
                sign,
                exp: S::min_normal_exp() - S::Exp::ONE,
                mant: S::Mant::ZERO,
            }
        }
    }

    #[inline]
    fn make_largest_finite(sign: bool) -> Self {
        Self {
            category: FpCategory::Normal,
            sign,
            exp: S::max_normal_exp(),
            mant: if S::FORMAT == Format::NoInfNanAllOnes {
                (S::Mant::ONE << S::PREC_BITS) - S::Mant::TWO
            } else {
                (S::Mant::ONE << S::PREC_BITS) - S::Mant::ONE
            },
        }
    }

    #[inline]
    fn make_smallest_subnormal(sign: bool) -> Self {
        Self {
            category: FpCategory::Subnormal,
            sign,
            exp: S::min_subnormal_exp(),
            mant: S::Mant::ONE << S::MANT_FRAC_BITS,
        }
    }

    #[inline]
    fn nan_is_quiet(self) -> bool {
        !self.get_nan_info().1
    }

    #[inline]
    fn get_nan_info(self) -> (S::Mant, bool) {
        debug_assert_eq!(self.category, FpCategory::Nan);

        match S::FORMAT {
            Format::Ieee => {
                let quiet_mask = S::Mant::ONE << (S::MANT_FRAC_BITS - 1);
                let signaling = self.mant & quiet_mask == S::Mant::ZERO;
                (self.mant, signaling)
            }
            Format::X87 => {
                let int_mask = S::Mant::ONE << S::MANT_FRAC_BITS;
                if self.exp != S::max_normal_exp() + S::Exp::ONE
                    || self.mant & int_mask == S::Mant::ZERO
                {
                    // invalid operand
                    (S::Mant::ZERO, true)
                } else {
                    // proper NaN
                    let quiet_mask = S::Mant::ONE << (S::MANT_FRAC_BITS - 1);
                    let signaling = self.mant & quiet_mask == S::Mant::ZERO;
                    (self.mant, signaling)
                }
            }
            Format::NoInfNanAllOnes | Format::NoInfNanNegZero => {
                // Source format does not have NaN payload.
                (S::Mant::ZERO, false)
            }
        }
    }

    #[inline]
    fn propagate_nan(self) -> (Self, bool) {
        let (payload, signaling) = self.get_nan_info();
        (Self::make_qnan::<true>(self.sign, payload), signaling)
    }

    #[inline]
    fn make_qnan<const SELF_PAYLOAD: bool>(sign: bool, payload: S::Mant) -> Self {
        match S::FORMAT {
            Format::Ieee => {
                let quiet_mask = S::Mant::ONE << (S::MANT_FRAC_BITS - 1);
                let payload_mask = if SELF_PAYLOAD {
                    S::Mant::MAX
                } else {
                    quiet_mask - S::Mant::ONE
                };
                let payload = payload & payload_mask;
                Self {
                    category: FpCategory::Nan,
                    sign,
                    exp: S::max_normal_exp() + S::Exp::ONE,
                    mant: quiet_mask | payload,
                }
            }
            Format::X87 => {
                let quiet_mask = S::Mant::from(0b11_u8) << (S::MANT_FRAC_BITS - 1);
                let payload_mask = if SELF_PAYLOAD {
                    S::Mant::MAX
                } else {
                    (S::Mant::ONE << (S::MANT_FRAC_BITS - 1)) - S::Mant::ONE
                };
                let payload = payload & payload_mask;
                Self {
                    category: FpCategory::Nan,
                    sign,
                    exp: S::max_normal_exp() + S::Exp::ONE,
                    mant: quiet_mask | payload,
                }
            }
            Format::NoInfNanAllOnes => Self {
                category: FpCategory::Nan,
                sign,
                exp: S::max_normal_exp(),
                mant: (S::Mant::ONE << S::MANT_FRAC_BITS) - S::Mant::ONE,
            },
            Format::NoInfNanNegZero => Self {
                category: FpCategory::Nan,
                sign: true,
                exp: S::min_normal_exp() - S::Exp::ONE,
                mant: S::Mant::ZERO,
            },
        }
    }

    #[inline]
    pub(crate) fn make_default_qnan() -> Self {
        Self::make_qnan::<true>(false, S::Mant::ZERO)
    }

    fn check_consistency(self) {
        match self.category {
            FpCategory::Nan => match S::FORMAT {
                Format::Ieee => {
                    assert_eq!(self.exp, S::max_normal_exp() + S::Exp::ONE);
                    assert_ne!(self.mant, S::Mant::ZERO);
                    assert!(self.mant < S::Mant::ONE << S::MANT_FRAC_BITS);
                }
                Format::X87 => {
                    assert!(self.mant < S::Mant::ONE << S::PREC_BITS);
                    if self.exp == S::max_normal_exp() + S::Exp::ONE {
                        assert!(
                            self.mant >> S::MANT_FRAC_BITS == S::Mant::ZERO
                                || self.mant & !(S::Mant::ONE << S::MANT_FRAC_BITS)
                                    != S::Mant::ZERO
                        );
                    } else {
                        assert!(self.mant >> S::MANT_FRAC_BITS == S::Mant::ZERO);
                    }
                }
                Format::NoInfNanAllOnes => {
                    assert_eq!(self.exp, S::max_normal_exp());
                    assert_eq!(
                        self.mant,
                        (S::Mant::ONE << S::MANT_FRAC_BITS) - S::Mant::ONE
                    );
                }
                Format::NoInfNanNegZero => {
                    assert!(self.sign);
                    assert_eq!(self.exp, S::min_normal_exp() - S::Exp::ONE);
                    assert_eq!(self.mant, S::Mant::ZERO);
                }
            },
            FpCategory::Infinite => match S::FORMAT {
                Format::Ieee => {
                    assert_eq!(self.exp, S::max_normal_exp() + S::Exp::ONE);
                    assert_eq!(self.mant, S::Mant::ZERO);
                }
                Format::X87 => {
                    assert_eq!(self.exp, S::max_normal_exp() + S::Exp::ONE);
                    assert_eq!(self.mant, S::Mant::ONE << S::MANT_FRAC_BITS);
                }
                Format::NoInfNanAllOnes | Format::NoInfNanNegZero => {
                    panic!("format does not have infinity");
                }
            },
            FpCategory::Zero => {
                assert_eq!(self.exp, S::min_normal_exp() - S::Exp::ONE);
                assert_eq!(self.mant, S::Mant::ZERO);
                if S::FORMAT == Format::NoInfNanNegZero {
                    assert!(!self.sign);
                }
            }
            FpCategory::Subnormal => {
                assert!(self.exp >= S::min_subnormal_exp());
                if S::FORMAT == Format::X87 {
                    assert!(self.exp <= S::min_normal_exp());
                } else {
                    assert!(self.exp < S::min_normal_exp());
                }
                assert!(self.mant >= S::Mant::ONE << S::MANT_FRAC_BITS);
                assert!(self.mant < S::Mant::ONE << S::PREC_BITS);

                let shift: u32 = (S::min_normal_exp() - self.exp).cast_into();
                assert!(self.mant & !(S::Mant::MAX << shift) == S::Mant::ZERO);
            }
            FpCategory::Normal => {
                assert!(self.exp >= S::min_normal_exp() && self.exp <= S::max_normal_exp());
                assert!(self.mant >= S::Mant::ONE << S::MANT_FRAC_BITS);
                assert!(self.mant < S::Mant::ONE << S::PREC_BITS);
                if S::FORMAT == Format::NoInfNanAllOnes {
                    assert!(
                        !(self.exp == S::max_normal_exp()
                            && self.mant == (S::Mant::ONE << S::PREC_BITS) - S::Mant::ONE)
                    );
                }
            }
        }
    }
}

impl<S: Semantics> core::fmt::Debug for IeeeFloat<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

impl<S: Semantics> PartialEq for IeeeFloat<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self.category, other.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => false,
            (FpCategory::Infinite, FpCategory::Infinite) => self.sign == other.sign,
            (FpCategory::Infinite, _) | (_, FpCategory::Infinite) => false,
            (FpCategory::Zero, FpCategory::Zero) => true,
            (FpCategory::Zero, _) | (_, FpCategory::Zero) => false,
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => self.sign == other.sign && self.exp == other.exp && self.mant == other.mant,
        }
    }
}

impl<S: Semantics> PartialOrd for IeeeFloat<S> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match (self.category, other.category) {
            (FpCategory::Nan, _) | (_, FpCategory::Nan) => None,
            (FpCategory::Infinite, FpCategory::Infinite) => Some((!self.sign).cmp(&!other.sign)),
            (FpCategory::Infinite, _) => {
                if self.sign {
                    Some(core::cmp::Ordering::Less)
                } else {
                    Some(core::cmp::Ordering::Greater)
                }
            }
            (_, FpCategory::Infinite) => {
                if other.sign {
                    Some(core::cmp::Ordering::Greater)
                } else {
                    Some(core::cmp::Ordering::Less)
                }
            }
            (FpCategory::Zero, FpCategory::Zero) => Some(core::cmp::Ordering::Equal),
            (FpCategory::Zero, _) => {
                if other.sign {
                    Some(core::cmp::Ordering::Greater)
                } else {
                    Some(core::cmp::Ordering::Less)
                }
            }
            (_, FpCategory::Zero) => {
                if self.sign {
                    Some(core::cmp::Ordering::Less)
                } else {
                    Some(core::cmp::Ordering::Greater)
                }
            }
            (
                FpCategory::Normal | FpCategory::Subnormal,
                FpCategory::Normal | FpCategory::Subnormal,
            ) => match (self.sign, other.sign) {
                (true, false) => Some(core::cmp::Ordering::Less),
                (false, true) => Some(core::cmp::Ordering::Greater),
                (false, false) => Some(
                    self.exp
                        .cmp(&other.exp)
                        .then_with(|| self.mant.cmp(&other.mant)),
                ),
                (true, true) => Some(
                    other
                        .exp
                        .cmp(&self.exp)
                        .then_with(|| other.mant.cmp(&self.mant)),
                ),
            },
        }
    }
}

impl<S: Semantics> core::ops::Neg for IeeeFloat<S> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        if S::FORMAT == Format::NoInfNanNegZero
            && matches!(self.category, FpCategory::Nan | FpCategory::Zero)
        {
            // Changing the sign would toggle between zero and NaN.
            self
        } else {
            Self {
                sign: !self.sign,
                ..self
            }
        }
    }
}

impl<S: Semantics> core_num::dec2flt::Float for IeeeFloat<S> {
    const SIG_BITS: u32 = S::MANT_FRAC_BITS;

    type Mant = S::Mant;
    type Exp = S::Exp;

    #[inline]
    fn zero(negative: bool) -> Self {
        IeeeFloat::make_zero(negative)
    }

    #[inline]
    fn inf(negative: bool) -> Self {
        Self::make_inf(negative)
    }

    #[inline]
    fn nan() -> Self {
        Self::make_default_qnan()
    }

    #[inline]
    fn overflow_value(negative: bool, round: Round) -> Self {
        IeeeFloat::make_overflow_value(negative, round)
    }

    #[inline]
    fn underflow_value(negative: bool, round: Round) -> Self {
        IeeeFloat::make_underflow_value(negative, round)
    }

    #[inline]
    fn build_value(
        negative: bool,
        exp: Self::Exp,
        mant: Self::Mant,
        loss: RoundLoss,
        round: Round,
    ) -> (Self, FpStatus) {
        IeeeFloat::round_and_classify(negative, exp, mant, loss, round)
    }
}

impl<S: Semantics> core_num::fmt_float::GeneralFormat for IeeeFloat<S> {
    fn already_rounded_value_should_use_exponential(&self) -> bool {
        let abs = self.abs();
        let e16 = IeeeFloat::from_uint(10_000_000_000_000_000, Round::NearestTiesToEven).0;
        let em4 = crate::F128::from_uint(1) / crate::F128::from_uint(10_000);
        let em4 = IeeeFloat::convert_from(em4.0, Round::NearestTiesToEven).0;
        (abs.category != FpCategory::Zero && abs < em4) || abs >= e16
    }
}

impl<S: Semantics> core_num::flt2dec::DecodableFloat for IeeeFloat<S> {
    const IS_LARGE: bool = S::PREC_BITS > 53 || S::EXP_BITS > 11;

    fn decode(self) -> (/*negative?*/ bool, core_num::flt2dec::FullDecoded) {
        use core_num::flt2dec::{Decoded, FullDecoded};
        let decoded = match self.category {
            FpCategory::Nan => FullDecoded::Nan,
            FpCategory::Infinite => FullDecoded::Infinite,
            FpCategory::Zero => FullDecoded::Zero,
            FpCategory::Normal | FpCategory::Subnormal => {
                let mant: u128 = self.mant.into();
                if self.exp < S::min_normal_exp() {
                    let shift: u32 = (S::min_normal_exp() - self.exp).cast_into();
                    let mant = mant >> (shift - 1);
                    let exp =
                        S::min_normal_exp() - S::Exp::ONE - S::Exp::cast_from(S::MANT_FRAC_BITS);
                    // neighbors: (mant - 2, exp) -- (mant, exp) -- (mant + 2, exp)
                    FullDecoded::Finite(Decoded {
                        mant,
                        minus: 1,
                        plus: 1,
                        exp: exp.cast_into(),
                        inclusive: true,
                    })
                } else {
                    let exp: i16 = (self.exp - S::Exp::cast_from(S::MANT_FRAC_BITS)).cast_into();
                    let even = (self.mant & S::Mant::ONE) == S::Mant::ZERO;
                    let minnorm_mant = S::Mant::ONE << S::MANT_FRAC_BITS;
                    if self.mant == minnorm_mant {
                        // neighbors: (maxmant, exp - 1) -- (minnormmant, exp) -- (minnormmant + 1, exp)
                        // where maxmant = minnormmant * 2 - 1
                        FullDecoded::Finite(Decoded {
                            mant: mant << 2,
                            minus: 1,
                            plus: 2,
                            exp: exp - 2,
                            inclusive: even,
                        })
                    } else {
                        // neighbors: (mant - 1, exp) -- (mant, exp) -- (mant + 1, exp)
                        FullDecoded::Finite(Decoded {
                            mant: mant << 1,
                            minus: 1,
                            plus: 1,
                            exp: exp - 1,
                            inclusive: even,
                        })
                    }
                }
            }
        };
        (self.sign, decoded)
    }
}

macro_rules! impl_float_traits {
    {
        impl $ty:ty {
            type Bits = $bits:ty;
            const ZERO: Self = $zero:expr;
            const NAN: Self = $nan:expr;
            const INFINITY: Self = $inf:expr;
        }
    } => {
        impl $crate::Float for $ty {
            type Bits = $bits;

            const ZERO: Self = $zero;

            const NAN: Self = $nan;

            const INFINITY: Self = $inf;

            #[inline]
            fn from_bits(bits: Self::Bits) -> Self {
                Self($crate::ieee_float::IeeeFloat::from_bits(bits))
            }

            #[inline]
            fn to_bits(self) -> Self::Bits {
                self.0.to_bits()
            }

            #[inline]
            fn from_uint_ex(value: u128, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = $crate::ieee_float::IeeeFloat::from_uint(value, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn from_int_ex(value: i128, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = $crate::ieee_float::IeeeFloat::from_int(value, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn to_uint_ex(self, bits: u32, round: $crate::Round) -> (Option<u128>, $crate::FpStatus) {
                let (res_value, res_status) = self.0.to_uint(bits, round);
                (res_value, res_status)
            }

            #[inline]
            fn to_int_ex(self, bits: u32, round: $crate::Round) -> (Option<i128>, $crate::FpStatus) {
                let (res_value, res_status) = self.0.to_int(bits, round);
                (res_value, res_status)
            }

            fn from_str_ex(s: &str, round: $crate::Round) -> Result<(Self, $crate::FpStatus), $crate::ParseFloatError> {
                let (res_value, res_status) = $crate::ieee_float::IeeeFloat::from_str(s, round)?;
                Ok((Self(res_value), res_status))
            }

            #[inline]
            fn is_nan(self) -> bool {
                self.0.category() == core::num::FpCategory::Nan
            }

            #[inline]
            fn is_infinite(self) -> bool {
                self.0.category() == core::num::FpCategory::Infinite
            }

            #[inline]
            fn is_finite(self) -> bool {
                !matches!(self.0.category(), core::num::FpCategory::Infinite |core::num::FpCategory::Nan)
            }

            #[inline]
            fn is_subnormal(self) -> bool {
                self.0.category() == core::num::FpCategory::Subnormal
            }

            #[inline]
            fn is_normal(self) -> bool {
                self.0.category() == core::num::FpCategory::Normal
            }

            #[inline]
            fn classify(self) -> core::num::FpCategory {
                self.0.category()
            }

            #[inline]
            fn is_sign_positive(self) -> bool {
                !self.0.sign()
            }

            #[inline]
            fn is_sign_negative(self) -> bool {
                self.0.sign()
            }

            #[inline]
            fn abs(self) -> Self {
                Self(self.0.abs())
            }

            #[inline]
            fn copysign(self, sign: Self) -> Self {
                Self(self.0.set_sign(sign.0.sign()))
            }

            #[inline]
            fn round_int_ex(self, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.round_int(round);
                (Self(res_value), res_status)
            }

            fn scalbn_ex(self, exp: i32, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.scalbn(exp, round);
                (Self(res_value), res_status)
            }

            fn frexp(self) -> (Self, i32) {
                let (res_value, res_exp) = self.0.frexp();
                (Self(res_value), res_exp)
            }

            #[inline]
            fn add_ex(self, rhs: Self, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.add(rhs.0, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn sub_ex(self, rhs: Self, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.sub(rhs.0, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn mul_ex(self, rhs: Self, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.mul(rhs.0, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn div_ex(self, rhs: Self, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.div(rhs.0, round);
                (Self(res_value), res_status)
            }

            #[inline]
            fn rem_ex(self, rhs: Self) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) = self.0.rem(rhs.0);
                (Self(res_value), res_status)
            }
        }

        impl Default for $ty {
            #[inline]
            fn default() -> Self {
                <Self as $crate::Float>::ZERO
            }
        }

        impl core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt_debug(f)
            }
        }

        impl core::fmt::Display for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt_display(f)
            }
        }

        impl core::fmt::LowerExp for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt_exp(f, false)
            }
        }

        impl core::fmt::UpperExp for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt_exp(f, true)
            }
        }

        impl core::str::FromStr for $ty {
            type Err = $crate::ParseFloatError;

            /// Parses a floating-point number from a base-10 string, rounding
            /// to the rounding to nearest with ties to even.
            ///
            /// See [`Float::from_str_ex`](crate::Float::from_str_ex) for details
            /// on the accepted syntax.
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$ty as $crate::Float>::from_str_ex(s, $crate::Round::NearestTiesToEven)
                    .map(|(v, _)| v)
            }
        }

        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl PartialOrd for $ty {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }

            #[inline]
            fn lt(&self, other: &Self) -> bool {
                self.0 < other.0
            }

            #[inline]
            fn le(&self, other: &Self) -> bool {
                self.0 <= other.0
            }

            #[inline]
            fn gt(&self, other: &Self) -> bool {
                self.0 > other.0
            }

            #[inline]
            fn ge(&self, other: &Self) -> bool {
                self.0 >= other.0
            }
        }

        impl core::ops::Neg for $ty {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl core::ops::Add for $ty {
            type Output = Self;

            /// Add `self` and `rhs`, rounding to the nearest with ties to even.
            #[inline]
            fn add(self, rhs: Self) -> Self {
                $crate::Float::add_ex(self, rhs, $crate::Round::NearestTiesToEven).0
            }
        }

        impl core::ops::Sub for $ty {
            type Output = Self;

            /// Subtract `rhs` from `self`, rounding to the nearest with ties to even.
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                $crate::Float::sub_ex(self, rhs, $crate::Round::NearestTiesToEven).0
            }
        }

        impl core::ops::Mul for $ty {
            type Output = Self;

            /// Multiply `self` and `rhs`, rounding to the nearest with ties to even.
            #[inline]
            fn mul(self, rhs: Self) -> Self {
                $crate::Float::mul_ex(self, rhs, $crate::Round::NearestTiesToEven).0
            }
        }

        impl core::ops::Div for $ty {
            type Output = Self;

            /// Divide `self` by `rhs`, rounding to the nearest with ties to even.
            #[inline]
            fn div(self, rhs: Self) -> Self {
                $crate::Float::div_ex(self, rhs, $crate::Round::NearestTiesToEven).0
            }
        }

        impl core::ops::Rem for $ty {
            type Output = Self;

            /// Calculate the remainder of `self` divided by `rhs`.
            #[inline]
            fn rem(self, rhs: Self) -> Self {
                $crate::Float::rem_ex(self, rhs).0
            }
        }

        $crate::ieee_float::impl_convert_from!($crate::F16 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F32 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F64 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F128 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::X87F80 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F8E5M2 as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F8E4M3B8Nnz as $ty);
        $crate::ieee_float::impl_convert_from!($crate::F8E4M3Nao as $ty);
    };
}

macro_rules! impl_convert_from {
    ($from:ty as $to:ty) => {
        impl $crate::FloatConvertFrom<$from> for $to {
            #[inline]
            fn convert_from_ex(value: $from, round: $crate::Round) -> (Self, $crate::FpStatus) {
                let (res_value, res_status) =
                    $crate::ieee_float::IeeeFloat::convert_from(value.0, round);
                (Self(res_value), res_status)
            }
        }
    };
}

pub(crate) use {impl_convert_from, impl_float_traits};
