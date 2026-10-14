//! Float formatting, which produces the same output as the Rust standard
//! library does for its primitive float types.
//!
//! Formatting has two steps. First, the decimal digits and the position of
//! the decimal point are calculated, either with `shortest::shortest` (when
//! no precision is given) or with `exact` (when the value has to be rounded
//! to a given precision). Then, they are written in decimal or exponential
//! notation, taking into account the sign, the width, the fill and the
//! alignment.

use core::fmt::{Alignment, Formatter, Write as _};
use core::num::FpCategory;

use super::{IeeeFloat, Semantics};
use crate::utils::{DEC_CHUNK_LEN, DEC_CHUNK_POW10};

mod exact;
mod scale;
mod shortest;

use exact::Limit;

/// Implements `core::fmt::Display`.
pub(super) fn display<S: Semantics>(
    value: IeeeFloat<S>,
    f: &mut Formatter<'_>,
) -> core::fmt::Result {
    let (sign, decoded) = decode(value);
    if let Some(precision) = f.precision() {
        fmt_exact(f, sign, &decoded, Limit::FracDigits(precision), false)
    } else {
        fmt_shortest(f, sign, &decoded, Notation::Dec)
    }
}

/// Implements `core::fmt::Debug`.
pub(super) fn debug<S: Semantics>(value: IeeeFloat<S>, f: &mut Formatter<'_>) -> core::fmt::Result {
    let (sign, decoded) = decode(value);
    if let Some(precision) = f.precision() {
        fmt_exact(f, sign, &decoded, Limit::FracDigits(precision), false)
    } else {
        fmt_shortest(f, sign, &decoded, Notation::Debug)
    }
}

/// Implements `core::fmt::LowerExp` and `core::fmt::UpperExp`.
pub(super) fn exp<S: Semantics>(
    value: IeeeFloat<S>,
    f: &mut Formatter<'_>,
    upper: bool,
) -> core::fmt::Result {
    let (sign, decoded) = decode(value);
    if let Some(precision) = f.precision() {
        // One integer digit and `precision` fractional digits.
        let num_digits = precision.saturating_add(1);
        fmt_exact(f, sign, &decoded, Limit::SigDigits(num_digits), upper)
    } else {
        fmt_shortest(f, sign, &decoded, Notation::Exp { upper })
    }
}

/// Absolute value of a float.
enum Decoded {
    Nan,
    Infinite,
    Zero,
    Finite(Finite),
}

/// Finite and non-zero value `mant * 2^exp`, where `2^exp` is the distance to
/// the next representable value.
///
/// `mant` is less than `2^MAX_PREC_BITS`, `exp` is not less than `MIN_EXP`
/// and the value is less than `2^MAX_TOP`.
struct Finite {
    mant: u128,
    exp: i32,
    /// Whether the distance to the previous representable value is
    /// `2^(exp - 1)` instead of `2^exp`, which happens when the previous
    /// value has a smaller exponent.
    ///
    /// Like in the Rust standard library, this is also set for the smallest
    /// normal value, although the subnormal value before it is `2^exp` away.
    /// The only consequence is that the shortest representation of that
    /// value is searched in a rounding interval that is narrower than it
    /// could be.
    asymmetric: bool,
}

/// Largest number of significand bits of a format that can be formatted.
///
/// Only `decode` is generic over the format. The rest of the code supports
/// any value within the limits of `Finite`, which are derived from this
/// constant and `MAX_EXP_BITS`. Both are the ones of binary128.
const MAX_PREC_BITS: u32 = 113;

/// Largest number of exponent bits of a format that can be formatted.
const MAX_EXP_BITS: u32 = 15;

/// Smallest `Finite::exp`.
///
/// With the default exponent bias, the smallest exponent of a normal value
/// is `2 - 2^(EXP_BITS - 1)`, and the last bit of its significand weighs
/// `2^(PREC_BITS - 1)` times less.
const MIN_EXP: i32 = 2 - (1 << (MAX_EXP_BITS - 1)) - (MAX_PREC_BITS as i32 - 1);

/// Every `Finite` is less than `2^MAX_TOP`.
///
/// With the default exponent bias, the largest exponent of a normal value
/// is `2^(EXP_BITS - 1)` (one less in the formats that have infinities).
const MAX_TOP: i32 = (1 << (MAX_EXP_BITS - 1)) + 1;

/// Returns the sign and the absolute value of `value`.
#[inline]
fn decode<S: Semantics>(value: IeeeFloat<S>) -> (bool, Decoded) {
    // The values of `S` must be within the limits of `Finite`.
    const {
        assert!(S::PREC_BITS <= MAX_PREC_BITS && S::EXP_BITS <= MAX_EXP_BITS);
    }
    if cfg!(debug_assertions) {
        let min_exp: i32 = S::min_subnormal_exp().into();
        let max_exp: i32 = S::max_normal_exp().into();
        assert!(min_exp >= MIN_EXP && max_exp < MAX_TOP);
    }

    let decoded = match value.category {
        FpCategory::Nan => Decoded::Nan,
        FpCategory::Infinite => Decoded::Infinite,
        FpCategory::Zero => Decoded::Zero,
        FpCategory::Normal | FpCategory::Subnormal => {
            // The value is `mant * 2^(exp - MANT_FRAC_BITS)`.
            let mant: u128 = value.mant.into();
            let exp: i32 = value.exp.into();
            let min_exp: i32 = S::min_normal_exp().into();
            let frac_bits = S::MANT_FRAC_BITS as i32;
            if exp < min_exp {
                // Subnormal values are multiples of the smallest one.
                Decoded::Finite(Finite {
                    mant: mant >> (min_exp - exp),
                    exp: min_exp - frac_bits,
                    asymmetric: false,
                })
            } else {
                Decoded::Finite(Finite {
                    mant,
                    exp: exp - frac_bits,
                    asymmetric: mant == 1 << S::MANT_FRAC_BITS,
                })
            }
        }
    };
    (value.sign, decoded)
}

