use crate::F32;
use crate::math_aux::sfpm64e16::log_core;
use crate::simple_fp::{Sfp, SfpM64E16};
use crate::traits::FloatImpExt as _;

// GENERATE: consts SfpM64E16 LN_2
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1

impl crate::generic::InvHyperbolic for F32 {
    fn asinh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM64E16) -> SfpM64E16 {
            // GENERATE: ln_1p_poly SfpM64E16 7 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFE146536); // -4.999999999991267347e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAA794DA42FD); // 3.333333329741674718e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFCA4B7BA5D2A); // -2.499999499831865214e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCBEEB5DE1FA9FA); // 1.999966906073777153e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAA8BD6F072AEC30F); // -1.665490707643317964e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x8FE9E51EB9520FD5); // 1.405406761171708590e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xCE7E72C1F2C11001); // -1.008271184385596542e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        if x.exponent() < -20 {
            // asinh(x) ~= x for small x
            return x;
        }

        let x = SfpM64E16::from_ieee_float(x.0);

        // t1 = |x| + sqrt(x^2 + 1)
        let t1 = x.abs() + (x.square() + SfpM64E16::one()).sqrt();

        // t2 = ln(t1) = |asinh(x)|

        // t2 = |asinh(x)| = ln(|x| + sqrt(x^2 + 1)) = ln(t1)
        let t2 = ln(t1);

        // asinh(x) = |asinh(x)| * sgn(x)
        Self(t2.set_sign(x.sign()).to_ieee_float())
    }

    fn acosh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM64E16) -> SfpM64E16 {
            // GENERATE: ln_1p_poly SfpM64E16 7 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFE146536); // -4.999999999991267347e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAA794DA42FD); // 3.333333329741674718e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFCA4B7BA5D2A); // -2.499999499831865214e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCBEEB5DE1FA9FA); // 1.999966906073777153e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAA8BD6F072AEC30F); // -1.665490707643317964e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x8FE9E51EB9520FD5); // 1.405406761171708590e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xCE7E72C1F2C11001); // -1.008271184385596542e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        let x = SfpM64E16::from_ieee_float(x.0);

        // t1 = x + sqrt(x^2 - 1)
        let t1 = x + (x.square() - SfpM64E16::one()).sqrt();

        // t2 = acosh(x) = ln(x + sqrt(x^2 - 1)) = ln(t1)
        let t2 = ln(t1);

        Self(t2.to_ieee_float())
    }

    fn atanh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM64E16) -> SfpM64E16 {
            // GENERATE: ln_1p_poly SfpM64E16 8 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFFF795EC); // -4.999999999999850531e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAA99831AC4); // 3.333333333255323519e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFE812F54B44); // -2.499999986073267291e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCC4B823C00BC5); // 1.999998795881197519e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAA927263CCBE044); // -1.666608922088917986e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x921EA36C699C1580); // 1.426950011176835922e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xFA89BE59EBF36590); // -1.223330374900741354e-1
            const K9: SfpM64E16 = Sfp::newp(-4, 0xB20D62315D3B945E); // 8.693958962581208796e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8, K9]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        if x.exponent() < -20 {
            // asinh(x) ~= x for small x
            return x;
        }

        let x = SfpM64E16::from_ieee_float(x.0);
        let one = SfpM64E16::one();

        // t1 = (1 + x) / (1 - x)
        let t1 = (one + x) / (one - x);

        // t2 = atanh(x) = 0.5 * ln((1 + x) / (1 - x)) = 0.5 * ln(t1)
        let t2 = ln(t1).scalbn(-1);

        Self(t2.to_ieee_float())
    }
}

impl crate::math::InvHyperbolic for F32 {
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
