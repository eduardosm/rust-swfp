use crate::F32;
use crate::Float as _;
use crate::simple_fp::{Sfp, SfpM64E16};

// GENERATE: consts SfpM64E16 LOG2_E LN_2 LOG2_10 LOG10_2 LN_10
const LOG2_E: SfpM64E16 = Sfp::newp(0, 0xB8AA3B295C17F0BC); // 1.442695040888963407e0
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1
const LOG2_10: SfpM64E16 = Sfp::newp(1, 0xD49A784BCD1B8AFE); // 3.321928094887362348e0
const LOG10_2: SfpM64E16 = Sfp::newp(-2, 0x9A209A84FBCFF799); // 3.010299956639811952e-1
const LN_10: SfpM64E16 = Sfp::newp(1, 0x935D8DDDAAA8AC17); // 2.302585092994045684e0

impl crate::generic::Exp for F32 {
    fn exp_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = x - SfpM64E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM64E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp_m1_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = x - SfpM64E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM64E16::one().scalbn(k) - SfpM64E16::one());

        Self(y.to_ieee_float())
    }

    fn exp2_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = (x - SfpM64E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM64E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp2_m1_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = (x - SfpM64E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM64E16::one().scalbn(k) - SfpM64E16::one());

        Self(y.to_ieee_float())
    }

    fn exp10_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = (x - SfpM64E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp10(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM64E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp10_m1_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

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
        let r = (x - SfpM64E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0108305 0.0108305
        const K2: SfpM64E16 = Sfp::newp(-1, 0x800000000000006A); // 5.000000000000000057e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAAA57E9144); // 1.666666666654906303e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAAA43B8C82); // 4.166666666630093304e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x8888AB53BD0DF725); // 8.333365737556939513e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60B90E56EC24405); // 1.388894498373998321e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp10(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM64E16::one().scalbn(k) - SfpM64E16::one());

        Self(y.to_ieee_float())
    }
}

impl crate::math::Exp for F32 {
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
