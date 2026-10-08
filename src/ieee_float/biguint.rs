//! Unsigned big integers with a fixed capacity.

/// Unsigned big integer with a capacity of `N` 64-bit limbs.
///
/// Operations panic if the result does not fit.
pub(super) struct BigUInt<const N: usize> {
    /// Number of limbs in use. `limbs[len..]` are zero, and `limbs[len - 1]`
    /// is not zero (unless `len` is zero).
    len: usize,
    /// Little-endian limbs.
    limbs: [u64; N],
}

impl<const N: usize> BigUInt<N> {
    #[inline]
    pub(super) fn from_u64(value: u64) -> Self {
        let mut limbs = [0; N];
        limbs[0] = value;
        Self {
            len: usize::from(value != 0),
            limbs,
        }
    }

    /// Creates a big integer from its decimal digits, starting from the most
    /// significant one.
    pub(super) fn from_digits(digits: impl Iterator<Item = u8>) -> Self {
        // Largest `n` such that `10^n < 2^64`.
        const CHUNK_LEN: u32 = 19;

        let mut r = Self::from_u64(0);
        let mut chunk = 0;
        let mut chunk_len = 0;
        for digit in digits {
            chunk = chunk * 10 + u64::from(digit);
            chunk_len += 1;
            if chunk_len == CHUNK_LEN {
                r.mul_add_small(const { 10u64.pow(CHUNK_LEN) }, chunk);
                chunk = 0;
                chunk_len = 0;
            }
        }
        if chunk_len != 0 {
            r.mul_add_small(10u64.pow(chunk_len), chunk);
        }
        r
    }