/// Calculates `floor(x * log10(2))`.
///
/// The result is exact for `-20000 <= x <= 20000`, which is enough for any
/// exponent from `MIN_EXP` to `MAX_TOP`.
#[inline]
fn floor_log10_pow2(x: i32) -> i32 {
    const {
        assert!(MIN_EXP >= -20000 && MAX_TOP <= 20000);
    }
    // 1292913986 = floor(log10(2) * 2^32)
    ((i64::from(x) * 1292913986) >> 32) as i32
}

/// Notation used when no precision is given.
#[derive(Clone, Copy)]
enum Notation {
    /// Decimal notation.
    Dec,
    /// Exponential notation.
    Exp { upper: bool },
    /// Decimal notation with at least one fractional digit, or exponential
    /// notation for values that are too small or too large.
    Debug,
}

/// Formats the shortest decimal representation of `±decoded`.
fn fmt_shortest(
    f: &mut Formatter<'_>,
    sign: bool,
    decoded: &Decoded,
    notation: Notation,
) -> core::fmt::Result {
    let sign = sign_str(f, sign);
    // The value is `d * 10^k`.
    let (d, k) = match decoded {
        Decoded::Nan => return write_text(f, "", "NaN"),
        Decoded::Infinite => return write_text(f, sign, "inf"),
        Decoded::Zero => {
            let text = match notation {
                Notation::Dec => "0",
                Notation::Exp { upper: false } => "0e0",
                Notation::Exp { upper: true } => "0E0",
                Notation::Debug => "0.0",
            };
            return write_text(f, sign, text);
        }
        Decoded::Finite(value) => shortest::shortest(value),
    };

    let mut buf = NumBuf::new();
    buf.set_digits(d);
    // The value is `0.<digits> * 10^point`.
    let point = k + buf.num_digits() as i32;
    match notation {
        Notation::Dec => buf.write_dec(f, sign, point, 0),
        Notation::Exp { upper } => buf.write_exp(f, sign, point, 0, upper),
        Notation::Debug => {
            // Like the Rust standard library, exponential notation is used
            // when the value is less than `1e-4` or not less than `1e16`.
            if matches!(point - 1, -4..16) {
                buf.write_dec(f, sign, point, 1)
            } else {
                buf.write_exp(f, sign, point, 0, false)
            }
        }
    }
}

/// Formats `±decoded` rounded to `limit`, in exponential notation when
/// `limit` is a number of significant digits and in decimal notation when it
/// is a number of fractional digits.
fn fmt_exact(
    f: &mut Formatter<'_>,
    sign: bool,
    decoded: &Decoded,
    limit: Limit,
    upper: bool,
) -> core::fmt::Result {
    let sign = sign_str(f, sign);
    match decoded {
        Decoded::Nan => write_text(f, "", "NaN"),
        Decoded::Infinite => write_text(f, sign, "inf"),
        Decoded::Zero => write_exact_int(f, sign, 0, 0, limit, upper),
        Decoded::Finite(value) => {
            if let Some((d, k)) = exact::try_exact_fast(value, limit) {
                write_exact_int(f, sign, d, k, limit, upper)
            } else {
                // Using a buffer sized for the worst case has a noticeable
                // cost. The second size is enough for any value up to
                // binary64.
                match exact::exact_slow_buf_len(value, limit) {
                    0..=64 => fmt_exact_slow::<64>(f, sign, value, limit, upper),
                    65..=800 => fmt_exact_slow::<800>(f, sign, value, limit, upper),
                    _ => fmt_exact_slow::<MAX_EXACT_DIGITS>(f, sign, value, limit, upper),
                }
            }
        }
    }
}

/// Largest length of the buffer needed by `exact::exact_slow` for a value
/// within the limits of `Finite`, see `tests::test_max_exact_digits`.
const MAX_EXACT_DIGITS: usize = 11583;

/// Formats `value` like `fmt_exact`, using the slow path with a buffer of
/// `N` bytes, which must be at least `exact::exact_slow_buf_len(value, limit)`.
// Never inlined, so the stack frame of `fmt_exact` does not include the large
// buffers of every size, only the frame of the called instance does.
#[inline(never)]
fn fmt_exact_slow<const N: usize>(
    f: &mut Formatter<'_>,
    sign: &str,
    value: &Finite,
    limit: Limit,
    upper: bool,
) -> core::fmt::Result {
    let mut buf = [0; N];
    let (len, point) = exact::exact_slow(&mut buf, value, limit);
    write_exact(f, sign, &buf[..len], point, limit, upper)
}

/// Writes `0.<digits> * 10^point`, which has already been rounded to `limit`,
/// as `fmt_exact` does. `digits` is empty when the value is zero.
fn write_exact(
    f: &mut Formatter<'_>,
    sign: &str,
    digits: &[u8],
    point: i32,
    limit: Limit,
    upper: bool,
) -> core::fmt::Result {
    if digits.is_empty() {
        return write_exact_int(f, sign, 0, 0, limit, upper);
    }
    match limit {
        Limit::SigDigits(num_digits) => write_exp(f, sign, digits, point, num_digits, upper),
        Limit::FracDigits(frac_digits) => write_dec(f, sign, digits, point, frac_digits),
    }
}

/// Writes `d * 10^k`, which has already been rounded to `limit`, as
/// `fmt_exact` does.
fn write_exact_int(
    f: &mut Formatter<'_>,
    sign: &str,
    d: u128,
    k: i32,
    limit: Limit,
    upper: bool,
) -> core::fmt::Result {
    let mut buf = NumBuf::new();
    buf.set_digits(d);
    // The value is `0.<digits> * 10^point`. Zero is always `0.0 * 10^1`, so
    // it is written like any other number.
    let point = if d == 0 {
        1
    } else {
        k + buf.num_digits() as i32
    };
    match limit {
        Limit::SigDigits(num_digits) => buf.write_exp(f, sign, point, num_digits, upper),
        Limit::FracDigits(frac_digits) => buf.write_dec(f, sign, point, frac_digits),
    }
}

