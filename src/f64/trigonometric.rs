use crate::math_aux::reduce_pi_2_large;
use crate::simple_fp::{Sfp, SfpM64E16, SfpM128E16};
use crate::traits::FloatImpExt as _;
use crate::{F64, Float as _};

// GENERATE: consts SfpM128E16 PI FRAC_PI_4 FRAC_PI_180
const PI: SfpM128E16 = Sfp::newp(1, 0xC90FDAA22168C234C4C6628B80DC1CD1); // 3.1415926535897932384626433832795028842e0
const FRAC_PI_4: SfpM128E16 = Sfp::newp(-1, 0xC90FDAA22168C234C4C6628B80DC1CD1); // 7.8539816339744830961566084581987572105e-1
const FRAC_PI_180: SfpM128E16 = Sfp::newp(-6, 0x8EFA351294E9C8AE0EC5F66E9485C4D9); // 1.7453292519943295769236907684886127134e-2

impl crate::generic::Trigonometric for F64 {
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

        if let Some(xi) = x_int.to_int(64) {
            let x_frac = x_abs - x_int;
            let xi = xi as u64;

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
            let xe = x.exponent() - 52;

            // EXP2_MOD45[i] = mod(2^(i + 3), 45)
            const EXP2_MOD45: [u8; 12] = [1, 2, 4, 8, 16, 32, 19, 38, 31, 17, 34, 23];

            // t = xm * mod(2^xe, 45)
            debug_assert!(xe > 3);
            let index = ((xe - 3) % 12) as usize;
            let t = xm * u64::from(EXP2_MOD45[index]);
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
        let y = SfpM128E16::from_ieee_float(x.0) * FRAC_PI_180;
        sin_core(y)
    }

    fn cosd_reduced(x: Self) -> Self {
        let y = SfpM128E16::from_ieee_float(x.0) * FRAC_PI_180;
        cos_core(y)
    }

    fn sind_cosd_reduced(x: Self) -> (Self, Self) {
        let y = SfpM128E16::from_ieee_float(x.0) * FRAC_PI_180;
        (sin_core(y), cos_core(y))
    }

    fn tand_reduced(x: Self, inv: bool) -> Self {
        let y = SfpM128E16::from_ieee_float(x.0) * FRAC_PI_180;
        tan_core(y, inv)
    }

    fn sinpi_reduced(x: Self) -> Self {
        let y = SfpM128E16::from_ieee_float(x.0) * PI;
        sin_core(y)
    }

    fn cospi_reduced(x: Self) -> Self {
        let y = SfpM128E16::from_ieee_float(x.0) * PI;
        cos_core(y)
    }

    fn sinpi_cospi_reduced(x: Self) -> (Self, Self) {
        let y = SfpM128E16::from_ieee_float(x.0) * PI;
        (sin_core(y), cos_core(y))
    }

    fn tanpi_reduced(x: Self, inv: bool) -> Self {
        let y = SfpM128E16::from_ieee_float(x.0) * PI;
        tan_core(y, inv)
    }
}

/// Reduces the angle argument `x`, returning `(n, y)` such that:
/// * `|y| <= π/4`
/// * `0 <= n <= 3`
/// * `x = 2*π*M + π/2*n + y`
/// * `M` is an integer
fn reduce_pi_2(x: F64) -> (u8, SfpM128E16) {
    let x_orig = x;
    let x = SfpM128E16::from_ieee_float(x.0);

    if x.abs() <= FRAC_PI_4 {
        // reduction not needed
        return (0, x);
    }

    // GENERATE: reduce_pi_2::medium_consts SfpM64E16 SfpM128E16
    const FRAC_2_PI: SfpM128E16 = Sfp::newp(-1, 0xA2F9836E4E441529FC2757D1F534DDC1); // 6.3661977236758134307553505349005744814e-1
    const FRAC_PI_2_P0: SfpM64E16 = Sfp::newp(0, 0xC90FDAA22168C234); // 1.570796326794896619e0
    const FRAC_PI_2_P0EX: SfpM128E16 = Sfp::newp(-64, 0xC4C6628B80DC1CD129024E088A67CC74); // 8.3337429185208783282958644685340386073e-20

    if let Some(n) = (x * FRAC_2_PI).to_int_round::<i64>() {
        let f_n = SfpM128E16::from_int(n);

        let r = x - f_n * SfpM128E16::from_sfp(FRAC_PI_2_P0);
        let y = r - f_n * FRAC_PI_2_P0EX;

        (n as u8 & 3, y)
    } else {
        let mant = x_orig.mant();
        let x_chunks = [
            ((mant >> 29) as u32),
            (((mant >> 5) & 0x00FF_FFFF) as u32),
            (((mant << 19) & 0x00FF_FFFF) as u32),
        ];
        let e0 = x_orig.exponent() - 23;
        let jk = 6;

        let mut qp: [u64; 20] = [0; 20];
        let (ih, jz, n, qe) = reduce_pi_2_large(x_chunks.as_ref(), e0, jk, &mut qp);

        // compress qp into fw
        let mut fw = SfpM128E16::ZERO;
        for &qp_i in qp[..=jz].iter().rev() {
            fw = fw.scalbn(-24) + SfpM128E16::from_int(qp_i);
        }

        let y = fw.scalbn(qe).set_sign(ih != 0);

        if x.sign() {
            (n.wrapping_neg() & 3, -y)
        } else {
            (n & 3, y)
        }
    }
}

