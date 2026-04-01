use crate::F16;
use crate::simple_fp::{Sfp, SfpM32E16, SfpM64E16};

// GENERATE: consts SfpM32E16 PI FRAC_PI_2 FRAC_180_PI FRAC_1_PI
const PI: SfpM32E16 = Sfp::newp(1, 0xC90FDAA2); // 3.14159265e0
const FRAC_PI_2: SfpM32E16 = Sfp::newp(0, 0xC90FDAA2); // 1.57079633e0
const FRAC_180_PI: SfpM32E16 = Sfp::newp(5, 0xE52EE0D3); // 5.72957795e1
const FRAC_1_PI: SfpM32E16 = Sfp::newp(-2, 0xA2F9836E); // 3.18309886e-1

impl crate::generic::InvTrigonometric for F16 {
    #[inline]
    fn pi() -> Self {
        Self(PI.to_ieee_float())
    }

    #[inline]
    fn frac_pi_2() -> Self {
        Self(FRAC_PI_2.to_ieee_float())
    }

    #[inline]
    fn asin_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = asin_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acos_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = acos_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = atan_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2_finite(y: Self, x: Self) -> Self {
        let y = SfpM64E16::from_ieee_float(y.0);
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan2_core(y, x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn asind_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = asin_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acosd_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = acos_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atand_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = atan_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2d_finite(y: Self, x: Self) -> Self {
        // GENERATE: consts SfpM64E16 FRAC_180_PI
        const FRAC_180_PI: SfpM64E16 = Sfp::newp(5, 0xE52EE0D31E0FBDC3); // 5.729577951308232088e1

        let y = SfpM64E16::from_ieee_float(y.0);
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan2_core(y, x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn asinpi_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = asin_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acospi_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = acos_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atanpi_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);
        let r = atan_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2pi_finite(y: Self, x: Self) -> Self {
        // GENERATE: consts SfpM64E16 FRAC_1_PI
        const FRAC_1_PI: SfpM64E16 = Sfp::newp(-2, 0xA2F9836E4E44152A); // 3.183098861837906715e-1

        let y = SfpM64E16::from_ieee_float(y.0);
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan2_core(y, x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }
}

fn asin_core(x: SfpM32E16) -> SfpM32E16 {
    #[inline]
    fn asin_poly(x: SfpM32E16, x2: SfpM32E16, x3: SfpM32E16) -> SfpM32E16 {
        // GENERATE: asin_poly SfpM32E16 5
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAAB1298); // 1.66668215e-1
        const K5: SfpM32E16 = Sfp::newp(-4, 0x99754DF4); // 7.49307718e-2
        const K7: SfpM32E16 = Sfp::newp(-5, 0xBB3816C0); // 4.57077874e-2
        const K9: SfpM32E16 = Sfp::newp(-6, 0xBD94E3B8); // 2.31422851e-2
        const K11: SfpM32E16 = Sfp::newp(-5, 0xB340DFA3); // 4.37630401e-2

        x + horner!(x3, x2, [K3, K5, K7, K9, K11])
    }

    if x.exponent() < -1 {
        // |x| < 0.5
        let x2 = x.square();
        let x3 = x2 * x;

        asin_poly(x, x2, x3)
    } else {
        // |x| >= 0.5
        // |asin(x)| = π/2 - 2 * asin(sqrt((1 - |x|) / 2))

        // y = sqrt((1 - |x|) / 2)
        let y2 = (SfpM32E16::one() - x.abs()).scalbn(-1);
        let y = y2.sqrt();
        let y3 = y2 * y;

        // t1 = asin(y)
        let t1 = asin_poly(y, y2, y3);

        // t3 = π/2 - 2 * asin(sqrt((1 - |x|) / 2)) = π/2 - 2 * t1
        let t3 = FRAC_PI_2 - t1.scalbn(1);

        t3.set_sign(x.sign())
    }
}

fn acos_core(x: SfpM32E16) -> SfpM32E16 {
    #[inline]
    fn asin_poly(x: SfpM32E16, x2: SfpM32E16, x3: SfpM32E16) -> SfpM32E16 {
        // GENERATE: asin_poly SfpM32E16 5
        const K3: SfpM32E16 = Sfp::newp(-3, 0xAAAB1298); // 1.66668215e-1
        const K5: SfpM32E16 = Sfp::newp(-4, 0x99754DF4); // 7.49307718e-2
        const K7: SfpM32E16 = Sfp::newp(-5, 0xBB3816C0); // 4.57077874e-2
        const K9: SfpM32E16 = Sfp::newp(-6, 0xBD94E3B8); // 2.31422851e-2
        const K11: SfpM32E16 = Sfp::newp(-5, 0xB340DFA3); // 4.37630401e-2

        x + horner!(x3, x2, [K3, K5, K7, K9, K11])
    }

    // acos(x) = π/2 - asin(x)
    if x.exponent() < -1 {
        // |x| < 0.5
        let x2 = x.square();
        let x3 = x2 * x;

        // t1 = asin(x)
        let t1 = asin_poly(x, x2, x3);

        // acos(x) = π/2 - asin(x) = π/2 - t1
        FRAC_PI_2 - t1
    } else {
        // |x| >= 0.5
        // |asin(x)| = π/2 - 2 * asin(sqrt((1 - |x|) / 2))

        // y = sqrt((1 - |x|) / 2)
        let y2 = (SfpM32E16::one() - x.abs()).scalbn(-1);
        let y = y2.sqrt();
        let y3 = y2 * y;

        // t1 = asin(y)
        let t1 = asin_poly(y, y2, y3);

        // t2 = 2 * asin(y) = 2 * t1
        let t2 = t1.scalbn(1);

        if x.sign() {
            // acos(x) = π/2 + |asin(x)|
            //         = π/2 + (π/2 - 2 * asin(y))
            //         = π - 2 * asin(y)
            //         = π - t2
            let pi = FRAC_PI_2.scalbn(1);
            pi - t2
        } else {
            // acos(x) = π/2 - |asin(x)|
            //         = π/2 - (π/2 - 2 * asin(y))
            //         = 2 * asin(y)
            //         = t2
            t2
        }
    }
}

fn atan_core(x: SfpM32E16) -> SfpM32E16 {
    #[inline]
    fn atan_poly(x: SfpM32E16, x2: SfpM32E16, x3: SfpM32E16) -> SfpM32E16 {
        // GENERATE: atan_poly SfpM32E16 8 -0.00001 1.0
        const K3: SfpM32E16 = Sfp::newn(-2, 0xAAAA367A); // -3.33329871e-1
        const K5: SfpM32E16 = Sfp::newp(-3, 0xCCB3A016); // 1.99903966e-1
        const K7: SfpM32E16 = Sfp::newn(-3, 0x9143AEE2); // -1.41859753e-1
        const K9: SfpM32E16 = Sfp::newp(-4, 0xD88DDB79); // 1.05739321e-1
        const K11: SfpM32E16 = Sfp::newn(-4, 0x96DEC1B0); // -7.36670620e-2
        const K13: SfpM32E16 = Sfp::newp(-5, 0xA86F6584); // 4.11218610e-2
        const K15: SfpM32E16 = Sfp::newn(-7, 0xF7EE75CA); // -1.51325369e-2
        const K17: SfpM32E16 = Sfp::newp(-9, 0xABD9F74C); // 2.62224472e-3

        x + horner!(x3, x2, [K3, K5, K7, K9, K11, K13, K15, K17])
    }

    if x.abs() <= SfpM32E16::one() {
        let x2 = x.square();
        let x3 = x2 * x;
        atan_poly(x, x2, x3)
    } else {
        // atan(x) = ±pi/2 - atan(1 / x)
        let inv_x = x.recip();
        let inv_x2 = inv_x.square();
        let inv_x3 = inv_x2 * inv_x;

        FRAC_PI_2.set_sign(x.sign()) - atan_poly(inv_x, inv_x2, inv_x3)
    }
}

fn atan2_core(mut n: SfpM64E16, mut d: SfpM64E16) -> SfpM64E16 {
    // GENERATE: consts SfpM64E16 FRAC_PI_2
    const FRAC_PI_2: SfpM64E16 = Sfp::newp(0, 0xC90FDAA22168C235); // 1.570796326794896619e0

    #[inline]
    fn atan_poly(x: SfpM64E16, x2: SfpM64E16, x3: SfpM64E16) -> SfpM64E16 {
        // GENERATE: atan_poly SfpM64E16 13 -0.00001 1.0
        const K3: SfpM64E16 = Sfp::newn(-2, 0xAAAAAAA16F390AE2); // -3.333333322585635061e-1
        const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCC833707C71A2); // 1.999999314685933561e-1
        const K7: SfpM64E16 = Sfp::newn(-3, 0x9248B4EE8B2FAC04); // -1.428554792972104721e-1
        const K9: SfpM64E16 = Sfp::newp(-4, 0xE382F8F03AC175E6); // 1.110896538078527221e-1
        const K11: SfpM64E16 = Sfp::newn(-4, 0xB9D56B8ED026383E); // -9.073909787732069341e-2
        const K13: SfpM64E16 = Sfp::newp(-4, 0x9BB3598C78378D71); // 7.602567634514991846e-2
        const K15: SfpM64E16 = Sfp::newn(-4, 0x81B9382F69738876); // -6.334155935945219664e-2
        const K17: SfpM64E16 = Sfp::newp(-5, 0xCC332814F140D4F0); // 4.985347420650062129e-2
        const K19: SfpM64E16 = Sfp::newn(-5, 0x8D3E7AF803664C55); // -3.448341402710182150e-2
        const K21: SfpM64E16 = Sfp::newp(-6, 0x9DA595CE12CB220A); // 1.924399622711216262e-2
        const K23: SfpM64E16 = Sfp::newn(-7, 0x80867FB575E46D89); // -7.844567027716099920e-3
        const K25: SfpM64E16 = Sfp::newp(-9, 0x850490625491171C); // 2.029690980935209974e-3
        const K27: SfpM64E16 = Sfp::newn(-12, 0x81663FE1280CB229); // -2.468097919923415844e-4

        x + horner!(
            x3,
            x2,
            [K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25, K27]
        )
    }

    let ysgn = n.sign();
    let xsgn = d.sign();

    let mut off = SfpM64E16::ZERO;
    if xsgn {
        off = SfpM64E16::two().set_sign(ysgn);
    }
    if n.abs() > d.abs() {
        core::mem::swap(&mut n, &mut d);
        n = -n;
        off = off + SfpM64E16::one().set_sign(ysgn ^ xsgn);
    }

    // atan2(y, x) = atan(n/d) + off * π/2
    let z = n / d;
    let z2 = z.square();
    let z3 = z2 * z;
    atan_poly(z, z2, z3) + off * FRAC_PI_2
}

impl crate::math::InvTrigonometric for F16 {
    fn asin(self) -> Self {
        crate::generic::asin(self)
    }

    fn acos(self) -> Self {
        crate::generic::acos(self)
    }

    fn atan(self) -> Self {
        crate::generic::atan(self)
    }

    fn atan2(self, other: Self) -> Self {
        crate::generic::atan2(self, other)
    }

    fn asind(self) -> Self {
        crate::generic::asind(self)
    }

    fn acosd(self) -> Self {
        crate::generic::acosd(self)
    }

    fn atand(self) -> Self {
        crate::generic::atand(self)
    }

    fn atan2d(self, other: Self) -> Self {
        crate::generic::atan2d(self, other)
    }

    fn asinpi(self) -> Self {
        crate::generic::asinpi(self)
    }

    fn acospi(self) -> Self {
        crate::generic::acospi(self)
    }

    fn atanpi(self) -> Self {
        crate::generic::atanpi(self)
    }

    fn atan2pi(self, other: Self) -> Self {
        crate::generic::atan2pi(self, other)
    }
}
