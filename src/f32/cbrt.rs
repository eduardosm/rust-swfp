use crate::F32;
use crate::simple_fp::{Sfp, SfpM32E16, SfpM64E16};

// GENERATE: consts SfpM64E16 FRAC_1_3
const FRAC_1_3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAB); // 3.333333333333333333e-1

// GENERATE: consts SfpM32E16 INV_CBRT_2 INV_CBRT_4
const INV_CBRT_2: SfpM32E16 = Sfp::newp(-1, 0xCB2FF52A); // 7.93700526e-1
const INV_CBRT_4: SfpM32E16 = Sfp::newp(-1, 0xA14517CC); // 6.29960525e-1

impl crate::generic::Cbrt for F32 {
    #[inline]
    fn cbrt_finite(x: Self) -> Self {
        let x = SfpM32E16::from_ieee_float(x.0);

        // Split |x| = 2^k * r such as
        // * k is an integer
        // * 1 <= r < 8
        let k = x.exponent();
        // 0 <= kmod3 <= 2
        let kmod3 = (((k + 153) as u16) % 3) as i16;

        // 1 <= r < 8
        let r = x.abs().set_exp(kmod3);

        // t1 ~= cbrt(1 / r) with a polynomial approximation
        // The polynomial approximation is for range [1, 2]
        // cbrt(1 / r) = cbrt(1 / r') * cbrt(1 / 2^kmod3)
        let t0 = {
            // GENERATE: inv_cbrt_poly SfpM32E16 3
            const K0: SfpM32E16 = Sfp::newp(0, 0xB2193911); // 1.39139474e0
            const K1: SfpM32E16 = Sfp::newn(-2, 0xF9C75185); // -4.87848804e-1
            const K2: SfpM32E16 = Sfp::newp(-4, 0xC257A8AC); // 9.48937585e-2

            let r0 = r.set_exp(0);
            K0 + horner!(r0, r0, [K1, K2])
        };
        let t1 = match kmod3 {
            0 => t0,
            1 => t0 * INV_CBRT_2,
            2 => t0 * INV_CBRT_4,
            _ => unreachable!(),
        };

        let r = SfpM64E16::from_sfp(r);

        // ti ~= cbrt(1 / r)
        // initially, ti = t1
        let mut ti = SfpM64E16::from_sfp(t1);

        // refine ti with Newton iterations
        // for each iteration: ti = ti - (1 / 3) * (r^2 * ti^4 - ti)
        ti = ti - FRAC_1_3 * (r * ti.square().square() - ti);
        ti = ti - FRAC_1_3 * (r * ti.square().square() - ti);
        ti = ti - FRAC_1_3 * (r * ti.square().square() - ti);

        // t2 = cbrt(r) = r * cbrt(1 / r)^2 = ti^2 * r
        let t2 = ti.square() * r;

        // y = cbrt(x)
        //   = sgn(x) * cbrt(r) * 2^((k - mod(k, 3)) / 3)
        //   = sgn(x) * t4 * 2^((k - mod(k, 3)) / 3)
        let y = t2.scalbn((k - kmod3) / 3).set_sign(x.sign());

        Self(y.to_ieee_float())
    }
}

impl crate::math::Cbrt for F32 {
    fn cbrt(self) -> Self {
        crate::generic::cbrt(self)
    }
}
