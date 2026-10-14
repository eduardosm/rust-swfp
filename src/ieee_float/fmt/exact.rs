//! Decimal digits of a float, rounded to a given number of digits with ties
//! to even.
//!
//! When the result has few digits, `try_exact_fast` attempts to calculate it
//! as an integer, with a single division by a power of ten (see
//! `scale::Pow10::try_scale`).
//!
//! Otherwise, `exact_slow` generates the digits with big integers: the value
//! is written as `(num / den) * 10^k`, where `num / den < 1`, and each chunk
//! of 19 digits is `floor(num * 10^19 / den)`, which leaves the remainder of
//! that division in `num` for the next chunk. When the value is less than
//! 1/2, `den` is a power of two, so the division only has to split `num`.

use super::scale::{MAX_LIMBS, Pow10, SMALL_LIMBS, fraction, required_limbs};
use super::{Finite, floor_log10_pow2, u64_to_digits};
use crate::utils::{DEC_CHUNK_LEN, DEC_CHUNK_POW10, RoundLoss};

/// Position of the last digit of a rounded value.
#[derive(Clone, Copy)]
pub(super) enum Limit {
    /// Number of significant digits, which is not zero.
    SigDigits(usize),
    /// Number of fractional digits.
    FracDigits(usize),
}

/// Largest number of significant digits supported by `try_exact_fast`.
///
/// It needs an integer with one more digit to fit in a `u128`.
const MAX_FAST_SIG_DIGITS: usize = 37;

/// Largest total number of digits supported by `try_exact_fast` when the
/// number of fractional digits is given.
const MAX_FAST_FRAC_TOTAL_DIGITS: i64 = 38;

/// Returns `top` such that `2^(top - 1) <= v < 2^top`, where `v` is `value`.
#[inline]
fn calc_top(value: &Finite) -> i32 {
    value.exp + (128 - value.mant.leading_zeros()) as i32
}

/// Returns `k_up` such that `v / 10^k_up` is in `[1/20, 1)`, where `v` is
/// `value`.
#[inline]
pub(super) fn calc_k_up(value: &Finite) -> i32 {
    // `10^(k_up - 1) <= 2^top < 10^k_up`
    floor_log10_pow2(calc_top(value)) + 1
}

/// Attempts to round `value` to `limit`.
///
/// Returns `(d, k)` such that the rounded value is `d * 10^k`. With
/// `Limit::SigDigits(n)`, `d` has `n` digits. With `Limit::FracDigits(n)`,
/// `k` is `-n` and `d` might be zero.
///
/// Returns `None` when the result has too many digits or cannot be
/// determined by the fast path.
pub(super) fn try_exact_fast(value: &Finite, limit: Limit) -> Option<(u128, i32)> {
    let m = value.mant;
    let e = value.exp;

    match limit {
        Limit::SigDigits(n) => {
            if n > MAX_FAST_SIG_DIGITS {
                return None;
            }
            let n = n as u32;

            // `10^(k_lo - 1) <= 2^(top - 1)` and `2^top < 10^(k_lo + 1)`, so
            // `v` has `k_lo` or `k_lo + 1` integer digits and `x` has `n` or
            // `n + 1`.
            let top = calc_top(value);
            let k_lo = floor_log10_pow2(top - 1) + 1;
            let j = k_lo - n as i32;
            let (x, loss) = Pow10::new(j)?.try_scale(m, e)?;

            let pow10_n = 10u128.pow(n);
            let (mut d, loss, mut k) = if x >= pow10_n {
                // Drop the last digit.
                let (x, digit) = div_rem_10(x);
                let loss = match digit {
                    0 if loss == RoundLoss::Zero => RoundLoss::Zero,
                    0..=4 => RoundLoss::HalfDown,
                    5 if loss == RoundLoss::Zero => RoundLoss::Halfway,
                    _ => RoundLoss::HalfUp,
                };
                (x, loss, j + 1)
            } else {
                (x, loss, j)
            };
            if rounds_up(d, loss) {
                d += 1;
                if d == pow10_n {
                    d = pow10_n / 10;
                    k += 1;
                }
            }
            Some((d, k))
        }
        Limit::FracDigits(n) => {
            // `v < 10^k_up`, so `x < 10^max_digits`.
            let k_up = calc_k_up(value);
            let max_digits = i64::from(k_up).saturating_add(i64::try_from(n).unwrap_or(i64::MAX));
            if max_digits > MAX_FAST_FRAC_TOTAL_DIGITS {
                return None;
            }
            // `max_digits` is small, so `n` is not much greater than `-k_up`.
            let k = -(n as i32);
            if max_digits < 0 {
                // `x < 1/10`
                return Some((0, k));
            }

            let (x, loss) = Pow10::new(k)?.try_scale(m, e)?;
            Some((x + u128::from(rounds_up(x, loss)), k))
        }
    }
}

