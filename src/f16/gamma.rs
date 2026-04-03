use crate::math_aux::reduce_half_revs_sfp;
use crate::math_aux::sfpm32e16::log_core;
use crate::simple_fp::{Sfp, SfpM32E16};
use crate::traits::FloatImpExt as _;
use crate::{F16, Float as _};

// GENERATE: consts SfpM32E16 PI LN_2 LN_PI
const PI: SfpM32E16 = Sfp::newp(1, 0xC90FDAA2); // 3.14159265e0
const LN_2: SfpM32E16 = Sfp::newp(-1, 0xB17217F8); // 6.93147181e-1
const LN_PI: SfpM32E16 = Sfp::newp(0, 0x92868247); // 1.14472989e0

impl crate::generic::Gamma for F16 {
    fn gamma_finite(x: Self) -> Self {
        if x >= Self::from_int(10) {
            // Overflow
            return Self::INFINITY;
        } else if x <= Self::from_int(-13) {
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

        let x = SfpM32E16::from_ieee_float(x.0);
        let k = SfpM32E16::newp(1, 0x17 << 27); // 2.875

        let d = x - k;
        let i = d.to_int_round::<i8>().unwrap();
        let f = d - SfpM32E16::from_int(i);

        // gf = Γ(k + f)
        let gf = {
            // GENERATE: gamma_poly SfpM32E16 8 2.875 -0.5 0.5
            const K0: SfpM32E16 = Sfp::newp(0, 0xE4D3B5F2); // 1.78771090e0
            const K1: SfpM32E16 = Sfp::newp(0, 0xC793AA39); // 1.55919388e0
            const K2: SfpM32E16 = Sfp::newp(0, 0x8688C931); // 1.05104937e0
            const K3: SfpM32E16 = Sfp::newp(-2, 0xF0FA44E6); // 4.70659402e-1
            const K4: SfpM32E16 = Sfp::newp(-3, 0xC1597BAE); // 1.88817914e-1
            const K5: SfpM32E16 = Sfp::newp(-5, 0xF0E41477); // 5.88112640e-2
            const K6: SfpM32E16 = Sfp::newp(-6, 0x920B103A); // 1.78275411e-2
            const K7: SfpM32E16 = Sfp::newp(-8, 0x8E26AAEB); // 4.33810564e-3
            const K8: SfpM32E16 = Sfp::newp(-10, 0x91E76D8C); // 1.11315930e-3

            K0 + horner!(f, f, [K1, K2, K3, K4, K5, K6, K7, K8])
        };

        let y = match i.cmp(&0) {
            core::cmp::Ordering::Equal => gf,
            core::cmp::Ordering::Greater => {
                // gi = prod(j = 0 to i - 1, k + f + j)
                let mut v = k + f;
                let mut gi = v;
                for _ in 1..i {
                    v = v + SfpM32E16::one();
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
                    v = v + SfpM32E16::one();
                    gi = gi * v;
                }
                // Γ(x) = Γ(k + f) / prod(j = 0 to |i| - 1, x + j) = gf / gi
                gf / gi
            }
        };

        Self(y.to_ieee_float())
    }

    fn ln_gamma_finite(x: Self) -> (Self, i8) {
        fn ln(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: ln_1p_poly SfpM32E16 3 -0.0001 0.0625
            const K2: SfpM32E16 = Sfp::newn(-2, 0xFFFEF30E); // -4.99991985e-1
            const K3: SfpM32E16 = Sfp::newp(-2, 0xAA4320F0); // 3.32543401e-1
            const K4: SfpM32E16 = Sfp::newn(-3, 0xE91848D3); // -2.27631700e-1

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo + horner!(lo2, lo, [K2, K3, K4]);

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        fn sin(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: sin_poly SfpM32E16 3
            const K3: SfpM32E16 = Sfp::newn(-3, 0xAAAAA2C3); // -1.66666549e-1
            const K5: SfpM32E16 = Sfp::newp(-7, 0x8883AC60); // 8.33217462e-3
            const K7: SfpM32E16 = Sfp::newn(-13, 0xCCA60716); // -1.95168062e-4

            let x2 = x.square();
            let x3 = x2 * x;

            x + horner!(x3, x2, [K3, K5, K7])
        }

        fn cos(x: SfpM32E16) -> SfpM32E16 {
            // GENERATE: cos_poly SfpM32E16 3
            const K4: SfpM32E16 = Sfp::newp(-5, 0xAAAAA554); // 4.16666468e-2
            const K6: SfpM32E16 = Sfp::newn(-10, 0xB60641DF); // -1.38873629e-3
            const K8: SfpM32E16 = Sfp::newp(-16, 0xCCFFFCF0); // 2.44378988e-5

            let x2 = x.square();
            let x4 = x2.square();

            let t = horner!(x4, x2, [K4, K6, K8]);
            SfpM32E16::one() + (t - x2.scalbn(-1))
        }

        let x = SfpM32E16::from_ieee_float(x.0);
        let (y, sign) = if x.abs() <= SfpM32E16::from_int(20) {
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

            let k = SfpM32E16::two();

            let d = x - k;
            let i = d.to_int_round::<i8>().unwrap();
            let f = d - SfpM32E16::from_int(i);

            // lgf = ln(Γ(k + f))
            let lgf = {
                // GENERATE: ln_gamma_poly SfpM32E16 8 2 -0.5 0.50001
                const K1: SfpM32E16 = Sfp::newp(-2, 0xD8773076); // 4.22784342e-1
                const K2: SfpM32E16 = Sfp::newp(-2, 0xA51A639B); // 3.22466958e-1
                const K3: SfpM32E16 = Sfp::newn(-4, 0x89F07DDD); // -6.73532327e-2
                const K4: SfpM32E16 = Sfp::newp(-6, 0xA89ED7AA); // 2.05835545e-2
                const K5: SfpM32E16 = Sfp::newn(-8, 0xF164FAC8); // -7.36677404e-3
                const K6: SfpM32E16 = Sfp::newp(-9, 0xBBA88FD2); // 2.86344062e-3
                const K7: SfpM32E16 = Sfp::newn(-10, 0xABEACEC3); // -1.31162428e-3
                const K8: SfpM32E16 = Sfp::newp(-11, 0x9EBAF7B1); // 6.05508201e-4

                horner!(f, f, [K1, K2, K3, K4, K5, K6, K7, K8])
            };

            match i.cmp(&0) {
                core::cmp::Ordering::Equal => (lgf, 1),
                core::cmp::Ordering::Greater => {
                    // gi = prod(j = 0 to i - 1, k + f + j)
                    let mut v = k + f;
                    let mut gi = v;
                    for _ in 1..i {
                        v = v + SfpM32E16::one();
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
                        v = v + SfpM32E16::one();
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
            let nx = if reflect { SfpM32E16::one() - x } else { x };

            // p = P(1 / nx)
            let p = {
                // GENERATE: gamma_lanczos_poly SfpM32E16 1 0.5 1.52e-5 0.051
                const K0: SfpM32E16 = Sfp::newp(1, 0xA06C8DD0); // 2.50662561e0
                const K1: SfpM32E16 = Sfp::newp(-3, 0xD655ECE3); // 2.09312154e-1

                K0 + nx.recip() * K1
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
                let lgx = LN_PI - ln(sinpix.abs() * p) - (nx - SfpM32E16::half()) * ln(nx) + nx;
                let sign = if sinpix.sign() { -1 } else { 1 };

                (lgx, sign)
            } else {
                // ln(abs(Γ(x))) = ln(Γ(nx))
                let lgx = (nx - SfpM32E16::half()) * ln_nx - nx + ln(p);
                (lgx, 1)
            }
        };

        (Self(y.to_ieee_float()), sign)
    }
}

impl crate::math::Gamma for F16 {
    fn gamma(self) -> Self {
        crate::generic::gamma(self)
    }

    fn ln_gamma(self) -> (Self, i8) {
        crate::generic::ln_gamma(self)
    }
}
