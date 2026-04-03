use crate::math_aux::reduce_pi_2_large;
use crate::simple_fp::{Sfp, SfpM32E16, SfpM64E16};
use crate::traits::FloatImpExt as _;
use crate::{F32, Float as _};

// GENERATE: consts SfpM64E16 PI FRAC_PI_4 FRAC_PI_180
const PI: SfpM64E16 = Sfp::newp(1, 0xC90FDAA22168C235); // 3.141592653589793238e0
const FRAC_PI_4: SfpM64E16 = Sfp::newp(-1, 0xC90FDAA22168C235); // 7.853981633974483096e-1
const FRAC_PI_180: SfpM64E16 = Sfp::newp(-6, 0x8EFA351294E9C8AE); // 1.745329251994329577e-2

impl crate::generic::Trigonometric for F32 {
    fn sin_finite(x: Self) -> Self {
        let (n, y) = reduce_pi_2(x);
        match n {
            0 => sin_core(y),
            1 => cos_core(y),
            2 => -sin_core(y),
            3 => -cos_core(y),
            _ => unreachable!(),
        }
    }

    fn cos_finite(x: Self) -> Self {
        let (n, y) = reduce_pi_2(x);
        match n {
            0 => cos_core(y),
            1 => -sin_core(y),
            2 => -cos_core(y),
            3 => sin_core(y),
            _ => unreachable!(),
        }
    }

    fn sin_cos_finite(x: Self) -> (Self, Self) {
        let (n, y) = reduce_pi_2(x);
        let sin = sin_core(y);
        let cos = cos_core(y);
        let (sin, cos) = match n {
            0 => (sin, cos),
            1 => (cos, -sin),
            2 => (-sin, -cos),
            3 => (-cos, sin),
            _ => unreachable!(),
        };
        (sin, cos)
    }

    fn tan_finite(x: Self) -> Self {
        let (n, y) = reduce_pi_2(x);
        tan_core(y, (n & 1) != 0)
    }

    fn reduce_90_deg(x: Self) -> (u8, Self) {
        if x.abs() <= Self::from_uint(45) {
            return (0, x);
        }

        let x_abs = x.abs();
        let x_int = x_abs.round_ties_even();

        if let Some(xi) = x_int.to_uint(32) {
            let x_frac = x_abs - x_int;
            let xi = xi as u32;

            let mut n = (xi / 90) as u8;
            let mut rem = (xi % 90) as i8;
            if rem >= 45 {
                n = n.wrapping_add(1);
                rem -= 90;
            }

            let y = x_frac + Self::from_int(rem.into());

            if x.sign() {
                (n.wrapping_neg() & 3, -y)
            } else {
                (n & 3, y)
            }
        } else {
            // |x| = xm * 2^xe
            let xm = x.mant();
            let xe = x.exponent() - 23;

            // EXP2_MOD45[i] = mod(2^(i + 3), 45)
            const EXP2_MOD45: [u8; 12] = [1, 2, 4, 8, 16, 32, 19, 38, 31, 17, 34, 23];

            // t = xm * mod(2^xe, 45)
            debug_assert!(xe > 3);
            let index = ((xe - 3) % 12) as usize;
            let t = xm * u32::from(EXP2_MOD45[index]);
            // rem45 = mod(|x|, 45)
            let rem45 = (t % 45) as u16;
            // rem360 = mod(|x|, 360) = mod(|x|, 45) * 8
            let rem360 = rem45 * 8;

            let mut n = (rem360 / 90) as u8;
            let mut rem90 = (rem360 % 90) as i16;
            if rem90 >= 45 {
                n += 1;
                rem90 -= 90;
            }

            let y = Self::from_int(rem90.into());

            if x.sign() {
                (n.wrapping_neg() & 3, -y)
            } else {
                (n & 3, y)
            }
        }
    }