/// Returns whether a value truncated to `d`, with a truncation loss of
/// `loss`, is rounded up.
#[inline]
fn rounds_up(d: u128, loss: RoundLoss) -> bool {
    match loss {
        RoundLoss::Zero | RoundLoss::HalfDown => false,
        RoundLoss::Halfway => !d.is_multiple_of(2),
        RoundLoss::HalfUp => true,
    }
}

/// Returns `(x / 10, x % 10)`.
#[inline]
fn div_rem_10(x: u128) -> (u128, u8) {
    // 128-bit divisions are slow, so they are avoided when possible.
    if let Ok(x) = u64::try_from(x) {
        (u128::from(x / 10), (x % 10) as u8)
    } else {
        (x / 10, (x % 10) as u8)
    }
}

/// Returns a length of the buffer that is enough for
/// `exact_slow(_, value, limit)`.
pub(super) fn exact_slow_buf_len(value: &Finite, limit: Limit) -> usize {
    // `v < 10^k_up`
    let k_up = i64::from(calc_k_up(value));

    let wanted = match limit {
        Limit::SigDigits(n) => i64::try_from(n).unwrap_or(i64::MAX),
        Limit::FracDigits(n) => k_up
            .saturating_add(i64::try_from(n).unwrap_or(i64::MAX))
            .max(0),
    };
    // The last non-zero digit of `v` is not below the units when
    // `value.exp >= 0`, or below the `-value.exp`-th fractional digit
    // otherwise.
    let available = k_up + i64::from(value.exp.min(0).unsigned_abs());

    // Digits are generated in chunks, and rounding up might add one digit.
    wanted.min(available) as usize + DEC_CHUNK_LEN + 1
}

/// Rounds `value` to `limit`, writing the ASCII digits of the result at the
/// beginning of `buf`, which must have at least
/// `exact_slow_buf_len(value, limit)` bytes.
///
/// Returns `(len, k)` such that the rounded value is `0.<digits> * 10^k`,
/// where `<digits>` are `buf[..len]` followed by zeros. The first digit is
/// not zero. With `Limit::SigDigits(n)`, `len <= n`. With
/// `Limit::FracDigits(n)`, `len <= k + n`, and `len` is zero when the
/// rounded value is zero.
pub(super) fn exact_slow(buf: &mut [u8], value: &Finite, limit: Limit) -> (usize, i32) {
    debug_assert!(buf.len() >= exact_slow_buf_len(value, limit));

    let m = value.mant;
    let e = value.exp;
    // `v / 10^k_up` is in `[1/20, 1)`.
    let k_up = calc_k_up(value);

    // Using `BigUInt` sized for the worst case has a noticeable cost.
    match required_limbs(m, e, k_up) {
        0..=SMALL_LIMBS => exact_slow_impl::<SMALL_LIMBS>(buf, m, e, k_up, limit),
        _ => exact_slow_impl::<MAX_LIMBS>(buf, m, e, k_up, limit),
    }
}