/// Returns the sign to be written before the absolute value of a number
/// which is not NaN.
#[inline]
fn sign_str(f: &Formatter<'_>, sign: bool) -> &'static str {
    if sign {
        "-"
    } else if f.sign_plus() {
        "+"
    } else {
        ""
    }
}

/// Writes `0.<digits> * 10^point` in decimal notation, with at least
/// `min_frac_digits` fractional digits.
///
/// `digits` must not be empty.
fn write_dec(
    f: &mut Formatter<'_>,
    sign: &str,
    digits: &[u8],
    point: i32,
    min_frac_digits: usize,
) -> core::fmt::Result {
    if point <= 0 {
        // 0.[zeros][digits][zeros]
        let num_zeros = point.unsigned_abs() as usize;
        let num_frac_digits = num_zeros + digits.len();
        let parts = [
            Part::Str("0."),
            Part::Zeros(num_zeros),
            Part::Digits(digits),
            Part::Zeros(min_frac_digits.saturating_sub(num_frac_digits)),
        ];
        write_parts(f, sign, &parts)
    } else if let Some((int_digits, frac_digits)) = digits.split_at_checked(point as usize)
        && !frac_digits.is_empty()
    {
        // [digits].[digits][zeros]
        let parts = [
            Part::Digits(int_digits),
            Part::Str("."),
            Part::Digits(frac_digits),
            Part::Zeros(min_frac_digits.saturating_sub(frac_digits.len())),
        ];
        write_parts(f, sign, &parts)
    } else {
        // [digits][zeros] or [digits][zeros].[zeros]
        let num_zeros = point as usize - digits.len();
        let parts = [
            Part::Digits(digits),
            Part::Zeros(num_zeros),
            Part::Str("."),
            Part::Zeros(min_frac_digits),
        ];
        if min_frac_digits != 0 {
            write_parts(f, sign, &parts)
        } else {
            write_parts(f, sign, &parts[..2])
        }
    }
}

/// Writes `0.<digits> * 10^point` in exponential notation, with at least
/// `min_digits` significant digits.
///
/// `digits` must not be empty.
fn write_exp(
    f: &mut Formatter<'_>,
    sign: &str,
    digits: &[u8],
    point: i32,
    min_digits: usize,
    upper: bool,
) -> core::fmt::Result {
    // `0.<digits> * 10^point = <digit>.<digits> * 10^(point - 1)`
    let (int_digit, frac_digits) = digits.split_at(1);
    let exp = point - 1;
    let exp_mark = match (upper, exp < 0) {
        (false, false) => "e",
        (false, true) => "e-",
        (true, false) => "E",
        (true, true) => "E-",
    };
    let mut exp_buf = [0; MAX_U32_DIGITS];
    let exp_start = u64_to_digits(&mut exp_buf, exp.unsigned_abs().into());
    let exp_digits = &exp_buf[exp_start..];

    let parts = [
        Part::Digits(int_digit),
        Part::Str("."),
        Part::Digits(frac_digits),
        Part::Zeros(min_digits.saturating_sub(digits.len())),
        Part::Str(exp_mark),
        Part::Digits(exp_digits),
    ];
    if digits.len() > 1 || min_digits > 1 {
        write_parts(f, sign, &parts)
    } else {
        write_parts(f, sign, &[parts[0], parts[4], parts[5]])
    }
}

/// Piece of a formatted number.
#[derive(Clone, Copy)]
enum Part<'a> {
    /// Text that is not made of digits.
    Str(&'a str),
    /// ASCII decimal digits.
    Digits(&'a [u8]),
    /// Run of `'0'`.
    Zeros(usize),
}

impl Part<'_> {
    #[inline]
    fn len(&self) -> usize {
        match *self {
            Part::Str(s) => s.len(),
            Part::Digits(s) => s.len(),
            Part::Zeros(n) => n,
        }
    }
}

/// Writes `sign` followed by `parts`, padded according to the width, the
/// fill and the alignment of `f`.
fn write_parts(f: &mut Formatter<'_>, sign: &str, parts: &[Part<'_>]) -> core::fmt::Result {
    let parts_len = parts.iter().map(Part::len).sum::<usize>();
    write_padded(f, sign, parts_len, |f, sign| {
        // Writing to `f` has a noticeable cost, so numbers that are not too
        // long are written at once.
        let mut buf = [0; 64];
        if let Some(buf) = buf.get_mut(..(sign.len() + parts_len)) {
            let (sign_buf, mut rest) = buf.split_at_mut(sign.len());
            sign_buf.copy_from_slice(sign.as_bytes());
            for part in parts {
                let part_buf;
                (part_buf, rest) = rest.split_at_mut(part.len());
                match *part {
                    Part::Str(s) => part_buf.copy_from_slice(s.as_bytes()),
                    Part::Digits(s) => part_buf.copy_from_slice(s),
                    Part::Zeros(_) => part_buf.fill(b'0'),
                }
            }
            f.write_str(core::str::from_utf8(buf).unwrap())
        } else {
            if !sign.is_empty() {
                f.write_str(sign)?;
            }
            for part in parts {
                match *part {
                    Part::Str(s) => f.write_str(s)?,
                    Part::Digits(s) => f.write_str(core::str::from_utf8(s).unwrap())?,
                    Part::Zeros(n) => write_zeros(f, n)?,
                }
            }
            Ok(())
        }
    })
}

/// Writes a run of `'0'`.
fn write_zeros(f: &mut Formatter<'_>, mut n: usize) -> core::fmt::Result {
    const ZEROS: &str = "00000000000000000000000000000000";
    while n > ZEROS.len() {
        f.write_str(ZEROS)?;
        n -= ZEROS.len();
    }
    if n != 0 {
        f.write_str(&ZEROS[..n])?;
    }
    Ok(())
}

/// Writes `sign` followed by `text`, padded according to the width, the fill
/// and the alignment of `f`.
fn write_text(f: &mut Formatter<'_>, sign: &str, text: &str) -> core::fmt::Result {
    write_padded(f, sign, text.len(), |f, sign| {
        if !sign.is_empty() {
            f.write_str(sign)?;
        }
        f.write_str(text)
    })
}