    fn sind_reduced(x: Self) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * FRAC_PI_180;
        sin_core(y)
    }

    fn cosd_reduced(x: Self) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * FRAC_PI_180;
        cos_core(y)
    }

    fn sind_cosd_reduced(x: Self) -> (Self, Self) {
        let y = SfpM64E16::from_ieee_float(x.0) * FRAC_PI_180;
        (sin_core(y), cos_core(y))
    }

    fn tand_reduced(x: Self, inv: bool) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * FRAC_PI_180;
        tan_core(y, inv)
    }

    fn sinpi_reduced(x: Self) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * PI;
        sin_core(y)
    }

    fn cospi_reduced(x: Self) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * PI;
        cos_core(y)
    }

    fn sinpi_cospi_reduced(x: Self) -> (Self, Self) {
        let y = SfpM64E16::from_ieee_float(x.0) * PI;
        (sin_core(y), cos_core(y))
    }

    fn tanpi_reduced(x: Self, inv: bool) -> Self {
        let y = SfpM64E16::from_ieee_float(x.0) * PI;
        tan_core(y, inv)
    }
}

/// Reduces the angle argument `x`, returning `(n, y)` such that:
/// * `|y| <= π/4`
/// * `0 <= n <= 3`
/// * `x = 2*π*M + π/2*n + y`
/// * `M` is an integer
fn reduce_pi_2(x: F32) -> (u8, SfpM64E16) {
    let x_orig = x;
    let x = SfpM64E16::from_ieee_float(x.0);

    if x.abs() <= FRAC_PI_4 {
        // reduction not needed
        return (0, x);
    }

    // GENERATE: reduce_pi_2::medium_consts SfpM32E16 SfpM64E16
    const FRAC_2_PI: SfpM64E16 = Sfp::newp(-1, 0xA2F9836E4E44152A); // 6.366197723675813431e-1
    const FRAC_PI_2_P0: SfpM32E16 = Sfp::newp(0, 0xC90FDAA2); // 1.57079633e0
    const FRAC_PI_2_P0EX: SfpM64E16 = Sfp::newp(-34, 0x85A308D313198A2E); // 6.077100506506192601e-11

    if let Some(n) = (x * FRAC_2_PI).to_int_round::<i32>() {
        let f_n = SfpM64E16::from_int(n);

        let r = x - f_n * SfpM64E16::from_sfp(FRAC_PI_2_P0);
        let y = r - f_n * FRAC_PI_2_P0EX;

        (n as u8 & 3, y)
    } else {
        let mant = x_orig.mant();
        let x_chunks = [mant];
        let e0 = x_orig.exponent() - 23;
        let jk = 4;

        let mut qp: [u64; 20] = [0; 20];
        let (ih, jz, n, qe) = reduce_pi_2_large(x_chunks.as_ref(), e0, jk, &mut qp);

        // compress qp into fw
        let mut fw = SfpM64E16::ZERO;
        for &qp_i in qp[..=jz].iter().rev() {
            fw = fw.scalbn(-24) + SfpM64E16::from_int(qp_i);
        }

        let y = fw.scalbn(qe).set_sign(ih != 0);

        if x.sign() {
            (n.wrapping_neg() & 3, -y)
        } else {
            (n & 3, y)
        }
    }
}

fn sin_core(x: SfpM64E16) -> F32 {
    // GENERATE: sin_poly SfpM64E16 6
    const K3: SfpM64E16 = Sfp::newn(-3, 0xAAAAAAAAAAAA4606); // -1.666666666666663175e-1
    const K5: SfpM64E16 = Sfp::newp(-7, 0x8888888887C2825A); // 8.333333333322340785e-3
    const K7: SfpM64E16 = Sfp::newn(-13, 0xD00D00CE060BE8CD); // -1.984126982974668876e-4
    const K9: SfpM64E16 = Sfp::newp(-19, 0xB8EF1AB95E459691); // 2.755731366981927738e-6
    const K11: SfpM64E16 = Sfp::newn(-26, 0xD72F3117C28C8DFD); // -2.505075452463560166e-8
    const K13: SfpM64E16 = Sfp::newp(-33, 0xAEC8E562550A787C); // 1.589658041882563595e-10

    let x2 = x.square();
    let x3 = x2 * x;

    let r = horner!(x3, x2, [K3, K5, K7, K9, K11, K13]) + x;

    F32(r.to_ieee_float())
}