/// Implementation of `exact_slow` with big integers of `N` limbs, which must
/// be at least `required_limbs(m, e, k_up)`.
// Never inlined, so the stack frame of `exact_slow` does not include the
// large `BigUInt`s of every size, only the frame of the called instance does.
#[inline(never)]
fn exact_slow_impl<const N: usize>(
    buf: &mut [u8],
    m: u128,
    e: i32,
    k_up: i32,
    limit: Limit,
) -> (usize, i32) {
    debug_assert!(N >= required_limbs(m, e, k_up));

    // `v = (num / den) * 10^k_up`
    let (mut num, den) = fraction::<N>(m, e, k_up);
    // When `k_up <= 0`, which means that `v < 1/2`, `den` is `2^den_log2`,
    // so dividing by it does not need a long division.
    let den_log2 = (k_up <= 0).then(|| den.bit_len() - 1);
    // Returns `floor(num * 10^DEC_CHUNK_LEN / den)`, which are the next
    // `DEC_CHUNK_LEN` digits, and leaves the remainder in `num`.
    let mut next_chunk = || {
        num.mul_add_small(DEC_CHUNK_POW10, 0);
        let chunk = match den_log2 {
            Some(n) => num.div_rem_pow2(n),
            None => num.shl_div_rem(0, &den),
        };
        (chunk, num.is_zero())
    };

    // `chunk[pos..]` are the generated digits that have not been used yet,
    // and `at_end` tells whether the digits after them are all zeros.
    let mut chunk = [0; DEC_CHUNK_LEN];
    let (first_chunk, mut at_end) = next_chunk();
    write_chunk(&mut chunk, first_chunk);
    // The first digit is zero when `v < 10^(k_up - 1)`.
    let mut pos = usize::from(first_chunk < DEC_CHUNK_POW10 / 10);
    let mut k = k_up - pos as i32;

    let wanted = match limit {
        Limit::SigDigits(n) => n,
        Limit::FracDigits(n) => {
            let wanted = i64::from(k).saturating_add(i64::try_from(n).unwrap_or(i64::MAX));
            if wanted < 0 {
                // `v < 10^k <= 10^(-n - 1)`, which rounds to zero.
                return (0, 0);
            }
            usize::try_from(wanted).unwrap_or(usize::MAX)
        }
    };

    let mut len = 0;
    loop {
        let n = (wanted - len).min(DEC_CHUNK_LEN - pos);
        buf[len..(len + n)].copy_from_slice(&chunk[pos..(pos + n)]);
        len += n;
        pos += n;
        if pos != DEC_CHUNK_LEN {
            // `len == wanted`, and the next digit is `chunk[pos]`.
            break;
        }
        if at_end {
            // The value is exact, because the rest of digits are zeros.
            return (len, k);
        }
        let (next, next_at_end) = next_chunk();
        write_chunk(&mut chunk, next);
        at_end = next_at_end;
        pos = 0;
    }

    // Round according to the digits that have not been used.
    let rounds_up = match chunk[pos] {
        b'0'..=b'4' => false,
        b'5' if at_end && chunk[(pos + 1)..].iter().all(|&c| c == b'0') => {
            // Tie, which rounds to even (`b'0'` is even).
            len != 0 && !buf[len - 1].is_multiple_of(2)
        }
        _ => true,
    };
    if rounds_up {
        if let Some(i) = buf[..len].iter().rposition(|&c| c != b'9') {
            buf[i] += 1;
            buf[(i + 1)..len].fill(b'0');
        } else {
            // `0.99...9 * 10^k` (or zero, when there are no digits) is
            // rounded up to `0.10...0 * 10^(k + 1)`, which has one more
            // digit when the number of fractional digits is fixed.
            k += 1;
            if matches!(limit, Limit::FracDigits(_)) {
                len += 1;
            }
            buf[0] = b'1';
            buf[1..len].fill(b'0');
        }
    }
    (len, k)
}

/// Writes the `DEC_CHUNK_LEN` ASCII digits of `value`, with leading zeros.
#[inline]
fn write_chunk(chunk: &mut [u8; DEC_CHUNK_LEN], value: u64) {
    debug_assert!(value < DEC_CHUNK_POW10);
    let start = u64_to_digits(chunk, value);
    chunk[..start].fill(b'0');
}

#[cfg(test)]
mod tests {
    use std::string::{String, ToString as _};
    use std::vec;

    use rand::RngExt as _;

    use super::super::Finite;
    use super::super::scale::required_limbs;
    use super::super::tests::{
        create_prng, exp_range, for_each_semantics, gen_finite, scale_reference,
    };
    use super::{
        Limit, calc_k_up, exact_slow, exact_slow_buf_len, exact_slow_impl, try_exact_fast,
    };
    use crate::ieee_float::Semantics;
    use crate::utils::RoundLoss;

    /// Rounds `value` to `limit` with exact integer arithmetic.
    ///
    /// Returns `(digits, point)` such that the rounded value is
    /// `0.<digits> * 10^point`, where `digits` has as many digits as `limit`
    /// allows, or it is empty when the rounded value is zero.
    fn reference(value: &Finite, limit: Limit) -> (String, i32) {
        // Returns `value / 10^j` rounded to an integer.
        let round = |j: i32| {
            let (x, loss) = scale_reference(&value.mant.into(), value.exp, j);
            match loss {
                RoundLoss::Zero | RoundLoss::HalfDown => x,
                RoundLoss::Halfway if x.is_even() => x,
                RoundLoss::Halfway | RoundLoss::HalfUp => x + 1u32,
            }
        };

        match limit {
            Limit::SigDigits(n) => {
                // `10^(k_up - 2) < v < 10^k_up`
                let k_up = calc_k_up(value);
                // The right position of the decimal point is the only one
                // that gives `n` digits.
                for point in (k_up - 1)..=(k_up + 1) {
                    let digits = round(point - n as i32).to_string();
                    if digits.len() == n {
                        return (digits, point);
                    }
                }
                unreachable!();
            }
            Limit::FracDigits(n) => {
                let x = round(-(n as i32));
                if x == 0 {
                    (String::new(), 0)
                } else {
                    let digits = x.to_string();
                    let point = digits.len() as i32 - n as i32;
                    (digits, point)
                }
            }
        }
    }