    #[inline]
    pub(super) fn is_zero(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(super) fn bit_len(&self) -> usize {
        match self.limbs[..self.len].last() {
            Some(&top) => self.len * 64 - top.leading_zeros() as usize,
            None => 0,
        }
    }

    /// Calculates `self = self * m + a`.
    #[inline]
    fn mul_add_small(&mut self, m: u64, a: u64) {
        mul_add_small_impl(&mut self.limbs, &mut self.len, m, a);
    }

    /// Calculates `self = self * 5^n`.
    #[inline]
    pub(super) fn mul_pow5(&mut self, n: u64) {
        mul_pow5_impl(&mut self.limbs, &mut self.len, n);
    }

    /// Calculates `self = self * 2^n`.
    #[inline]
    pub(super) fn shl(&mut self, n: usize) {
        shl_impl(&mut self.limbs, &mut self.len, n);
    }

    /// Calculates `q = floor(self * 2^bits / den)`, replaces `self` with the
    /// remainder `self * 2^bits - q * den` and returns `q`.
    ///
    /// `self < den` and `1 <= bits <= 64` must hold, so `q < 2^64`, and `den`
    /// must be normalized (the most significant bit of its most significant
    /// limb must be set).
    ///
    /// This is a step of Algorithm D from "The Art of Computer Programming",
    /// Vol. 2, Section 4.3.1, by Donald E. Knuth.
    #[inline]
    pub(super) fn shl_div_rem(&mut self, bits: u32, den: &Self) -> u64 {
        debug_assert!(*self < *den);
        shl_div_rem_impl(&mut self.limbs, &mut self.len, bits, &den.limbs[..den.len])
    }
}

// The functions below implement the operations of `BigUInt` on `x`, the
// integer made of `limbs[..*len]`, where `limbs` has the whole capacity.
//
// They are not generic and never inlined, so their code is shared by the
// `BigUInt`s of every size.

/// Implementation of `BigUInt::mul_add_small`.
#[inline(never)]
fn mul_add_small_impl(limbs: &mut [u64], len: &mut usize, m: u64, a: u64) {
    debug_assert_ne!(m, 0);

    let mut carry = a;
    for limb in limbs[..*len].iter_mut() {
        (*limb, carry) = limb.carrying_mul(m, carry);
    }
    if carry != 0 {
        limbs[*len] = carry;
        *len += 1;
    }
}

/// Implementation of `BigUInt::mul_pow5`.
#[inline(never)]
fn mul_pow5_impl(limbs: &mut [u64], len: &mut usize, mut n: u64) {
    // Largest `n` such that `5^n < 2^64`.
    const MAX_SMALL_N: u32 = 27;

    while n >= u64::from(MAX_SMALL_N) {
        mul_add_small_impl(limbs, len, const { 5u64.pow(MAX_SMALL_N) }, 0);
        n -= u64::from(MAX_SMALL_N);
    }
    if n != 0 {
        mul_add_small_impl(limbs, len, 5u64.pow(n as u32), 0);
    }
}

/// Implementation of `BigUInt::shl`.
#[inline(never)]
fn shl_impl(limbs: &mut [u64], len: &mut usize, n: usize) {
    if *len == 0 {
        return;
    }

    let limb_shift = n / 64;
    let bit_shift = (n % 64) as u32;
    let mut new_len = *len + limb_shift;
    if bit_shift == 0 {
        limbs.copy_within(..*len, limb_shift);
    } else {
        let carry = limbs[*len - 1] >> (64 - bit_shift);
        if carry != 0 {
            limbs[new_len] = carry;
            new_len += 1;
        }
        // Iterate downwards, so every limb is read before it is overwritten.
        for i in (1..*len).rev() {
            limbs[i + limb_shift] = (limbs[i] << bit_shift) | (limbs[i - 1] >> (64 - bit_shift));
        }
        limbs[limb_shift] = limbs[0] << bit_shift;
    }
    limbs[..limb_shift].fill(0);
    *len = new_len;
}

/// Implementation of `BigUInt::shl_div_rem`, where `den` only has the limbs
/// in use.
#[inline(never)]
fn shl_div_rem_impl(limbs: &mut [u64], len: &mut usize, bits: u32, den: &[u64]) -> u64 {
    let n = den.len();
    debug_assert!(n != 0 && den[n - 1] >> 63 == 1);
    debug_assert!((1..=64).contains(&bits));

    // `x < den * 2^bits <= 2^(64 * (n + 1))`, so it fits in limbs `0..=n`.
    shl_impl(limbs, len, bits as usize);

    // Estimate `q` from the two most significant limbs of `x` and the most
    // significant limb of `den`. Since `den` is normalized, the estimate is
    // never too small and at most 2 too large (Theorem B in Section 4.3.1).
    let top = (u128::from(limbs[n]) << 64) | u128::from(limbs[n - 1]);
    let mut q = (top / u128::from(den[n - 1])).min(u128::from(u64::MAX)) as u64;

    // `x -= q * den`
    let mut carry = 0;
    for (limb, &d) in limbs[..n].iter_mut().zip(den) {
        // `q * d + carry <= (2^64 - 1)^2 + (2^64 - 1) = (2^64 - 1) * 2^64`,
        // so `hi` is `2^64 - 1` at most, and only when `lo` is zero, which
        // cannot borrow. Therefore, the next `carry` (`hi` plus borrow)
        // also fits in 64 bits.
        let (lo, hi) = q.carrying_mul(d, carry);
        let (diff, borrow) = limb.overflowing_sub(lo);
        *limb = diff;
        carry = hi + u64::from(borrow);
    }
    let (diff, mut negative) = limbs[n].overflowing_sub(carry);
    limbs[n] = diff;

    // While the estimate was too large, the result has wrapped around
    // `2^(64 * (n + 1))`. Adding `den` back eventually carries out of
    // limb `n`, which undoes the wraparound.
    while negative {
        q -= 1;
        let mut carry = false;
        for (limb, &d) in limbs[..n].iter_mut().zip(den) {
            let (sum, c1) = limb.overflowing_add(d);
            let (sum, c2) = sum.overflowing_add(u64::from(carry));
            *limb = sum;
            carry = c1 || c2;
        }
        let (sum, carry) = limbs[n].overflowing_add(u64::from(carry));
        limbs[n] = sum;
        negative = !carry;
    }

    *len = n + 1;
    while *len != 0 && limbs[*len - 1] == 0 {
        *len -= 1;
    }
    q
}

impl<const N: usize> PartialEq for BigUInt<N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.limbs[..self.len] == other.limbs[..other.len]
    }
}

impl<const N: usize> Eq for BigUInt<N> {}

