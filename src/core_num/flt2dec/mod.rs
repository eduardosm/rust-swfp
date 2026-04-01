/*!

Floating-point number to decimal conversion routines.

# Problem statement

We are given the floating-point number `v = f * 2^e` with an integer `f`,
and its bounds `minus` and `plus` such that any number between `v - minus` and
`v + plus` will be rounded to `v`. For the simplicity we assume that
this range is exclusive. Then we would like to get the unique decimal
representation `V = 0.d[0..n-1] * 10^k` such that:

- `d[0]` is non-zero.

- It's correctly rounded when parsed back: `v - minus < V < v + plus`.
  Furthermore it is shortest such one, i.e., there is no representation
  with less than `n` digits that is correctly rounded.

- It's closest to the original value: `abs(V - v) <= 10^(k-n) / 2`. Note that
  there might be two representations satisfying this uniqueness requirement,
  in which case some tie-breaking mechanism is used.

We will call this mode of operation as to the *shortest* mode. This mode is used
when there is no additional constraint, and can be thought as a "natural" mode
as it matches the ordinary intuition (it at least prints `0.1f32` as "0.1").

We have two more modes of operation closely related to each other. In these modes
we are given either the number of significant digits `n` or the last-digit
limitation `limit` (which determines the actual `n`), and we would like to get
the representation `V = 0.d[0..n-1] * 10^k` such that:

- `d[0]` is non-zero, unless `n` was zero in which case only `k` is returned.

- It's closest to the original value: `abs(V - v) <= 10^(k-n) / 2`. Again,
  there might be some tie-breaking mechanism.

When `limit` is given but not `n`, we set `n` such that `k - n = limit`
so that the last digit `d[n-1]` is scaled by `10^(k-n) = 10^limit`.
If such `n` is negative, we clip it to zero so that we will only get `k`.
We are also limited by the supplied buffer. This limitation is used to print
the number up to given number of fractional digits without knowing
the correct `k` beforehand.

We will call the mode of operation requiring `n` as to the *exact* mode,
and one requiring `limit` as to the *fixed* mode. The exact mode is a subset of
the fixed mode: the sufficiently large last-digit limitation will eventually fill
the supplied buffer and let the algorithm to return.

# Implementation overview

It is easy to get the floating point printing correct but slow (Russ Cox has
[demonstrated](https://research.swtch.com/ftoa) how it's easy), or incorrect but
fast (naïve division and modulo). But it is surprisingly hard to print
floating point numbers correctly *and* efficiently.

There are two classes of algorithms widely known to be correct.

- The "Dragon" family of algorithm is first described by Guy L. Steele Jr. and
  Jon L. White. They rely on the fixed-size big integer for their correctness.
  A slight improvement was found later, which is posthumously described by
  Robert G. Burger and R. Kent Dybvig. David Gay's `dtoa.c` routine is
  a popular implementation of this strategy.

- The "Grisu" family of algorithm is first described by Florian Loitsch.
  They use very cheap integer-only procedure to determine the close-to-correct
  representation which is at least guaranteed to be shortest. The variant,
  Grisu3, actively detects if the resulting representation is incorrect.

We implement both algorithms with necessary tweaks to suit our requirements.
In particular, published literatures are short of the actual implementation
difficulties like how to avoid arithmetic overflows. Each implementation,
available in `strategy::dragon` and `strategy::grisu` respectively,
extensively describes all necessary justifications and many proofs for them.
(It is still difficult to follow though. You have been warned.)

Both implementations expose two public functions:

- `format_shortest(decoded, buf)`, which always needs at least
  `MAX_SIG_DIGITS` digits of buffer. Implements the shortest mode.

- `format_exact(decoded, buf, limit)`, which accepts as small as
  one digit of buffer. Implements exact and fixed modes.

They try to fill the `u8` buffer with digits and returns the number of digits
written and the exponent `k`. They are total for all finite `f32` and `f64`
inputs (Grisu internally falls back to Dragon if necessary).

The rendered digits are formatted into the actual string form with
four functions:

- `to_shortest_str` prints the shortest representation, which can be padded by
  zeroes to make *at least* given number of fractional digits.

- `to_shortest_exp_str` prints the shortest representation, which can be
  padded by zeroes when its exponent is in the specified ranges,
  or can be printed in the exponential form such as `1.23e45`.

- `to_exact_exp_str` prints the exact representation with given number of
  digits in the exponential form.

- `to_exact_fixed_str` prints the fixed representation with *exactly*
  given number of fractional digits.

They all return a slice of preallocated `Part` array, which corresponds to
the individual part of strings: a fixed string, a part of rendered digits,
a number of zeroes or a small (`u16`) number. The caller is expected to
provide a large enough buffer and `Part` array, and to assemble the final
string from resulting `Part`s itself.

All algorithms and formatting functions are accompanied by extensive tests
in `coretests::num::flt2dec` module. It also shows how to use individual
functions.

*/

use super::numfmt::{Formatted, Part};

use decoder::decode;
pub(crate) use decoder::{DecodableFloat, Decoded, FullDecoded};

mod decoder;
mod estimator;

/// Digit-generation algorithms.
pub(super) mod strategy {
    mod dragon;
    pub(in super::super) mod dragon_large;
    pub(in super::super) mod grisu;
}

/// The minimum size of buffer necessary for the shortest mode.
///
/// It is a bit non-trivial to derive, but this is one plus the maximal number of
/// significant decimal digits from formatting algorithms with the shortest result.
/// The exact formula is `ceil(# bits in mantissa * log_10 2 + 1)`.
pub(super) const MAX_SIG_DIGITS: usize = 36;

/// When `d` contains decimal digits, increase the last digit and propagate carry.
/// Returns a next digit when it causes the length to change.
fn round_up(d: &mut [u8]) -> Option<u8> {
    match d.iter().rposition(|&c| c != b'9') {
        Some(i) => {
            // d[i+1..n] is all nines
            d[i] += 1;
            d[i + 1..].fill(b'0');
            None
        }
        None if d.is_empty() => {
            // an empty buffer rounds up (a bit strange but reasonable)
            Some(b'1')
        }
        None => {
            // 999..999 rounds to 1000..000 with an increased exponent
            d[0] = b'1';
            d[1..].fill(b'0');
            Some(b'0')
        }
    }
}

/// Formats given decimal digits `0.<...buf...> * 10^exp` into the decimal form
/// with at least given number of fractional digits. The result is stored to
/// the supplied parts array and a slice of written parts is returned.
///
/// `frac_digits` can be less than the number of actual fractional digits in `buf`;
/// it will be ignored and full digits will be printed. It is only used to print
/// additional zeroes after rendered digits. Thus `frac_digits` of 0 means that
/// it will only print given digits and nothing else.
fn digits_to_dec_str<'a>(
    buf: &'a [u8],
    exp: i16,
    frac_digits: usize,
    parts: &'a mut [Part<'a>],
) -> &'a [Part<'a>] {
    assert!(!buf.is_empty());
    assert!(buf[0] > b'0');
    assert!(parts.len() >= 4);

    // if there is the restriction on the last digit position, `buf` is assumed to be
    // left-padded with the virtual zeroes. the number of virtual zeroes, `nzeroes`,
    // equals to `max(0, exp + frac_digits - buf.len())`, so that the position of
    // the last digit `exp - buf.len() - nzeroes` is no more than `-frac_digits`:
    //
    //                       |<-virtual->|
    //       |<---- buf ---->|  zeroes   |     exp
    //    0. 1 2 3 4 5 6 7 8 9 _ _ _ _ _ _ x 10
    //    |                  |           |
    // 10^exp    10^(exp-buf.len())   10^(exp-buf.len()-nzeroes)
    //
    // `nzeroes` is individually calculated for each case in order to avoid overflow.

    if exp <= 0 {
        // the decimal point is before rendered digits: [0.][000...000][1234][____]
        let minus_exp = -(exp as i32) as usize;
        parts[0] = Part::Copy("0.");
        parts[1] = Part::Zero(minus_exp);
        parts[2] = Part::Copy(core::str::from_utf8(buf).unwrap());
        if frac_digits > buf.len() && frac_digits - buf.len() > minus_exp {
            parts[3] = Part::Zero((frac_digits - buf.len()) - minus_exp);
            &parts[..4]
        } else {
            &parts[..3]
        }
    } else {
        let exp = exp as usize;
        if exp < buf.len() {
            // the decimal point is inside rendered digits: [12][.][34][____]
            parts[0] = Part::Copy(core::str::from_utf8(&buf[..exp]).unwrap());
            parts[1] = Part::Copy(".");
            parts[2] = Part::Copy(core::str::from_utf8(&buf[exp..]).unwrap());
            if frac_digits > buf.len() - exp {
                parts[3] = Part::Zero(frac_digits - (buf.len() - exp));
                &parts[..4]
            } else {
                &parts[..3]
            }
        } else {
            // the decimal point is after rendered digits: [1234][____0000] or [1234][__][.][__].
            parts[0] = Part::Copy(core::str::from_utf8(buf).unwrap());
            parts[1] = Part::Zero(exp - buf.len());
            if frac_digits > 0 {
                parts[2] = Part::Copy(".");
                parts[3] = Part::Zero(frac_digits);
                &parts[..4]
            } else {
                &parts[..2]
            }
        }
    }
}

/// Formats the given decimal digits `0.<...buf...> * 10^exp` into the exponential
/// form with at least the given number of significant digits. When `upper` is `true`,
/// the exponent will be prefixed by `E`; otherwise that's `e`. The result is
/// stored to the supplied parts array and a slice of written parts is returned.
///
/// `min_digits` can be less than the number of actual significant digits in `buf`;
/// it will be ignored and full digits will be printed. It is only used to print
/// additional zeroes after rendered digits. Thus, `min_digits == 0` means that
/// it will only print the given digits and nothing else.
fn digits_to_exp_str<'a>(
    buf: &'a [u8],
    exp: i16,
    min_ndigits: usize,
    upper: bool,
    parts: &'a mut [Part<'a>],
) -> &'a [Part<'a>] {
    assert!(!buf.is_empty());
    assert!(buf[0] > b'0');
    assert!(parts.len() >= 6);

    let mut n = 0;

    parts[n] = Part::Copy(core::str::from_utf8(&buf[..1]).unwrap());
    n += 1;

    if buf.len() > 1 || min_ndigits > 1 {
        parts[n] = Part::Copy(".");
        parts[n + 1] = Part::Copy(core::str::from_utf8(&buf[1..]).unwrap());
        n += 2;
        if min_ndigits > buf.len() {
            parts[n] = Part::Zero(min_ndigits - buf.len());
            n += 1;
        }
    }

    // 0.1234 x 10^exp = 1.234 x 10^(exp-1)
    let exp = exp as i32 - 1; // avoid underflow when exp is i16::MIN
    if exp < 0 {
        parts[n] = Part::Copy(if upper { "E-" } else { "e-" });
        parts[n + 1] = Part::Num(-exp as u16);
    } else {
        parts[n] = Part::Copy(if upper { "E" } else { "e" });
        parts[n + 1] = Part::Num(exp as u16);
    }
    &parts[..n + 2]
}

/// Sign formatting options.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum Sign {
    /// Prints `-` for any negative value.
    Minus, // -inf -1 -0  0  1  inf nan
    /// Prints `-` for any negative value, or `+` otherwise.
    MinusPlus, // -inf -1 -0 +0 +1 +inf nan
}

