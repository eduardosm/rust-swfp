use crate::math_aux::reduce_half_revs_sfp;
use crate::math_aux::sfpm64e16::log_core;
use crate::simple_fp::{Sfp, SfpM64E16};
use crate::traits::FloatImpExt as _;
use crate::{F32, Float as _};

// GENERATE: consts SfpM64E16 PI LN_2 LN_PI
const PI: SfpM64E16 = Sfp::newp(1, 0xC90FDAA22168C235); // 3.141592653589793238e0
const LN_2: SfpM64E16 = Sfp::newp(-1, 0xB17217F7D1CF79AC); // 6.931471805599453094e-1
const LN_PI: SfpM64E16 = Sfp::newp(0, 0x928682473D0DE85F); // 1.144729885849400174e0

impl crate::generic::Gamma for F32 {
    fn gamma_finite(x: Self) -> Self {
        if x >= Self::from_int(36) {
            // Overflow
            return Self::INFINITY;
        } else if x <= Self::from_int(-43) {
            // Underflow
            let xi = (-x).to_uint(32).unwrap();
            return Self::ZERO.set_sign((xi & 1) == 0);
        }

        // Split x = k + f + i, such as:
        //  * k is some constant
        //  * i is an integer
        //  * -0.5 <= f <= 0.5
        // so, Γ(x) = Γ(k + f + i)
        //
        // when i = 0:
        //   Γ(x) = Γ(k + f)
        //
        // when i > 0:
        //  Γ(x) = Γ(k + f + i) = Γ(k + f) * prod(j = 0 to i - 1, k + f + j)
        //
        // when i < 0:
        //  Γ(k + f) = Γ(k + f + i) * prod(j = 0 to |i| - 1, k + f + i + j)
        //           = Γ(k + f + i) * prod(j = 0 to |i| - 1, x + j)
        //  Γ(x) = Γ(k + f + i) = Γ(k + f) / prod(j = 0 to |i| - 1, x + j)

        let x = SfpM64E16::from_ieee_float(x.0);
        let k = SfpM64E16::newp(1, 0x17 << 59); // 2.875

        let d = x - k;
        let i = d.to_int_round::<i8>().unwrap();
        let f = d - SfpM64E16::from_int(i);

        // gf = Γ(k + f)
        let gf = {
            // GENERATE: gamma_poly SfpM64E16 16 2.875 -0.5 0.5
            const K0: SfpM64E16 = Sfp::newp(0, 0xE4D3B5F2BB8916AB); // 1.787710898896940311e0
            const K1: SfpM64E16 = Sfp::newp(0, 0xC793AA6EE7C86087); // 1.559193901207950554e0
            const K2: SfpM64E16 = Sfp::newp(0, 0x8688C8CA4A0C44B6); // 1.051049326681182581e0
            const K3: SfpM64E16 = Sfp::newp(-2, 0xF0FA1678573089D9); // 4.706580182933928897e-1
            const K4: SfpM64E16 = Sfp::newp(-3, 0xC159AC51D747F32C); // 1.888186383201357453e-1
            const K5: SfpM64E16 = Sfp::newp(-5, 0xF0F959865CD0E7B2); // 5.883154841085434074e-2
            const K6: SfpM64E16 = Sfp::newp(-6, 0x9207B69E98EDEB9C); // 1.782594364047239925e-2
            const K7: SfpM64E16 = Sfp::newp(-8, 0x8A916503540B863E); // 4.228758166079025337e-3
            const K8: SfpM64E16 = Sfp::newp(-10, 0x8FE8042D715CDA89); // 1.097918043088986035e-3
            const K9: SfpM64E16 = Sfp::newp(-13, 0xCC0448C15F0AF0B1); // 1.945655204378488568e-4
            const K10: SfpM64E16 = Sfp::newp(-15, 0xD9FA03B4A86FCD60); // 5.196967578993331879e-5
            const K11: SfpM64E16 = Sfp::newp(-18, 0xA4EBD1E360B0B3F6); // 4.915033918311622105e-6
            const K12: SfpM64E16 = Sfp::newp(-19, 0xA41572752ED30A49); // 2.445038821348528761e-6
            const K13: SfpM64E16 = Sfp::newn(-23, 0x9DD711B449F8E961); // -1.470000612507324134e-7
            const K14: SfpM64E16 = Sfp::newp(-23, 0xAF0A78EB1838FDD4); // 1.630195486987184651e-7
            const K15: SfpM64E16 = Sfp::newn(-25, 0xC8562D954B6EB1E2); // -4.664450721912152569e-8
            const K16: SfpM64E16 = Sfp::newp(-26, 0xA9DBE6E4527FBE1E); // 1.977418920497313117e-8

            K0 + horner!(
                f,
                f,
                [
                    K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16
                ]
            )
        };

        let y = match i.cmp(&0) {
            core::cmp::Ordering::Equal => gf,
            core::cmp::Ordering::Greater => {
                // gi = prod(j = 0 to i - 1, k + f + j)
                let mut v = k + f;
                let mut gi = v;
                for _ in 1..i {
                    v = v + SfpM64E16::one();
                    gi = gi * v;
                }
                // Γ(x) = Γ(k + f) * prod(j = 0 to i - 1, k + f + j) = gf * gi
                gf * gi
            }
            core::cmp::Ordering::Less => {
                // gi = prod(j = 0 to |i| - 1, x + j)
                let mut v = x;
                let mut gi = v;
                for _ in 1..-i {
                    v = v + SfpM64E16::one();
                    gi = gi * v;
                }
                // Γ(x) = Γ(k + f) / prod(j = 0 to |i| - 1, x + j) = gf / gi
                gf / gi
            }
        };

        Self(y.to_ieee_float())
    }

