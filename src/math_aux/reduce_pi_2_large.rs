use crate::simple_fp::{Sfp, SfpM128E16};
use crate::traits::{CastInto, UInt};

/// Reduces a large angle argument `x` (in radians) with integer arithmetic
/// (Payne-Hanek), returning `(n, y)` such that:
/// * `|y| <= π/4`
/// * `0 <= n <= 3`
/// * `|x| = 2*π*M + π/2*n + y`
/// * `M` is an integer
///
/// The input is `|x| = m * 2^e`, with `m < 2^53` and `e >= 2`.
///
/// `|x| * 2/π` is calculated modulo 4 with an absolute error less than
/// 2^-201. Since its distance to the nearest integer is at least ~2^-61.5 for
/// all finite `f64`, the fraction has a relative error less than 2^-139. Its
/// highest 128 bits are multiplied by π/2, and the result is truncated to
/// `M::BITS` bits.
pub(crate) fn reduce_pi_2_large<M: UInt>(m: u64, e: i16) -> (u8, Sfp<M, i16>)
where
    u128: CastInto<M>,
{
    // GENERATE: consts SfpM128E16 FRAC_PI_2
    const FRAC_PI_2: SfpM128E16 = Sfp::newp(0, 0xC90FDAA22168C234C4C6628B80DC1CD1); // 1.5707963267948966192313216916397514421e0

    // 2 / π ~= sum(FRAC_2_PI_BITS[i] * 2^(-64 * (i + 1)))
    // GENERATE: reduce_pi_2::frac_2_pi_bits
    static FRAC_2_PI_BITS: [u64; 20] = [
        0xA2F9836E4E441529,
        0xFC2757D1F534DDC0,
        0xDB6295993C439041,
        0xFE5163ABDEBBC561,
        0xB7246E3A424DD2E0,
        0x06492EEA09D1921C,
        0xFE1DEB1CB129A73E,
        0xE88235F52EBB4484,
        0xE99C7026B45F7E41,
        0x3991D639835339F4,
        0x9C845F8BBDF9283B,
        0x1FF897FFDE05980F,
        0xEF2F118B5A0A6D1F,
        0x6D367ECF27CB09B7,
        0x4F463F669E5FEA2D,
        0x7527BAC7EBE5F17B,
        0x3D0739F78A5292EA,
        0x6BFB5FB11F8D5D08,
        0x56033046FC7B6BAB,
        0xF0CFBC209AF4361D,
    ];

    debug_assert!(m < (1 << 53));
    debug_assert!(e >= 2);

    // With `2/π = sum(b[i] * 2^-i)` (`i >= 1`), the terms of
    // `|x| * 2/π = sum(m * b[i] * 2^(e - i))` with `i < e - 1` are multiples
    // of 4, so they can be discarded. `G = sum(b[i] * 2^(e - i))` (`i >= e - 1`)
    // is taken from the bits of `FRAC_2_PI_BITS` starting at `b[e - 1]`, which
    // is bit `e - 2` (counting from 0, starting at the most significant bit of
    // the first word).
    let pos = (e - 2) as usize;
    let (w, s) = (pos / 64, pos % 64);
    let window = |i: usize| -> u64 {
        let pair = (u128::from(FRAC_2_PI_BITS[i]) << 64) | u128::from(FRAC_2_PI_BITS[i + 1]);
        ((pair << s) >> 64) as u64
    };

    // q = m * G, with 256 bits of G, so the truncation error is less than
    // 2^53 * 2^-254 = 2^-201. Only the lowest 256 bits of the result are
    // needed: bits 254..256 are the integer part (modulo 4) of `|x| * 2/π`,
    // and bits 0..254 the fraction.
    let mut q = [0u64; 4];
    let mut carry = 0;
    for (i, q_i) in q.iter_mut().enumerate().rev() {
        let t = u128::from(m) * u128::from(window(w + i)) + carry;
        *q_i = t as u64;
        carry = t >> 64;
    }

    let mut n = (q[0] >> 62) as u8;
    // fraction = (hi * 2^128 + lo) * 2^-254
    let mut hi = ((u128::from(q[0]) << 64) | u128::from(q[1])) & (u128::MAX >> 2);
    let mut lo = (u128::from(q[2]) << 64) | u128::from(q[3]);

    let neg = (hi >> 125) != 0;
    if neg {
        // Round to nearest, the fraction becomes negative (fraction - 1)
        n = n.wrapping_add(1);
        let borrow;
        (lo, borrow) = 0u128.overflowing_sub(lo);
        hi = 0u128.wrapping_sub(hi).wrapping_sub(u128::from(borrow)) & (u128::MAX >> 2);
    }

    // Normalize (lz >= 2), keeping the highest 128 bits.
    let (top, lz) = if hi != 0 {
        let lz = hi.leading_zeros();
        ((hi << lz) | (lo >> (128 - lz)), lz)
    } else if lo != 0 {
        let lz = lo.leading_zeros();
        (lo << lz, 128 + lz)
    } else {
        return (n & 3, Sfp::ZERO);
    };

    // |fraction| ~= top * 2^(128 - lz) * 2^-254 and
    // π/2 ~= FRAC_PI_2.mant() * 2^-127, so
    // |y| = |fraction| * π/2 ~= p * 2^128 * 2^(-126 - lz) * 2^-127
    let (_, p) = top.wide_mul(FRAC_PI_2.mant());
    let y = Sfp::from_int(p).scalbn(-125 - lz as i16).set_sign(neg);

    (n & 3, y)
}
