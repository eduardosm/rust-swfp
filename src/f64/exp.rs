use crate::F64;
use crate::Float as _;
use crate::simple_fp::{Sfp, SfpM128E16};

// GENERATE: consts SfpM128E16 LOG2_E LN_2 LOG2_10 LOG10_2 LN_10
const LOG2_E: SfpM128E16 = Sfp::newp(0, 0xB8AA3B295C17F0BBBE87FED0691D3E89); // 1.4426950408889634073599246810018921374e0
const LN_2: SfpM128E16 = Sfp::newp(-1, 0xB17217F7D1CF79ABC9E3B39803F2F6AF); // 6.9314718055994530941723212145817656808e-1
const LOG2_10: SfpM128E16 = Sfp::newp(1, 0xD49A784BCD1B8AFE492BF6FF4DAFDB4D); // 3.3219280948873623478703194294893901759e0
const LOG10_2: SfpM128E16 = Sfp::newp(-2, 0x9A209A84FBCFF7988F8959AC0B7C9178); // 3.0102999566398119521373889472449302677e-1
const LN_10: SfpM128E16 = Sfp::newp(1, 0x935D8DDDAAA8AC16EA56D62B82D30A29); // 2.3025850929940456840179914546843642076e0

impl crate::generic::Exp for F64 {
    #[inline]
    fn exp_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k*ln(2) + r
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_E).to_int_round() else {
            if x.sign() {
                return Self::ZERO;
            } else {
                return Self::INFINITY;
            }
        };
        let r = x - SfpM128E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0108305 0.0108305
        const K2: SfpM128E16 = Sfp::newp(-1, 0x8000000000000000000000000A18C4AB); // 5.0000000000000000000000000000049780825e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAB89C9072); // 1.6666666666666666666666666666683855098e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAA906471425FAC6); // 4.1666666666666666666666581769781087337e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888737131C645523); // 8.3333333333333333333333162960238389375e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B631A94ABA6245389070); // 1.3888888888888888929423141017382472144e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D03869DC24B3F3CE60A0); // 1.9841269841269841327294377469557908182e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00A2843DAE88AAA271BE8B259); // 2.4801587301508312293963425605330140238e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB3B8378C5AE72809FFA254E4); // 2.7557319223898965515590551277993695419e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2956D5F7CFB94E966758D60E38A0E); // 2.7557386565713058282698357331832748571e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD7324D7EDC9508860022CAAAC56AC467); // 2.5052169220091465487491176317014703209e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM128E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    #[inline]
    fn exp_m1_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k*ln(2) + r
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_E).to_int_round() else {
            if x.sign() {
                return Self::from_int(-1);
            } else {
                return Self::INFINITY;
            }
        };
        let r = x - SfpM128E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0108305 0.0108305
        const K2: SfpM128E16 = Sfp::newp(-1, 0x8000000000000000000000000A18C4AB); // 5.0000000000000000000000000000049780825e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAB89C9072); // 1.6666666666666666666666666666683855098e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAA906471425FAC6); // 4.1666666666666666666666581769781087337e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888737131C645523); // 8.3333333333333333333333162960238389375e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B631A94ABA6245389070); // 1.3888888888888888929423141017382472144e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D03869DC24B3F3CE60A0); // 1.9841269841269841327294377469557908182e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00A2843DAE88AAA271BE8B259); // 2.4801587301508312293963425605330140238e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB3B8378C5AE72809FFA254E4); // 2.7557319223898965515590551277993695419e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2956D5F7CFB94E966758D60E38A0E); // 2.7557386565713058282698357331832748571e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD7324D7EDC9508860022CAAAC56AC467); // 2.5052169220091465487491176317014703209e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM128E16::one().scalbn(k) - SfpM128E16::one());

        Self(y.to_ieee_float())
    }

    #[inline]
    fn exp2_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k + r*log2(e)
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = x.to_int_round() else {
            if x.sign() {
                return Self::ZERO;
            } else {
                return Self::INFINITY;
            }
        };
        let r = (x - SfpM128E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0108305 0.0108305
        const K2: SfpM128E16 = Sfp::newp(-1, 0x8000000000000000000000000A18C4AB); // 5.0000000000000000000000000000049780825e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAB89C9072); // 1.6666666666666666666666666666683855098e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAA906471425FAC6); // 4.1666666666666666666666581769781087337e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888737131C645523); // 8.3333333333333333333333162960238389375e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B631A94ABA6245389070); // 1.3888888888888888929423141017382472144e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D03869DC24B3F3CE60A0); // 1.9841269841269841327294377469557908182e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00A2843DAE88AAA271BE8B259); // 2.4801587301508312293963425605330140238e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB3B8378C5AE72809FFA254E4); // 2.7557319223898965515590551277993695419e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2956D5F7CFB94E966758D60E38A0E); // 2.7557386565713058282698357331832748571e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD7324D7EDC9508860022CAAAC56AC467); // 2.5052169220091465487491176317014703209e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM128E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    #[inline]
    fn exp2_m1_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k + r*log2(e)
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = x.to_int_round() else {
            if x.sign() {
                return Self::from_int(-1);
            } else {
                return Self::INFINITY;
            }
        };
        let r = (x - SfpM128E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0108305 0.0108305
        const K2: SfpM128E16 = Sfp::newp(-1, 0x8000000000000000000000000A18C4AB); // 5.0000000000000000000000000000049780825e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAB89C9072); // 1.6666666666666666666666666666683855098e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAA906471425FAC6); // 4.1666666666666666666666581769781087337e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888737131C645523); // 8.3333333333333333333333162960238389375e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B631A94ABA6245389070); // 1.3888888888888888929423141017382472144e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D03869DC24B3F3CE60A0); // 1.9841269841269841327294377469557908182e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00A2843DAE88AAA271BE8B259); // 2.4801587301508312293963425605330140238e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB3B8378C5AE72809FFA254E4); // 2.7557319223898965515590551277993695419e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2956D5F7CFB94E966758D60E38A0E); // 2.7557386565713058282698357331832748571e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD7324D7EDC9508860022CAAAC56AC467); // 2.5052169220091465487491176317014703209e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM128E16::one().scalbn(k) - SfpM128E16::one());

        Self(y.to_ieee_float())
    }

    #[inline]
    fn exp10_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k*log10(2) + r*log10(e)
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_10).to_int_round() else {
            if x.sign() {
                return Self::ZERO;
            } else {
                return Self::INFINITY;
            }
        };
        let r = (x - SfpM128E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0108305 0.0108305
        const K2: SfpM128E16 = Sfp::newp(-1, 0x8000000000000000000000000A18C4AB); // 5.0000000000000000000000000000049780825e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAB89C9072); // 1.6666666666666666666666666666683855098e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAA906471425FAC6); // 4.1666666666666666666666581769781087337e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888737131C645523); // 8.3333333333333333333333162960238389375e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B631A94ABA6245389070); // 1.3888888888888888929423141017382472144e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D03869DC24B3F3CE60A0); // 1.9841269841269841327294377469557908182e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00A2843DAE88AAA271BE8B259); // 2.4801587301508312293963425605330140238e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB3B8378C5AE72809FFA254E4); // 2.7557319223898965515590551277993695419e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2956D5F7CFB94E966758D60E38A0E); // 2.7557386565713058282698357331832748571e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD7324D7EDC9508860022CAAAC56AC467); // 2.5052169220091465487491176317014703209e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp10(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM128E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    #[inline]
    fn exp10_m1_finite(x: Self) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);

        // x = k*log10(2) + r*log10(e)
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_10).to_int_round() else {
            if x.sign() {
                return Self::from_int(-1);
            } else {
                return Self::INFINITY;
            }
        };
        let r = (x - SfpM128E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^6

        let rs = r.scalbn(-6);

        // GENERATE: exp_m1_poly SfpM128E16 10 -0.0054153 0.0054153
        const K2: SfpM128E16 = Sfp::newp(-1, 0x80000000000000000000000000028662); // 5.0000000000000000000000000000000048628e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAE2714); // 1.6666666666666666666666666666666683451e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAAAA9063504E7FE); // 4.1666666666666666666666666334982557119e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888887371BF7CC20); // 8.3333333333333333333333332667881663459e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60B60BF9DC72320C5533EB); // 1.3888888888888888889522302469313391423e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00D00D00D00DAE6FA172D6A361C8); // 1.9841269841269841270738881568299702821e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00CD34692EF88B07A57D79EA3); // 2.4801587301582364453824784808344725831e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB61187086145EDD9AC001B25); // 2.7557319223980458282527743484252534437e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F283A83674384D1BD615B789557197); // 2.7557336059888092042953577046406556867e-7
        const K11: SfpM128E16 = Sfp::newp(-26, 0xD73233CF5C813EF152D91100C4771E0B); // 2.5052123593392505270409191775314627009e-8

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..6 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp10(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM128E16::one().scalbn(k) - SfpM128E16::one());

        Self(y.to_ieee_float())
    }
}

impl crate::math::Exp for F64 {
    fn exp(self) -> Self {
        crate::generic::exp(self)
    }

    fn exp_m1(self) -> Self {
        crate::generic::exp_m1(self)
    }

    fn exp2(self) -> Self {
        crate::generic::exp2(self)
    }

    fn exp2_m1(self) -> Self {
        crate::generic::exp2_m1(self)
    }

    fn exp10(self) -> Self {
        crate::generic::exp10(self)
    }

    fn exp10_m1(self) -> Self {
        crate::generic::exp10_m1(self)
    }
}