    /// Checks `digits` and `point`, which are the result of a slow path,
    /// against the reference.
    fn check_slow_result(value: &Finite, limit: Limit, digits: &[u8], point: i32) {
        let (expected_digits, expected_point) = reference(value, limit);
        let context = (value.mant, value.exp, expected_digits.as_str());

        // Trailing zeros might be omitted.
        let (expected_digits, expected_zeros) = expected_digits.as_bytes().split_at(digits.len());
        assert_eq!(digits, expected_digits, "{context:?}");
        assert!(expected_zeros.iter().all(|&c| c == b'0'), "{context:?}");
        if !digits.is_empty() {
            assert_eq!(point, expected_point, "{context:?}");
        }
    }

    /// Checks both paths for `value` rounded to `limit`.
    ///
    /// Returns whether the fast path was able to handle it.
    fn check(value: &Finite, limit: Limit) -> bool {
        let mut buf = vec![0; exact_slow_buf_len(value, limit)];
        let (len, point) = exact_slow(&mut buf, value, limit);
        check_slow_result(value, limit, &buf[..len], point);

        let fast = try_exact_fast(value, limit);
        if let Some((d, k)) = fast {
            let (expected_digits, expected_point) = reference(value, limit);
            let context = (value.mant, value.exp);
            if d == 0 {
                assert_eq!(expected_digits, "", "{context:?}");
            } else {
                let digits = d.to_string();
                assert_eq!(digits, expected_digits, "{context:?}");
                assert_eq!(k + digits.len() as i32, expected_point, "{context:?}");
            }
            if let Limit::FracDigits(n) = limit {
                assert_eq!(k, -(n as i32), "{context:?}");
            }
        }
        fast.is_some()
    }

    fn gen_limit(rng: &mut impl rand::RngExt) -> Limit {
        let n = match rng.random_range(0..20) {
            0 => rng.random_range(46..=130),
            1 => [767, 768, 800, 1100, 5000, 12000, 17000][rng.random_range(0..7)],
            _ => rng.random_range(0..=45),
        };
        if rng.random() {
            Limit::SigDigits(n + 1)
        } else {
            Limit::FracDigits(n)
        }
    }

    #[test]
    fn test_random() {
        fn test<S: Semantics>(rng: &mut impl rand::RngExt) {
            for _ in 0..3000 {
                check(&gen_finite::<S>(rng), gen_limit(rng));
            }
        }

        let mut rng = create_prng();
        for_each_semantics!(test(&mut rng));
    }

    /// Checks every value of the 8-bit formats.
    #[test]
    fn test_exhaustive() {
        fn test<S: Semantics>() {
            let (min_exp, max_exp) = exp_range::<S>();
            let min_mant = 1u128 << (S::PREC_BITS - 1);
            let max_mant = 2 * min_mant - 1;

            let check_all = |mant, exp| {
                let value = Finite {
                    mant,
                    exp,
                    asymmetric: false,
                };
                for n in 0..=24 {
                    // The fast path is always exact for these values.
                    assert!(check(&value, Limit::SigDigits(n + 1)));
                    assert!(check(&value, Limit::FracDigits(n)));
                }
            };
            for mant in 1..min_mant {
                check_all(mant, min_exp);
            }
            for exp in min_exp..=max_exp {
                for mant in min_mant..=max_mant {
                    check_all(mant, exp);
                }
            }
        }

        test::<crate::f8e5m2::F8E5M2Semantics>();
        test::<crate::f8e4m3b8nnz::F8E4M3B8NnzSemantics>();
        test::<crate::f8e4m3nao::F8E4M3NaoSemantics>();
    }