    fn ln_gamma_finite(x: Self) -> (Self, i8) {
        fn ln(x: SfpM64E16) -> SfpM64E16 {
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

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        fn sin(x: SfpM64E16) -> SfpM64E16 {
            // GENERATE: sin_poly SfpM64E16 5
            const K3: SfpM64E16 = Sfp::newn(-3, 0xAAAAAAAAA91D198C); // -1.666666666663135563e-1
            const K5: SfpM64E16 = Sfp::newp(-7, 0x888888864E280F31); // 8.333333325227790763e-3
            const K7: SfpM64E16 = Sfp::newn(-13, 0xD00CFCA8F30E3A39); // -1.984126379823826394e-4
            const K9: SfpM64E16 = Sfp::newp(-19, 0xB8EBBC225A2D26F9); // 2.755535236744545341e-6
            const K11: SfpM64E16 = Sfp::newn(-26, 0xD4B2924F17F19652); // -2.476125315268813741e-8

            let x2 = x.square();
            let x3 = x2 * x;

            x + horner!(x3, x2, [K3, K5, K7, K9, K11])
        }

        fn cos(x: SfpM64E16) -> SfpM64E16 {
            // GENERATE: cos_poly SfpM64E16 5
            const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAAA9865DBB); // 4.166666666660176291e-2
            const K6: SfpM64E16 = Sfp::newn(-10, 0xB60B60B3B22AF5A2); // -1.388888887820925452e-3
            const K8: SfpM64E16 = Sfp::newp(-16, 0xD00CFD511A621F40); // 2.480158094260118932e-5
            const K10: SfpM64E16 = Sfp::newn(-22, 0x93F01362DC3BED65); // -2.755556177525308368e-7
            const K12: SfpM64E16 = Sfp::newp(-29, 0x8DE34535C3BF5065); // 2.064738870992259184e-9

            let x2 = x.square();
            let x4 = x2.square();

            let t = horner!(x4, x2, [K4, K6, K8, K10, K12]);
            SfpM64E16::one() + (t - x2.scalbn(-1))
        }

        let x = SfpM64E16::from_ieee_float(x.0);
        let (y, sign) = if x.abs() <= SfpM64E16::from_int(45) {
            // Split x = k + f + i, such as:
            //  * k is some constant
            //  * i is an integer
            //  * -0.5 <= f <= 0.5
            // so, Γ(x) = Γ(k + f + i)
            //
            // when i = 0:
            //   Γ(x) = Γ(k + f)
            //
            // when i > 0:
            //  Γ(x) = Γ(k + f + i) = Γ(k + f) * prod(j = 0 to i - 1, k + f + j)
            //
            // when i < 0:
            //  Γ(k + f) = Γ(k + f + i) * prod(j = 0 to |i| - 1, k + f + i + j)
            //           = Γ(k + f + i) * prod(j = 0 to |i| - 1, x + j)
            //  Γ(x) = Γ(k + f + i) = Γ(k + f) / prod(j = 0 to |i| - 1, x + j)

            let k = SfpM64E16::two();

            let d = x - k;
            let i = d.to_int_round::<i8>().unwrap();
            let f = d - SfpM64E16::from_int(i);

            // lgf = ln(Γ(k + f))
            let lgf = {
                // GENERATE: ln_gamma_poly SfpM64E16 15 2 -0.5 0.50001
                const K1: SfpM64E16 = Sfp::newp(-2, 0xD8773039049F4ED7); // 4.227843350984686807e-1
                const K2: SfpM64E16 = Sfp::newp(-2, 0xA51A662530A058EF); // 3.224670334241756596e-1
                const K3: SfpM64E16 = Sfp::newn(-4, 0x89F000D2B147615D); // -6.735230105383366542e-2
                const K4: SfpM64E16 = Sfp::newp(-6, 0xA89915629FD44FE5); // 2.058080841833968134e-2
                const K5: SfpM64E16 = Sfp::newn(-8, 0xF2027DF8F4944588); // -7.385550985337226748e-3
                const K6: SfpM64E16 = Sfp::newp(-9, 0xBD6EB91ABDD96402); // 2.890510741728210967e-3
                const K7: SfpM64E16 = Sfp::newn(-10, 0x9C5637C7DFC9E0E7); // -1.192755040335293494e-3
                const K8: SfpM64E16 = Sfp::newp(-11, 0x859AD0FE70E821CC); // 5.096616801986424556e-4
                const K9: SfpM64E16 = Sfp::newn(-13, 0xE9FAC8E3DFCD853A); // -2.231403616334676907e-4
                const K10: SfpM64E16 = Sfp::newp(-14, 0xD0BCDB4B7F8C65E6); // 9.953390177481185373e-5
                const K11: SfpM64E16 = Sfp::newn(-15, 0xBCD94DFAE745AC62); // -4.502507355970531764e-5
                const K12: SfpM64E16 = Sfp::newp(-16, 0xA8B7C3687701AAC6); // 2.011273210530979716e-5
                const K13: SfpM64E16 = Sfp::newn(-17, 0x982E464629B75908); // -9.070680129365463227e-6
                const K14: SfpM64E16 = Sfp::newp(-18, 0xB4D890772D8FED20); // 5.389629434620917064e-6
                const K15: SfpM64E16 = Sfp::newn(-19, 0xB6F5949B0EED815A); // -2.726306003227484658e-6

                horner!(
                    f,
                    f,
                    [
                        K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15
                    ]
                )
            };

            match i.cmp(&0) {
                core::cmp::Ordering::Equal => (lgf, 1),
                core::cmp::Ordering::Greater => {
                    // gi = prod(j = 0 to i - 1, k + f + j)
                    let mut v = k + f;
                    let mut gi = v;
                    for _ in 1..i {
                        v = v + SfpM64E16::one();
                        gi = gi * v;
                    }
                    // ln(abs(Γ(x))) = ln(Γ(k + f)) + ln(prod(j = 0 to i - 1, k + f + j)) = lgf + ln(gi)
                    (lgf + ln(gi), 1)
                }
                core::cmp::Ordering::Less => {
                    // gi = prod(j = 0 to |i| - 1, x + j)
                    let mut v = x;
                    let mut gi = v;
                    for _ in 1..-i {
                        v = v + SfpM64E16::one();
                        gi = gi * v;
                    }
                    // ln(abs(Γ(x))) = ln(Γ(k + f)) - ln(abs(prod(j = 0 to |i| - 1, x + j))) = lgf - ln(abs(gi))
                    let sign = if gi.sign() { -1 } else { 1 };
                    (lgf - ln(gi.abs()), sign)
                }
            }
        } else {
            // Use Lanczos approximation:
            // Γ(x) = √(2π) * (x + g - 0.5)^(x - 0.5) * exp(-(x + g - 0.5)) * Ag(x - 1)
            //      = (x + g - 0.5)^(x - 0.5) * exp(-(x + g - 0.5)) * P(1 / x)
            // with let P(1 / x) = √(2π) * Ag(x - 1) = Γ(x) / ((x + g - 0.5)^(x - 0.5) * exp(-(x + g - 0.5)))
            //
            // choose g = 0.5, so:
            // ln(Γ(x)) = (x - 0.5) * ln(x) - x + ln(P(1 / x))
            //
            // For x < 0.5, use reflection formula:
            // Γ(x)*Γ(1-x) = π/sin(πx) => Γ(x) = π/(sin(πx)*Γ(1-x))

            // nx = x or 1 - x, so nx >= 0.5
            let reflect = x.sign();
            let nx = if reflect { SfpM64E16::one() - x } else { x };

            // p = P(1 / nx)
            let p = {
                // GENERATE: gamma_lanczos_poly SfpM64E16 4 0.5 2.93e-39 0.022223
                const K0: SfpM64E16 = Sfp::newp(1, 0xA06C98FFB139A4EB); // 2.506628274631021387e0
                const K1: SfpM64E16 = Sfp::newp(-3, 0xD5E621541DAC0C36); // 2.088856895056033093e-1
                const K2: SfpM64E16 = Sfp::newp(-7, 0x8E997DB5D00683AB); // 8.703587306854749480e-3
                const K3: SfpM64E16 = Sfp::newn(-8, 0xDC4E7583C2AE9BF8); // -6.723220234126487095e-3
                const K4: SfpM64E16 = Sfp::newn(-12, 0xF42FCEC8EDDC4AD9); // -4.657492619263783405e-4

                let inv_nx = nx.recip();
                K0 + horner!(inv_nx, inv_nx, [K1, K2, K3, K4])
            };

            let ln_nx = ln(nx);

            if reflect {
                let (n, z) = reduce_half_revs_sfp(x);
                let z_rad = z * PI;
                let sinpix = match n {
                    0 => sin(z_rad),
                    1 => cos(z_rad),
                    2 => -sin(z_rad),
                    3 => -cos(z_rad),
                    _ => unreachable!(),
                };

                // ln(abs(Γ(x))) = ln(π) - ln(abs(sin(πx))) - ln(Γ(1-x))
                let lgx = LN_PI - ln(sinpix.abs() * p) - (nx - SfpM64E16::half()) * ln(nx) + nx;
                let sign = if sinpix.sign() { -1 } else { 1 };

                (lgx, sign)
            } else {
                // ln(abs(Γ(x))) = ln(Γ(nx))
                let lgx = (nx - SfpM64E16::half()) * ln_nx - nx + ln(p);
                (lgx, 1)
            }
        };

        (Self(y.to_ieee_float()), sign)
    }
}

impl crate::math::Gamma for F32 {
    fn gamma(self) -> Self {
        crate::generic::gamma(self)
    }

    fn ln_gamma(self) -> (Self, i8) {
        crate::generic::ln_gamma(self)
    }
}
