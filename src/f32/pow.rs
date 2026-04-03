use crate::F32;
use crate::math_aux::sfpm128e16::log_core;
use crate::simple_fp::{Sfp, SfpM128E16};

// GENERATE: consts SfpM128E16 LN_2 LOG2_E
const LN_2: SfpM128E16 = Sfp::newp(-1, 0xB17217F7D1CF79ABC9E3B39803F2F6AF); // 6.9314718055994530941723212145817656808e-1
const LOG2_E: SfpM128E16 = Sfp::newp(0, 0xB8AA3B295C17F0BBBE87FED0691D3E89); // 1.4426950408889634073599246810018921374e0

impl crate::generic::Pow for F32 {
    fn pow_finite(x: Self, y: Self, sign: bool) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);
        let y = SfpM128E16::from_ieee_float(y.0);
        pow_core(x, y, sign)
    }

    fn powi_finite(x: Self, y: i32) -> Self {
        let x = SfpM128E16::from_ieee_float(x.0);
        let sign = x.sign() && (y & 1) != 0;
        let y = SfpM128E16::from_int(y);
        pow_core(x, y, sign)
    }
}

fn pow_core(x: SfpM128E16, y: SfpM128E16, sign: bool) -> F32 {
    #[inline]
    fn ln(x: SfpM128E16) -> SfpM128E16 {
        // GENERATE: ln_1p_poly SfpM128E16 10 -0.00001 0.015625
        const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFF38A3FB07BA6E5); // -4.9999999999999999999999484667897942096e-1
        const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAA1301411AC15A0344); // 3.3333333333333333331727551914729008914e-1
        const K4: SfpM128E16 = Sfp::newn(-3, 0xFFFFFFFFFFFFFB0199610568A2BB7510); // -2.4999999999999998267443758344511132043e-1
        const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCC260DAEECFB6E4E6AA6118); // 1.9999999999999074370706811564527328844e-1
        const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAA9E2B12F02E0A04B415E12FF8); // -1.6666666666382485748818844339413961970e-1
        const K7: SfpM128E16 = Sfp::newp(-3, 0x92492489031F459F2E87F4962E1495AA); // 1.4285714231735304476715502735856501788e-1
        const K8: SfpM128E16 = Sfp::newn(-4, 0xFFFFF7328FCD72305DECB5734DC61359); // -1.2499993441631460469066548847303191335e-1
        const K9: SfpM128E16 = Sfp::newp(-4, 0xE38B8AFADE8DAF20A0C95B8E0EA07F61); // 1.1110600069134232552219948581757297085e-1
        const K10: SfpM128E16 = Sfp::newn(-4, 0xCC4B0C288D98A3E9D2D6A136284B9F4D); // -9.9752516734678948878307558054302137430e-2
        const K11: SfpM128E16 = Sfp::newp(-4, 0xAC3A52B23778512759684B35D555CD2D); // 8.4095617355100101611618810088092089093e-2

        let (k, lo, ln_hi) = log_core(x);
        let lo2 = lo.square();
        let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]);

        // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
        k * LN_2 + (ln_hi + ln_lo)
    }

    #[inline]
    fn exp(x: SfpM128E16) -> SfpM128E16 {
        // x = k*ln(2) + r
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_E).to_int_round() else {
            if x.sign() {
                return SfpM128E16::ZERO;
            } else {
                return SfpM128E16::INFINITY;
            }
        };
        let r = x - SfpM128E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^4

        let rs = r.scalbn(-4);

        // GENERATE: exp_m1_poly SfpM128E16 9 -0.0216608 0.0216608
        const K2: SfpM128E16 = Sfp::newp(-1, 0x800000000000000000000034819DEEC5); // 5.0000000000000000000000000066272288092e-1
        const K3: SfpM128E16 = Sfp::newp(-3, 0xAAAAAAAAAAAAAAAAA910F142CC915B62); // 1.6666666666666666666658193767947496168e-1
        const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAA87C8899F6573A11); // 4.1666666666666666666637811863381114105e-2
        const K5: SfpM128E16 = Sfp::newp(-7, 0x888888888888909FEED70B41C4AF4D31); // 8.3333333333333350878770715131364071254e-3
        const K6: SfpM128E16 = Sfp::newp(-10, 0xB60B60B60B60C1FA20171FA9B5F53DD2); // 1.3888888888888892123228690916861705051e-3
        const K7: SfpM128E16 = Sfp::newp(-13, 0xD00D00CFDA245CBF09615956754D4771); // 1.9841269840140497220276947582416499918e-4
        const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00CFD85F4D1482EE1599C828E73A); // 2.4801587300126500532075214060594908244e-5
        const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF9B2A0E034EB8929B5B5DA2BB7C69); // 2.7557605708980918710648003314163235546e-6
        const K10: SfpM128E16 = Sfp::newp(-22, 0x93F2E36581E20A8178EB33C2E92F5D09); // 2.7557608167720408359976377926825683075e-7

        let rs2 = rs.square();
        let mut t = rs + horner!(rs2, rs, [K2, K3, K4, K5, K6, K7, K8, K9, K10]);

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..4 {
            t = t.square() + t.scalbn(1);
        }

        // exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        (t + SfpM128E16::one()).scalbn(k)
    }

    // |z| = |x|^y = exp(y * ln(|x|))
    let absz = exp(y * ln(x.abs()));
    let z = absz.set_sign(sign);

    F32(z.round_prec(85).to_ieee_float())
}

impl crate::math::Pow for F32 {
    fn pow(self, other: Self) -> Self {
        crate::generic::pow(self, other)
    }

    fn powi(self, n: i32) -> Self {
        crate::generic::powi(self, n)
    }
}