    /// Checks small multiples of negative powers of two, many of which are
    /// ties when rounded to few digits.
    #[test]
    fn test_ties() {
        fn test<S: Semantics>() {
            let (min_exp, max_exp) = exp_range::<S>();
            for int in 1..=1100u128 {
                let int_bits = 128 - int.leading_zeros();
                if int_bits > S::PREC_BITS {
                    break;
                }
                for frac_bits in 0..=6 {
                    // `int * 2^-frac_bits`
                    let shift = S::PREC_BITS - int_bits;
                    let value = Finite {
                        mant: int << shift,
                        exp: -frac_bits - shift as i32,
                        asymmetric: false,
                    };
                    if !(min_exp..=max_exp).contains(&value.exp) {
                        continue;
                    }
                    for n in 0..=7 {
                        // The fast path is always exact for these values.
                        assert!(check(&value, Limit::SigDigits(n + 1)));
                        assert!(check(&value, Limit::FracDigits(n)));
                    }
                }
            }
        }

        for_each_semantics!(test());
    }

    /// Checks that the fast path handles most values when few digits are
    /// requested.
    #[test]
    fn test_fast_path() {
        fn test<S: Semantics>(rng: &mut impl rand::RngExt, max_digits: usize) {
            let num_inputs = 20_000;
            let mut num_handled = 0;
            for _ in 0..num_inputs {
                let value = gen_finite::<S>(rng);
                let limit = Limit::SigDigits(rng.random_range(1..=max_digits));
                if check(&value, limit) {
                    num_handled += 1;
                }

                // The fast path is always exact when the power of ten is
                // small and the result has few digits (less than 19 integer
                // digits here).
                if value.exp + S::PREC_BITS as i32 <= 60 {
                    let limit = Limit::FracDigits(rng.random_range(0..=17));
                    assert!(check(&value, limit));
                }
            }
            assert!(
                num_handled >= num_inputs - num_inputs / 1000,
                "only {num_handled} of {num_inputs} inputs handled",
            );
        }

        // With more digits, there are few bits left to tell whether the
        // value is close to a tie, so the fast path gives up more often.
        let mut rng = create_prng();
        test::<crate::f32::F32Semantics>(&mut rng, 30);
        test::<crate::f64::F64Semantics>(&mut rng, 30);
        test::<crate::x87f80::X87F80Semantics>(&mut rng, 30);
        test::<crate::f128::F128Semantics>(&mut rng, 30);
    }

    /// Checks the values that need the largest buffers and big integers.
    #[test]
    fn test_extremes() {
        fn test<S: Semantics>() {
            let (min_exp, max_exp) = exp_range::<S>();
            let min_mant = 1u128 << (S::PREC_BITS - 1);
            let max_mant = 2 * min_mant - 1;

            for (mant, exp) in [
                (1, min_exp),
                (min_mant - 1, min_exp),
                (min_mant, min_exp),
                (max_mant, min_exp),
                (min_mant, max_exp),
                (max_mant - 1, max_exp),
            ] {
                let value = Finite {
                    mant,
                    exp,
                    asymmetric: false,
                };
                for n in [0, 1, 36, 37, 38, 5000, 11500, 12000, 17000, 20000] {
                    check(&value, Limit::SigDigits(n + 1));
                    check(&value, Limit::FracDigits(n));
                }
            }
        }

        for_each_semantics!(test());
    }

    /// Checks that `required_limbs` is enough by running `exact_slow_impl`
    /// with exactly that number of limbs (which panics if it is too small).
    #[test]
    fn test_required_limbs() {
        macro_rules! exact_slow_any {
            ($buf:expr, $m:expr, $e:expr, $k_up:expr, $limit:expr; $($limbs:literal)*) => {
                match required_limbs($m, $e, $k_up) {
                    $($limbs => exact_slow_impl::<$limbs>($buf, $m, $e, $k_up, $limit),)*
                    limbs => panic!("unexpected number of limbs: {limbs}"),
                }
            };
        }

        let mut rng = create_prng();
        for _ in 0..50_000 {
            let mant_bits = rng.random_range(1..=113);
            let mant = rng.random::<u128>() >> (128 - mant_bits);
            let exp = rng.random_range(-900..=900);

            // All ones is the largest significand for its number of bits.
            for mant in [mant | 1, u128::MAX >> (128 - mant_bits)] {
                let value = Finite {
                    mant,
                    exp,
                    asymmetric: false,
                };
                let limit = gen_limit(&mut rng);
                let k_up = calc_k_up(&value);

                let mut buf = vec![0; exact_slow_buf_len(&value, limit)];
                let (len, point) = exact_slow_any!(
                    &mut buf, mant, exp, k_up, limit;
                    4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20
                );
                check_slow_result(&value, limit, &buf[..len], point);
            }
        }
    }
}
