use crate::simple_fp::{Sfp, SfpM32E16};

/// Returns `(k, lo, ln_hi)` such as `ln(x) = k * ln(2) + ln_hi + ln(lo)`
#[inline]
pub(crate) fn log_core(x: SfpM32E16) -> (SfpM32E16, SfpM32E16, SfpM32E16) {
    // GENERATE: ln_table SfpM32E16 4
    // LN_TBL[i] = ln(1 + i / 16)
    static LN_TBL: [SfpM32E16; 16] = [
        Sfp::ZERO,                 // 0.0
        Sfp::newp(-5, 0xF8518601), // 6.06246218e-2
        Sfp::newp(-4, 0xF1383B71), // 1.17783036e-1
        Sfp::newp(-3, 0xAFF98385), // 1.71850257e-1
        Sfp::newp(-3, 0xE47FBE3D), // 2.23143551e-1
        Sfp::newp(-2, 0x8B3AE55D), // 2.71933715e-1
        Sfp::newp(-2, 0xA30C5E11), // 3.18453731e-1
        Sfp::newp(-2, 0xB9CEBFB6), // 3.62905494e-1
        Sfp::newp(-2, 0xCF991F66), // 4.05465108e-1
        Sfp::newp(-2, 0xE47FBE3D), // 4.46287103e-1
        Sfp::newp(-2, 0xF8947AFD), // 4.85507816e-1
        Sfp::newp(-1, 0x85F39721), // 5.23248144e-1
        Sfp::newp(-1, 0x8F42FAF4), // 5.59615788e-1
        Sfp::newp(-1, 0x983EB99A), // 5.94707108e-1
        Sfp::newp(-1, 0xA0EC7F42), // 6.28608659e-1
        Sfp::newp(-1, 0xA9516933), // 6.61398482e-1
    ];

    // GENERATE: ln_lo_scale_table SfpM32E16 4
    // LN_LO_SCALE_TBL[i] = 1 / (1 + i / 16)
    static LN_LO_SCALE_TBL: [SfpM32E16; 16] = [
        Sfp::newp(0, 0x80000000),  // 1.00000000e0
        Sfp::newp(-1, 0xF0F0F0F1), // 9.41176471e-1
        Sfp::newp(-1, 0xE38E38E4), // 8.88888889e-1
        Sfp::newp(-1, 0xD79435E5), // 8.42105263e-1
        Sfp::newp(-1, 0xCCCCCCCD), // 8.00000000e-1
        Sfp::newp(-1, 0xC30C30C3), // 7.61904762e-1
        Sfp::newp(-1, 0xBA2E8BA3), // 7.27272727e-1
        Sfp::newp(-1, 0xB21642C8), // 6.95652174e-1
        Sfp::newp(-1, 0xAAAAAAAB), // 6.66666667e-1
        Sfp::newp(-1, 0xA3D70A3D), // 6.40000000e-1
        Sfp::newp(-1, 0x9D89D89E), // 6.15384615e-1
        Sfp::newp(-1, 0x97B425ED), // 5.92592593e-1
        Sfp::newp(-1, 0x92492492), // 5.71428571e-1
        Sfp::newp(-1, 0x8D3DCB09), // 5.51724138e-1
        Sfp::newp(-1, 0x88888889), // 5.33333333e-1
        Sfp::newp(-1, 0x84210842), // 5.16129032e-1
    ];

    // x = 2^k * m
    let k = x.exponent();
    let m = x.mant();

    // ln(m) = ln(hi) + ln(lo)
    let hi = (m >> (32 - 5) & 0xF) as usize;
    let lo = x.set_exp(0) * LN_LO_SCALE_TBL[hi] - SfpM32E16::one();
    let ln_hi = LN_TBL[hi];

    (SfpM32E16::from_int(k), lo, ln_hi)
}
