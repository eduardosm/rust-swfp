use crate::F64;
use crate::simple_fp::{Sfp, SfpM64E16, SfpM128E16};

// GENERATE: consts SfpM128E16 FRAC_1_3
const FRAC_1_3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB); // 3.3333333333333333333333333333333333333e-1

// GENERATE: consts SfpM64E16 INV_CBRT_2 INV_CBRT_4
const INV_CBRT_2: SfpM64E16 = Sfp::newp(-1, 0xCB2FF529EB71E416); // 7.937005259840997374e-1
const INV_CBRT_4: SfpM64E16 = Sfp::newp(-1, 0xA14517CC6B945711); // 6.299605249474365824e-1

impl crate::generic::Cbrt for F64 {
    fn cbrt_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);

        // Split |x| = 2^k * r such as
        // * k is an integer
        // * 1 <= r < 8
        let k = x.exponent();
        // 0 <= kmod3 <= 2
        let kmod3 = (((k + 1077) as u16) % 3) as i16;

        // 1 <= r < 8
        let r = x.abs().set_exp(kmod3);

        // t1 ~= cbrt(1 / r) with a polynomial approximation
        // The polynomial approximation is for range [1, 2]
        // cbrt(1 / r) = cbrt(1 / r') * cbrt(1 / 2^kmod3)
        let t0 = {
            // GENERATE: inv_cbrt_poly SfpM64E16 5
            const K0: SfpM64E16 = Sfp::newp(0, 0xD611504AAC2110BE); // 1.672403370343325444e0
            const K1: SfpM64E16 = Sfp::newn(0, 0x96FD660D8871253B); // -1.179608112914942420e0
            const K2: SfpM64E16 = Sfp::newp(-1, 0xB3D2126E24FEF83D); // 7.024241942317494025e-1
            const K3: SfpM64E16 = Sfp::newn(-3, 0xE5BE822B32981BA9); // -2.243595446458571523e-1
            const K4: SfpM64E16 = Sfp::newp(-6, 0xEE7396EA967CBA9E); // 2.910785175181692985e-2

            let r0 = r.set_exp(0);
            K0 + horner!(r0, r0, [K1, K2, K3, K4])
        };
        let t1 = match kmod3 {
            0 => t0,
            1 => t0 * INV_CBRT_2,
            2 => t0 * INV_CBRT_4,
            _ => unreachable!(),
        };

        let r = SfpM128E16::from_sfp(r);

        // ti ~= cbrt(1 / r)
        // initially, ti = t1
        let mut ti = SfpM128E16::from_sfp(t1);

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

impl crate::math::Cbrt for F64 {
    fn cbrt(self) -> Self {
        crate::generic::cbrt(self)
    }
}