/// Returns the static byte string corresponding to the sign to be formatted.
/// It can be either `""`, `"+"` or `"-"`.
fn determine_sign(sign: Sign, decoded: &FullDecoded, negative: bool) -> &'static str {
    match (*decoded, sign) {
        (FullDecoded::Nan, _) => "",
        (_, Sign::Minus) => {
            if negative {
                "-"
            } else {
                ""
            }
        }
        (_, Sign::MinusPlus) => {
            if negative {
                "-"
            } else {
                "+"
            }
        }
    }
}

/// Formats the given floating point number into the decimal form with at least
/// given number of fractional digits. The result is stored to the supplied parts
/// array while utilizing given byte buffer as a scratch. `upper` is currently
/// unused but left for the future decision to change the case of non-finite values,
/// i.e., `inf` and `nan`. The first part to be rendered is always a `Part::Sign`
/// (which can be an empty string if no sign is rendered).
///
/// `format_shortest` should be the underlying digit-generation function.
/// It should return the part of the buffer that it initialized.
/// You probably would want `strategy::grisu::format_shortest` for this.
///
/// `frac_digits` can be less than the number of actual fractional digits in `v`;
/// it will be ignored and full digits will be printed. It is only used to print
/// additional zeroes after rendered digits. Thus `frac_digits` of 0 means that
/// it will only print given digits and nothing else.
///
/// The byte buffer should be at least `MAX_SIG_DIGITS` bytes long.
/// There should be at least 4 parts available, due to the worst case like
/// `[+][0.][0000][2][0000]` with `frac_digits = 10`.
pub(super) fn to_shortest_str<'a, T, F>(
    mut format_shortest: F,
    v: T,
    sign: Sign,
    frac_digits: usize,
    buf: &'a mut [u8],
    parts: &'a mut [Part<'a>],
) -> Formatted<'a>
where
    T: DecodableFloat,
    F: FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
{
    assert!(parts.len() >= 4);
    assert!(buf.len() >= MAX_SIG_DIGITS);

    let (negative, full_decoded) = decode(v);
    let sign = determine_sign(sign, &full_decoded, negative);
    match full_decoded {
        FullDecoded::Nan => {
            parts[0] = Part::Copy("NaN");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Infinite => {
            parts[0] = Part::Copy("inf");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Zero => {
            if frac_digits > 0 {
                // [0.][0000]
                parts[0] = Part::Copy("0.");
                parts[1] = Part::Zero(frac_digits);
                Formatted {
                    sign,
                    parts: &parts[..2],
                }
            } else {
                parts[0] = Part::Copy("0");
                Formatted {
                    sign,
                    parts: &parts[..1],
                }
            }
        }
        FullDecoded::Finite(ref decoded) => {
            let (buf, exp) = format_shortest(decoded, buf);
            Formatted {
                sign,
                parts: digits_to_dec_str(buf, exp, frac_digits, parts),
            }
        }
    }
}

/// Formats the given floating point number into the decimal form or
/// the exponential form, depending on the resulting exponent. The result is
/// stored to the supplied parts array while utilizing given byte buffer
/// as a scratch. `upper` is used to determine the case of non-finite values
/// (`inf` and `nan`) or the case of the exponent prefix (`e` or `E`).
/// The first part to be rendered is always a `Part::Sign` (which can be
/// an empty string if no sign is rendered).
///
/// `format_shortest` should be the underlying digit-generation function.
/// It should return the part of the buffer that it initialized.
/// You probably would want `strategy::grisu::format_shortest` for this.
///
/// The `dec_bounds` is a tuple `(lo, hi)` such that the number is formatted
/// as decimal only when `10^lo <= V < 10^hi`. Note that this is the *apparent* `V`
/// instead of the actual `v`! Thus any printed exponent in the exponential form
/// cannot be in this range, avoiding any confusion.
///
/// The byte buffer should be at least `MAX_SIG_DIGITS` bytes long.
/// There should be at least 6 parts available, due to the worst case like
/// `[+][1][.][2345][e][-][6]`.
pub(super) fn to_shortest_exp_str<'a, T, F>(
    mut format_shortest: F,
    v: T,
    sign: Sign,
    dec_bounds: (i16, i16),
    upper: bool,
    buf: &'a mut [u8],
    parts: &'a mut [Part<'a>],
) -> Formatted<'a>
where
    T: DecodableFloat,
    F: FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
{
    assert!(parts.len() >= 6);
    assert!(buf.len() >= MAX_SIG_DIGITS);
    assert!(dec_bounds.0 <= dec_bounds.1);

    let (negative, full_decoded) = decode(v);
    let sign = determine_sign(sign, &full_decoded, negative);
    match full_decoded {
        FullDecoded::Nan => {
            parts[0] = Part::Copy("NaN");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Infinite => {
            parts[0] = Part::Copy("inf");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Zero => {
            parts[0] = if dec_bounds.0 <= 0 && 0 < dec_bounds.1 {
                Part::Copy("0")
            } else {
                Part::Copy(if upper { "0E0" } else { "0e0" })
            };
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Finite(ref decoded) => {
            let (buf, exp) = format_shortest(decoded, buf);
            let vis_exp = exp as i32 - 1;
            let parts = if dec_bounds.0 as i32 <= vis_exp && vis_exp < dec_bounds.1 as i32 {
                digits_to_dec_str(buf, exp, 0, parts)
            } else {
                digits_to_exp_str(buf, exp, 0, upper, parts)
            };
            Formatted { sign, parts }
        }
    }
}

/// Returns a rather crude approximation (upper bound) for the maximum buffer size
/// calculated from the given decoded exponent.
///
/// The exact limit is:
///
/// - when `exp < 0`, the maximum length is `ceil(log_10 (5^-exp * (2^64 - 1)))`.
/// - when `exp >= 0`, the maximum length is `ceil(log_10 (2^exp * (2^64 - 1)))`.
///
/// `ceil(log_10 (x^exp * (2^64 - 1)))` is less than `ceil(log_10 (2^64 - 1)) +
/// ceil(exp * log_10 x)`, which is in turn less than `20 + (1 + exp * log_10 x)`.
/// We use the facts that `log_10 2 < 5/16` and `log_10 5 < 12/16`, which is
/// enough for our purposes.
///
/// Why do we need this? `format_exact` functions will fill the entire buffer
/// unless limited by the last digit restriction, but it is possible that
/// the number of digits requested is ridiculously large (say, 30,000 digits).
/// The vast majority of buffer will be filled with zeroes, so we don't want to
/// allocate all the buffer beforehand. Consequently, for any given arguments,
/// 826 bytes of buffer should be sufficient for `f64`. Compare this with
/// the actual number for the worst case: 770 bytes (when `exp = -1074`).
fn estimate_max_buf_len(exp: i16) -> usize {
    21 + ((if exp < 0 { -12 } else { 5 } * exp as i32) as usize >> 4)
}

/// Formats given floating point number into the exponential form with
/// exactly given number of significant digits. The result is stored to
/// the supplied parts array while utilizing given byte buffer as a scratch.
/// `upper` is used to determine the case of the exponent prefix (`e` or `E`).
/// The first part to be rendered is always a `Part::Sign` (which can be
/// an empty string if no sign is rendered).
///
/// `format_exact` should be the underlying digit-generation function.
/// It should return the part of the buffer that it initialized.
/// You probably would want `strategy::grisu::format_exact` for this.
///
/// The byte buffer should be at least `ndigits` bytes long unless `ndigits` is
/// so large that only the fixed number of digits will be ever written.
/// (The tipping point for `f64` is about 800, so 1000 bytes should be enough.)
/// There should be at least 6 parts available, due to the worst case like
/// `[+][1][.][2345][e][-][6]`.
pub(super) fn to_exact_exp_str<'a, T, F>(
    mut format_exact: F,
    v: T,
    sign: Sign,
    ndigits: usize,
    upper: bool,
    buf: &'a mut [u8],
    parts: &'a mut [Part<'a>],
) -> Formatted<'a>
where
    T: DecodableFloat,
    F: FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
{
    assert!(parts.len() >= 6);
    assert!(ndigits > 0);

    let (negative, full_decoded) = decode(v);
    let sign = determine_sign(sign, &full_decoded, negative);
    match full_decoded {
        FullDecoded::Nan => {
            parts[0] = Part::Copy("NaN");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Infinite => {
            parts[0] = Part::Copy("inf");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Zero => {
            if ndigits > 1 {
                // [0.][0000][e0]
                parts[0] = Part::Copy("0.");
                parts[1] = Part::Zero(ndigits - 1);
                parts[2] = Part::Copy(if upper { "E0" } else { "e0" });
                Formatted {
                    sign,
                    parts: &parts[..3],
                }
            } else {
                parts[0] = Part::Copy(if upper { "0E0" } else { "0e0" });
                Formatted {
                    sign,
                    parts: &parts[..1],
                }
            }
        }
        FullDecoded::Finite(ref decoded) => {
            let maxlen = estimate_max_buf_len(decoded.exp);
            assert!(buf.len() >= ndigits || buf.len() >= maxlen);

            let trunc = if ndigits < maxlen { ndigits } else { maxlen };
            let (buf, exp) = format_exact(decoded, &mut buf[..trunc], i16::MIN);
            Formatted {
                sign,
                parts: digits_to_exp_str(buf, exp, ndigits, upper, parts),
            }
        }
    }
}

/// Formats given floating point number into the decimal form with exactly
/// given number of fractional digits. The result is stored to the supplied parts
/// array while utilizing given byte buffer as a scratch. `upper` is currently
/// unused but left for the future decision to change the case of non-finite values,
/// i.e., `inf` and `nan`. The first part to be rendered is always a `Part::Sign`
/// (which can be an empty string if no sign is rendered).
///
/// `format_exact` should be the underlying digit-generation function.
/// It should return the part of the buffer that it initialized.
/// You probably would want `strategy::grisu::format_exact` for this.
///
/// The byte buffer should be enough for the output unless `frac_digits` is
/// so large that only the fixed number of digits will be ever written.
/// (The tipping point for `f64` is about 800, and 1000 bytes should be enough.)
/// There should be at least 4 parts available, due to the worst case like
/// `[+][0.][0000][2][0000]` with `frac_digits = 10`.
pub(super) fn to_exact_fixed_str<'a, T, F>(
    mut format_exact: F,
    v: T,
    sign: Sign,
    frac_digits: usize,
    buf: &'a mut [u8],
    parts: &'a mut [Part<'a>],
) -> Formatted<'a>
where
    T: DecodableFloat,
    F: FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
{
    assert!(parts.len() >= 4);

    let (negative, full_decoded) = decode(v);
    let sign = determine_sign(sign, &full_decoded, negative);
    match full_decoded {
        FullDecoded::Nan => {
            parts[0] = Part::Copy("NaN");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Infinite => {
            parts[0] = Part::Copy("inf");
            Formatted {
                sign,
                parts: &parts[..1],
            }
        }
        FullDecoded::Zero => {
            if frac_digits > 0 {
                // [0.][0000]
                parts[0] = Part::Copy("0.");
                parts[1] = Part::Zero(frac_digits);
                Formatted {
                    sign,
                    parts: &parts[..2],
                }
            } else {
                parts[0] = Part::Copy("0");
                Formatted {
                    sign,
                    parts: &parts[..1],
                }
            }
        }
        FullDecoded::Finite(ref decoded) => {
            let maxlen = estimate_max_buf_len(decoded.exp);
            assert!(buf.len() >= maxlen);

            // it *is* possible that `frac_digits` is ridiculously large.
            // `format_exact` will end rendering digits much earlier in this case,
            // because we are strictly limited by `maxlen`.
            let limit = if frac_digits < 0x8000 {
                -(frac_digits as i16)
            } else {
                i16::MIN
            };
            let (buf, exp) = format_exact(decoded, &mut buf[..maxlen], limit);
            if exp <= limit {
                // the restriction couldn't been met, so this should render like zero no matter
                // `exp` was. this does not include the case that the restriction has been met
                // only after the final rounding-up; it's a regular case with `exp = limit + 1`.
                debug_assert_eq!(buf.len(), 0);
                if frac_digits > 0 {
                    // [0.][0000]
                    parts[0] = Part::Copy("0.");
                    parts[1] = Part::Zero(frac_digits);
                    Formatted {
                        sign,
                        parts: &parts[..2],
                    }
                } else {
                    parts[0] = Part::Copy("0");
                    Formatted {
                        sign,
                        parts: &parts[..1],
                    }
                }
            } else {
                Formatted {
                    sign,
                    parts: digits_to_dec_str(buf, exp, frac_digits, parts),
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_arithmetic)]
mod tests {
    use std::string::String;
    use std::{format, vec};

    use super::super::numfmt::{Formatted, Part};
    use super::Sign::{Minus, MinusPlus};
    use super::decoder::{DecodableFloat, Decoded, FullDecoded, decode};
    use super::{
        MAX_SIG_DIGITS, Sign, round_up, to_exact_exp_str, to_exact_fixed_str, to_shortest_exp_str,
        to_shortest_str,
    };
    use crate::ieee_float::IeeeFloat;

    fn decode_finite<T: DecodableFloat + std::fmt::Debug>(v: T) -> Decoded {
        match decode(v).1 {
            FullDecoded::Finite(decoded) => decoded,
            full_decoded => panic!("expected finite, got {full_decoded:?} instead for {v:?}"),
        }
    }

    macro_rules! check_shortest {
        ($f:ident($v:expr) => $buf:expr, $exp:expr) => (
            check_shortest!($f($v) => $buf, $exp;
                            "shortest mismatch for v={v}: actual {actual:?}, expected {expected:?}",
                            v = stringify!($v))
        );

        ($f:ident{$($k:ident: $v:expr),+} => $buf:expr, $exp:expr) => (
            check_shortest!($f{$($k: $v),+} => $buf, $exp;
                            "shortest mismatch for {v:?}: actual {actual:?}, expected {expected:?}",
                            v = Decoded { $($k: $v),+ })
        );

        ($f:ident($v:expr) => $buf:expr, $exp:expr; $fmt:expr, $($key:ident = $val:expr),*) => ({
            let v = $v.to_soft();
            let mut buf = [b'_'; MAX_SIG_DIGITS];
            let (buf, k) = $f(&decode_finite(v), &mut buf);
            assert!((buf, k) == ($buf, $exp),
                    $fmt, actual = (core::str::from_utf8(buf).unwrap(), k),
                          expected = (core::str::from_utf8($buf).unwrap(), $exp),
                          $($key = $val),*);
        });

        ($f:ident{$($k:ident: $v:expr),+} => $buf:expr, $exp:expr;
                                             $fmt:expr, $($key:ident = $val:expr),*) => ({
            let mut buf = [b'_'; MAX_SIG_DIGITS];
            let (buf, k) = $f(&Decoded { $($k: $v),+ }, &mut buf);
            assert!((buf, k) == ($buf, $exp),
                    $fmt, actual = (core::str::from_utf8(buf).unwrap(), k),
                          expected = (core::str::from_utf8($buf).unwrap(), $exp),
                          $($key = $val),*);
        })
    }

    macro_rules! try_exact {
        ($f:ident($decoded:expr) => $buf:expr, $expected:expr, $expectedk:expr;
                                    $fmt:expr, $($key:ident = $val:expr),*) => ({
            let (buf, k) = $f($decoded, &mut $buf[..$expected.len()], i16::MIN);
            assert!((buf, k) == ($expected, $expectedk),
                    $fmt, actual = (core::str::from_utf8(buf).unwrap(), k),
                          expected = (core::str::from_utf8($expected).unwrap(), $expectedk),
                          $($key = $val),*);
        })
    }

    macro_rules! try_fixed {
        ($f:ident($decoded:expr) => $buf:expr, $request:expr, $expected:expr, $expectedk:expr;
                                    $fmt:expr, $($key:ident = $val:expr),*) => ({
            let (buf, k) = $f($decoded, &mut $buf[..], $request);
            assert!((buf, k) == ($expected, $expectedk),
                    $fmt, actual = (core::str::from_utf8(buf).unwrap(), k),
                          expected = (core::str::from_utf8($expected).unwrap(), $expectedk),
                          $($key = $val),*);
        })
    }

    fn check_exact<F, T>(mut f: F, v: T, vstr: &str, expected: &[u8], expectedk: i16)
    where
        T: DecodableFloat + std::fmt::Debug,
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        // use a large enough buffer
        let mut buf = [b'_'; 1024];
        let mut expected_ = [b'_'; 1024];

        let decoded = decode_finite(v);
        let cut = expected.iter().position(|&c| c == b' ');

        // check significant digits
        for i in 1..cut.unwrap_or(expected.len() - 1) {
            expected_[..i].copy_from_slice(&expected[..i]);
            let mut expectedk_ = expectedk;
            if expected[i] >= b'5' {
                // check if this is a rounding-to-even case.
                // we avoid rounding ...x5000... (with infinite zeroes) to ...(x+1) when x is even.
                if !(i + 1 < expected.len()
                    && expected[i - 1] & 1 == 0
                    && expected[i] == b'5'
                    && expected[i + 1] == b' ')
                {
                    // if this returns true, expected_[..i] is all `9`s and being rounded up.
                    // we should always return `100..00` (`i` digits) instead, since that's
                    // what we can came up with `i` digits anyway. `round_up` assumes that
                    // the adjustment to the length is done by caller, which we simply ignore.
                    if round_up(&mut expected_[..i]).is_some() {
                        expectedk_ += 1;
                    }
                }
            }

            try_exact!(f(&decoded) => &mut buf, &expected_[..i], expectedk_;
                       "exact sigdigit mismatch for v={v}, i={i}: \
                        actual {actual:?}, expected {expected:?}",
                       v = vstr, i = i);
            try_fixed!(f(&decoded) => &mut buf, expectedk_ - i as i16, &expected_[..i], expectedk_;
                       "fixed sigdigit mismatch for v={v}, i={i}: \
                        actual {actual:?}, expected {expected:?}",
                       v = vstr, i = i);
        }

        // check exact rounding for zero- and negative-width cases
        let start;
        if expected[0] > b'5' {
            try_fixed!(f(&decoded) => &mut buf, expectedk, b"1", expectedk + 1;
                       "zero-width rounding-up mismatch for v={v}: \
                        actual {actual:?}, expected {expected:?}",
                       v = vstr);
            start = 1;
        } else {
            start = 0;
        }
        for i in start..-10 {
            try_fixed!(f(&decoded) => &mut buf, expectedk - i, b"", expectedk;
                       "rounding-down mismatch for v={v}, i={i}: \
                        actual {actual:?}, expected {expected:?}",
                       v = vstr, i = -i);
        }

        // check infinite zero digits
        if let Some(cut) = cut {
            for i in cut..expected.len() - 1 {
                expected_[..cut].copy_from_slice(&expected[..cut]);
                for c in &mut expected_[cut..i] {
                    *c = b'0';
                }

                try_exact!(f(&decoded) => &mut buf, &expected_[..i], expectedk;
                           "exact infzero mismatch for v={v}, i={i}: \
                            actual {actual:?}, expected {expected:?}",
                           v = vstr, i = i);
                try_fixed!(f(&decoded) => &mut buf, expectedk - i as i16, &expected_[..i], expectedk;
                           "fixed infzero mismatch for v={v}, i={i}: \
                            actual {actual:?}, expected {expected:?}",
                           v = vstr, i = i);
            }
        }
    }

    trait TestableFloat {
        type Soft: DecodableFloat + std::fmt::Debug;

        fn to_soft(self) -> Self::Soft;

        fn ldexpi(f: i64, exp: isize) -> Self::Soft;
    }

    /*impl TestableFloat for f16 {
        type Soft = IeeeFloat<crate::f16::F16Semantics>;

        fn to_soft(self) -> Self::Soft {
            IeeeFloat::from_bits(self.to_bits())
        }

        fn ldexpi(f: i64, exp: isize) -> Self::Soft {
            let rnd = crate::Round::NearestTiesToEven;
            IeeeFloat::from_int(f.into(), rnd)
                .0
                .scalbn(exp.try_into().unwrap(), rnd)
                .0
        }
    }*/

    impl TestableFloat for f32 {
        type Soft = IeeeFloat<crate::f32::F32Semantics>;

        fn to_soft(self) -> Self::Soft {
            IeeeFloat::from_bits(self.to_bits())
        }

        fn ldexpi(f: i64, exp: isize) -> Self::Soft {
            let rnd = crate::Round::NearestTiesToEven;
            IeeeFloat::from_int(f.into(), rnd)
                .0
                .scalbn(exp.try_into().unwrap(), rnd)
                .0
        }
    }

    impl TestableFloat for f64 {
        type Soft = IeeeFloat<crate::f64::F64Semantics>;

        fn to_soft(self) -> Self::Soft {
            IeeeFloat::from_bits(self.to_bits())
        }

        fn ldexpi(f: i64, exp: isize) -> Self::Soft {
            let rnd = crate::Round::NearestTiesToEven;
            IeeeFloat::from_int(f.into(), rnd)
                .0
                .scalbn(exp.try_into().unwrap(), rnd)
                .0
        }
    }

    fn check_exact_one<F, T>(
        mut f: F,
        x: i64,
        e: isize,
        tstr: &str,
        expected: &[u8],
        expectedk: i16,
    ) where
        T: TestableFloat,
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        // use a large enough buffer
        let mut buf = [b'_'; 1024];
        let v = T::ldexpi(x, e);
        let decoded = decode_finite(v);

        try_exact!(f(&decoded) => &mut buf, &expected, expectedk;
                   "exact mismatch for v={x}p{e}{t}: actual {actual:?}, expected {expected:?}",
                   x = x, e = e, t = tstr);
        try_fixed!(f(&decoded) => &mut buf, expectedk - expected.len() as i16, &expected, expectedk;
                   "fixed mismatch for v={x}p{e}{t}: actual {actual:?}, expected {expected:?}",
                   x = x, e = e, t = tstr);
    }

    macro_rules! check_exact {
        ($f:ident($v:expr) => $buf:expr, $exp:expr) => {
            let v = $v.to_soft();
            check_exact(|d, b, k| $f(d, b, k), v, stringify!($v), $buf, $exp)
        };
    }

    macro_rules! check_exact_one {
        ($f:ident($x:expr, $e:expr; $t:ty) => $buf:expr, $exp:expr) => {
            check_exact_one::<_, $t>(|d, b, k| $f(d, b, k), $x, $e, stringify!($t), $buf, $exp)
        };
    }

    // in the following comments, three numbers are spaced by 1 ulp apart,
    // and the second one is being formatted.
    //
    // some tests are derived from [1].
    //
    // [1] Vern Paxson, A Program for Testing IEEE Decimal-Binary Conversion
    //     ftp://ftp.ee.lbl.gov/testbase-report.ps.Z
    //  or https://www.icir.org/vern/papers/testbase-report.pdf

    /*pub fn f16_shortest_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [MaybeUninit<u8>]) -> (&'a [u8], i16),
    {
        // 0.0999145507813
        // 0.0999755859375
        // 0.100036621094
        check_shortest!(f(0.1f16) => b"1", 0);

        // 0.3330078125
        // 0.333251953125 (1/3 in the default rounding)
        // 0.33349609375
        check_shortest!(f(1.0f16/3.0) => b"3333", 0);

        // 10^1 * 0.3138671875
        // 10^1 * 0.3140625
        // 10^1 * 0.3142578125
        check_shortest!(f(3.14f16) => b"314", 1);

        // 10^18 * 0.31415916243714048
        // 10^18 * 0.314159196796878848
        // 10^18 * 0.314159231156617216
        check_shortest!(f(3.1415e4f16) => b"3141", 5);

        // regression test for decoders
        // 10^2 * 0.31984375
        // 10^2 * 0.32
        // 10^2 * 0.3203125
        check_shortest!(f(ldexp_f16(1.0, 5)) => b"32", 2);

        // 10^5 * 0.65472
        // 10^5 * 0.65504
        // 10^5 * 0.65536
        check_shortest!(f(f16::MAX) => b"655", 5);

        // 10^-4 * 0.60975551605224609375
        // 10^-4 * 0.6103515625
        // 10^-4 * 0.61094760894775390625
        check_shortest!(f(f16::MIN_POSITIVE) => b"6104", -4);

        // 10^-9 * 0
        // 10^-9 * 0.59604644775390625
        // 10^-8 * 0.11920928955078125
        let minf16 = ldexp_f16(1.0, -24);
        check_shortest!(f(minf16) => b"6", -7);
    }*/

    /*pub fn f16_exact_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [MaybeUninit<u8>], i16) -> (&'a [u8], i16),
    {
        let minf16 = ldexp_f16(1.0, -24);

        check_exact!(f(0.1f16)            => b"999755859375     ", -1);
        check_exact!(f(0.5f16)            => b"5                ", 0);
        check_exact!(f(1.0f16/3.0)        => b"333251953125     ", 0);
        check_exact!(f(3.141f16)          => b"3140625          ", 1);
        check_exact!(f(3.141e4f16)        => b"31408            ", 5);
        check_exact!(f(f16::MAX)          => b"65504            ", 5);
        check_exact!(f(f16::MIN_POSITIVE) => b"6103515625       ", -4);
        check_exact!(f(minf16)            => b"59604644775390625", -7);

        // FIXME(f16): these should gain the check_exact_one tests like `f32` and `f64` have,
        // but these values are not easy to generate. The algorithm from the Paxon paper [1] needs
        // to be adapted to binary16.
    }*/

    pub(super) fn f32_shortest_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
    {
        // 0.0999999940395355224609375
        // 0.100000001490116119384765625
        // 0.10000000894069671630859375
        check_shortest!(f(0.1f32) => b"1", 0);

        // 0.333333313465118408203125
        // 0.3333333432674407958984375 (1/3 in the default rounding)
        // 0.33333337306976318359375
        check_shortest!(f(1.0f32/3.0) => b"33333334", 0);

        // 10^1 * 0.31415917873382568359375
        // 10^1 * 0.31415920257568359375
        // 10^1 * 0.31415922641754150390625
        check_shortest!(f(3.141592f32) => b"3141592", 1);

        // 10^18 * 0.31415916243714048
        // 10^18 * 0.314159196796878848
        // 10^18 * 0.314159231156617216
        check_shortest!(f(3.141592e17f32) => b"3141592", 18);

        // regression test for decoders
        // 10^8 * 0.3355443
        // 10^8 * 0.33554432
        // 10^8 * 0.33554436
        check_shortest!(f((1u32 << 25) as f32) => b"33554432", 8);

        // 10^39 * 0.340282326356119256160033759537265639424
        // 10^39 * 0.34028234663852885981170418348451692544
        // 10^39 * 0.340282366920938463463374607431768211456
        check_shortest!(f(f32::MAX) => b"34028235", 39);

        // 10^-37 * 0.1175494210692441075487029444849287348827...
        // 10^-37 * 0.1175494350822287507968736537222245677818...
        // 10^-37 * 0.1175494490952133940450443629595204006810...
        check_shortest!(f(f32::MIN_POSITIVE) => b"11754944", -37);

        // 10^-44 * 0
        // 10^-44 * 0.1401298464324817070923729583289916131280...
        // 10^-44 * 0.2802596928649634141847459166579832262560...
        let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)
        check_shortest!(f(minf32) => b"1", -44);
    }

    pub(super) fn f32_exact_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)

        check_exact!(f(0.1f32)            => b"100000001490116119384765625             ", 0);
        check_exact!(f(0.5f32)            => b"5                                       ", 0);
        check_exact!(f(1.0f32/3.0)        => b"3333333432674407958984375               ", 0);
        check_exact!(f(3.141592f32)       => b"31415920257568359375                    ", 1);
        check_exact!(f(3.141592e17f32)    => b"314159196796878848                      ", 18);
        check_exact!(f(f32::MAX)          => b"34028234663852885981170418348451692544  ", 39);
        check_exact!(f(f32::MIN_POSITIVE) => b"1175494350822287507968736537222245677818", -37);
        check_exact!(f(minf32)            => b"1401298464324817070923729583289916131280", -44);

        // [1], Table 16: Stress Inputs for Converting 24-bit Binary to Decimal, < 1/2 ULP
        check_exact_one!(f(12676506, -102; f32) => b"2",            -23);
        check_exact_one!(f(12676506, -103; f32) => b"12",           -23);
        check_exact_one!(f(15445013,   86; f32) => b"119",           34);
        check_exact_one!(f(13734123, -138; f32) => b"3941",         -34);
        check_exact_one!(f(12428269, -130; f32) => b"91308",        -32);
        check_exact_one!(f(15334037, -146; f32) => b"171900",       -36);
        check_exact_one!(f(11518287,  -41; f32) => b"5237910",       -5);
        check_exact_one!(f(12584953, -145; f32) => b"28216440",     -36);
        check_exact_one!(f(15961084, -125; f32) => b"375243281",    -30);
        check_exact_one!(f(14915817, -146; f32) => b"1672120916",   -36);
        check_exact_one!(f(10845484, -102; f32) => b"21388945814",  -23);
        check_exact_one!(f(16431059,  -61; f32) => b"712583594561", -11);

        // [1], Table 17: Stress Inputs for Converting 24-bit Binary to Decimal, > 1/2 ULP
        check_exact_one!(f(16093626,   69; f32) => b"1",             29);
        check_exact_one!(f( 9983778,   25; f32) => b"34",            15);
        check_exact_one!(f(12745034,  104; f32) => b"259",           39);
        check_exact_one!(f(12706553,   72; f32) => b"6001",          29);
        check_exact_one!(f(11005028,   45; f32) => b"38721",         21);
        check_exact_one!(f(15059547,   71; f32) => b"355584",        29);
        check_exact_one!(f(16015691,  -99; f32) => b"2526831",      -22);
        check_exact_one!(f( 8667859,   56; f32) => b"62458507",      24);
        check_exact_one!(f(14855922,  -82; f32) => b"307213267",    -17);
        check_exact_one!(f(14855922,  -83; f32) => b"1536066333",   -17);
        check_exact_one!(f(10144164, -110; f32) => b"78147796834",  -26);
        check_exact_one!(f(13248074,   95; f32) => b"524810279937",  36);
    }

    pub(super) fn f64_shortest_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
    {
        // 0.0999999999999999777955395074968691915273...
        // 0.1000000000000000055511151231257827021181...
        // 0.1000000000000000333066907387546962127089...
        check_shortest!(f(0.1f64) => b"1", 0);

        // this example is explicitly mentioned in the paper.
        // 10^3 * 0.0999999999999999857891452847979962825775...
        // 10^3 * 0.1 (exact)
        // 10^3 * 0.1000000000000000142108547152020037174224...
        check_shortest!(f(100.0f64) => b"1", 3);

        // 0.3333333333333332593184650249895639717578...
        // 0.3333333333333333148296162562473909929394... (1/3 in the default rounding)
        // 0.3333333333333333703407674875052180141210...
        check_shortest!(f(1.0f64/3.0) => b"3333333333333333", 0);

        // explicit test case for equally closest representations.
        // Dragon has its own tie-breaking rule; Grisu should fall back.
        // 10^1 * 0.1000007629394531027955395074968691915273...
        // 10^1 * 0.100000762939453125 (exact)
        // 10^1 * 0.1000007629394531472044604925031308084726...
        check_shortest!(f(1.00000762939453125f64) => b"10000076293945313", 1);

        // 10^1 * 0.3141591999999999718085064159822650253772...
        // 10^1 * 0.3141592000000000162174274009885266423225...
        // 10^1 * 0.3141592000000000606263483859947882592678...
        check_shortest!(f(3.141592f64) => b"3141592", 1);

        // 10^18 * 0.314159199999999936
        // 10^18 * 0.3141592 (exact)
        // 10^18 * 0.314159200000000064
        check_shortest!(f(3.141592e17f64) => b"3141592", 18);

        // regression test for decoders
        // 10^20 * 0.18446744073709549568
        // 10^20 * 0.18446744073709551616
        // 10^20 * 0.18446744073709555712
        check_shortest!(f((1u128 << 64) as f64) => b"18446744073709552", 20);

        // pathological case: high = 10^23 (exact). tie breaking should always prefer that.
        // 10^24 * 0.099999999999999974834176
        // 10^24 * 0.099999999999999991611392
        // 10^24 * 0.100000000000000008388608
        check_shortest!(f(1.0e23f64) => b"1", 24);

        // 10^309 * 0.1797693134862315508561243283845062402343...
        // 10^309 * 0.1797693134862315708145274237317043567980...
        // 10^309 * 0.1797693134862315907729305190789024733617...
        check_shortest!(f(f64::MAX) => b"17976931348623157", 309);

        // 10^-307 * 0.2225073858507200889024586876085859887650...
        // 10^-307 * 0.2225073858507201383090232717332404064219...
        // 10^-307 * 0.2225073858507201877155878558578948240788...
        check_shortest!(f(f64::MIN_POSITIVE) => b"22250738585072014", -307);

        // 10^-323 * 0
        // 10^-323 * 0.4940656458412465441765687928682213723650...
        // 10^-323 * 0.9881312916824930883531375857364427447301...
        let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)
        check_shortest!(f(minf64) => b"5", -323);
    }

    pub(super) fn f64_exact_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)

        check_exact!(f(0.1f64)            => b"1000000000000000055511151231257827021181", 0);
        check_exact!(f(0.45f64)           => b"4500000000000000111022302462515654042363", 0);
        check_exact!(f(0.5f64)            => b"5                                       ", 0);
        check_exact!(f(0.95f64)           => b"9499999999999999555910790149937383830547", 0);
        check_exact!(f(100.0f64)          => b"1                                       ", 3);
        check_exact!(f(999.5f64)          => b"9995000000000000000000000000000000000000", 3);
        check_exact!(f(1.0f64/3.0)        => b"3333333333333333148296162562473909929394", 0);
        check_exact!(f(3.141592f64)       => b"3141592000000000162174274009885266423225", 1);
        check_exact!(f(3.141592e17f64)    => b"3141592                                 ", 18);
        check_exact!(f(1.0e23f64)         => b"99999999999999991611392                 ", 23);
        check_exact!(f(f64::MAX)          => b"1797693134862315708145274237317043567980", 309);
        check_exact!(f(f64::MIN_POSITIVE) => b"2225073858507201383090232717332404064219", -307);
        check_exact!(f(minf64)            => b"4940656458412465441765687928682213723650\
                                               5980261432476442558568250067550727020875\
                                               1865299836361635992379796564695445717730\
                                               9266567103559397963987747960107818781263\
                                               0071319031140452784581716784898210368871\
                                               8636056998730723050006387409153564984387\
                                               3124733972731696151400317153853980741262\
                                               3856559117102665855668676818703956031062\
                                               4931945271591492455329305456544401127480\
                                               1297099995419319894090804165633245247571\
                                               4786901472678015935523861155013480352649\
                                               3472019379026810710749170333222684475333\
                                               5720832431936092382893458368060106011506\
                                               1698097530783422773183292479049825247307\
                                               7637592724787465608477820373446969953364\
                                               7017972677717585125660551199131504891101\
                                               4510378627381672509558373897335989936648\
                                               0994116420570263709027924276754456522908\
                                               7538682506419718265533447265625         ", -323);

        // [1], Table 3: Stress Inputs for Converting 53-bit Binary to Decimal, < 1/2 ULP
        check_exact_one!(f(8511030020275656,  -342; f64) => b"9",                       -87);
        check_exact_one!(f(5201988407066741,  -824; f64) => b"46",                     -232);
        check_exact_one!(f(6406892948269899,   237; f64) => b"141",                      88);
        check_exact_one!(f(8431154198732492,    72; f64) => b"3981",                     38);
        check_exact_one!(f(6475049196144587,    99; f64) => b"41040",                    46);
        check_exact_one!(f(8274307542972842,   726; f64) => b"292084",                  235);
        check_exact_one!(f(5381065484265332,  -456; f64) => b"2891946",                -121);
        check_exact_one!(f(6761728585499734, -1057; f64) => b"43787718",               -302);
        check_exact_one!(f(7976538478610756,   376; f64) => b"122770163",               130);
        check_exact_one!(f(5982403858958067,   377; f64) => b"1841552452",              130);
        check_exact_one!(f(5536995190630837,    93; f64) => b"54835744350",              44);
        check_exact_one!(f(7225450889282194,   710; f64) => b"389190181146",            230);
        check_exact_one!(f(7225450889282194,   709; f64) => b"1945950905732",           230);
        check_exact_one!(f(8703372741147379,   117; f64) => b"14460958381605",           52);
        check_exact_one!(f(8944262675275217, -1001; f64) => b"417367747458531",        -285);
        check_exact_one!(f(7459803696087692,  -707; f64) => b"1107950772878888",       -196);
        check_exact_one!(f(6080469016670379,  -381; f64) => b"12345501366327440",       -98);
        check_exact_one!(f(8385515147034757,   721; f64) => b"925031711960365024",      233);
        check_exact_one!(f(7514216811389786,  -828; f64) => b"4198047150284889840",    -233);
        check_exact_one!(f(8397297803260511,  -345; f64) => b"11716315319786511046",    -87);
        check_exact_one!(f(6733459239310543,   202; f64) => b"432810072844612493629",    77);
        check_exact_one!(f(8091450587292794,  -473; f64) => b"3317710118160031081518", -126);

        // [1], Table 4: Stress Inputs for Converting 53-bit Binary to Decimal, > 1/2 ULP
        check_exact_one!(f(6567258882077402,   952; f64) => b"3",                       303);
        check_exact_one!(f(6712731423444934,   535; f64) => b"76",                      177);
        check_exact_one!(f(6712731423444934,   534; f64) => b"378",                     177);
        check_exact_one!(f(5298405411573037,  -957; f64) => b"4350",                   -272);
        check_exact_one!(f(5137311167659507,  -144; f64) => b"23037",                   -27);
        check_exact_one!(f(6722280709661868,   363; f64) => b"126301",                  126);
        check_exact_one!(f(5344436398034927,  -169; f64) => b"7142211",                 -35);
        check_exact_one!(f(8369123604277281,  -853; f64) => b"13934574",               -240);
        check_exact_one!(f(8995822108487663,  -780; f64) => b"141463449",              -218);
        check_exact_one!(f(8942832835564782,  -383; f64) => b"4539277920",              -99);
        check_exact_one!(f(8942832835564782,  -384; f64) => b"22696389598",             -99);
        check_exact_one!(f(8942832835564782,  -385; f64) => b"113481947988",            -99);
        check_exact_one!(f(6965949469487146,  -249; f64) => b"7700366561890",           -59);
        check_exact_one!(f(6965949469487146,  -250; f64) => b"38501832809448",          -59);
        check_exact_one!(f(6965949469487146,  -251; f64) => b"192509164047238",         -59);
        check_exact_one!(f(7487252720986826,   548; f64) => b"6898586531774201",        181);
        check_exact_one!(f(5592117679628511,   164; f64) => b"13076622631878654",        66);
        check_exact_one!(f(8887055249355788,   665; f64) => b"136052020756121240",      217);
        check_exact_one!(f(6994187472632449,   690; f64) => b"3592810217475959676",     224);
        check_exact_one!(f(8797576579012143,   588; f64) => b"89125197712484551899",    193);
        check_exact_one!(f(7363326733505337,   272; f64) => b"558769757362301140950",    98);
        check_exact_one!(f(8549497411294502,  -448; f64) => b"1176257830728540379990", -118);
    }

    pub(super) fn more_shortest_sanity_test<F>(mut f: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
    {
        check_shortest!(f{mant: 99_999_999_999_999_999, minus: 1, plus: 1,
                          exp: 0, inclusive: true} => b"1", 18);
        check_shortest!(f{mant: 99_999_999_999_999_999, minus: 1, plus: 1,
                          exp: 0, inclusive: false} => b"99999999999999999", 17);
    }

    fn to_string_with_parts<F>(mut f: F) -> String
    where
        F: for<'a> FnMut(&'a mut [u8], &'a mut [Part<'a>]) -> Formatted<'a>,
    {
        let mut buf = [0; 1024];
        let mut parts = [Part::Zero(0); 16];
        let formatted = f(&mut buf, &mut parts);
        let mut ret = vec![0; formatted.len()];
        assert_eq!(formatted.write(&mut ret), Some(ret.len()));
        String::from_utf8(ret).unwrap()
    }

    pub(super) fn to_shortest_str_test<F>(mut f_: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
    {
        fn to_string<T, F>(f: &mut F, v: T, sign: Sign, frac_digits: usize) -> String
        where
            T: TestableFloat,
            F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
        {
            let v = v.to_soft();
            to_string_with_parts(|buf, parts| {
                to_shortest_str(|d, b| f(d, b), v, sign, frac_digits, buf, parts)
            })
        }

        let f = &mut f_;

        assert_eq!(to_string(f, 0.0, Minus, 0), "0");
        assert_eq!(to_string(f, 0.0, Minus, 0), "0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 0), "+0");
        assert_eq!(to_string(f, -0.0, Minus, 0), "-0");
        assert_eq!(to_string(f, -0.0, MinusPlus, 0), "-0");
        assert_eq!(to_string(f, 0.0, Minus, 1), "0.0");
        assert_eq!(to_string(f, 0.0, Minus, 1), "0.0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 1), "+0.0");
        assert_eq!(to_string(f, -0.0, Minus, 8), "-0.00000000");
        assert_eq!(to_string(f, -0.0, MinusPlus, 8), "-0.00000000");

        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 0), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 0), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, MinusPlus, 0), "+inf");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 0), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 1), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, MinusPlus, 64), "NaN");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 0), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 1), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, MinusPlus, 64), "-inf");

        assert_eq!(to_string(f, 3.14, Minus, 0), "3.14");
        assert_eq!(to_string(f, 3.14, Minus, 0), "3.14");
        assert_eq!(to_string(f, 3.14, MinusPlus, 0), "+3.14");
        assert_eq!(to_string(f, -3.14, Minus, 0), "-3.14");
        assert_eq!(to_string(f, -3.14, Minus, 0), "-3.14");
        assert_eq!(to_string(f, -3.14, MinusPlus, 0), "-3.14");
        assert_eq!(to_string(f, 3.14, Minus, 1), "3.14");
        assert_eq!(to_string(f, 3.14, Minus, 2), "3.14");
        assert_eq!(to_string(f, 3.14, MinusPlus, 4), "+3.1400");
        assert_eq!(to_string(f, -3.14, Minus, 8), "-3.14000000");
        assert_eq!(to_string(f, -3.14, Minus, 8), "-3.14000000");
        assert_eq!(to_string(f, -3.14, MinusPlus, 8), "-3.14000000");

        assert_eq!(to_string(f, 7.5e-11, Minus, 0), "0.000000000075");
        assert_eq!(to_string(f, 7.5e-11, Minus, 3), "0.000000000075");
        assert_eq!(to_string(f, 7.5e-11, Minus, 12), "0.000000000075");
        assert_eq!(to_string(f, 7.5e-11, Minus, 13), "0.0000000000750");

        assert_eq!(to_string(f, 1.9971e20, Minus, 0), "199710000000000000000");
        assert_eq!(to_string(f, 1.9971e20, Minus, 1), "199710000000000000000.0");
        assert_eq!(
            to_string(f, 1.9971e20, Minus, 8),
            "199710000000000000000.00000000"
        );

        /*{
            // f16
            assert_eq!(to_string(f, f16::MAX, Minus, 0), "65500");
            assert_eq!(to_string(f, f16::MAX, Minus, 1), "65500.0");
            assert_eq!(to_string(f, f16::MAX, Minus, 8), "65500.00000000");

            let minf16 = ldexp_f16(1.0, -24);
            assert_eq!(to_string(f, minf16, Minus, 0), "0.00000006");
            assert_eq!(to_string(f, minf16, Minus, 8), "0.00000006");
            assert_eq!(to_string(f, minf16, Minus, 9), "0.000000060");
        }*/

        {
            // f32
            assert_eq!(
                to_string(f, f32::MAX, Minus, 0),
                format!("34028235{:0>31}", "")
            );
            assert_eq!(
                to_string(f, f32::MAX, Minus, 1),
                format!("34028235{:0>31}.0", "")
            );
            assert_eq!(
                to_string(f, f32::MAX, Minus, 8),
                format!("34028235{:0>31}.00000000", "")
            );

            let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)
            assert_eq!(to_string(f, minf32, Minus, 0), format!("0.{:0>44}1", ""));
            assert_eq!(to_string(f, minf32, Minus, 45), format!("0.{:0>44}1", ""));
            assert_eq!(to_string(f, minf32, Minus, 46), format!("0.{:0>44}10", ""));
        }

        {
            // f64
            assert_eq!(
                to_string(f, f64::MAX, Minus, 0),
                format!("17976931348623157{:0>292}", "")
            );
            assert_eq!(
                to_string(f, f64::MAX, Minus, 1),
                format!("17976931348623157{:0>292}.0", "")
            );
            assert_eq!(
                to_string(f, f64::MAX, Minus, 8),
                format!("17976931348623157{:0>292}.00000000", "")
            );

            let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)
            assert_eq!(to_string(f, minf64, Minus, 0), format!("0.{:0>323}5", ""));
            assert_eq!(to_string(f, minf64, Minus, 324), format!("0.{:0>323}5", ""));
            assert_eq!(
                to_string(f, minf64, Minus, 325),
                format!("0.{:0>323}50", "")
            );
        }

        if cfg!(miri) {
            // Miri is too slow
            return;
        }

        // very large output
        assert_eq!(
            to_string(f, 1.1, Minus, 50000),
            format!("1.1{:0>49999}", "")
        );
    }

    pub(super) fn to_shortest_exp_str_test<F>(mut f_: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
    {
        fn to_string<T, F>(
            f: &mut F,
            v: T,
            sign: Sign,
            exp_bounds: (i16, i16),
            upper: bool,
        ) -> String
        where
            T: TestableFloat,
            F: for<'a> FnMut(&Decoded, &'a mut [u8]) -> (&'a [u8], i16),
        {
            let v = v.to_soft();
            to_string_with_parts(|buf, parts| {
                to_shortest_exp_str(|d, b| f(d, b), v, sign, exp_bounds, upper, buf, parts)
            })
        }

        let f = &mut f_;

        assert_eq!(to_string(f, 0.0, Minus, (-4, 16), false), "0");
        assert_eq!(to_string(f, 0.0, Minus, (-4, 16), false), "0");
        assert_eq!(to_string(f, 0.0, MinusPlus, (-4, 16), false), "+0");
        assert_eq!(to_string(f, -0.0, Minus, (-4, 16), false), "-0");
        assert_eq!(to_string(f, -0.0, MinusPlus, (-4, 16), false), "-0");
        assert_eq!(to_string(f, 0.0, Minus, (0, 0), true), "0E0");
        assert_eq!(to_string(f, 0.0, Minus, (0, 0), false), "0e0");
        assert_eq!(to_string(f, 0.0, MinusPlus, (5, 9), false), "+0e0");
        assert_eq!(to_string(f, -0.0, Minus, (0, 0), true), "-0E0");
        assert_eq!(to_string(f, -0.0, MinusPlus, (5, 9), false), "-0e0");

        assert_eq!(to_string(f, 1.0 / 0.0, Minus, (-4, 16), false), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, Minus, (-4, 16), true), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, MinusPlus, (-4, 16), true), "+inf");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, (0, 0), false), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, (0, 0), true), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, MinusPlus, (5, 9), true), "NaN");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, (0, 0), false), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, (0, 0), true), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, MinusPlus, (5, 9), true), "-inf");

        assert_eq!(to_string(f, 3.14, Minus, (-4, 16), false), "3.14");
        assert_eq!(to_string(f, 3.14, MinusPlus, (-4, 16), false), "+3.14");
        assert_eq!(to_string(f, -3.14, Minus, (-4, 16), false), "-3.14");
        assert_eq!(to_string(f, -3.14, MinusPlus, (-4, 16), false), "-3.14");
        assert_eq!(to_string(f, 3.14, Minus, (0, 0), true), "3.14E0");
        assert_eq!(to_string(f, 3.14, Minus, (0, 0), false), "3.14e0");
        assert_eq!(to_string(f, 3.14, MinusPlus, (5, 9), false), "+3.14e0");
        assert_eq!(to_string(f, -3.14, Minus, (0, 0), true), "-3.14E0");
        assert_eq!(to_string(f, -3.14, Minus, (0, 0), false), "-3.14e0");
        assert_eq!(to_string(f, -3.14, MinusPlus, (5, 9), false), "-3.14e0");

        assert_eq!(to_string(f, 0.1, Minus, (-4, 16), false), "0.1");
        assert_eq!(to_string(f, 0.1, Minus, (-4, 16), false), "0.1");
        assert_eq!(to_string(f, 0.1, MinusPlus, (-4, 16), false), "+0.1");
        assert_eq!(to_string(f, -0.1, Minus, (-4, 16), false), "-0.1");
        assert_eq!(to_string(f, -0.1, MinusPlus, (-4, 16), false), "-0.1");
        assert_eq!(to_string(f, 0.1, Minus, (0, 0), true), "1E-1");
        assert_eq!(to_string(f, 0.1, Minus, (0, 0), false), "1e-1");
        assert_eq!(to_string(f, 0.1, MinusPlus, (5, 9), false), "+1e-1");
        assert_eq!(to_string(f, -0.1, Minus, (0, 0), true), "-1E-1");
        assert_eq!(to_string(f, -0.1, Minus, (0, 0), false), "-1e-1");
        assert_eq!(to_string(f, -0.1, MinusPlus, (5, 9), false), "-1e-1");

        assert_eq!(to_string(f, 7.5e-11, Minus, (-4, 16), false), "7.5e-11");
        assert_eq!(
            to_string(f, 7.5e-11, Minus, (-11, 10), false),
            "0.000000000075"
        );
        assert_eq!(to_string(f, 7.5e-11, Minus, (-10, 11), false), "7.5e-11");

        assert_eq!(to_string(f, 1.9971e20, Minus, (-4, 16), false), "1.9971e20");
        assert_eq!(
            to_string(f, 1.9971e20, Minus, (-20, 21), false),
            "199710000000000000000"
        );
        assert_eq!(
            to_string(f, 1.9971e20, Minus, (-21, 20), false),
            "1.9971e20"
        );

        // the true value of 1.0e23f64 is less than 10^23, but that shouldn't matter here
        assert_eq!(to_string(f, 1.0e23, Minus, (22, 23), false), "1e23");
        assert_eq!(
            to_string(f, 1.0e23, Minus, (23, 24), false),
            "100000000000000000000000"
        );
        assert_eq!(to_string(f, 1.0e23, Minus, (24, 25), false), "1e23");

        /*{
            // f16
            assert_eq!(to_string(f, f16::MAX, Minus, (-2, 2), false), "6.55e4");
            assert_eq!(to_string(f, f16::MAX, Minus, (-4, 4), false), "6.55e4");
            assert_eq!(to_string(f, f16::MAX, Minus, (-5, 5), false), "65500");

            let minf16 = ldexp_f16(1.0, -24);
            assert_eq!(to_string(f, minf16, Minus, (-2, 2), false), "6e-8");
            assert_eq!(to_string(f, minf16, Minus, (-7, 7), false), "6e-8");
            assert_eq!(to_string(f, minf16, Minus, (-8, 8), false), "0.00000006");
        }*/

        {
            // f32
            assert_eq!(
                to_string(f, f32::MAX, Minus, (-4, 16), false),
                "3.4028235e38"
            );
            assert_eq!(
                to_string(f, f32::MAX, Minus, (-39, 38), false),
                "3.4028235e38"
            );
            assert_eq!(
                to_string(f, f32::MAX, Minus, (-38, 39), false),
                format!("34028235{:0>31}", "")
            );

            let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)
            assert_eq!(to_string(f, minf32, Minus, (-4, 16), false), "1e-45");
            assert_eq!(to_string(f, minf32, Minus, (-44, 45), false), "1e-45");
            assert_eq!(
                to_string(f, minf32, Minus, (-45, 44), false),
                format!("0.{:0>44}1", "")
            );
        }

        {
            // f64
            assert_eq!(
                to_string(f, f64::MAX, Minus, (-4, 16), false),
                "1.7976931348623157e308"
            );
            assert_eq!(
                to_string(f, f64::MAX, Minus, (-308, 309), false),
                format!("17976931348623157{:0>292}", "")
            );
            assert_eq!(
                to_string(f, f64::MAX, Minus, (-309, 308), false),
                "1.7976931348623157e308"
            );

            let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)
            assert_eq!(to_string(f, minf64, Minus, (-4, 16), false), "5e-324");
            assert_eq!(
                to_string(f, minf64, Minus, (-324, 323), false),
                format!("0.{:0>323}5", "")
            );
            assert_eq!(to_string(f, minf64, Minus, (-323, 324), false), "5e-324");
        }
        assert_eq!(to_string(f, 1.1, Minus, (i16::MIN, i16::MAX), false), "1.1");
    }

    pub(super) fn to_exact_exp_str_test<F>(mut f_: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        fn to_string<T, F>(f: &mut F, v: T, sign: Sign, ndigits: usize, upper: bool) -> String
        where
            T: TestableFloat,
            F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
        {
            let v = v.to_soft();
            to_string_with_parts(|buf, parts| {
                to_exact_exp_str(|d, b, l| f(d, b, l), v, sign, ndigits, upper, buf, parts)
            })
        }

        let f = &mut f_;

        assert_eq!(to_string(f, 0.0, Minus, 1, true), "0E0");
        assert_eq!(to_string(f, 0.0, Minus, 1, false), "0e0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 1, false), "+0e0");
        assert_eq!(to_string(f, -0.0, Minus, 1, true), "-0E0");
        assert_eq!(to_string(f, -0.0, MinusPlus, 1, false), "-0e0");
        assert_eq!(to_string(f, 0.0, Minus, 2, true), "0.0E0");
        assert_eq!(to_string(f, 0.0, Minus, 2, false), "0.0e0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 2, false), "+0.0e0");
        assert_eq!(to_string(f, -0.0, Minus, 8, false), "-0.0000000e0");
        assert_eq!(to_string(f, -0.0, MinusPlus, 8, false), "-0.0000000e0");

        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 1, false), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 1, true), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, MinusPlus, 1, true), "+inf");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 8, false), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 8, true), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, MinusPlus, 8, true), "NaN");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 64, false), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 64, true), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, MinusPlus, 64, true), "-inf");

        assert_eq!(to_string(f, 3.14, Minus, 1, true), "3E0");
        assert_eq!(to_string(f, 3.14, Minus, 1, false), "3e0");
        assert_eq!(to_string(f, 3.14, MinusPlus, 1, false), "+3e0");
        assert_eq!(to_string(f, -3.14, Minus, 2, true), "-3.1E0");
        assert_eq!(to_string(f, -3.14, Minus, 2, false), "-3.1e0");
        assert_eq!(to_string(f, -3.14, MinusPlus, 2, false), "-3.1e0");
        assert_eq!(to_string(f, 3.14, Minus, 3, true), "3.14E0");
        assert_eq!(to_string(f, 3.14, Minus, 3, false), "3.14e0");
        assert_eq!(to_string(f, 3.14, MinusPlus, 3, false), "+3.14e0");
        assert_eq!(to_string(f, -3.14, Minus, 4, true), "-3.140E0");
        assert_eq!(to_string(f, -3.14, Minus, 4, false), "-3.140e0");
        assert_eq!(to_string(f, -3.14, MinusPlus, 4, false), "-3.140e0");

        assert_eq!(to_string(f, 0.195, Minus, 1, false), "2e-1");
        assert_eq!(to_string(f, 0.195, Minus, 1, true), "2E-1");
        assert_eq!(to_string(f, 0.195, MinusPlus, 1, true), "+2E-1");
        assert_eq!(to_string(f, -0.195, Minus, 2, false), "-2.0e-1");
        assert_eq!(to_string(f, -0.195, Minus, 2, true), "-2.0E-1");
        assert_eq!(to_string(f, -0.195, MinusPlus, 2, true), "-2.0E-1");
        assert_eq!(to_string(f, 0.195, Minus, 3, false), "1.95e-1");
        assert_eq!(to_string(f, 0.195, Minus, 3, true), "1.95E-1");
        assert_eq!(to_string(f, 0.195, MinusPlus, 3, true), "+1.95E-1");
        assert_eq!(to_string(f, -0.195, Minus, 4, false), "-1.950e-1");
        assert_eq!(to_string(f, -0.195, Minus, 4, true), "-1.950E-1");
        assert_eq!(to_string(f, -0.195, MinusPlus, 4, true), "-1.950E-1");

        assert_eq!(to_string(f, 9.5, Minus, 1, false), "1e1");
        assert_eq!(to_string(f, 9.5, Minus, 2, false), "9.5e0");
        assert_eq!(to_string(f, 9.5, Minus, 3, false), "9.50e0");
        assert_eq!(
            to_string(f, 9.5, Minus, 30, false),
            "9.50000000000000000000000000000e0"
        );

        assert_eq!(to_string(f, 1.0e25, Minus, 1, false), "1e25");
        assert_eq!(to_string(f, 1.0e25, Minus, 2, false), "1.0e25");
        assert_eq!(
            to_string(f, 1.0e25, Minus, 15, false),
            "1.00000000000000e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 16, false),
            "1.000000000000000e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 17, false),
            "1.0000000000000001e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 18, false),
            "1.00000000000000009e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 19, false),
            "1.000000000000000091e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 20, false),
            "1.0000000000000000906e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 21, false),
            "1.00000000000000009060e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 22, false),
            "1.000000000000000090597e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 23, false),
            "1.0000000000000000905970e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 24, false),
            "1.00000000000000009059697e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 25, false),
            "1.000000000000000090596966e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 26, false),
            "1.0000000000000000905969664e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 27, false),
            "1.00000000000000009059696640e25"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 30, false),
            "1.00000000000000009059696640000e25"
        );

        assert_eq!(to_string(f, 1.0e-6, Minus, 1, false), "1e-6");
        assert_eq!(to_string(f, 1.0e-6, Minus, 2, false), "1.0e-6");
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 16, false),
            "1.000000000000000e-6"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 17, false),
            "9.9999999999999995e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 18, false),
            "9.99999999999999955e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 19, false),
            "9.999999999999999547e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 20, false),
            "9.9999999999999995475e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 30, false),
            "9.99999999999999954748111825886e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 40, false),
            "9.999999999999999547481118258862586856139e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 50, false),
            "9.9999999999999995474811182588625868561393872369081e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 60, false),
            "9.99999999999999954748111825886258685613938723690807819366455e-7"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 70, false),
            "9.999999999999999547481118258862586856139387236908078193664550781250000e-7"
        );

        /*{
            assert_eq!(to_string(f, f16::MAX, Minus, 1, false), "7e4");
            assert_eq!(to_string(f, f16::MAX, Minus, 2, false), "6.6e4");
            assert_eq!(to_string(f, f16::MAX, Minus, 4, false), "6.550e4");
            assert_eq!(to_string(f, f16::MAX, Minus, 5, false), "6.5504e4");
            assert_eq!(to_string(f, f16::MAX, Minus, 6, false), "6.55040e4");
            assert_eq!(to_string(f, f16::MAX, Minus, 16, false), "6.550400000000000e4");

            let minf16 = ldexp_f16(1.0, -24);
            assert_eq!(to_string(f, minf16, Minus, 1, false), "6e-8");
            assert_eq!(to_string(f, minf16, Minus, 2, false), "6.0e-8");
            assert_eq!(to_string(f, minf16, Minus, 4, false), "5.960e-8");
            assert_eq!(to_string(f, minf16, Minus, 8, false), "5.9604645e-8");
            assert_eq!(to_string(f, minf16, Minus, 16, false), "5.960464477539062e-8");
            assert_eq!(to_string(f, minf16, Minus, 17, false), "5.9604644775390625e-8");
            assert_eq!(to_string(f, minf16, Minus, 18, false), "5.96046447753906250e-8");
            assert_eq!(to_string(f, minf16, Minus, 24, false), "5.96046447753906250000000e-8");
        }*/

        assert_eq!(to_string(f, f32::MAX, Minus, 1, false), "3e38");
        assert_eq!(to_string(f, f32::MAX, Minus, 2, false), "3.4e38");
        assert_eq!(to_string(f, f32::MAX, Minus, 4, false), "3.403e38");
        assert_eq!(to_string(f, f32::MAX, Minus, 8, false), "3.4028235e38");
        assert_eq!(
            to_string(f, f32::MAX, Minus, 16, false),
            "3.402823466385289e38"
        );
        assert_eq!(
            to_string(f, f32::MAX, Minus, 32, false),
            "3.4028234663852885981170418348452e38"
        );
        assert_eq!(
            to_string(f, f32::MAX, Minus, 64, false),
            "3.402823466385288598117041834845169254400000000000000000000000000e38"
        );

        let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)
        assert_eq!(to_string(f, minf32, Minus, 1, false), "1e-45");
        assert_eq!(to_string(f, minf32, Minus, 2, false), "1.4e-45");
        assert_eq!(to_string(f, minf32, Minus, 4, false), "1.401e-45");
        assert_eq!(to_string(f, minf32, Minus, 8, false), "1.4012985e-45");
        assert_eq!(
            to_string(f, minf32, Minus, 16, false),
            "1.401298464324817e-45"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 32, false),
            "1.4012984643248170709237295832899e-45"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 64, false),
            "1.401298464324817070923729583289916131280261941876515771757068284e-45"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 128, false),
            "1.401298464324817070923729583289916131280261941876515771757068283\
                    8897910826858606014866381883621215820312500000000000000000000000e-45"
        );

        if cfg!(miri) {
            // Miri is too slow
            return;
        }

        assert_eq!(to_string(f, f64::MAX, Minus, 1, false), "2e308");
        assert_eq!(to_string(f, f64::MAX, Minus, 2, false), "1.8e308");
        assert_eq!(to_string(f, f64::MAX, Minus, 4, false), "1.798e308");
        assert_eq!(to_string(f, f64::MAX, Minus, 8, false), "1.7976931e308");
        assert_eq!(
            to_string(f, f64::MAX, Minus, 16, false),
            "1.797693134862316e308"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 32, false),
            "1.7976931348623157081452742373170e308"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 64, false),
            "1.797693134862315708145274237317043567980705675258449965989174768e308"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 128, false),
            "1.797693134862315708145274237317043567980705675258449965989174768\
                    0315726078002853876058955863276687817154045895351438246423432133e308"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 256, false),
            "1.797693134862315708145274237317043567980705675258449965989174768\
                    0315726078002853876058955863276687817154045895351438246423432132\
                    6889464182768467546703537516986049910576551282076245490090389328\
                    9440758685084551339423045832369032229481658085593321233482747978e308"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 512, false),
            "1.797693134862315708145274237317043567980705675258449965989174768\
                    0315726078002853876058955863276687817154045895351438246423432132\
                    6889464182768467546703537516986049910576551282076245490090389328\
                    9440758685084551339423045832369032229481658085593321233482747978\
                    2620414472316873817718091929988125040402618412485836800000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000e308"
        );

        // okay, this is becoming tough. fortunately for us, this is almost the worst case.
        let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)
        assert_eq!(to_string(f, minf64, Minus, 1, false), "5e-324");
        assert_eq!(to_string(f, minf64, Minus, 2, false), "4.9e-324");
        assert_eq!(to_string(f, minf64, Minus, 4, false), "4.941e-324");
        assert_eq!(to_string(f, minf64, Minus, 8, false), "4.9406565e-324");
        assert_eq!(
            to_string(f, minf64, Minus, 16, false),
            "4.940656458412465e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 32, false),
            "4.9406564584124654417656879286822e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 64, false),
            "4.940656458412465441765687928682213723650598026143247644255856825e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 128, false),
            "4.940656458412465441765687928682213723650598026143247644255856825\
                    0067550727020875186529983636163599237979656469544571773092665671e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 256, false),
            "4.940656458412465441765687928682213723650598026143247644255856825\
                    0067550727020875186529983636163599237979656469544571773092665671\
                    0355939796398774796010781878126300713190311404527845817167848982\
                    1036887186360569987307230500063874091535649843873124733972731696e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 512, false),
            "4.940656458412465441765687928682213723650598026143247644255856825\
                    0067550727020875186529983636163599237979656469544571773092665671\
                    0355939796398774796010781878126300713190311404527845817167848982\
                    1036887186360569987307230500063874091535649843873124733972731696\
                    1514003171538539807412623856559117102665855668676818703956031062\
                    4931945271591492455329305456544401127480129709999541931989409080\
                    4165633245247571478690147267801593552386115501348035264934720193\
                    7902681071074917033322268447533357208324319360923828934583680601e-324"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 1024, false),
            "4.940656458412465441765687928682213723650598026143247644255856825\
                    0067550727020875186529983636163599237979656469544571773092665671\
                    0355939796398774796010781878126300713190311404527845817167848982\
                    1036887186360569987307230500063874091535649843873124733972731696\
                    1514003171538539807412623856559117102665855668676818703956031062\
                    4931945271591492455329305456544401127480129709999541931989409080\
                    4165633245247571478690147267801593552386115501348035264934720193\
                    7902681071074917033322268447533357208324319360923828934583680601\
                    0601150616980975307834227731832924790498252473077637592724787465\
                    6084778203734469699533647017972677717585125660551199131504891101\
                    4510378627381672509558373897335989936648099411642057026370902792\
                    4276754456522908753868250641971826553344726562500000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000e-324"
        );

        // very large output
        assert_eq!(
            to_string(f, 0.0, Minus, 50000, false),
            format!("0.{:0>49999}e0", "")
        );
        assert_eq!(
            to_string(f, 1.0e1, Minus, 50000, false),
            format!("1.{:0>49999}e1", "")
        );
        assert_eq!(
            to_string(f, 1.0e0, Minus, 50000, false),
            format!("1.{:0>49999}e0", "")
        );
        assert_eq!(
            to_string(f, 1.0e-1, Minus, 50000, false),
            format!(
                "1.000000000000000055511151231257827021181583404541015625{:0>49945}\
                            e-1",
                ""
            )
        );
        assert_eq!(
            to_string(f, 1.0e-20, Minus, 50000, false),
            format!(
                "9.999999999999999451532714542095716517295037027873924471077157760\
                            66783064379706047475337982177734375{:0>49901}e-21",
                ""
            )
        );
    }

    pub(super) fn to_exact_fixed_str_test<F>(mut f_: F)
    where
        F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
    {
        fn to_string<T, F>(f: &mut F, v: T, sign: Sign, frac_digits: usize) -> String
        where
            T: TestableFloat,
            F: for<'a> FnMut(&Decoded, &'a mut [u8], i16) -> (&'a [u8], i16),
        {
            let v = v.to_soft();
            to_string_with_parts(|buf, parts| {
                to_exact_fixed_str(|d, b, l| f(d, b, l), v, sign, frac_digits, buf, parts)
            })
        }

        let f = &mut f_;

        assert_eq!(to_string(f, 0.0, Minus, 0), "0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 0), "+0");
        assert_eq!(to_string(f, -0.0, Minus, 0), "-0");
        assert_eq!(to_string(f, -0.0, MinusPlus, 0), "-0");
        assert_eq!(to_string(f, 0.0, Minus, 1), "0.0");
        assert_eq!(to_string(f, 0.0, MinusPlus, 1), "+0.0");
        assert_eq!(to_string(f, -0.0, Minus, 8), "-0.00000000");
        assert_eq!(to_string(f, -0.0, MinusPlus, 8), "-0.00000000");

        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 0), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, Minus, 1), "inf");
        assert_eq!(to_string(f, 1.0 / 0.0, MinusPlus, 64), "+inf");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 0), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, Minus, 1), "NaN");
        assert_eq!(to_string(f, 0.0 / 0.0, MinusPlus, 64), "NaN");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 0), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, Minus, 1), "-inf");
        assert_eq!(to_string(f, -1.0 / 0.0, MinusPlus, 64), "-inf");

        assert_eq!(to_string(f, 3.14, Minus, 0), "3");
        assert_eq!(to_string(f, 3.14, Minus, 0), "3");
        assert_eq!(to_string(f, 3.14, MinusPlus, 0), "+3");
        assert_eq!(to_string(f, -3.14, Minus, 0), "-3");
        assert_eq!(to_string(f, -3.14, Minus, 0), "-3");
        assert_eq!(to_string(f, -3.14, MinusPlus, 0), "-3");
        assert_eq!(to_string(f, 3.14, Minus, 1), "3.1");
        assert_eq!(to_string(f, 3.14, Minus, 2), "3.14");
        assert_eq!(to_string(f, 3.14, MinusPlus, 4), "+3.1400");
        assert_eq!(to_string(f, -3.14, Minus, 8), "-3.14000000");
        assert_eq!(to_string(f, -3.14, Minus, 8), "-3.14000000");
        assert_eq!(to_string(f, -3.14, MinusPlus, 8), "-3.14000000");

        assert_eq!(to_string(f, 0.195, Minus, 0), "0");
        assert_eq!(to_string(f, 0.195, MinusPlus, 0), "+0");
        assert_eq!(to_string(f, -0.195, Minus, 0), "-0");
        assert_eq!(to_string(f, -0.195, Minus, 0), "-0");
        assert_eq!(to_string(f, -0.195, MinusPlus, 0), "-0");
        assert_eq!(to_string(f, 0.195, Minus, 1), "0.2");
        assert_eq!(to_string(f, 0.195, Minus, 2), "0.20");
        assert_eq!(to_string(f, 0.195, MinusPlus, 4), "+0.1950");
        assert_eq!(to_string(f, -0.195, Minus, 5), "-0.19500");
        assert_eq!(to_string(f, -0.195, Minus, 6), "-0.195000");
        assert_eq!(to_string(f, -0.195, MinusPlus, 8), "-0.19500000");

        assert_eq!(to_string(f, 999.5, Minus, 0), "1000");
        assert_eq!(to_string(f, 999.5, Minus, 1), "999.5");
        assert_eq!(to_string(f, 999.5, Minus, 2), "999.50");
        assert_eq!(to_string(f, 999.5, Minus, 3), "999.500");
        assert_eq!(
            to_string(f, 999.5, Minus, 30),
            "999.500000000000000000000000000000"
        );

        assert_eq!(to_string(f, 0.5, Minus, 0), "0");
        assert_eq!(to_string(f, 0.5, Minus, 1), "0.5");
        assert_eq!(to_string(f, 0.5, Minus, 2), "0.50");
        assert_eq!(to_string(f, 0.5, Minus, 3), "0.500");

        assert_eq!(to_string(f, 0.95, Minus, 0), "1");
        assert_eq!(to_string(f, 0.95, Minus, 1), "0.9"); // because it really is less than 0.95
        assert_eq!(to_string(f, 0.95, Minus, 2), "0.95");
        assert_eq!(to_string(f, 0.95, Minus, 3), "0.950");
        assert_eq!(to_string(f, 0.95, Minus, 10), "0.9500000000");
        assert_eq!(
            to_string(f, 0.95, Minus, 30),
            "0.949999999999999955591079014994"
        );

        assert_eq!(to_string(f, 0.095, Minus, 0), "0");
        assert_eq!(to_string(f, 0.095, Minus, 1), "0.1");
        assert_eq!(to_string(f, 0.095, Minus, 2), "0.10");
        assert_eq!(to_string(f, 0.095, Minus, 3), "0.095");
        assert_eq!(to_string(f, 0.095, Minus, 4), "0.0950");
        assert_eq!(to_string(f, 0.095, Minus, 10), "0.0950000000");
        assert_eq!(
            to_string(f, 0.095, Minus, 30),
            "0.095000000000000001110223024625"
        );

        assert_eq!(to_string(f, 0.0095, Minus, 0), "0");
        assert_eq!(to_string(f, 0.0095, Minus, 1), "0.0");
        assert_eq!(to_string(f, 0.0095, Minus, 2), "0.01");
        assert_eq!(to_string(f, 0.0095, Minus, 3), "0.009"); // really is less than 0.0095
        assert_eq!(to_string(f, 0.0095, Minus, 4), "0.0095");
        assert_eq!(to_string(f, 0.0095, Minus, 5), "0.00950");
        assert_eq!(to_string(f, 0.0095, Minus, 10), "0.0095000000");
        assert_eq!(
            to_string(f, 0.0095, Minus, 30),
            "0.009499999999999999764077607267"
        );

        assert_eq!(to_string(f, 7.5e-11, Minus, 0), "0");
        assert_eq!(to_string(f, 7.5e-11, Minus, 3), "0.000");
        assert_eq!(to_string(f, 7.5e-11, Minus, 10), "0.0000000001");
        assert_eq!(to_string(f, 7.5e-11, Minus, 11), "0.00000000007"); // ditto
        assert_eq!(to_string(f, 7.5e-11, Minus, 12), "0.000000000075");
        assert_eq!(to_string(f, 7.5e-11, Minus, 13), "0.0000000000750");
        assert_eq!(to_string(f, 7.5e-11, Minus, 20), "0.00000000007500000000");
        assert_eq!(
            to_string(f, 7.5e-11, Minus, 30),
            "0.000000000074999999999999999501"
        );

        assert_eq!(to_string(f, 1.0e25, Minus, 0), "10000000000000000905969664");
        assert_eq!(
            to_string(f, 1.0e25, Minus, 1),
            "10000000000000000905969664.0"
        );
        assert_eq!(
            to_string(f, 1.0e25, Minus, 3),
            "10000000000000000905969664.000"
        );

        assert_eq!(to_string(f, 1.0e-6, Minus, 0), "0");
        assert_eq!(to_string(f, 1.0e-6, Minus, 3), "0.000");
        assert_eq!(to_string(f, 1.0e-6, Minus, 6), "0.000001");
        assert_eq!(to_string(f, 1.0e-6, Minus, 9), "0.000001000");
        assert_eq!(to_string(f, 1.0e-6, Minus, 12), "0.000001000000");
        assert_eq!(to_string(f, 1.0e-6, Minus, 22), "0.0000010000000000000000");
        assert_eq!(to_string(f, 1.0e-6, Minus, 23), "0.00000099999999999999995");
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 24),
            "0.000000999999999999999955"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 25),
            "0.0000009999999999999999547"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 35),
            "0.00000099999999999999995474811182589"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 45),
            "0.000000999999999999999954748111825886258685614"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 55),
            "0.0000009999999999999999547481118258862586856139387236908"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 65),
            "0.00000099999999999999995474811182588625868561393872369080781936646"
        );
        assert_eq!(
            to_string(f, 1.0e-6, Minus, 75),
            "0.000000999999999999999954748111825886258685613938723690807819366455078125000"
        );

        /*{
            assert_eq!(to_string(f, f16::MAX, Minus, 0), "65504");
            assert_eq!(to_string(f, f16::MAX, Minus, 1), "65504.0");
            assert_eq!(to_string(f, f16::MAX, Minus, 2), "65504.00");
        }*/

        assert_eq!(
            to_string(f, f32::MAX, Minus, 0),
            "340282346638528859811704183484516925440"
        );
        assert_eq!(
            to_string(f, f32::MAX, Minus, 1),
            "340282346638528859811704183484516925440.0"
        );
        assert_eq!(
            to_string(f, f32::MAX, Minus, 2),
            "340282346638528859811704183484516925440.00"
        );

        if cfg!(miri) {
            // Miri is too slow
            return;
        }

        /*{
            let minf16 = ldexp_f16(1.0, -24);
            assert_eq!(to_string(f, minf16, Minus, 0), "0");
            assert_eq!(to_string(f, minf16, Minus, 1), "0.0");
            assert_eq!(to_string(f, minf16, Minus, 2), "0.00");
            assert_eq!(to_string(f, minf16, Minus, 4), "0.0000");
            assert_eq!(to_string(f, minf16, Minus, 8), "0.00000006");
            assert_eq!(to_string(f, minf16, Minus, 10), "0.0000000596");
            assert_eq!(to_string(f, minf16, Minus, 15), "0.000000059604645");
            assert_eq!(to_string(f, minf16, Minus, 20), "0.00000005960464477539");
            assert_eq!(to_string(f, minf16, Minus, 24), "0.000000059604644775390625");
            assert_eq!(to_string(f, minf16, Minus, 32), "0.00000005960464477539062500000000");
        }*/

        let minf32 = f32::from_bits(1); // smallest subnormal f32 (2^-149)
        assert_eq!(to_string(f, minf32, Minus, 0), "0");
        assert_eq!(to_string(f, minf32, Minus, 1), "0.0");
        assert_eq!(to_string(f, minf32, Minus, 2), "0.00");
        assert_eq!(to_string(f, minf32, Minus, 4), "0.0000");
        assert_eq!(to_string(f, minf32, Minus, 8), "0.00000000");
        assert_eq!(to_string(f, minf32, Minus, 16), "0.0000000000000000");
        assert_eq!(
            to_string(f, minf32, Minus, 32),
            "0.00000000000000000000000000000000"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 64),
            "0.0000000000000000000000000000000000000000000014012984643248170709"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 128),
            "0.0000000000000000000000000000000000000000000014012984643248170709\
                    2372958328991613128026194187651577175706828388979108268586060149"
        );
        assert_eq!(
            to_string(f, minf32, Minus, 256),
            "0.0000000000000000000000000000000000000000000014012984643248170709\
                    2372958328991613128026194187651577175706828388979108268586060148\
                    6638188362121582031250000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000"
        );

        assert_eq!(
            to_string(f, f64::MAX, Minus, 0),
            "1797693134862315708145274237317043567980705675258449965989174768\
                    0315726078002853876058955863276687817154045895351438246423432132\
                    6889464182768467546703537516986049910576551282076245490090389328\
                    9440758685084551339423045832369032229481658085593321233482747978\
                    26204144723168738177180919299881250404026184124858368"
        );
        assert_eq!(
            to_string(f, f64::MAX, Minus, 10),
            "1797693134862315708145274237317043567980705675258449965989174768\
                    0315726078002853876058955863276687817154045895351438246423432132\
                    6889464182768467546703537516986049910576551282076245490090389328\
                    9440758685084551339423045832369032229481658085593321233482747978\
                    26204144723168738177180919299881250404026184124858368.0000000000"
        );

        let minf64 = f64::from_bits(1); // smallest subnormal f64 (2^-1074)
        assert_eq!(to_string(f, minf64, Minus, 0), "0");
        assert_eq!(to_string(f, minf64, Minus, 1), "0.0");
        assert_eq!(to_string(f, minf64, Minus, 10), "0.0000000000");
        assert_eq!(
            to_string(f, minf64, Minus, 100),
            "0.0000000000000000000000000000000000000000000000000000000000000000\
                    000000000000000000000000000000000000"
        );
        assert_eq!(
            to_string(f, minf64, Minus, 1000),
            "0.0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0000000000000000000000000000000000000000000000000000000000000000\
                    0004940656458412465441765687928682213723650598026143247644255856\
                    8250067550727020875186529983636163599237979656469544571773092665\
                    6710355939796398774796010781878126300713190311404527845817167848\
                    9821036887186360569987307230500063874091535649843873124733972731\
                    6961514003171538539807412623856559117102665855668676818703956031\
                    0624931945271591492455329305456544401127480129709999541931989409\
                    0804165633245247571478690147267801593552386115501348035264934720\
                    1937902681071074917033322268447533357208324319360923828934583680\
                    6010601150616980975307834227731832924790498252473077637592724787\
                    4656084778203734469699533647017972677717585125660551199131504891\
                    1014510378627381672509558373897335989937"
        );

        // very large output
        assert_eq!(to_string(f, 0.0, Minus, 50000), format!("0.{:0>50000}", ""));
        assert_eq!(
            to_string(f, 1.0e1, Minus, 50000),
            format!("10.{:0>50000}", "")
        );
        assert_eq!(
            to_string(f, 1.0e0, Minus, 50000),
            format!("1.{:0>50000}", "")
        );
        assert_eq!(
            to_string(f, 1.0e-1, Minus, 50000),
            format!(
                "0.1000000000000000055511151231257827021181583404541015625{:0>49945}",
                ""
            )
        );
        assert_eq!(
            to_string(f, 1.0e-20, Minus, 50000),
            format!(
                "0.0000000000000000000099999999999999994515327145420957165172950370\
                            2787392447107715776066783064379706047475337982177734375{:0>49881}",
                ""
            )
        );
    }
}