impl<const N: usize> PartialOrd for BigUInt<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for BigUInt<N> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Since there are no leading zero limbs, a longer integer is greater.
        self.len.cmp(&other.len).then_with(|| {
            let lhs = self.limbs[..self.len].iter().rev();
            let rhs = other.limbs[..other.len].iter().rev();
            lhs.cmp(rhs)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::string::String;
    use std::vec::Vec;

    use rand::RngExt as _;

    use super::BigUInt;

    fn create_prng() -> impl rand::Rng {
        use rand::SeedableRng as _;
        rand_pcg::Pcg64::seed_from_u64(0x8E3F_51A6_0C2D_947B)
    }

    #[test]
    fn test_basic_ops() {
        fn to_rug<const N: usize>(x: &BigUInt<N>) -> rug::Integer {
            rug::Integer::from_digits(&x.limbs[..x.len], rug::integer::Order::Lsf)
        }

        let mut rng = create_prng();
        for _ in 0..1000 {
            let num_digits = rng.random_range(1..=1000);
            let mut digits = String::with_capacity(num_digits);
            for _ in 0..num_digits {
                digits.push(char::from(rng.random_range(b'0'..=b'9')));
            }
            let mut x = BigUInt::<128>::from_digits(digits.bytes().map(|c| c - b'0'));
            let mut expected: rug::Integer = digits.parse().unwrap();
            assert_eq!(to_rug(&x), expected);
            assert_eq!(x.bit_len(), expected.significant_bits() as usize);

            let n = rng.random_range(0..200);
            x.mul_pow5(n);
            expected *= rug::Integer::from(rug::Integer::u_pow_u(5, n as u32));
            assert_eq!(to_rug(&x), expected);

            let n = rng.random_range(0..300);
            x.shl(n);
            expected <<= n as u32;
            assert_eq!(to_rug(&x), expected);
            assert_eq!(x.bit_len(), expected.significant_bits() as usize);

            let y_digits = &digits[..rng.random_range(0..=digits.len())];
            let y = BigUInt::<128>::from_digits(y_digits.bytes().map(|c| c - b'0'));
            let y_rug = to_rug(&y);
            assert_eq!(x.cmp(&y), expected.cmp(&y_rug));
            assert_eq!(y.cmp(&x), y_rug.cmp(&expected));
        }
    }

    #[test]
    fn test_shl_div_rem() {
        fn to_rug(x: &BigUInt<8>) -> rug::Integer {
            rug::Integer::from_digits(&x.limbs[..x.len], rug::integer::Order::Lsf)
        }

        fn from_rug(x: &rug::Integer) -> BigUInt<8> {
            let limbs = x.to_digits::<u64>(rug::integer::Order::Lsf);
            let mut r = BigUInt::from_u64(0);
            r.limbs[..limbs.len()].copy_from_slice(&limbs);
            r.len = limbs.len();
            r
        }

        /// Generates a limb, often with an extreme value, to make the
        /// quotient estimate in `shl_div_rem` as wrong as possible.
        fn gen_limb(rng: &mut impl rand::RngExt) -> u64 {
            match rng.random_range(0..4) {
                0 => 0,
                1 => u64::MAX,
                2 => 1 << 63,
                _ => rng.random(),
            }
        }

        fn gen_int(rng: &mut impl rand::RngExt, n: usize) -> rug::Integer {
            let limbs: Vec<u64> = (0..n).map(|_| gen_limb(rng)).collect();
            rug::Integer::from_digits(&limbs, rug::integer::Order::Lsf)
        }

        let mut rng = create_prng();
        // Number of times the quotient estimate is too large by 1 or 2.
        let mut num_est_errors = [0; 3];
        for _ in 0..100_000 {
            let n = rng.random_range(1..=6);
            let den_rug = gen_int(&mut rng, n) | (rug::Integer::from(1) << (64 * n as u32 - 1));
            let num_rug = gen_int(&mut rng, n) % &den_rug;
            let bits: u32 = if rng.random() {
                64
            } else {
                rng.random_range(1..=64)
            };

            let shifted = num_rug.clone() << bits;
            let (expected_q, expected_rem) = shifted.clone().div_rem(den_rug.clone());

            let top = (shifted.clone() >> (64 * (n as u32 - 1)))
                .to_u128()
                .unwrap();
            let d1 = (den_rug.clone() >> (64 * (n as u32 - 1)))
                .to_u128()
                .unwrap();
            let estimate = (top / d1).min(u128::from(u64::MAX));
            let est_error = estimate - expected_q.to_u128().unwrap();
            num_est_errors[est_error as usize] += 1;

            let mut x = from_rug(&num_rug);
            let den = from_rug(&den_rug);
            let q = x.shl_div_rem(bits, &den);
            assert_eq!(q, expected_q);
            assert_eq!(to_rug(&x), expected_rem);
            assert_eq!(
                x.len,
                x.limbs[..x.len]
                    .iter()
                    .rposition(|&l| l != 0)
                    .map_or(0, |i| i + 1)
            );
        }
        assert!(num_est_errors[1] != 0 && num_est_errors[2] != 0);
    }
}
