use crate::simple_fp::{Sfp, SfpM64E16};

/// Returns `(k, lo, ln_hi)` such as `ln(x) = k * ln(2) + ln_hi + ln(lo)`
#[inline]
pub(crate) fn log_core(x: SfpM64E16) -> (SfpM64E16, SfpM64E16, SfpM64E16) {
    // GENERATE: ln_table SfpM64E16 4
    // LN_TBL[i] = ln(1 + i / 16)
    static LN_TBL: [SfpM64E16; 16] = [
        Sfp::ZERO,                         // 0.0
        Sfp::newp(-5, 0xF85186008B15330C), // 6.062462181643484258e-2
        Sfp::newp(-4, 0xF1383B7157972F4F), // 1.177830356563834545e-1
        Sfp::newp(-3, 0xAFF983853C9E9E44), // 1.718502569266592223e-1
        Sfp::newp(-3, 0xE47FBE3CD4D10D61), // 2.231435513142097558e-1
        Sfp::newp(-2, 0x8B3AE55D5D30701D), // 2.719337154836417588e-1
        Sfp::newp(-2, 0xA30C5E10E2F613E8), // 3.184537311185346158e-1
        Sfp::newp(-2, 0xB9CEBFB5DE8034E7), // 3.629054936893684531e-1
        Sfp::newp(-2, 0xCF991F65FCC25F96), // 4.054651081081643820e-1
        Sfp::newp(-2, 0xE47FBE3CD4D10D61), // 4.462871026284195115e-1
        Sfp::newp(-2, 0xF8947AFD7837659B), // 4.855078157817008078e-1
        Sfp::newp(-1, 0x85F39721295415B5), // 5.232481437645478365e-1
        Sfp::newp(-1, 0x8F42FAF3820681EF), // 5.596157879354226862e-1
        Sfp::newp(-1, 0x983EB99A7885F0FE), // 5.947071077466927895e-1
        Sfp::newp(-1, 0xA0EC7F4233957323), // 6.286086594223741377e-1
        Sfp::newp(-1, 0xA9516932DE2D5774), // 6.613984822453650083e-1
    ];

    // GENERATE: ln_lo_scale_table SfpM64E16 4
    // LN_LO_SCALE_TBL[i] = 1 / (1 + i / 16)
    static LN_LO_SCALE_TBL: [SfpM64E16; 16] = [
        Sfp::newp(0, 0x8000000000000000),  // 1.000000000000000000e0
        Sfp::newp(-1, 0xF0F0F0F0F0F0F0F1), // 9.411764705882352941e-1
        Sfp::newp(-1, 0xE38E38E38E38E38E), // 8.888888888888888889e-1
        Sfp::newp(-1, 0xD79435E50D79435E), // 8.421052631578947368e-1
        Sfp::newp(-1, 0xCCCCCCCCCCCCCCCD), // 8.000000000000000000e-1
        Sfp::newp(-1, 0xC30C30C30C30C30C), // 7.619047619047619048e-1
        Sfp::newp(-1, 0xBA2E8BA2E8BA2E8C), // 7.272727272727272727e-1
        Sfp::newp(-1, 0xB21642C8590B2164), // 6.956521739130434783e-1
        Sfp::newp(-1, 0xAAAAAAAAAAAAAAAB), // 6.666666666666666667e-1
        Sfp::newp(-1, 0xA3D70A3D70A3D70A), // 6.400000000000000000e-1
        Sfp::newp(-1, 0x9D89D89D89D89D8A), // 6.153846153846153846e-1
        Sfp::newp(-1, 0x97B425ED097B425F), // 5.925925925925925926e-1
        Sfp::newp(-1, 0x9249249249249249), // 5.714285714285714286e-1
        Sfp::newp(-1, 0x8D3DCB08D3DCB08D), // 5.517241379310344827e-1
        Sfp::newp(-1, 0x8888888888888889), // 5.333333333333333334e-1
        Sfp::newp(-1, 0x8421084210842108), // 5.161290322580645161e-1
    ];

    // x = 2^k * m
    let k = x.exponent();
    let m = x.mant();

    // ln(m) = ln(hi) + ln(lo)
    let hi = (m >> (64 - 5) & 0xF) as usize;
    let lo = x.set_exp(0) * LN_LO_SCALE_TBL[hi] - SfpM64E16::one();
    let ln_hi = LN_TBL[hi];

    (SfpM64E16::from_int(k), lo, ln_hi)
}
