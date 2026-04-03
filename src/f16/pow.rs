use crate::F16;
use crate::math_aux::sfpm64e16::log_core;
use crate::simple_fp::{Sfp, SfpM64E16};

// GENERATE: consts SfpM64E16 LN_2 LOG2_E
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1
const LOG2_E: SfpM64E16 = Sfp::newp(0, 0xB8AA3B295C17F0BC); // 1.442695040888963407e0

impl crate::generic::Pow for F16 {
    fn pow_finite(x: Self, y: Self, sign: bool) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let y = SfpM64E16::from_ieee_float(y.0);
        pow_core(x, y, sign)
    }

    fn powi_finite(x: Self, y: i32) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let sign = x.sign() && (y & 1) != 0;
        let y = SfpM64E16::from_int(y);
        pow_core(x, y, sign)
    }
}

fn pow_core(x: SfpM64E16, y: SfpM64E16, sign: bool) -> F16 {
    #[inline]
    fn ln(x: SfpM64E16) -> SfpM64E16 {
        // GENERATE: ln_1p_poly SfpM64E16 6 -0.0001 0.0625
        const K2: SfpM64E16 = Sfp::newn(-2, 0xFFFFFFFF91AB93B2); // -4.999999999498278259e-1
        const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAA237EA980A9); // 3.333333175972539354e-1
        const K4: SfpM64E16 = Sfp::newn(-3, 0xFFFF9189EE3722C6); // -2.499983539997209333e-1
        const K5: SfpM64E16 = Sfp::newp(-3, 0xCCB7FA272CDE1E25); // 1.999205671702083345e-1
        const K6: SfpM64E16 = Sfp::newn(-3, 0xA8AACA2E92935D0B); // -1.647140112805972509e-1
        const K7: SfpM64E16 = Sfp::newp(-4, 0xF345BFDCFD5B6F60); // 1.187853802982041507e-1

        let (k, lo, ln_hi) = log_core(x);
        let lo2 = lo.square();
        let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7]);

        // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
        k * LN_2 + (ln_hi + ln_lo)
    }

    #[inline]
    fn exp(x: SfpM64E16) -> SfpM64E16 {
        // x = k*ln(2) + r
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_E).to_int_round() else {
            if x.sign() {
                return SfpM64E16::ZERO;
            } else {
                return SfpM64E16::INFINITY;
            }
        };
        let r = x - SfpM64E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM64E16 5 -0.0216609 0.0216609
        const K2: SfpM64E16 = Sfp::newp(-1, 0x8000000000001A81); // 5.000000000000003678e-1
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA57E95031); // 1.666666666478502929e-1
        const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAA43B86262); // 4.166666666081484045e-2
        const K5: SfpM64E16 = Sfp::newp(-7, 0x888913B538B6C1AA); // 8.333462949745948728e-3
        const K6: SfpM64E16 = Sfp::newp(-10, 0xB60C2173D3E2218B); // 1.388911326934098377e-3

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        (t + SfpM64E16::one()).scalbn(k)
    }

    // |z| = |x|^y = exp(y * ln(|x|))
    let absz = exp(y * ln(x.abs()));
    let z = absz.set_sign(sign);

    F16(z.round_prec(46).to_ieee_float())
}

impl crate::math::Pow for F16 {
    fn pow(self, other: Self) -> Self {
        crate::generic::pow(self, other)
    }

    fn powi(self, n: i32) -> Self {
        crate::generic::powi(self, n)
    }
}
