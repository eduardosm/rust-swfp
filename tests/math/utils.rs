const SPREAD_SQRT2: u64 = 0x6A09_E667_F3BC_C909; // 2^64 * frac(sqrt(2))

/// Evenly spread values in `[0, 2^bits)` (Weyl sequence based on the golden
/// ratio).
pub(crate) fn spread(bits: u32, n: u32, seed: u64) -> impl Clone + Iterator<Item = u64> {
    const K: u64 = 0x9E37_79B9_7F4A_7C15; // 2^64 / golden ratio
    let start = seed.wrapping_mul(SPREAD_SQRT2);
    (1..=u64::from(n)).map(move |i| start.wrapping_add(i.wrapping_mul(K)) >> (64 - bits))
}

/// Evenly spread values in `[0, 2^bits)`, like [`spread`] but with up to 128
/// bits.
pub(crate) fn spread_u128(bits: u32, n: u32, seed: u64) -> impl Clone + Iterator<Item = u128> {
    const K: u128 = 0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C835; // 2^128 / golden ratio (odd)
    const SQRT2: u128 = 0x6A09_E667_F3BC_C908_B2FB_1366_EA95_7D3E; // 2^128 * frac(sqrt(2))
    let start = u128::from(seed).wrapping_mul(SQRT2);
    (1..=u128::from(n)).map(move |i| start.wrapping_add(i.wrapping_mul(K)) >> (128 - bits))
}

/// Evenly spread pairs of values in `[0, 2^bits)^2` (R2 sequence, the 2D
/// generalization of the golden ratio sequence, based on the plastic number).
///
/// The pairs are spread evenly over the square, unlike the product of two 1D
/// sequences, which repeats each coordinate many times.
pub(crate) fn spread2(bits: u32, n: u32, seed: u64) -> impl Clone + Iterator<Item = (u64, u64)> {
    const K1: u64 = 0xC13F_A9A9_02A6_328F; // 2^64 / plastic number
    const K2: u64 = 0x91E1_0DA5_C79E_7B1C; // 2^64 / plastic number^2
    let start = seed.wrapping_mul(SPREAD_SQRT2);
    (1..=u64::from(n)).map(move |i| {
        (
            start.wrapping_add(i.wrapping_mul(K1)) >> (64 - bits),
            start.wrapping_add(i.wrapping_mul(K2)) >> (64 - bits),
        )
    })
}

#[test]
fn test_spread() {
    for (bits, n) in [(8, 50), (32, 1000), (53, 100_000)] {
        let mut v: Vec<u64> = spread(bits, n, 0x1234).collect();
        assert_eq!(v.len(), n as usize);
        assert!(v.iter().all(|&x| x < (1 << bits)));

        // The values are evenly spread: the largest gap between consecutive
        // values (including the one that wraps around) is a small multiple of
        // the average gap
        v.sort_unstable();
        let max_gap = v
            .windows(2)
            .map(|w| w[1] - w[0])
            .chain([v[0] + (1 << bits) - v[v.len() - 1]])
            .max()
            .unwrap();
        assert!(max_gap <= 2 * ((1 << bits) / u64::from(n)) + 2);
    }
}

#[test]
fn test_spread_u128() {
    for (bits, n) in [(8, 50), (64, 1000), (113, 100_000)] {
        let mut v: Vec<u128> = spread_u128(bits, n, 0x1234).collect();
        assert_eq!(v.len(), n as usize);
        assert!(v.iter().all(|&x| x < (1 << bits)));

        // The values are evenly spread: the largest gap between consecutive
        // values (including the one that wraps around) is a small multiple of
        // the average gap
        v.sort_unstable();
        let max_gap = v
            .windows(2)
            .map(|w| w[1] - w[0])
            .chain([v[0] + (1 << bits) - v[v.len() - 1]])
            .max()
            .unwrap();
        assert!(max_gap <= 2 * ((1 << bits) / u128::from(n)) + 2);
    }
}

#[test]
fn test_spread2() {
    // 2^14 pairs in a grid of 2^6 x 2^6 cells, 4 per cell on average
    let bits = 16;
    let n = 1 << 14;
    let mut cells = vec![0u32; 1 << 12];
    for (a, b) in spread2(bits, n, 0x1234) {
        assert!(a < (1 << bits) && b < (1 << bits));
        cells[((a >> (bits - 6)) << 6 | (b >> (bits - 6))) as usize] += 1;
    }

    // The pairs are evenly spread: every cell has a few of them (with random
    // pairs, many cells would be empty)
    assert!(cells.iter().all(|&c| (1..=8).contains(&c)));
}
