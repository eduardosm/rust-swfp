use crate::F16;
use crate::simple_fp::{Sfp, SfpM32E16};
use crate::traits::FloatImpExt as _;

// GENERATE: consts SfpM32E16 LOG2_E LN_2
const LOG2_E: SfpM32E16 = Sfp::newp(0, 0xB8AA3B29); // 1.44269504e0
const LN_2: SfpM32E16 = Sfp::newp(-1, 0xB17217F8); // 6.93147181e-1

impl crate::generic::Hyperbolic for F16 {
    fn sinh_finite(x: Self) -> Self {
        let (sinh, _) = sinh_cosh_inner(x);
        Self(sinh.to_ieee_float())
    }

    fn cosh_finite(x: Self) -> Self {
        let (_, cosh) = sinh_cosh_inner(x);
        Self(cosh.to_ieee_float())
    }

    fn sinh_cosh_finite(x: Self) -> (Self, Self) {
        let (sinh, cosh) = sinh_cosh_inner(x);
        (Self(sinh.to_ieee_float()), Self(cosh.to_ieee_float()))
    }

    fn tanh_finite(x: Self) -> Self {
        let (sinh, cosh) = sinh_cosh_inner(x);
        if sinh.is_infinite() && cosh.is_infinite() {
            Self::ONE.set_sign(sinh.sign())
        } else {
            Self((sinh / cosh).to_ieee_float())
        }
    }
}

fn sinh_cosh_inner(x: F16) -> (SfpM32E16, SfpM32E16) {
    #[inline]
    fn exp_m1_core(r: SfpM32E16) -> (SfpM32E16, SfpM32E16) {
        // exp(r) = exp(r / s)^s
        // exp(-r) = exp(-r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM32E16 3 -0.0216609 0.0216609
        const K2: SfpM32E16 = Sfp::newp(-2, 0xFFFFFFFF); // 5.00000000e-1
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAAB840B); // 1.66669906e-1
        const K4: SfpM32E16 = Sfp::newp(-5, 0xAAABA208); // 4.16675882e-2

        let rs2 = rs.square();
        let mrs = -rs;
        let mut t1 = rs + horner!(rs2, rs, [K2, K3, K4]);
        let mut t2 = mrs + horner!(rs2, mrs, [K2, K3, K4]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t1 = t1.square() + t1.scalbn(1);
            t2 = t2.square() + t2.scalbn(1);
        }

        (t1, t2)
    }

    let x = SfpM32E16::from_ieee_float(x.0);

    // x = k*ln(2) + r
    // k is an integer
    // |r| <= 0.5*ln(2)
    let Some(k) = (x * LOG2_E).to_int_round::<i16>() else {
        return (SfpM32E16::INFINITY.set_sign(x.sign()), SfpM32E16::INFINITY);
    };
    let r = x - SfpM32E16::from_int(k) * LN_2;

    // t1 = exp(r) - 1
    // t2 = exp(-r) - 1
    let (t1a, t1b) = exp_m1_core(r);

    // sinh(x) = (exp(x) - exp(-x)) / 2
    // cosh(x) = (exp(x) + exp(-x)) / 2

    if k == 0 {
        let s = (t1a - t1b).scalbn(-1);
        let c = (t1a + t1b).scalbn(-1) + SfpM32E16::one();
        (s, c)
    } else {
        let t2a = (t1a + SfpM32E16::one()).scalbn(k - 1);
        let t2b = (t1b + SfpM32E16::one()).scalbn(-k - 1);

        let s = t2a - t2b;
        let c = t2a + t2b;
        (s, c)
    }
}

impl crate::math::Hyperbolic for F16 {
    fn sinh(self) -> Self {
        crate::generic::sinh(self)
    }

    fn cosh(self) -> Self {
        crate::generic::cosh(self)
    }

    fn sinh_cosh(self) -> (Self, Self) {
        crate::generic::sinh_cosh(self)
    }

    fn tanh(self) -> Self {
        crate::generic::tanh(self)
    }
}
