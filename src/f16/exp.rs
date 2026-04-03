use crate::F16;
use crate::Float as _;
use crate::simple_fp::{Sfp, SfpM32E16};

// GENERATE: consts SfpM32E16 LOG2_E LN_2 LOG2_10 LOG10_2 LN_10
const LOG2_E: SfpM32E16 = Sfp::newp(0, 0xB8AA3B29); // 1.44269504e0
const LN_2: SfpM32E16 = Sfp::newp(-1, 0xB17217F8); // 6.93147181e-1
const LOG2_10: SfpM32E16 = Sfp::newp(1, 0xD49A784C); // 3.32192809e0
const LOG10_2: SfpM32E16 = Sfp::newp(-2, 0x9A209A85); // 3.01029996e-1
const LN_10: SfpM32E16 = Sfp::newp(1, 0x935D8DDE); // 2.30258509e0

impl crate::generic::Exp for F16 {
    fn exp_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = x - SfpM32E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-1, 0x8000F5D2); // 5.00014652e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAABB110); // 1.66670577e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM32E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp_m1_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = x - SfpM32E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-1, 0x8000F5D2); // 5.00014652e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAABB110); // 1.66670577e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM32E16::one().scalbn(k) - SfpM32E16::one());

        Self(y.to_ieee_float())
    }

    fn exp2_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = (x - SfpM32E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-1, 0x8000F5D2); // 5.00014652e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAABB110); // 1.66670577e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM32E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp2_m1_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = (x - SfpM32E16::from_int(k)) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0108305 0.0108305
        const K2: SfpM32E16 = Sfp::newp(-1, 0x80003D7A); // 5.00003664e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAAAEC44); // 1.66667644e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp2(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM32E16::one().scalbn(k) - SfpM32E16::one());

        Self(y.to_ieee_float())
    }

    fn exp10_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = (x - SfpM32E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-1, 0x8000F5D2); // 5.00014652e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAABB110); // 1.66670577e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // exp10(x) = exp(r) * 2^k = (t + 1) * 2^k
        let y = (t + SfpM32E16::one()).scalbn(k);

        Self(y.to_ieee_float())
    }

    fn exp10_m1_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

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
        let r = (x - SfpM32E16::from_int(k) * LOG10_2) * LN_10;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 2 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-1, 0x8000F5D2); // 5.00014652e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAABB110); // 1.66670577e-1

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // y = exp10(x) - 1 = exp(r) * 2^k - 1 = t * 2^k + (2^k - 1)
        let y = t.scalbn(k) + (SfpM32E16::one().scalbn(k) - SfpM32E16::one());

        Self(y.to_ieee_float())
    }
}

impl crate::math::Exp for F16 {
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
