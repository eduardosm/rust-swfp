use super::{IeeeFloat, Semantics};
use crate::{FpStatus, ParseFloatError, Round};

mod lemire;
mod slow;

// Like Rust standard library, the following EBNF grammar is accepted:
//
// ```
// Float  ::= Sign? ( 'inf' | 'infinity' | 'nan' | Number )
// Number ::= ( Digit+ |
//              Digit+ '.' Digit* |
//              Digit* '.' Digit+ ) Exp?
// Exp    ::= 'e' Sign? Digit+
// Sign   ::= [+-]
// Digit  ::= [0-9]
// ```

enum ParsedFloat<'a> {
    Nan(bool),
    Infinity(bool),
    Zero(bool),
    Finite(ParsedFinite<'a>),
    FiniteUnderflow(bool),
    FiniteOverflow(bool),
}

/// Finite number `±w * 10^exp`, where `w` is the integer whose decimal digits
/// are `digits0` followed by `digits1`.
///
/// `w` is not zero and does not have leading zeros, and the value is not
/// known to underflow or overflow (see `parse_string`).
struct ParsedFinite<'a> {
    sign: bool,
    exp: i64,
    digits0: &'a [u8],
    digits1: &'a [u8],
}

pub(super) fn parse<S: Semantics>(
    s: &[u8],
    round: Round,
) -> Result<(IeeeFloat<S>, FpStatus), ParseFloatError> {
    match parse_string(s, max_underflow_sci_exp::<S>(), min_overflow_sci_exp::<S>())? {
        ParsedFloat::Nan(sign) => Ok((IeeeFloat::make_default_qnan().set_sign(sign), FpStatus::OK)),
        ParsedFloat::Infinity(sign) => Ok((IeeeFloat::make_inf(sign), FpStatus::OK)),
        ParsedFloat::Zero(sign) => Ok((IeeeFloat::make_zero(sign), FpStatus::OK)),
        ParsedFloat::Finite(parsed) => {
            if let Some(result) = lemire::try_convert(&parsed, round) {
                return Ok(result);
            }

            Ok(slow::convert(&parsed, round))
        }
        ParsedFloat::FiniteUnderflow(sign) => Ok(IeeeFloat::make_underflow_result(sign, round)),
        ParsedFloat::FiniteOverflow(sign) => Ok(IeeeFloat::make_overflow_result(sign, round)),
    }
}

/// Parses `s`.
///
/// A non-zero finite value in `[10^e, 10^(e + 1))` is reported as an underflow
/// when `e <= max_underflow_sci_exp` and as an overflow when
/// `e >= min_overflow_sci_exp`.
fn parse_string(
    s: &[u8],
    max_underflow_sci_exp: i64,
    min_overflow_sci_exp: i64,
) -> Result<ParsedFloat<'_>, ParseFloatError> {
    if s.is_empty() {
        return Err(ParseFloatError::mk_empty());
    }

    let (sign, rest) = take_sign(s);
    if rest.eq_ignore_ascii_case(b"inf") || rest.eq_ignore_ascii_case(b"infinity") {
        return Ok(ParsedFloat::Infinity(sign));
    } else if rest.eq_ignore_ascii_case(b"nan") {
        return Ok(ParsedFloat::Nan(sign));
    }

    // Integer digits
    let (digits0, rest) = take_digits(rest);
    // Fractional digits
    let (digits1, rest) = if let Some(rest) = rest.strip_prefix(b".") {
        take_digits(rest)
    } else {
        let empty: &[u8] = &[];
        (empty, rest)
    };

    if digits0.is_empty() && digits1.is_empty() {
        // Empty strings have already been handled above.
        return Err(ParseFloatError::mk_invalid());
    }

    let (exp, rest) = if let Some(rest) = strip_e(rest) {
        let (exp_sign, mut rest) = take_sign(rest);
        let mut exp = 0u64;
        let mut has_digits = false;
        while let [digit @ b'0'..=b'9', new_rest @ ..] = rest {
            exp = exp.saturating_mul(10).saturating_add((digit - b'0').into());
            has_digits = true;
            rest = new_rest;
        }
        if !has_digits {
            return Err(ParseFloatError::mk_invalid());
        }
        let exp = if exp_sign {
            0i64.saturating_sub_unsigned(exp)
        } else {
            0i64.saturating_add_unsigned(exp)
        };
        (exp, rest)
    } else {
        (0, rest)
    };

    if !rest.is_empty() {
        // Garbage after the number
        return Err(ParseFloatError::mk_invalid());
    }

    // Move the decimal point after the last fractional digit.
    let exp = exp.saturating_sub(digits1.len() as i64);

    // Leading zeros are not significant.
    let digits0 = trim_leading_zeros(digits0);
    let digits1 = if digits0.is_empty() {
        trim_leading_zeros(digits1)
    } else {
        digits1
    };

    let num_digits = digits0.len() + digits1.len();
    if num_digits == 0 {
        return Ok(ParsedFloat::Zero(sign));
    }

    // The value is in `[10^sci_exp, 10^(sci_exp + 1))`.
    let sci_exp = exp.saturating_add(num_digits as i64 - 1);
    if sci_exp >= min_overflow_sci_exp {
        Ok(ParsedFloat::FiniteOverflow(sign))
    } else if sci_exp <= max_underflow_sci_exp {
        Ok(ParsedFloat::FiniteUnderflow(sign))
    } else {
        Ok(ParsedFloat::Finite(ParsedFinite {
            sign,
            exp,
            digits0,
            digits1,
        }))
    }
}

