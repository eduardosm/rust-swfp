use crate::F16;
use crate::math_aux::sfpm32e16::log_core;
use crate::simple_fp::{Sfp, SfpM32E16};

// GENERATE: consts SfpM32E16 LN_2
const LN_2: SfpM32E16 = Sfp::newp(-1, 0xB17217F8); // 6.93147181e-1

impl crate::generic::InvHyperbolic for F16 {
    fn asinh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        let x = SfpM32E16::from_ieee_float(x.0);

        // t1 = |x| + sqrt(x^2 + 1)
        let t1 = x.abs() + (x.square() + SfpM32E16::one()).sqrt();

        // t2 = ln(t1) = |asinh(x)|

        // t2 = |asinh(x)| = ln(|x| + sqrt(x^2 + 1)) = ln(t1)
        let t2 = ln(t1);

        // asinh(x) = |asinh(x)| * sgn(x)
        Self(t2.set_sign(x.sign()).to_ieee_float())
    }

    fn acosh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: ln_1p_poly SfpM32E16 2 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFCCFC7B); // -4.99610796e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xA023817D); // 3.12770888e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        let x = SfpM32E16::from_ieee_float(x.0);

        // t1 = x + sqrt(x^2 - 1)
        let t1 = x + (x.square() - SfpM32E16::one()).sqrt();

        // t2 = acosh(x) = ln(x + sqrt(x^2 - 1)) = ln(t1)
        let t2 = ln(t1);

        Self(t2.to_ieee_float())
    }

    fn atanh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        let x = SfpM32E16::from_ieee_float(x.0);
        let one = SfpM32E16::one();

        // t1 = (1 + x) / (1 - x)
        let t1 = (one + x) / (one - x);

        // t2 = atanh(x) = 0.5 * ln((1 + x) / (1 - x)) = 0.5 * ln(t1)
        let t2 = ln(t1).scalbn(-1);

        Self(t2.to_ieee_float())
    }
}

impl crate::math::InvHyperbolic for F16 {
    fn asinh(self) -> Self {
        crate::generic::asinh(self)
    }

    fn acosh(self) -> Self {
        crate::generic::acosh(self)
    }

    fn atanh(self) -> Self {
        crate::generic::atanh(self)
    }
}