/// Writes a number with the padding required by the width, the fill and the
/// alignment of `f`.
///
/// The number is `sign` followed by `len` more characters. It is written by
/// `write`, which is given the sign that it has to write.
#[inline]
fn write_padded(
    f: &mut Formatter<'_>,
    mut sign: &str,
    len: usize,
    write: impl FnOnce(&mut Formatter<'_>, &str) -> core::fmt::Result,
) -> core::fmt::Result {
    let mut padding = 0;
    let mut pre_padding = 0;
    let mut fill = f.fill();
    if let Some(mut width) = f.width() {
        let mut align = f.align();
        if f.sign_aware_zero_pad() {
            // The sign goes before the padding, which is made of zeros.
            f.write_str(sign)?;
            width = width.saturating_sub(sign.len());
            sign = "";
            fill = '0';
            align = Some(Alignment::Right);
        }
        padding = width.saturating_sub(sign.len() + len);
        pre_padding = match align {
            Some(Alignment::Left) => 0,
            Some(Alignment::Right) | None => padding,
            Some(Alignment::Center) => padding / 2,
        };
    }

    for _ in 0..pre_padding {
        f.write_char(fill)?;
    }
    write(f, sign)?;
    for _ in pre_padding..padding {
        f.write_char(fill)?;
    }
    Ok(())
}

/// Buffer where a number that is not too long is formatted in place, around
/// its digits, so it can be written at once: writing to a `Formatter` has a
/// noticeable cost.
///
/// The digits are at `start..NumBuf::DIGITS_END` and every other byte is
/// `b'0'`, so the runs of zeros of the formatted number are already there.
struct NumBuf {
    buf: [u8; Self::LEN],
    start: usize,
}

impl NumBuf {
    const LEN: usize = 128;

    /// The digits end in the middle of the buffer, which leaves room for
    /// what goes before them and what goes after them. There is always room
    /// for a sign and a decimal point before them.
    const DIGITS_END: usize = {
        assert!(Self::LEN / 2 >= MAX_U128_DIGITS + 2);
        Self::LEN / 2
    };

    /// Creates a buffer without digits.
    #[inline]
    fn new() -> Self {
        Self {
            buf: [b'0'; Self::LEN],
            start: Self::DIGITS_END,
        }
    }

    /// Sets the digits of the buffer, which must not have any, to the
    /// decimal digits of `value`.
    #[inline]
    fn set_digits(&mut self, value: u128) {
        debug_assert_eq!(self.start, Self::DIGITS_END);
        self.start = u128_to_digits(&mut self.buf[..Self::DIGITS_END], value);
    }

    #[inline]
    fn num_digits(&self) -> usize {
        Self::DIGITS_END - self.start
    }

    #[inline]
    fn digits(&self) -> &[u8] {
        &self.buf[self.start..Self::DIGITS_END]
    }

    /// Does the same as the function `write_dec` with the digits of the
    /// buffer.
    fn write_dec(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        point: i32,
        min_frac_digits: usize,
    ) -> core::fmt::Result {
        match self.dec_in_place(point, min_frac_digits) {
            Some((start, end)) if end <= Self::LEN => self.write(f, sign, start, end),
            Some((start, end)) => self.write_long(f, sign, start, end - Self::LEN),
            None => self.write_dec_unfit(f, sign, point, min_frac_digits),
        }
    }

    /// Does the same as `write_dec` when `dec_in_place` is not able to make
    /// the number fit in the buffer, which it must not have modified.
    // Never inlined because it is rarely needed.
    #[inline(never)]
    fn write_dec_unfit(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        point: i32,
        min_frac_digits: usize,
    ) -> core::fmt::Result {
        let digits_start = self.start;
        let num_zeros = point.unsigned_abs() as usize;
        let num_frac_digits = num_zeros.saturating_add(self.num_digits());
        if point <= 0 && min_frac_digits <= num_frac_digits {
            // 0.[zeros][digits], which is the case of the shortest
            // representation of a very small number. The zeros that are
            // before the digits in the buffer are written with them.
            let num_buf_zeros = num_zeros.min(digits_start);
            return write_padded(f, sign, num_frac_digits.saturating_add(2), |f, sign| {
                if !sign.is_empty() {
                    f.write_str(sign)?;
                }
                f.write_str("0.")?;
                write_zeros(f, num_zeros - num_buf_zeros)?;
                self.write_range(f, digits_start - num_buf_zeros, Self::DIGITS_END)
            });
        }

        write_dec(f, sign, self.digits(), point, min_frac_digits)
    }

    /// Does the same as the function `write_exp` with the digits of the
    /// buffer.
    fn write_exp(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        point: i32,
        min_digits: usize,
        upper: bool,
    ) -> core::fmt::Result {
        match self.exp_in_place(point, min_digits, upper) {
            Some((start, end)) => self.write(f, sign, start, end),
            None => write_exp(f, sign, self.digits(), point, min_digits, upper),
        }
    }

    /// Turns the digits into what `write_dec` writes after the sign.
    ///
    /// Returns the range of the buffer where it is, which is not at its
    /// beginning. Its end is after the end of the buffer when the zeros at
    /// the end of the number do not fit. Returns `None`, without modifying
    /// the digits, if anything else does not fit.
    #[inline]
    fn dec_in_place(&mut self, point: i32, min_frac_digits: usize) -> Option<(usize, usize)> {
        let digits_start = self.start;
        let num_digits = self.num_digits();
        if point <= 0 {
            // 0.[zeros][digits][zeros]
            let num_zeros = point.unsigned_abs() as usize;
            let start = digits_start.checked_sub(num_zeros.checked_add(2)?)?;
            if start == 0 {
                return None;
            }
            let num_frac_digits = num_zeros + num_digits;
            let end =
                Self::DIGITS_END.checked_add(min_frac_digits.saturating_sub(num_frac_digits))?;
            self.buf[start + 1] = b'.';
            Some((start, end))
        } else if (point as usize) < num_digits {
            // [digits].[digits][zeros]
            let num_int_digits = point as usize;
            let num_frac_digits = num_digits - num_int_digits;
            let end =
                Self::DIGITS_END.checked_add(min_frac_digits.saturating_sub(num_frac_digits))?;
            // The integer digits are moved to make room for the point.
            let int_end = digits_start + num_int_digits;
            self.buf
                .copy_within(digits_start..int_end, digits_start - 1);
            self.buf[int_end - 1] = b'.';
            Some((digits_start - 1, end))
        } else {
            // [digits][zeros] or [digits][zeros].[zeros]
            let num_zeros = point as usize - num_digits;
            let point_index = Self::DIGITS_END.checked_add(num_zeros)?;
            if min_frac_digits == 0 {
                Some((digits_start, point_index))
            } else {
                let end = point_index.checked_add(1)?.checked_add(min_frac_digits)?;
                *self.buf.get_mut(point_index)? = b'.';
                Some((digits_start, end))
            }
        }
    }

    /// Turns the digits into what `write_exp` writes after the sign.
    ///
    /// Returns the range of the buffer where it is, which is not at its
    /// beginning, or `None` if it does not fit. In the latter case, the
    /// buffer is not modified.
    #[inline]
    fn exp_in_place(
        &mut self,
        point: i32,
        min_digits: usize,
        upper: bool,
    ) -> Option<(usize, usize)> {
        let digits_start = self.start;
        let num_digits = self.num_digits();
        let exp = point - 1;
        let exp_abs = exp.unsigned_abs();
        let exp_len = exp_abs.checked_ilog10().map_or(1, |log| log as usize + 1);

        // [digit].[digits][zeros]e[-][exp] or [digit]e[-][exp]
        let num_zeros = min_digits.saturating_sub(num_digits);
        if num_zeros > Self::LEN - Self::DIGITS_END {
            return None;
        }
        let mantissa_end = Self::DIGITS_END + num_zeros;
        let exp_start = mantissa_end + 1 + usize::from(exp < 0);
        let end = exp_start + exp_len;
        if end > Self::LEN {
            return None;
        }

        let mut start = digits_start;
        if num_digits > 1 || min_digits > 1 {
            // The first digit is moved to make room for the point.
            start -= 1;
            self.buf[start] = self.buf[digits_start];
            self.buf[digits_start] = b'.';
        }
        self.buf[mantissa_end] = if upper { b'E' } else { b'e' };
        if exp < 0 {
            self.buf[mantissa_end + 1] = b'-';
        }
        // Most exponents have one or two digits, which do not need a call.
        match exp_abs {
            0..10 => self.buf[exp_start] = b'0' + exp_abs as u8,
            10..100 => {
                self.buf[exp_start..end].copy_from_slice(&DIGIT_PAIRS[exp_abs as usize]);
            }
            _ => {
                let actual_exp_start = u64_to_digits(&mut self.buf[..end], exp_abs.into());
                debug_assert_eq!(actual_exp_start, exp_start);
            }
        }

        Some((start, end))
    }

    /// Writes `sign` followed by the range `start..end` of the buffer, padded
    /// according to the width, the fill and the alignment of `f`.
    ///
    /// `start` must not be zero and every byte before it must be `b'0'`.
    #[inline]
    fn write(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        start: usize,
        end: usize,
    ) -> core::fmt::Result {
        if f.width().is_none() {
            self.write_unpadded(f, sign, start, end)
        } else {
            self.write_padded(f, sign, start, end)
        }
    }

    /// Does the same as `write`, without the padding.
    #[inline]
    fn write_unpadded(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        start: usize,
        end: usize,
    ) -> core::fmt::Result {
        let start = self.put_sign(sign, start);
        self.write_range(f, start, end)
    }

    /// Writes `sign` followed by the buffer from `start` and `num_zeros`
    /// zeros, padded according to the width, the fill and the alignment of
    /// `f`.
    ///
    /// `start` must not be zero.
    // Never inlined because it is rarely needed.
    #[inline(never)]
    fn write_long(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        start: usize,
        num_zeros: usize,
    ) -> core::fmt::Result {
        write_padded(f, sign, (Self::LEN - start) + num_zeros, |f, sign| {
            self.write_unpadded(f, sign, start, Self::LEN)?;
            write_zeros(f, num_zeros)
        })
    }

    /// Does the same as `write` when there is a width.
    // Never inlined, so it does not affect the code of the most common case
    // in `write`.
    #[inline(never)]
    fn write_padded(
        &mut self,
        f: &mut Formatter<'_>,
        sign: &str,
        start: usize,
        end: usize,
    ) -> core::fmt::Result {
        // Writing each character of the padding has a noticeable cost, so it
        // is added to the buffer when it is possible.
        let padding = f
            .width()
            .unwrap_or(0)
            .saturating_sub(sign.len() + (end - start));
        if f.sign_aware_zero_pad() {
            // The padding goes between the sign and the number, and it is
            // made of zeros, which are already in the buffer.
            if let Some(start) = start.checked_sub(padding)
                && start != 0
            {
                return self.write_unpadded(f, sign, start, end);
            }
        } else if let Ok(fill) = u8::try_from(f.fill())
            && fill.is_ascii()
        {
            let pre_padding = match f.align() {
                Some(Alignment::Left) => 0,
                Some(Alignment::Right) | None => padding,
                Some(Alignment::Center) => padding / 2,
            };
            if let Some(padded_start) = start.checked_sub(sign.len() + pre_padding)
                && let Some(padded_end) = end.checked_add(padding - pre_padding)
                && padded_end <= Self::LEN
            {
                let start = self.put_sign(sign, start);
                self.buf[padded_start..start].fill(fill);
                self.buf[end..padded_end].fill(fill);
                return self.write_range(f, padded_start, padded_end);
            }
        }

        write_padded(f, sign, end - start, |f, sign| {
            self.write_unpadded(f, sign, start, end)
        })
    }

    /// Puts `sign`, which has one character at most, before the index
    /// `start` of the buffer, which must not be zero.
    ///
    /// Returns the index where it starts.
    #[inline]
    fn put_sign(&mut self, sign: &str, start: usize) -> usize {
        debug_assert!(sign.len() <= 1);
        match sign.as_bytes().first() {
            Some(&sign) => {
                self.buf[start - 1] = sign;
                start - 1
            }
            None => start,
        }
    }

    /// Writes the range `start..end` of the buffer.
    #[inline]
    fn write_range(&self, f: &mut Formatter<'_>, start: usize, end: usize) -> core::fmt::Result {
        f.write_str(core::str::from_utf8(&self.buf[start..end]).unwrap())
    }
}

/// Maximum number of decimal digits of a `u32`.
const MAX_U32_DIGITS: usize = 10;

/// Maximum number of decimal digits of a `u128`.
const MAX_U128_DIGITS: usize = 39;

/// The two digits of every integer below 100.
static DIGIT_PAIRS: [[u8; 2]; 100] = {
    let mut pairs = [[0; 2]; 100];
    let mut i = 0;
    while i < 100 {
        pairs[i] = [b'0' + (i / 10) as u8, b'0' + (i % 10) as u8];
        i += 1;
    }
    pairs
};

/// Writes the ASCII decimal digits of `value` at the end of `buf` and
/// returns the index of the first one.
fn u128_to_digits(buf: &mut [u8], mut value: u128) -> usize {
    let mut end = buf.len();
    loop {
        // 128-bit divisions are slow, so they are avoided when possible.
        let (rest, chunk) = match u64::try_from(value) {
            Ok(value) => (0, value),
            Err(_) => (
                value / u128::from(DEC_CHUNK_POW10),
                (value % u128::from(DEC_CHUNK_POW10)) as u64,
            ),
        };
        let start = u64_to_digits(&mut buf[..end], chunk);
        if rest == 0 {
            return start;
        }
        // All the digits of the chunk are needed, including leading zeros.
        end -= DEC_CHUNK_LEN;
        buf[end..start].fill(b'0');
        value = rest;
    }
}

/// Writes the ASCII decimal digits of `value` at the end of `buf` and
/// returns the index of the first one.
#[inline]
fn u64_to_digits(buf: &mut [u8], mut value: u64) -> usize {
    /// Writes the four digits of `value`, which must be less than `10^4`.
    #[inline]
    fn write_4_digits(buf: &mut [u8; 4], value: u32) {
        buf[..2].copy_from_slice(&DIGIT_PAIRS[(value / 100) as usize]);
        buf[2..].copy_from_slice(&DIGIT_PAIRS[(value % 100) as usize]);
    }

    // Eight digits at a time while there are more, so most of the divisions
    // are 32-bit ones, which are cheaper.
    let mut start = buf.len();
    while value >= 100_000_000 {
        let chunk = (value % 100_000_000) as u32;
        value /= 100_000_000;
        start -= 8;
        let chunk_buf: &mut [u8; 8] = (&mut buf[start..(start + 8)]).try_into().unwrap();
        let (hi_buf, lo_buf) = chunk_buf.split_at_mut(4);
        write_4_digits(hi_buf.try_into().unwrap(), chunk / 10_000);
        write_4_digits(lo_buf.try_into().unwrap(), chunk % 10_000);
    }

    // The remaining digits, which are eight at most.
    let mut value = value as u32;
    if value >= 10_000 {
        start -= 4;
        let chunk_buf = (&mut buf[start..(start + 4)]).try_into().unwrap();
        write_4_digits(chunk_buf, value % 10_000);
        value /= 10_000;
    }
    if value >= 100 {
        start -= 2;
        buf[start..(start + 2)].copy_from_slice(&DIGIT_PAIRS[(value % 100) as usize]);
        value /= 100;
    }
    if value >= 10 {
        start -= 2;
        buf[start..(start + 2)].copy_from_slice(&DIGIT_PAIRS[value as usize]);
    } else {
        start -= 1;
        buf[start] = b'0' + value as u8;
    }
    start
}

#[cfg(test)]
mod tests {
    use std::format;
    use std::string::ToString as _;

    use rand::RngExt as _;

    use super::{
        Decoded, Finite, Formatter, MAX_EXACT_DIGITS, MAX_PREC_BITS, MAX_TOP, MAX_U128_DIGITS,
        MIN_EXP, NumBuf, decode, exact, floor_log10_pow2, u128_to_digits,
    };
    use crate::ieee_float::{Format, IeeeFloat, Semantics};
    use crate::utils::RoundLoss;
    use crate::{Float as _, Round};

    pub(super) fn create_prng() -> impl rand::Rng {
        use rand::SeedableRng as _;
        rand_pcg::Pcg64::seed_from_u64(0x3A91_C07E_55D2_F48B)
    }

    /// Calls `$f::<S>($args)` for the semantics of every format.
    macro_rules! for_each_semantics {
        ($f:ident($($args:expr),*)) => {
            $f::<crate::f8e5m2::F8E5M2Semantics>($($args),*);
            $f::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>($($args),*);
            $f::<crate::f8e4m3nao::F8E4M3NaoSemantics>($($args),*);
            $f::<crate::f16::F16Semantics>($($args),*);
            $f::<crate::f32::F32Semantics>($($args),*);
            $f::<crate::f64::F64Semantics>($($args),*);
            $f::<crate::f128::F128Semantics>($($args),*);
            $f::<crate::x87f80::X87F80Semantics>($($args),*);
        };
    }
    pub(super) use for_each_semantics;

    /// Returns the smallest and the largest value of `Finite::exp` for `S`.
    pub(super) fn exp_range<S: Semantics>() -> (i32, i32) {
        let min_exp: i32 = S::min_normal_exp().into();
        let max_exp: i32 = S::max_normal_exp().into();
        let frac_bits = S::MANT_FRAC_BITS as i32;
        (min_exp - frac_bits, max_exp - frac_bits)
    }

    /// Generates a finite and non-zero value of `S`.
    ///
    /// Subnormal values, the extreme exponents and the extreme significands
    /// are generated often.
    pub(super) fn gen_finite<S: Semantics>(rng: &mut impl rand::RngExt) -> Finite {
        let (min_exp, max_exp) = exp_range::<S>();
        let min_mant = 1u128 << (S::PREC_BITS - 1);
        let max_mant = 2 * min_mant - 1;

        if rng.random_ratio(1, 8) {
            // Subnormal
            let mant = match rng.random_range(0..4) {
                0 => 1,
                1 => min_mant - 1,
                _ => rng.random_range(1..min_mant),
            };
            return Finite {
                mant,
                exp: min_exp,
                asymmetric: false,
            };
        }

        let mut mant = match rng.random_range(0..8) {
            0 => min_mant,
            1 => max_mant,
            2 => min_mant + 1,
            _ => rng.random_range(min_mant..=max_mant),
        };
        let exp = match rng.random_range(0..8) {
            0 => min_exp,
            1 => max_exp,
            2 => rng.random_range(-200..=200).clamp(min_exp, max_exp),
            _ => rng.random_range(min_exp..=max_exp),
        };
        if S::FORMAT == Format::NoInfNanAllOnes && exp == max_exp && mant == max_mant {
            // This is the encoding of NaN.
            mant -= 1;
        }
        Finite {
            mant,
            exp,
            asymmetric: mant == min_mant,
        }
    }

    /// Calls `f` with the largest value of each exponent and each number of
    /// significand bits that are within the limits of `Finite`.
    pub(super) fn for_each_max_finite(mut f: impl FnMut(&Finite)) {
        for exp in MIN_EXP..MAX_TOP {
            for bits in 1..=MAX_PREC_BITS.min((MAX_TOP - exp) as u32) {
                f(&Finite {
                    mant: (1 << bits) - 1,
                    exp,
                    asymmetric: false,
                });
            }
        }
    }

    /// Calculates `floor(x)` and the rounding loss of `x = n * 2^e / 10^j`
    /// with exact integer arithmetic.
    pub(super) fn scale_reference(n: &rug::Integer, e: i32, j: i32) -> (rug::Integer, RoundLoss) {
        let pow2 = rug::Integer::from(1) << e.unsigned_abs();
        let pow10 = rug::Integer::from(rug::Integer::u_pow_u(10, j.unsigned_abs()));
        let mut num = n.clone();
        let mut den = rug::Integer::from(1);
        if e >= 0 {
            num *= pow2;
        } else {
            den *= pow2;
        }
        if j >= 0 {
            den *= pow10;
        } else {
            num *= pow10;
        }

        let (q, r) = num.div_rem(den.clone());
        let loss = if r == 0 {
            RoundLoss::Zero
        } else {
            match (r * 2u32).cmp(&den) {
                core::cmp::Ordering::Less => RoundLoss::HalfDown,
                core::cmp::Ordering::Equal => RoundLoss::Halfway,
                core::cmp::Ordering::Greater => RoundLoss::HalfUp,
            }
        };
        (q, loss)
    }

    #[test]
    fn test_floor_log10_pow2() {
        // `pow10 = 10^k <= pow2 = 2^x < 10^(k + 1)`
        let mut pow2 = rug::Integer::from(1);
        let mut pow10 = rug::Integer::from(1);
        let mut k = 0;
        for x in 0..=20000 {
            if pow10.clone() * 10u32 <= pow2 {
                pow10 *= 10u32;
                k += 1;
            }
            assert_eq!(floor_log10_pow2(x), k);
            if x != 0 {
                // `2^x` is not a power of ten, so `10^(-k - 1) < 2^-x < 10^-k`.
                assert_eq!(floor_log10_pow2(-x), -k - 1);
            }
            pow2 *= 2u32;
        }
    }

    #[test]
    fn test_u128_to_digits() {
        let mut rng = create_prng();
        let check = |value: u128| {
            let mut buf = [0; MAX_U128_DIGITS];
            let start = u128_to_digits(&mut buf, value);
            assert_eq!(&buf[start..], value.to_string().as_bytes());
        };
        check(0);
        check(u128::MAX);
        check(u128::from(u64::MAX));
        check(u128::from(u64::MAX) + 1);
        for i in 0..=38 {
            check(10u128.pow(i));
            check(10u128.pow(i) - 1);
        }
        for _ in 0..10_000 {
            let bits = rng.random_range(1..=128);
            check(rng.random::<u128>() >> (128 - bits));
        }
    }

    /// Checks that `NumBuf` writes the same as the functions that take the
    /// digits as a slice, with any padding.
    #[test]
    fn test_num_buf() {
        /// Value that is formatted by a closure.
        struct FmtWith<F>(F);

        impl<F: Fn(&mut Formatter<'_>) -> core::fmt::Result> core::fmt::Display for FmtWith<F> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                (self.0)(f)
            }
        }

        macro_rules! check {
            ($actual:expr, $expected:expr, $context:expr; $($spec:literal)*) => {
                $(
                    assert_eq!(
                        format!($spec, FmtWith($actual)),
                        format!($spec, FmtWith($expected)),
                        "{}, spec = {:?}", $context, $spec,
                    );
                )*
            };
        }
        // The precision and the sign flags are not used by these functions.
        macro_rules! check_all {
            ($actual:expr, $expected:expr, $context:expr) => {
                check!(
                    $actual, $expected, $context;
                    "{}" "{:1}" "{:20}" "{:<20}" "{:^21}" "{:>60}" "{:<60}" "{:^61}"
                    "{:-^129}" "{:#>130}" "{:<300}" "{:020}" "{:060}" "{:0129}" "{:0300}"
                    "{:<08}" "{:^030}" "{:é>30}" "{:é^300}" "{:é<5}"
                );
            };
        }

        let mut rng = create_prng();
        for _ in 0..10_000 {
            // Any number of digits, including the single digit of zero.
            let value = match rng.random_range(0..40) {
                0 => 0,
                39 => rng.random_range(10u128.pow(38)..=u128::MAX),
                num_digits => rng.random_range(10u128.pow(num_digits - 1)..10u128.pow(num_digits)),
            };
            let digits = value.to_string();
            let sign = ["", "-", "+"][rng.random_range(0..3)];
            // The numbers that fit in the buffer, the ones that do not, and
            // the ones in between.
            let point = match rng.random_range(0..3) {
                0 => rng.random_range(-5..=(digits.len() as i32 + 5)),
                1 => rng.random_range(-70..=140),
                _ => rng.random_range(-400..=400),
            };
            let min_digits = match rng.random_range(0..4) {
                0 => 0,
                1 => rng.random_range(0..=5),
                2 => rng.random_range(0..=70),
                _ => rng.random_range(0..=300),
            };
            let upper = rng.random();
            let context = format!(
                "digits = {digits}, sign = {sign:?}, point = {point}, min_digits = {min_digits}"
            );

            check_all!(
                |f: &mut Formatter<'_>| {
                    let mut buf = NumBuf::new();
                    buf.set_digits(value);
                    assert_eq!(buf.digits(), digits.as_bytes());
                    buf.write_dec(f, sign, point, min_digits)
                },
                |f: &mut Formatter<'_>| {
                    super::write_dec(f, sign, digits.as_bytes(), point, min_digits)
                },
                context
            );
            check_all!(
                |f: &mut Formatter<'_>| {
                    let mut buf = NumBuf::new();
                    buf.set_digits(value);
                    buf.write_exp(f, sign, point, min_digits, upper)
                },
                |f: &mut Formatter<'_>| {
                    super::write_exp(f, sign, digits.as_bytes(), point, min_digits, upper)
                },
                context
            );
        }
    }

    #[test]
    fn test_decode() {
        fn check<S: Semantics>(rng: &mut impl rand::RngExt) {
            for _ in 0..1000 {
                let expected = gen_finite::<S>(rng);
                // `expected.mant * 2^expected.exp` is exactly representable.
                let (value, status) = IeeeFloat::<S>::from_uint(expected.mant, Round::TowardZero);
                assert_eq!(status, crate::FpStatus::OK);
                let value = value.scalbn(expected.exp, Round::TowardZero).0;

                for value in [value, -value] {
                    let (sign, decoded) = decode(value);
                    assert_eq!(sign, value.sign());
                    let Decoded::Finite(actual) = decoded else {
                        panic!("not finite");
                    };
                    assert_eq!(actual.mant, expected.mant);
                    assert_eq!(actual.exp, expected.exp);
                    assert_eq!(actual.asymmetric, expected.asymmetric);
                }
            }
        }

        let mut rng = create_prng();
        for_each_semantics!(check(&mut rng));
    }

    /// Checks that `MAX_EXACT_DIGITS` is enough for any value within the
    /// limits of `Finite`.
    #[test]
    fn test_max_exact_digits() {
        let mut max_len = 0;
        // The length only depends on the exponent and the number of
        // significand bits of the value.
        for_each_max_finite(|value| {
            for limit in [
                exact::Limit::SigDigits(usize::MAX),
                exact::Limit::FracDigits(usize::MAX),
            ] {
                max_len = max_len.max(exact::exact_slow_buf_len(value, limit));
            }
        });
        assert_eq!(max_len, MAX_EXACT_DIGITS);
    }

    /// Checks the whole formatting of `f32` and `f64` against the Rust
    /// standard library.
    #[test]
    fn test_against_host() {
        macro_rules! check {
            ($host:expr, $soft:expr; $($spec:literal)*) => {
                $(
                    let expected = format!($spec, $host);
                    let actual = format!($spec, $soft);
                    assert_eq!(
                        actual, expected,
                        "value = {:e}, spec = {:?}", $host, $spec,
                    );
                )*
            };
        }
        macro_rules! check_all {
            ($host:expr, $soft:expr) => {
                check!(
                    $host, $soft;
                    "{}" "{:?}" "{:e}" "{:E}" "{:+}" "{:+?}" "{:+e}"
                    "{:.0}" "{:.1}" "{:.2}" "{:.3}" "{:.6}" "{:.10}" "{:.17}" "{:.20}"
                    "{:.37}" "{:.38}" "{:.40}" "{:.60}" "{:.100}" "{:.400}" "{:.1100}"
                    "{:.0e}" "{:.1e}" "{:.2e}" "{:.5e}" "{:.10E}" "{:.16e}" "{:.17e}"
                    "{:.35e}" "{:.36e}" "{:.37e}" "{:.38e}" "{:.40e}" "{:.100e}"
                    "{:.800e}" "{:.3?}" "{:.0?}" "{:+.3?}"
                    "{:10}" "{:<12.3}" "{:^+14.2e}" "{:>9?}" "{:012.4}" "{:+010}"
                    "{:*^20.3e}" "{:08.1?}" "{:#<3}" "{:30.20}" "{:<030.2E}"
                    "{:^08}" "{:+08.0}" "{:x>+12.1}" "{:^13?}" "{:^14?}"
                );
            };
        }

        let mut rng = create_prng();

        let check_f32 = |bits: u32| {
            check_all!(f32::from_bits(bits), crate::F32::from_bits(bits));
        };
        let check_f64 = |bits: u64| {
            check_all!(f64::from_bits(bits), crate::F64::from_bits(bits));
        };

        // Zeros, infinities, NaNs, subnormals, extreme values and powers of
        // two, with both signs.
        for sign in [0, 1 << 31] {
            for exp in 0..=0xFF {
                for mant in [0, 1, 2, 0x40_0000, 0x7F_FFFE, 0x7F_FFFF] {
                    check_f32(sign | (exp << 23) | mant);
                }
            }
        }
        for sign in [0, 1 << 63] {
            for exp in 0..=0x7FF {
                for mant in [0, 1, (1 << 52) - 1] {
                    check_f64(sign | (exp << 52) | mant);
                }
            }
        }

        // Small integers and decimal fractions, which have many ties.
        for i in 0..=2000u16 {
            for scale in [1.0, 2.0, 8.0, 10.0, 100.0, 1000.0, 1024.0] {
                check_f32((f32::from(i) / scale as f32).to_bits());
                check_f64((f64::from(i) / scale).to_bits());
                check_f64((f64::from(i) * scale * 1e15).to_bits());
            }
        }
        for e in -330..=310 {
            let s = format!("1e{e}");
            check_f32(s.parse::<f32>().unwrap().to_bits());
            check_f64(s.parse::<f64>().unwrap().to_bits());
            let s = format!("9.999999999999999e{e}");
            check_f64(s.parse::<f64>().unwrap().to_bits());
        }

        for _ in 0..3000 {
            check_f32(rng.random());
            check_f64(rng.random());
        }
    }
}