#[inline]
fn take_sign(s: &[u8]) -> (bool, &[u8]) {
    match s {
        [b'+', rest @ ..] => (false, rest),
        [b'-', rest @ ..] => (true, rest),
        _ => (false, s),
    }
}

#[inline]
fn take_digits(s: &[u8]) -> (&[u8], &[u8]) {
    let len = s
        .iter()
        .position(|&b| !b.is_ascii_digit())
        .unwrap_or(s.len());
    s.split_at(len)
}

#[inline]
fn strip_e(s: &[u8]) -> Option<&[u8]> {
    match s {
        [b'e' | b'E', rest @ ..] => Some(rest),
        _ => None,
    }
}

#[inline]
fn trim_leading_zeros(digits: &[u8]) -> &[u8] {
    let n = digits
        .iter()
        .position(|&c| c != b'0')
        .unwrap_or(digits.len());
    &digits[n..]
}

/// Returns an `e` such that any value in `[10^e, 10^(e + 1))` is guaranteed
/// to be at least `2^(max_normal_exp + 1)`, and so it overflows.
#[inline]
fn min_overflow_sci_exp<S: Semantics>() -> i64 {
    // Since `log2(10) > 3.3219`, `10^e >= 2^(max_exp + 1)` when
    // `e * 33219 >= (max_exp + 1) * 10000`.
    let max_exp: i32 = S::max_normal_exp().into();
    let lim = i64::from(max_exp + 1) * 10000;
    debug_assert!(lim > 0);
    (lim + 33218) / 33219
}

/// Returns an `e` such that any value in `[10^e, 10^(e + 1))` is guaranteed
/// to be less than `2^(min_subnormal_exp - 1)` (half of the smallest
/// subnormal), and so it underflows.
#[inline]
fn max_underflow_sci_exp<S: Semantics>() -> i64 {
    // Since `log2(10) > 3.3219`, `10^(e + 1) < 2^(min_exp - 1)` when
    // `e + 1 < 0` and `(e + 1) * 33219 <= (min_exp - 1) * 10000`.
    let min_exp: i32 = S::min_subnormal_exp().into();
    let lim = i64::from(min_exp - 1) * 10000;
    debug_assert!(lim < 0);
    lim.div_euclid(33219) - 1
}

#[cfg(test)]
mod tests {
    use core::ops::RangeInclusive;
    use std::format;
    use std::string::{String, ToString as _};

    use super::{ParsedFloat, max_underflow_sci_exp, min_overflow_sci_exp, parse, parse_string};
    use crate::ieee_float::{IeeeFloat, Semantics};
    use crate::traits::CastFrom as _;
    use crate::utils::RoundLoss;
    use crate::{FpStatus, Round};

    pub(super) const ALL_ROUND_MODES: [Round; 5] = [
        Round::NearestTiesToEven,
        Round::NearestTiesToAway,
        Round::TowardPositive,
        Round::TowardNegative,
        Round::TowardZero,
    ];

    pub(super) fn create_prng() -> impl rand::Rng {
        use rand::SeedableRng as _;
        rand_pcg::Pcg64::seed_from_u64(0xB05C_3028_6B30_9158)
    }