fn sin_core(x: SfpM128E16) -> F64 {
    // GENERATE: sin_poly SfpM128E16 12
    const K3: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAA095); // -1.6666666666666666666666666666666666477e-1
    const K5: SfpM128E16 = Sfp::newp(-7, 0x888888888888888888888888884312D5); // 8.3333333333333333333333333333331243104e-3
    const K7: SfpM128E16 = Sfp::newn(-13, 0xD00D00D00D00D00D00D00CFE34ADD7F4); // -1.9841269841269841269841269840466521786e-4
    const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB6399C7D560E37E2D703F684); // 2.7557319223985890652557317675701788376e-6
    const K11: SfpM128E16 = Sfp::newn(-26, 0xD7322B3FAA271C7F39F8375927FB460A); // -2.5052108385441718775050359794303019166e-8
    const K13: SfpM128E16 = Sfp::newp(-33, 0xB092309D43684BE41D2063E8FDFFFFF2); // 1.6059043836821614598135261708205578121e-10
    const K15: SfpM128E16 = Sfp::newn(-41, 0xD73F9F399DC0F3C8A5382D2EBAD534E9); // -7.6471637318198158733509342540544948335e-13
    const K17: SfpM128E16 = Sfp::newp(-49, 0xCA963B81855AB784DFDB8A54B047CBE7); // 2.8114572543453237536015460464151280112e-15
    const K19: SfpM128E16 = Sfp::newn(-57, 0x97A4DA33E716E0AB32DA40CED4785E5E); // -8.2206352461831668668075107087988496929e-18
    const K21: SfpM128E16 = Sfp::newp(-66, 0xB8DC774D67E80248D537F9F174AEB443); // 1.9572940397598479367549379968975283961e-20
    const K23: SfpM128E16 = Sfp::newn(-75, 0xBB0CD36AD4E168D401FB4B7D869C858E); // -3.8681054275062103917014958489518849502e-23
    const K25: SfpM128E16 = Sfp::newp(-84, 0x9EB627394AC9D8D5AB309671EE9F8F88); // 6.4103078893574637648930111673652337563e-26

    let x2 = x.square();
    let x3 = x2 * x;

    let r = horner!(
        x3,
        x2,
        [K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25]
    ) + x;

    F64(r.to_ieee_float())
}

fn cos_core(x: SfpM128E16) -> F64 {
    // GENERATE: cos_poly SfpM128E16 11
    const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAAAAAAAAA0F65E5); // 4.1666666666666666666666666666664797688e-2
    const K6: SfpM128E16 = Sfp::newn(-10, 0xB60B60B60B60B60B60B60B5C2204BBA9); // -1.3888888888888888888888888887760268947e-3
    const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00D00D00D00CFF1BBE5BF504A); // 2.4801587301587301587301584612636558537e-5
    const K10: SfpM128E16 = Sfp::newn(-22, 0x93F27DBBC4FAE39777B4146DD696E926); // -2.7557319223985890652553874501256576721e-7
    const K12: SfpM128E16 = Sfp::newp(-29, 0x8F76C77FC6C4BDA8D14E78DBB04DE947); // 2.0876756987868098976515936415241471781e-9
    const K14: SfpM128E16 = Sfp::newn(-37, 0xC9CBA54603E4E23AC7B4E636305D74E9); // -1.1470745597729723341975546123700489052e-11
    const K16: SfpM128E16 = Sfp::newp(-45, 0xD73F9F399DA9B8F16401A5B6204BC373); // 4.7794773323869157978196021092618545502e-14
    const K18: SfpM128E16 = Sfp::newn(-53, 0xB413C31D95F8B0AA663E2C12A9BC8EE3); // -1.5619206967496648328777608209892745780e-16
    const K20: SfpM128E16 = Sfp::newp(-62, 0xF2A15C7877388B4E7CEDA46BEF0B34D1); // 4.1103174540823860184683143373280703118e-19
    const K22: SfpM128E16 = Sfp::newn(-70, 0x8671248E5B13160E2E3D27BEA89008E6); // -8.8966228941542164611491475904841720625e-22
    const K24: SfpM128E16 = Sfp::newp(-80, 0xF7E62AEA82A6BB0B72698B4D47503BDE); // 1.6020103363982032604999202009517097012e-24

    let x2 = x.square();
    let x4 = x2.square();

    let r = horner!(x4, x2, [K4, K6, K8, K10, K12, K14, K16, K18, K20, K22, K24]) - x2.scalbn(-1)
        + SfpM128E16::one();

    F64(r.to_ieee_float())
}

