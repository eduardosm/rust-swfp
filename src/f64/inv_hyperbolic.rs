use crate::F64;
use crate::math_aux::sfpm128e16::log_core;
use crate::simple_fp::{Sfp, SfpM128E16};
use crate::traits::FloatImpExt as _;

// GENERATE: consts SfpM128E16 LN_2
const LN_2: SfpM128E16 = Sfp::newp(-1, 0xB17217F7D1CF79ABC9E3B39803F2F6AF); // 6.9314718055994530941723212145817656808e-1

impl crate::generic::InvHyperbolic for F64 {
    fn asinh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: ln_1p_poly SfpM128E16 12 -0.00001 0.015625
            const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFFFFFF16F59479F); // -4.9999999999999999999999999990808164997e-1
            const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAA9B2804C4BE9A3); // 3.3333333333333333333333293240034046042e-1
            const K4: SfpM128E16 = Sfp::newn(-3, 0xFFFFFFFFFFFFFFFFF47A89CE0991E2F1); // -2.4999999999999999999939006561610015738e-1
            const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCCCAA7604D17E66919861); // 1.9999999999999999953461991936312423447e-1
            const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAA6EAF5B63D262709B326B); // -1.6666666666666645856341530968105352731e-1
            const K7: SfpM128E16 = Sfp::newp(-3, 0x9249249248E1DBEE53B529687D44C830); // 1.4285714285708360469702707930960114499e-1
            const K8: SfpM128E16 = Sfp::newn(-4, 0xFFFFFFFF9D06F3C160D128880BF129D1); // -1.2499999998874809044900537166204302648e-1
            const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E38B1722134EC8F475DBBADB02F1C); // 1.1111110965272583982904133178697643877e-1
            const K10: SfpM128E16 = Sfp::newn(-4, 0xCCCCBB68685ADA6FC8505986AC614DB3); // -9.9999870418327520335284407501121774889e-2
            const K11: SfpM128E16 = Sfp::newp(-4, 0xBA2A786AFBFA447E8610651E025C71F3); // 9.0901318325902321662289945050430200859e-2
            const K12: SfpM128E16 = Sfp::newn(-4, 0xAA0CA486768123C919139338D116F1E0); // -8.3031926492197323147364540646048877048e-2
            const K13: SfpM128E16 = Sfp::newp(-4, 0x8F7D6E780ADC16880CD29C46D668B3E0); // 7.0063460386661486018349089735095949981e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo
                + horner!(
                    lo2,
                    lo,
                    [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13]
                );

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        #[inline]
        fn asinh_small(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: asinh_poly SfpM128E16 8 -0.01563 0.01563
            const K3: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAAAA881BE); // -1.6666666666666666666666666666666656267e-1
            const K5: SfpM128E16 = Sfp::newp(-4, 0x999999999999999999999990DAD60F12); // 7.4999999999999999999999999986202552629e-2
            const K7: SfpM128E16 = Sfp::newn(-5, 0xB6DB6DB6DB6DB6DB6DAAC11AFD6D886E); // -4.4642857142857142857142231441481550623e-2
            const K9: SfpM128E16 = Sfp::newp(-6, 0xF8E38E38E38E38DB7493D71EB371FD4D); // 3.0381944444444444430722214760153823829e-2
            const K11: SfpM128E16 = Sfp::newn(-6, 0xB745D1745D15C5EDE7482C5D58BE67B9); // -2.2372159090908924423831525819679103588e-2
            const K13: SfpM128E16 = Sfp::newp(-6, 0x8E276275FDF8BC740FAC7892DBC707F4); // 1.7352764421899895113771545568753769099e-2
            const K15: SfpM128E16 = Sfp::newn(-7, 0xE4CCC79DF907E8A059A4B2ECBEB6BFEE); // -1.3964838923030681788570140385947361860e-2
            const K17: SfpM128E16 = Sfp::newp(-7, 0xBD171FA7929D9D3C83DC5B17B2B57B7F); // 1.1541157639195143936019330877262322385e-2

            let x2 = x.square();
            let x3 = x2 * x;
            x + horner!(x3, x2, [K3, K5, K7, K9, K11, K13, K15, K17])
        }

        if x.exponent() < -50 {
            // asinh(x) ~= x for tiny x
            return x;
        }

        let x = SfpM128E16::from_ieee_float(x.0);

        let r = if x.exponent() <= -7 {
            asinh_small(x)
        } else {
            // t1 = |x| + sqrt(x^2 + 1)
            let t1 = x.abs() + (x.square() + SfpM128E16::one()).sqrt();

            // t2 = |asinh(x)| = ln(|x| + sqrt(x^2 + 1)) = ln(t1)
            let t2 = ln(t1);

            // asinh(x) = |asinh(x)| * sgn(x)
            t2.set_sign(x.sign())
        };

        Self(r.to_ieee_float())
    }

    fn acosh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: ln_1p_poly SfpM128E16 12 -0.00001 0.015625
            const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFFFFFF16F59479F); // -4.9999999999999999999999999990808164997e-1
            const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAA9B2804C4BE9A3); // 3.3333333333333333333333293240034046042e-1
            const K4: SfpM128E16 = Sfp::newn(-3, 0xFFFFFFFFFFFFFFFFF47A89CE0991E2F1); // -2.4999999999999999999939006561610015738e-1
            const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCCCAA7604D17E66919861); // 1.9999999999999999953461991936312423447e-1
            const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAA6EAF5B63D262709B326B); // -1.6666666666666645856341530968105352731e-1
            const K7: SfpM128E16 = Sfp::newp(-3, 0x9249249248E1DBEE53B529687D44C830); // 1.4285714285708360469702707930960114499e-1
            const K8: SfpM128E16 = Sfp::newn(-4, 0xFFFFFFFF9D06F3C160D128880BF129D1); // -1.2499999998874809044900537166204302648e-1
            const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E38B1722134EC8F475DBBADB02F1C); // 1.1111110965272583982904133178697643877e-1
            const K10: SfpM128E16 = Sfp::newn(-4, 0xCCCCBB68685ADA6FC8505986AC614DB3); // -9.9999870418327520335284407501121774889e-2
            const K11: SfpM128E16 = Sfp::newp(-4, 0xBA2A786AFBFA447E8610651E025C71F3); // 9.0901318325902321662289945050430200859e-2
            const K12: SfpM128E16 = Sfp::newn(-4, 0xAA0CA486768123C919139338D116F1E0); // -8.3031926492197323147364540646048877048e-2
            const K13: SfpM128E16 = Sfp::newp(-4, 0x8F7D6E780ADC16880CD29C46D668B3E0); // 7.0063460386661486018349089735095949981e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo
                + horner!(
                    lo2,
                    lo,
                    [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13]
                );

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        let x = SfpM128E16::from_ieee_float(x.0);

        // t1 = x + sqrt(x^2 - 1)
        let t1 = x + (x.square() - SfpM128E16::one()).sqrt();

        // t2 = acosh(x) = ln(x + sqrt(x^2 - 1)) = ln(t1)
        let t2 = ln(t1);

        Self(t2.to_ieee_float())
    }

    fn atanh_finite(x: Self) -> Self {
        #[inline]
        fn ln(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: ln_1p_poly SfpM128E16 13 -0.00001 0.015625
            const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFFFFFFFF06850E3); // -4.9999999999999999999999999999961561481e-1
            const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAAAA9765082F602); // 3.3333333333333333333333333138735756805e-1
            const K4: SfpM128E16 = Sfp::newn(-3, 0xFFFFFFFFFFFFFFFFFFEF57DB16B27963); // -2.4999999999999999999999655545175889109e-1
            const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCCCCC92C88A39B9C8D095); // 1.9999999999999999999692862460713059255e-1
            const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAAAA3375B0A85A5E4EDF45); // -1.6666666666666666505111137735865396512e-1
            const K7: SfpM128E16 = Sfp::newp(-3, 0x924924924923F4E0383D7769F88D69EE); // 1.4285714285714231101771158572603198931e-1
            const K8: SfpM128E16 = Sfp::newn(-4, 0xFFFFFFFFFEE7265D874340062F68DD8C); // -1.2499999999987527748529710602373729243e-1
            const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E38E2E01ECAC9A0D67AF2662B9869); // 1.1111111109131801206255797711711992385e-1
            const K10: SfpM128E16 = Sfp::newn(-4, 0xCCCCCC80E57C247FA5E531A560157318); // -9.9999997790915237138144862381320516125e-2
            const K11: SfpM128E16 = Sfp::newp(-4, 0xBA2E746E1BBD95FCCC48ACD9286599EB); // 9.0908918009033706070738883784403831482e-2
            const K12: SfpM128E16 = Sfp::newn(-4, 0xAAA5CA90BB197DC41A10ED4BC2C4A59F); // -8.3324034253916985936395142308734402968e-2
            const K13: SfpM128E16 = Sfp::newp(-4, 0x9CDDE201C326520CDE3F88AD11746D61); // 7.6595082930356248822102435148385903403e-2
            const K14: SfpM128E16 = Sfp::newn(-4, 0x84362FE38C05DEDEF2932AAAC5936FD5); // -6.4556478625948208444911174206711719985e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo
                + horner!(
                    lo2,
                    lo,
                    [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14]
                );

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        #[inline]
        fn ln_1p_small(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: ln_1p_poly SfpM128E16 8 -0.0002442 0.0002442
            const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFFFFFFFFE3259AC); // -4.9999999999999999999999999999995554475e-1
            const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAAAAAAAA4AA1620); // 3.3333333333333333333333333333318536604e-1
            const K4: SfpM128E16 = Sfp::newn(-2, 0x8000000000000000001808B2C1426C2A); // -2.5000000000000000000000994022014985010e-1
            const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCCCCCCD2B24483756E188); // 2.0000000000000000000001950941186808508e-1
            const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAA9FDB3B9F44BFAF41D98FA); // -1.6666666666666606657512604283836634451e-1
            const K7: SfpM128E16 = Sfp::newp(-3, 0x9249249249239FE04E4EADEE9B190181); // 1.4285714285714201611588889952827865096e-1
            const K8: SfpM128E16 = Sfp::newn(-3, 0x800000E682A18582EF1C0E73828B5132); // -1.2500001341746404784162704671114616987e-1
            const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E3ADDE2A3A89D686069955BB4D98B); // 1.1111112584724643188665070320841992284e-1

            let x2 = x.square();
            x + horner!(x2, x, [K2, K3, K4, K5, K6, K7, K8, K9])
        }

        let x = SfpM128E16::from_ieee_float(x.0);
        let one = SfpM128E16::one();

        let r = if x.exponent() <= -14 {
            // t1 = (1 + x) / (1 - x) - 1 = 2 * x / (1 - x)
            let t1 = x.scalbn(1) / (one - x);

            // atanh(x) = 0.5 * ln((1 + x) / (1 - x)) = 0.5 * ln(t1 + 1)
            ln_1p_small(t1).scalbn(-1)
        } else {
            // t1 = (1 + x) / (1 - x)
            let t1 = (one + x) / (one - x);

            // atanh(x) = 0.5 * ln((1 + x) / (1 - x)) = 0.5 * ln(t1)
            ln(t1).scalbn(-1)
        };

        Self(r.to_ieee_float())
    }
}

impl crate::math::InvHyperbolic for F64 {
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