    /// Parses `s`, which must be valid, with the range of `S`.
    pub(super) fn parse_string_for<S: Semantics>(s: &str) -> ParsedFloat<'_> {
        parse_string(
            s.as_bytes(),
            max_underflow_sci_exp::<S>(),
            min_overflow_sci_exp::<S>(),
        )
        .unwrap()
    }

    /// Formats `digits * 10^exp` with a random sign, a random position of the
    /// decimal point and some extra leading and trailing zeros.
    pub(super) fn format_input(rng: &mut impl rand::RngExt, digits: &str, exp: i64) -> String {
        let sign = if rng.random() { "-" } else { "" };
        let num_lz = rng.random_range(0..=2);
        let num_tz = rng.random_range(0..=2);
        let digits = "0".repeat(num_lz) + digits + &"0".repeat(num_tz);

        let point = rng.random_range(0..=digits.len());
        let (digits_i, digits_f) = digits.split_at(point);
        let exp = exp - num_tz as i64 + digits_f.len() as i64;
        format!("{sign}{digits_i}.{digits_f}e{exp}")
    }

    /// Generates `(digits, exp)` such that `digits * 10^exp` is a random value
    /// around the range of `S`, with up to `max_num_digits` digits.
    ///
    /// Some values overflow or underflow.
    pub(super) fn gen_arbitrary<S: Semantics>(
        rng: &mut impl rand::RngExt,
        max_num_digits: usize,
    ) -> (String, i64) {
        let num_digits = rng.random_range(1..=max_num_digits);
        let mut digits = String::with_capacity(num_digits);
        digits.push(char::from(rng.random_range(b'1'..=b'9')));
        for _ in 1..num_digits {
            digits.push(char::from(rng.random_range(b'0'..=b'9')));
        }

        // The value is in `[10^sci_exp, 10^(sci_exp + 1))`.
        let sci_exp = rng
            .random_range((max_underflow_sci_exp::<S>() - 2)..=(min_overflow_sci_exp::<S>() + 2));
        (digits, sci_exp - (num_digits as i64 - 1))
    }

    /// Generates `(digits, exp)` such that `digits * 10^exp` is `m * 2^e2`,
    /// where `m` has `PREC_BITS + 1` bits and `e2` is in `e2_range`.
    ///
    /// In the normal range, these values are either representable or the
    /// midpoint between two consecutive representable values.
    pub(super) fn gen_boundary<S: Semantics>(
        rng: &mut impl rand::RngExt,
        e2_range: RangeInclusive<i32>,
    ) -> (String, i64) {
        let m_bits = S::PREC_BITS + 1;
        let m = (rng.random::<u128>() >> (128 - m_bits)) | (1 << (m_bits - 1));

        let e2 = rng.random_range(e2_range);
        let m = rug::Integer::from(m);
        if e2 >= 0 {
            ((m << e2.unsigned_abs()).to_string(), 0)
        } else {
            // `m * 2^e2 = (m * 5^-e2) * 10^e2`
            let p5 = rug::Integer::from(rug::Integer::u_pow_u(5, e2.unsigned_abs()));
            ((m * p5).to_string(), i64::from(e2))
        }
    }

    /// Calculates the correctly rounded value of `±value` with MPFR, where
    /// `value` is a non-negative decimal number.
    pub(super) fn reference<S: Semantics>(
        sign: bool,
        value: &str,
        round: Round,
    ) -> (IeeeFloat<S>, FpStatus) {
        let value = rug::Float::parse(value).unwrap();

        // Truncating to `PREC_BITS + 1` bits gives the rounding bit in the
        // least significant bit, and the sticky bit in whether the truncation
        // is exact.
        let (value, ord) =
            rug::Float::with_val_round(S::PREC_BITS + 1, value, rug::float::Round::Zero);
        if value.is_zero() {
            assert!(ord.is_eq());
            return (IeeeFloat::make_zero(sign), FpStatus::OK);
        }
        let (m, e) = value.to_integer_exp().unwrap();
        assert_eq!(m.significant_bits(), S::PREC_BITS + 1);
        let m = m.to_u128().unwrap();
        let loss = match (m & 1 != 0, ord.is_ne()) {
            (false, false) => RoundLoss::Zero,
            (false, true) => RoundLoss::HalfDown,
            (true, false) => RoundLoss::Halfway,
            (true, true) => RoundLoss::HalfUp,
        };

        let exp = i64::from(e) + i64::from(S::PREC_BITS);
        let mant = S::Mant::cast_from(m >> 1);
        match S::Exp::try_from(exp) {
            Ok(exp) => IeeeFloat::round_and_classify(sign, exp, mant, loss, round),
            Err(_) if exp > 0 => IeeeFloat::make_overflow_result(sign, round),
            Err(_) => IeeeFloat::make_underflow_result(sign, round),
        }
    }

    #[test]
    fn test_zero_and_huge_exp() {
        type S = crate::f64::F64Semantics;

        let cases = [
            ("0", false, FpStatus::OK),
            ("-0.000", true, FpStatus::OK),
            ("0e99999999999999999999999", false, FpStatus::OK),
            ("-.0e-99999999999999999999999", true, FpStatus::OK),
            (
                "1e99999999999999999999999",
                false,
                FpStatus::OVERFLOW | FpStatus::INEXACT,
            ),
            (
                "-1e99999999999999999999999",
                true,
                FpStatus::OVERFLOW | FpStatus::INEXACT,
            ),
            (
                "1e-99999999999999999999999",
                false,
                FpStatus::UNDERFLOW | FpStatus::INEXACT,
            ),
            (
                "-1e-99999999999999999999999",
                true,
                FpStatus::UNDERFLOW | FpStatus::INEXACT,
            ),
        ];
        for (s, sign, status) in cases {
            let (value, actual_status) =
                parse::<S>(s.as_bytes(), Round::NearestTiesToEven).unwrap();
            assert_eq!(value.sign(), sign, "input = {s:?}");
            assert_eq!(actual_status, status, "input = {s:?}");
            let expected = if status.contains(FpStatus::OVERFLOW) {
                IeeeFloat::<S>::make_inf(sign)
            } else {
                IeeeFloat::<S>::make_zero(sign)
            };
            assert_eq!(value.to_bits(), expected.to_bits(), "input = {s:?}");
        }
    }
}
