use crate::F16;
use crate::math_aux::sfpm32e16::log_core;
use crate::simple_fp::{Sfp, SfpM32E16};

// GENERATE: consts SfpM32E16 LN_2 LOG2_E LOG10_E LOG10_2
const LN_2: SfpM32E16 = Sfp::newp(-1, 0xB17217F8); // 6.93147181e-1
const LOG2_E: SfpM32E16 = Sfp::newp(0, 0xB8AA3B29); // 1.44269504e0
const LOG10_E: SfpM32E16 = Sfp::newp(-2, 0xDE5BD8A9); // 4.34294482e-1
const LOG10_2: SfpM32E16 = Sfp::newp(-2, 0x9A209A85); // 3.01029996e-1

impl crate::generic::Log for F16 {
    #[inline]
    fn ln_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

        // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
        const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
        const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
        const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

        let (k, lo, ln_hi) = log_core(x);
        let lo2 = lo.square();
        let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

        // y = ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
        let y = k * LN_2 + (ln_hi + ln_lo);
        Self(y.to_ieee_float())
    }

    #[inline]
    fn ln_1p_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -5 {
            // GENERATE: ln_1p_poly SfpM32E16 4 -0.0625 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFFE5B2); // -4.99999216e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAAAA7ABC); // 3.33331905e-1
            const K4: SfpM32E16 = Sfp::newn(-2, 0x806A705E); // -2.50812065e-1
            const K5: SfpM32E16 = Sfp::newp(-3, 0xCDBF646D); // 2.00925416e-1

            let x2 = x.square();
            x + horner!(x2, x, [K2, K3, K4, K5])
        } else {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x + SfpM32E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            k * LN_2 + (ln_hi + ln_lo)
        };
        Self(y.to_ieee_float())
    }

    #[inline]
    fn log2_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

        // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
        const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
        const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
        const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

        let (k, lo, ln_hi) = log_core(x);
        let lo2 = lo.square();
        let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

        let y = k + (ln_hi + ln_lo) * LOG2_E;
        Self(y.to_ieee_float())
    }

    #[inline]
    fn log2_1p_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -5 {
            // GENERATE: ln_1p_poly SfpM32E16 4 -0.0625 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFFE5B2); // -4.99999216e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAAAA7ABC); // 3.33331905e-1
            const K4: SfpM32E16 = Sfp::newn(-2, 0x806A705E); // -2.50812065e-1
            const K5: SfpM32E16 = Sfp::newp(-3, 0xCDBF646D); // 2.00925416e-1

            let x2 = x.square();
            let t = x + horner!(x2, x, [K2, K3, K4, K5]);
            t * LOG2_E
        } else {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x + SfpM32E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            k + (ln_hi + ln_lo) * LOG2_E
        };
        Self(y.to_ieee_float())
    }

    #[inline]
    fn log10_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

        // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
        const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
        const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
        const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

        let (k, lo, ln_hi) = log_core(x);
        let lo2 = lo.square();
        let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

        let y = k * LOG10_2 + (ln_hi + ln_lo) * LOG10_E;
        Self(y.to_ieee_float())
    }

    #[inline]
    fn log10_1p_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -5 {
            // GENERATE: ln_1p_poly SfpM32E16 4 -0.0625 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFFE5B2); // -4.99999216e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAAAA7ABC); // 3.33331905e-1
            const K4: SfpM32E16 = Sfp::newn(-2, 0x806A705E); // -2.50812065e-1
            const K5: SfpM32E16 = Sfp::newp(-3, 0xCDBF646D); // 2.00925416e-1

            let x2 = x.square();
            let t = x + horner!(x2, x, [K2, K3, K4, K5]);
            t * LOG10_E
        } else {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x + SfpM32E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            k * LOG10_2 + (ln_hi + ln_lo) * LOG10_E
        };
        Self(y.to_ieee_float())
    }
}

impl crate::math::Log for F16 {
    fn ln(self) -> Self {
        crate::generic::ln(self)
    }

    fn ln_1p(self) -> Self {
        crate::generic::ln_1p(self)
    }

    fn log2(self) -> Self {
        crate::generic::log2(self)
    }

    fn log2_1p(self) -> Self {
        crate::generic::log2_1p(self)
    }

    fn log10(self) -> Self {
        crate::generic::log10(self)
    }

    fn log10_1p(self) -> Self {
        crate::generic::log10_1p(self)
    }
}
