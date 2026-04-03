use crate::F32;
use crate::math_aux::sfpm64e16::log_core;
use crate::simple_fp::{Sfp, SfpM64E16};

// GENERATE: consts SfpM64E16 LN_2 LOG2_E LOG10_E LOG10_2
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1
const LOG2_E: SfpM64E16 = Sfp::newp(0, 0xB8AA3B295C17F0BC); // 1.442695040888963407e0
const LOG10_E: SfpM64E16 = Sfp::newp(-2, 0xDE5BD8A937287195); // 4.342944819032518277e-1
const LOG10_2: SfpM64E16 = Sfp::newp(-2, 0x9A209A84FBCFF799); // 3.010299956639811952e-1

impl crate::generic::Log for F32 {
    fn ln_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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

        // y = ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
        let y = k * LN_2 + (ln_hi + ln_lo);
        Self(y.to_ieee_float())
    }

    fn ln_1p_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -8 {
            // GENERATE: ln_1p_poly SfpM64E16 6 -0.0078125 0.0078125
            const K2: SfpM64E16 = Sfp::newn(-1, 0x800000000000DF5F); // -5.000000000000030999e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAAAAAF26AD); // 3.333333333333412992e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFF901DEC800); // -2.499999995929716168e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCCCC205E29954); // 1.999999993726953908e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAAE2A5E2EEB906C); // -1.666800136551774295e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x924D0DEED8E750A0); // 1.428720643072901053e-1

            let x2 = x.square();
            x + horner!(x2, x, [K2, K3, K4, K5, K6, K7])
        } else {
            // GENERATE: ln_1p_poly SfpM64E16 8 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFFF795EC); // -4.999999999999850531e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAA99831AC4); // 3.333333333255323519e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFE812F54B44); // -2.499999986073267291e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCC4B823C00BC5); // 1.999998795881197519e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAA927263CCBE044); // -1.666608922088917986e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x921EA36C699C1580); // 1.426950011176835922e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xFA89BE59EBF36590); // -1.223330374900741354e-1
            const K9: SfpM64E16 = Sfp::newp(-4, 0xB20D62315D3B945E); // 8.693958962581208796e-2

            let (k, lo, ln_hi) = log_core(x + SfpM64E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8, K9]);

            k * LN_2 + (ln_hi + ln_lo)
        };
        Self(y.to_ieee_float())
    }

    fn log2_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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

        let y = k + (ln_hi + ln_lo) * LOG2_E;
        Self(y.to_ieee_float())
    }

    fn log2_1p_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -8 {
            // GENERATE: ln_1p_poly SfpM64E16 6 -0.0078125 0.0078125
            const K2: SfpM64E16 = Sfp::newn(-1, 0x800000000000DF5F); // -5.000000000000030999e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAAAAAF26AD); // 3.333333333333412992e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFF901DEC800); // -2.499999995929716168e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCCCC205E29954); // 1.999999993726953908e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAAE2A5E2EEB906C); // -1.666800136551774295e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x924D0DEED8E750A0); // 1.428720643072901053e-1

            let x2 = x.square();
            let t = x + horner!(x2, x, [K2, K3, K4, K5, K6, K7]);
            t * LOG2_E
        } else {
            // GENERATE: ln_1p_poly SfpM64E16 8 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFFF795EC); // -4.999999999999850531e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAA99831AC4); // 3.333333333255323519e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFE812F54B44); // -2.499999986073267291e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCC4B823C00BC5); // 1.999998795881197519e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAA927263CCBE044); // -1.666608922088917986e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x921EA36C699C1580); // 1.426950011176835922e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xFA89BE59EBF36590); // -1.223330374900741354e-1
            const K9: SfpM64E16 = Sfp::newp(-4, 0xB20D62315D3B945E); // 8.693958962581208796e-2

            let (k, lo, ln_hi) = log_core(x + SfpM64E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8, K9]);

            k + (ln_hi + ln_lo) * LOG2_E
        };
        Self(y.to_ieee_float())
    }

    fn log10_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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

        let y = k * LOG10_2 + (ln_hi + ln_lo) * LOG10_E;
        Self(y.to_ieee_float())
    }

    fn log10_1p_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let y = if x.exponent() <= -8 {
            // GENERATE: ln_1p_poly SfpM64E16 6 -0.0078125 0.0078125
            const K2: SfpM64E16 = Sfp::newn(-1, 0x800000000000DF5F); // -5.000000000000030999e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAAAAAF26AD); // 3.333333333333412992e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFF901DEC800); // -2.499999995929716168e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCCCC205E29954); // 1.999999993726953908e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAAE2A5E2EEB906C); // -1.666800136551774295e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x924D0DEED8E750A0); // 1.428720643072901053e-1

            let x2 = x.square();
            let t = x + horner!(x2, x, [K2, K3, K4, K5, K6, K7]);
            t * LOG10_E
        } else {
            // GENERATE: ln_1p_poly SfpM64E16 8 -0.0001 0.0625
            const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFFFFF795EC); // -4.999999999999850531e-1
            const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAA99831AC4); // 3.333333333255323519e-1
            const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFFFFE812F54B44); // -2.499999986073267291e-1
            const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCC4B823C00BC5); // 1.999998795881197519e-1
            const K6: SfpM64E16 = Sfp::newn(-3, 0xAAA927263CCBE044); // -1.666608922088917986e-1
            const K7: SfpM64E16 = Sfp::newp(-3, 0x921EA36C699C1580); // 1.426950011176835922e-1
            const K8: SfpM64E16 = Sfp::newn(-4, 0xFA89BE59EBF36590); // -1.223330374900741354e-1
            const K9: SfpM64E16 = Sfp::newp(-4, 0xB20D62315D3B945E); // 8.693958962581208796e-2

            let (k, lo, ln_hi) = log_core(x + SfpM64E16::one());
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8, K9]);

            k * LOG10_2 + (ln_hi + ln_lo) * LOG10_E
        };
        Self(y.to_ieee_float())
    }
}

impl crate::math::Log for F32 {
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
