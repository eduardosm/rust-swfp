use crate::simple_fp::{Sfp, SfpM16E8, SfpM32E16};
use crate::traits::FloatImpExt as _;
use crate::{F16, Float as _};

// GENERATE: consts SfpM32E16 PI FRAC_PI_4 FRAC_PI_180
const PI: SfpM32E16 = Sfp::newp(1, 0xC90FDAA2); // 3.14159265e0
const FRAC_PI_4: SfpM32E16 = Sfp::newp(-1, 0xC90FDAA2); // 7.85398163e-1
const FRAC_PI_180: SfpM32E16 = Sfp::newp(-6, 0x8EFA3513); // 1.74532925e-2

impl crate::generic::Trigonometric for F16 {
    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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
        let x_frac = x_abs - x_int;

        let xi = x_int.to_uint(32).unwrap() as u32;

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
    }

    #[inline]
    fn sind_reduced(x: Self) -> Self {
        let y = SfpM32E16::from_ieee_float(x.0) * FRAC_PI_180;
        sin_core(y)
    }

    #[inline]
    fn cosd_reduced(x: Self) -> Self {
        let y = SfpM32E16::from_ieee_float(x.0) * FRAC_PI_180;
        cos_core(y)
    }

    #[inline]
    fn sind_cosd_reduced(x: Self) -> (Self, Self) {
        let y = SfpM32E16::from_ieee_float(x.0) * FRAC_PI_180;
        (sin_core(y), cos_core(y))
    }

    #[inline]
    fn tand_reduced(x: Self, inv: bool) -> Self {
        if !inv && x.abs().bitwise_eq(Self::from_bits(0x3ADB)) {
            // Hard to round case
            return Self::from_bits(0x23A9).set_sign(x.sign());
        }

        let y = SfpM32E16::from_ieee_float(x.0) * FRAC_PI_180;
        tan_core(y, inv)
    }

    #[inline]
    fn sinpi_reduced(x: Self) -> Self {
        let y = SfpM32E16::from_ieee_float(x.0) * PI;
        sin_core(y)
    }

    #[inline]
    fn cospi_reduced(x: Self) -> Self {
        let y = SfpM32E16::from_ieee_float(x.0) * PI;
        cos_core(y)
    }

    #[inline]
    fn sinpi_cospi_reduced(x: Self) -> (Self, Self) {
        let y = SfpM32E16::from_ieee_float(x.0) * PI;
        (sin_core(y), cos_core(y))
    }

    #[inline]
    fn tanpi_reduced(x: Self, inv: bool) -> Self {
        if !inv && x.abs().bitwise_eq(Self::from_bits(0x1CE0)) {
            // Hard to round case
            return Self::from_bits(0x23A9).set_sign(x.sign());
        }

        let y = SfpM32E16::from_ieee_float(x.0) * PI;
        tan_core(y, inv)
    }
}

/// Reduces the angle argument `x`, returning `(n, y)` such that:
/// * `|y| <= π/4`
/// * `0 <= n <= 3`
/// * `x = 2*π*M + π/2*n + y`
/// * `M` is an integer
fn reduce_pi_2(x: F16) -> (u8, SfpM32E16) {
    let x = SfpM32E16::from_ieee_float(x.0);

    if x.abs() <= FRAC_PI_4 {
        // reduction not needed
        return (0, x);
    }

    // GENERATE: reduce_pi_2::medium_consts SfpM16E8 SfpM32E16
    const FRAC_2_PI: SfpM32E16 = Sfp::newp(-1, 0xA2F9836E); // 6.36619772e-1
    const FRAC_PI_2_P0: SfpM16E8 = Sfp::newp(0, 0xC90F); // 1.571e0
    const FRAC_PI_2_P0EX: SfpM32E16 = Sfp::newp(-16, 0xDAA22169); // 2.60631230e-5

    let n: i32 = (x * FRAC_2_PI).to_int_round().unwrap();
    let f_n = SfpM32E16::from_int(n);

    let r = x - f_n * SfpM32E16::from_sfp(FRAC_PI_2_P0);
    let y = r - f_n * FRAC_PI_2_P0EX;

    (n as u8 & 3, y)
}

fn sin_core(x: SfpM32E16) -> F16 {
    // GENERATE: sin_poly SfpM32E16 3
    const K3: SfpM32E16 = Sfp::newn(-3, 0xAAAAA2C3); // -1.66666549e-1
    const K5: SfpM32E16 = Sfp::newp(-7, 0x8883AC60); // 8.33217462e-3
    const K7: SfpM32E16 = Sfp::newn(-13, 0xCCA60716); // -1.95168062e-4

    let x2 = x.square();
    let x3 = x2 * x;

    let r = horner!(x3, x2, [K3, K5, K7]) + x;

    F16(r.to_ieee_float())
}

fn cos_core(x: SfpM32E16) -> F16 {
    // GENERATE: cos_poly SfpM32E16 3
    const K4: SfpM32E16 = Sfp::newp(-5, 0xAAAAA554); // 4.16666468e-2
    const K6: SfpM32E16 = Sfp::newn(-10, 0xB60641DF); // -1.38873629e-3
    const K8: SfpM32E16 = Sfp::newp(-16, 0xCCFFFCF0); // 2.44378988e-5

    let x2 = x.square();
    let x4 = x2.square();

    let r = horner!(x4, x2, [K4, K6, K8]) - x2.scalbn(-1) + SfpM32E16::one();

    F16(r.to_ieee_float())
}

fn tan_core(x: SfpM32E16, inv: bool) -> F16 {
    // GENERATE: tan_poly SfpM32E16 4
    const K3: SfpM32E16 = Sfp::newp(-2, 0xAAAA9C92); // 3.33332913e-1
    const K5: SfpM32E16 = Sfp::newp(-3, 0x888F5D40); // 1.33359391e-1
    const K7: SfpM32E16 = Sfp::newp(-5, 0xDB0201E5); // 5.34687113e-2
    const K9: SfpM32E16 = Sfp::newp(-6, 0xD191B08D); // 2.55821656e-2

    // let y = 0.5 * x
    // tan(x) = 2 * tan(y) / (1 - tan(y)^2)

    let y = x.scalbn(-1);
    let y2 = y.square();
    let y3 = y2 * y;

    // t1 = tan(y)
    let t1 = horner!(y3, y2, [K3, K5, K7, K9]) + y;

    // t2 = 2 * tan(y)
    let t2 = t1.scalbn(1);

    // t3 = 1 - tan(y)^2
    let t3 = SfpM32E16::one() - t1.square();

    let r = if inv {
        // -1/tan(x) = -t3 / t2
        -(t3 / t2)
    } else {
        // tan(x) = t2 / t3
        t2 / t3
    };

    F16(r.to_ieee_float())
}

impl crate::math::Trigonometric for F16 {
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
