use crate::F32;
use crate::simple_fp::{Sfp, SfpM64E16};
use crate::traits::FloatImpExt as _;

// GENERATE: consts SfpM64E16 LOG2_E LN_2
const LOG2_E: SfpM64E16 = Sfp::newp(0, 0xB8AA3B295C17F0BC); // 1.442695040888963407e0
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1

impl crate::generic::Hyperbolic for F32 {
    #[inline]
    fn sinh_finite(x: Self) -> Self {
        let (sinh, _) = sinh_cosh_inner(x);
        Self(sinh.to_ieee_float())
    }

    #[inline]
    fn cosh_finite(x: Self) -> Self {
        let (_, cosh) = sinh_cosh_inner(x);
        Self(cosh.to_ieee_float())
    }

    #[inline]
    fn sinh_cosh_finite(x: Self) -> (Self, Self) {
        let (sinh, cosh) = sinh_cosh_inner(x);
        (Self(sinh.to_ieee_float()), Self(cosh.to_ieee_float()))
    }

    #[inline]
    fn tanh_finite(x: Self) -> Self {
        let (sinh, cosh) = sinh_cosh_inner(x);
        if sinh.is_infinite() && cosh.is_infinite() {
            Self::ONE.set_sign(sinh.sign())
        } else {
            Self((sinh / cosh).to_ieee_float())
        }
    }
}

fn sinh_cosh_inner(x: F32) -> (SfpM64E16, SfpM64E16) {
    #[inline]
    fn exp_m1_core(r: SfpM64E16) -> (SfpM64E16, SfpM64E16) {
        // exp(r) = exp(r / s)^s
        // exp(-r) = exp(-r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216608 0.0216608
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E9B458); // 1.666666666478506404e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B8DEF9); // 4.166666666081494852e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B4E47F6ABB); // 8.333462948549164968e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C21735F40B59D); // 1.388911326726920896e-3

        let rs2 = rs.square();
        let mrs = -rs;
        let mut t1 = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);
        let mut t2 = mrs + horner!(rs2, mrs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t1 = t1.square() + t1.scalbn(1);
            t2 = t2.square() + t2.scalbn(1);
        }

        (t1, t2)
    }

    let x = SfpM64E16::from_ieee_float(x.0);

    // x = k*ln(2) + r
    // k is an integer
    // |r| <= 0.5*ln(2)
    let Some(k) = (x * LOG2_E).to_int_round::<i16>() else {
        return (SfpM64E16::INFINITY.set_sign(x.sign()), SfpM64E16::INFINITY);
    };
    let r = x - SfpM64E16::from_int(k) * LN_2;

    // t1 = exp(r) - 1
    // t2 = exp(-r) - 1
    let (t1a, t1b) = exp_m1_core(r);

    // sinh(x) = (exp(x) - exp(-x)) / 2
    // cosh(x) = (exp(x) + exp(-x)) / 2

    if k == 0 {
        let s = (t1a - t1b).scalbn(-1);
        let c = (t1a + t1b).scalbn(-1) + SfpM64E16::one();
        (s, c)
    } else {
        let t2a = (t1a + SfpM64E16::one()).scalbn(k - 1);
        let t2b = (t1b + SfpM64E16::one()).scalbn(-k - 1);

        let s = t2a - t2b;
        let c = t2a + t2b;
        (s, c)
    }
}

impl crate::math::Hyperbolic for F32 {
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