fn tan_core(x: SfpM128E16, inv: bool) -> F64 {
    // GENERATE: tan_poly SfpM128E16 19
    const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAAAAAAAAB016214); // 3.3333333333333333333333333333334168382e-1
    const K5: SfpM128E16 = Sfp::newp(-3, 0x888888888888888888888885C588E5AF); // 1.3333333333333333333333333332461890304e-1
    const K7: SfpM128E16 = Sfp::newp(-5, 0xDD0DD0DD0DD0DD0DD0DD1DAF3BC81407); // 5.3968253968253968253968257172898867177e-2
    const K9: SfpM128E16 = Sfp::newp(-6, 0xB327A4416087CF996B468B7C0A20F11A); // 2.1869488536155202821868886848717747242e-2
    const K11: SfpM128E16 = Sfp::newp(-7, 0x91371AAF3611E47AEF193FF32175E6BD); // 8.8632355299021965689312082900180700506e-3
    const K13: SfpM128E16 = Sfp::newp(-9, 0xEB69E870ABEEFD97F3FE05C1DAD6FEDD); // 3.5921280365724810118548157763739318336e-3
    const K15: SfpM128E16 = Sfp::newp(-10, 0xBED1B2295BAF1F7B61830CA9CD9E0ED4); // 1.4558343870513185331932533126008545389e-3
    const K17: SfpM128E16 = Sfp::newp(-11, 0x9AAC12401B373A36A6C125E777B09535); // 5.9002744094557589350923798297698199861e-4
    const K19: SfpM128E16 = Sfp::newp(-13, 0xFABEBB9A69FEBCF0D32EB5DA6EA2B133); // 2.3912911424384010639224223842747667818e-4
    const K21: SfpM128E16 = Sfp::newp(-14, 0xCB3F0C57AD24432B5C565397268AF835); // 9.6915379563038600565879159895474444418e-5
    const K23: SfpM128E16 = Sfp::newp(-15, 0xA4BEC77C76BF95C4EDEF10F5BB2FE36B); // 3.9278323988353646081665925288699405631e-5
    const K25: SfpM128E16 = Sfp::newp(-16, 0x858995DC09BF9CC2BE62AB570DB89DB0); // 1.5918903699500788365937053945390907305e-5
    const K27: SfpM128E16 = Sfp::newp(-18, 0xD87BB5291A4FC7795B04E57113E2A61B); // 6.4517031026106150085218208859885410067e-6
    const K29: SfpM128E16 = Sfp::newp(-19, 0xAF778E3AF8474308C158BDB319A4DD0C); // 2.6146622600096923690432596292329466381e-6
    const K31: SfpM128E16 = Sfp::newp(-20, 0x8E525B2DE6F908792C539791826EFE81); // 1.0603793247475567811506561936199497499e-6
    const K33: SfpM128E16 = Sfp::newp(-22, 0xE5031A4253F037BDE00E1F8675FB503D); // 4.2656831337505132027301605177115708543e-7
    const K35: SfpM128E16 = Sfp::newp(-23, 0xC504DE448868DF637BB9DF6F85F87E0F); // 1.8348825772856182264930729103233281847e-7
    const K37: SfpM128E16 = Sfp::newp(-25, 0xD75AB16CC8AFF087735313FEE4B6839A); // 5.0141073248743043066190488022645861405e-8
    const K39: SfpM128E16 = Sfp::newp(-25, 0xE90DF0FC139637C79CDB4DC86F1D151D); // 5.4262219552021913231015737705707455342e-8

    // let y = 0.5 * x
    // tan(x) = 2 * tan(y) / (1 - tan(y)^2)

    let y = x.scalbn(-1);
    let y2 = y.square();
    let y3 = y2 * y;

    // t1 = tan(y)
    let t1 = horner!(
        y3,
        y2,
        [
            K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25, K27, K29, K31, K33, K35, K37,
            K39
        ]
    ) + y;

    // t2 = 2 * tan(y)
    let t2 = t1.scalbn(1);

    // t3 = 1 - tan(y)^2
    let t3 = SfpM128E16::one() - t1.square();

    let r = if inv {
        // -1/tan(x) = -t3 / t2
        -(t3 / t2)
    } else {
        // tan(x) = t2 / t3
        t2 / t3
    };

    F64(r.to_ieee_float())
}

impl crate::math::Trigonometric for F64 {
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
