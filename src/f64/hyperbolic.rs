use crate::F64;
use crate::Float as _;
use crate::simple_fp::{Sfp, SfpM128E16};
use crate::traits::FloatImpExt as _;

// GENERATE: consts SfpM128E16 LOG2_E LN_2
const LOG2_E: SfpM128E16 = Sfp::newp(0, 0xB8AA3B295C17F0BBBE87FED0691D3E89); // 1.4426950408889634073599246810018921374e0
const LN_2: SfpM128E16 = Sfp::newp(-1, 0xB17217F7D1CF79ABC9E3B39803F2F6AF); // 6.9314718055994530941723212145817656808e-1

impl crate::generic::Hyperbolic for F64 {
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
        // Very hard to round case
        if x.abs().bitwise_eq(Self::from_bits(0x3E9E0000000000E1)) {
            return Self::from_bits(0x3E9DFFFFFFFFFEAF).set_sign(x.sign());
        }

        let (sinh, cosh) = sinh_cosh_inner(x);
        if sinh.is_infinite() && cosh.is_infinite() {
            Self::ONE.set_sign(sinh.sign())
        } else {
            Self((sinh / cosh).to_ieee_float())
        }
    }
}

fn sinh_cosh_inner(x: F64) -> (SfpM128E16, SfpM128E16) {
    #[inline]
    fn exp_m1_core(r: SfpM128E16) -> (SfpM128E16, SfpM128E16) {
        // exp(r) = exp(r / s)^s
        // exp(-r) = exp(-r / s)^s
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
        let mrs = -rs;
        let mut t1 = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);
        let mut t2 = mrs + horner!(rs2, mrs, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t1 = t1.square() + t1.scalbn(1);
            t2 = t2.square() + t2.scalbn(1);
        }

        (t1, t2)
    }

    let x = SfpM128E16::from_ieee_float(x.0);

    // x = k*ln(2) + r
    // k is an integer
    // |r| <= 0.5*ln(2)
    let Some(k) = (x * LOG2_E).to_int_round::<i16>() else {
        return (
            SfpM128E16::INFINITY.set_sign(x.sign()),
            SfpM128E16::INFINITY,
        );
    };
    let r = x - SfpM128E16::from_int(k) * LN_2;

    // t1 = exp(r) - 1
    // t2 = exp(-r) - 1
    let (t1a, t1b) = exp_m1_core(r);

    // sinh(x) = (exp(x) - exp(-x)) / 2
    // cosh(x) = (exp(x) + exp(-x)) / 2

    if k == 0 {
        let s = (t1a - t1b).scalbn(-1);
        let c = (t1a + t1b).scalbn(-1) + SfpM128E16::one();
        (s, c)
    } else {
        let t2a = (t1a + SfpM128E16::one()).scalbn(k - 1);
        let t2b = (t1b + SfpM128E16::one()).scalbn(-k - 1);

        let s = t2a - t2b;
        let c = t2a + t2b;
        (s, c)
    }
}

impl crate::math::Hyperbolic for F64 {
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