fn cos_core(x: SfpM64E16) -> F32 {
    // GENERATE: cos_poly SfpM64E16 6
    const K4: SfpM64E16 = Sfp::newp(-5, 0xAAAAAAAAAAAA5C94); // 4.166666666666659894e-2
    const K6: SfpM64E16 = Sfp::newn(-10, 0xB60B60B60A8A817D); // -1.388888888887402541e-3
    const K8: SfpM64E16 = Sfp::newp(-16, 0xD00D00CE568B654F); // 2.480158728941762910e-5
    const K10: SfpM64E16 = Sfp::newn(-22, 0x93F27C0337817602); // -2.755731433286911134e-7
    const K12: SfpM64E16 = Sfp::newp(-29, 0x8F74F4B79AECD23E); // 2.087572052380019896e-9
    const K14: SfpM64E16 = Sfp::newn(-37, 0xC7D6A4454AD6D727); // -1.135950038085136403e-11

    let x2 = x.square();
    let x4 = x2.square();

    let r = horner!(x4, x2, [K4, K6, K8, K10, K12, K14]) - x2.scalbn(-1) + SfpM64E16::one();

    F32(r.to_ieee_float())
}

fn tan_core(x: SfpM64E16, inv: bool) -> F32 {
    // GENERATE: tan_poly SfpM64E16 8
    const K3: SfpM64E16 = Sfp::newp(-2, 0xAAAAAAAAAA789AF9); // 3.333333333332444066e-1
    const K5: SfpM64E16 = Sfp::newp(-3, 0x88888888DA02D51D); // 1.333333333518592245e-1
    const K7: SfpM64E16 = Sfp::newp(-5, 0xDD0DD08279C2A37F); // 5.396825265016561316e-2
    const K9: SfpM64E16 = Sfp::newp(-6, 0xB327BC90A46317AA); // 2.186953381636803555e-2
    const K11: SfpM64E16 = Sfp::newp(-7, 0x91338150181B7AEE); // 8.862377435777195622e-3
    const K13: SfpM64E16 = Sfp::newp(-9, 0xEC080F8D34B2DA62); // 3.601554676793834079e-3
    const K15: SfpM64E16 = Sfp::newp(-10, 0xB707864FE54377BA); // 1.396403451460883264e-3
    const K17: SfpM64E16 = Sfp::newp(-11, 0xCE0F37F1E06E5EF1); // 7.860544105543986292e-4

    // let y = 0.5 * x
    // tan(x) = 2 * tan(y) / (1 - tan(y)^2)

    let y = x.scalbn(-1);
    let y2 = y.square();
    let y3 = y2 * y;

    // t1 = tan(y)
    let t1 = horner!(y3, y2, [K3, K5, K7, K9, K11, K13, K15, K17]) + y;

    // t2 = 2 * tan(y)
    let t2 = t1.scalbn(1);

    // t3 = 1 - tan(y)^2
    let t3 = SfpM64E16::one() - t1.square();

    let r = if inv {
        // -1/tan(x) = -t3 / t2
        -(t3 / t2)
    } else {
        // tan(x) = t2 / t3
        t2 / t3
    };

    F32(r.to_ieee_float())
}

impl crate::math::Trigonometric for F32 {
    fn sin(self) -> Self {
        crate::generic::sin(self)
    }

    fn cos(self) -> Self {
        crate::generic::cos(self)
    }

    fn sin_cos(self) -> (Self, Self) {
        crate::generic::sin_cos(self)
    }

    fn tan(self) -> Self {
        crate::generic::tan(self)
    }

    fn sind(self) -> Self {
        crate::generic::sind(self)
    }

    fn cosd(self) -> Self {
        crate::generic::cosd(self)
    }

    fn sind_cosd(self) -> (Self, Self) {
        crate::generic::sind_cosd(self)
    }

    fn tand(self) -> Self {
        crate::generic::tand(self)
    }

    fn sinpi(self) -> Self {
        crate::generic::sinpi(self)
    }

    fn cospi(self) -> Self {
        crate::generic::cospi(self)
    }

    fn sinpi_cospi(self) -> (Self, Self) {
        crate::generic::sinpi_cospi(self)
    }

    fn tanpi(self) -> Self {
        crate::generic::tanpi(self)
    }
}
