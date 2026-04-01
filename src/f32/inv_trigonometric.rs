use crate::F32;
use crate::simple_fp::{Sfp, SfpM64E16, SfpM128E16};

// GENERATE: consts SfpM64E16 PI FRAC_PI_2 FRAC_180_PI FRAC_1_PI
const PI: SfpM64E16 = Sfp::newp(1, 0xC90FDAA22168C235); // 3.141592653589793238e0
const FRAC_PI_2: SfpM64E16 = Sfp::newp(0, 0xC90FDAA22168C235); // 1.570796326794896619e0
const FRAC_180_PI: SfpM64E16 = Sfp::newp(5, 0xE52EE0D31E0FBDC3); // 5.729577951308232088e1
const FRAC_1_PI: SfpM64E16 = Sfp::newp(-2, 0xA2F9836E4E44152A); // 3.183098861837906715e-1

impl crate::generic::InvTrigonometric for F32 {
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
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = asin_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acos_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = acos_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan_core(x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2_finite(y: Self, x: Self) -> Self {
        let y = SfpM128E16::from_ieee_float(y.0);
        let x = SfpM128E16::from_ieee_float(x.0);
        let r = atan2_core(y, x);
        Self(r.to_ieee_float())
    }

    #[inline]
    fn asind_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = asin_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acosd_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = acos_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atand_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan_core(x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2d_finite(y: Self, x: Self) -> Self {
        // GENERATE: consts SfpM128E16 FRAC_180_PI
        const FRAC_180_PI: SfpM128E16 = Sfp::newp(5, 0xE52EE0D31E0FBDC30A97537F40D257D7); // 5.7295779513082320876798154814105170332e1

        let y = SfpM128E16::from_ieee_float(y.0);
        let x = SfpM128E16::from_ieee_float(x.0);
        let r = atan2_core(y, x) * FRAC_180_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn asinpi_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = asin_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn acospi_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = acos_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atanpi_finite(x: Self) -> Self {
        let x = SfpM64E16::from_ieee_float(x.0);
        let r = atan_core(x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }

    #[inline]
    fn atan2pi_finite(y: Self, x: Self) -> Self {
        // GENERATE: consts SfpM128E16 FRAC_1_PI
        const FRAC_1_PI: SfpM128E16 = Sfp::newp(-2, 0xA2F9836E4E441529FC2757D1F534DDC1); // 3.1830988618379067153776752674502872407e-1

        let y = SfpM128E16::from_ieee_float(y.0);
        let x = SfpM128E16::from_ieee_float(x.0);
        let r = atan2_core(y, x) * FRAC_1_PI;
        Self(r.to_ieee_float())
    }
}

fn asin_core(x: SfpM64E16) -> SfpM64E16 {
    #[inline]
    fn asin_poly(x: SfpM64E16, x2: SfpM64E16, x3: SfpM64E16) -> SfpM64E16 {
        // GENERATE: asin_poly SfpM64E16 12
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAAAA7C5BF3); // 1.666666666666255374e-1
        const K5: SfpM64E16 = Sfp::newp(-4, 0x99999999E3E0A190); // 7.500000000844437014e-2
        const K7: SfpM64E16 = Sfp::newp(-5, 0xB6DB6D89E124CD49); // 4.464285648834582829e-2
        const K9: SfpM64E16 = Sfp::newp(-6, 0xF8E39C7FC5A5508E); // 3.038197103721943678e-2
        const K11: SfpM64E16 = Sfp::newp(-6, 0xB74474493B8B074B); // 2.237150871393234525e-2
        const K13: SfpM64E16 = Sfp::newp(-6, 0x8E3CF8C91943F435); // 1.736305800367476437e-2
        const K15: SfpM64E16 = Sfp::newp(-7, 0xE3011F19C1FCE364); // 1.385524785203511790e-2
        const K17: SfpM64E16 = Sfp::newp(-7, 0xCA55DBA0EEEBAD7A); // 1.234957168683934890e-2
        const K19: SfpM64E16 = Sfp::newp(-8, 0xBE465FF03552B1C8); // 5.806729168777188656e-3
        const K21: SfpM64E16 = Sfp::newp(-6, 0xAF0A67DABE503497); // 2.136726650310019131e-2
        const K23: SfpM64E16 = Sfp::newn(-6, 0x9B13313B5DD7F3FD); // -1.893005004506686551e-2
        const K25: SfpM64E16 = Sfp::newp(-5, 0x89DFB78A3D294072); // 3.366061873732412207e-2

        x + horner!(
            x3,
            x2,
            [K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25]
        )
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
        let y2 = (SfpM64E16::one() - x.abs()).scalbn(-1);
        let y = y2.sqrt();
        let y3 = y2 * y;

        // t1 = asin(y)
        let t1 = asin_poly(y, y2, y3);

        // t3 = π/2 - 2 * asin(sqrt((1 - |x|) / 2)) = π/2 - 2 * t1
        let t3 = FRAC_PI_2 - t1.scalbn(1);

        t3.set_sign(x.sign())
    }
}

fn acos_core(x: SfpM64E16) -> SfpM64E16 {
    #[inline]
    fn asin_poly(x: SfpM64E16, x2: SfpM64E16, x3: SfpM64E16) -> SfpM64E16 {
        // GENERATE: asin_poly SfpM64E16 10
        const K3: SfpM64E16 = Sfp::newp(-3, 0xAAAAAAAA8E9D9AE5); // 1.666666666602886027e-1
        const K5: SfpM64E16 = Sfp::newp(-4, 0x999999B9FE084B5E); // 7.500000094274041658e-2
        const K7: SfpM64E16 = Sfp::newp(-5, 0xB6DB5FB9D1FDD002); // 4.464280503191227212e-2
        const K9: SfpM64E16 = Sfp::newp(-6, 0xF8E6AC8AF8B2C3DC); // 3.038343143250296808e-2
        const K11: SfpM64E16 = Sfp::newp(-6, 0xB7117720C9756FF8); // 2.234719531251865703e-2
        const K13: SfpM64E16 = Sfp::newp(-6, 0x904D7E1EB35B73C1); // 1.761507637784681137e-2
        const K15: SfpM64E16 = Sfp::newp(-7, 0xC7E27788D5034D63); // 1.219999001780554550e-2
        const K17: SfpM64E16 = Sfp::newp(-6, 0x9C3CE74EA83B78F7); // 1.907200982282780372e-2
        const K19: SfpM64E16 = Sfp::newn(-7, 0x9A04FA7708F79548); // -9.400601000506968626e-3
        const K21: SfpM64E16 = Sfp::newp(-5, 0x87B07A1252A6C3C7); // 3.312728580673657602e-2

        x + horner!(x3, x2, [K3, K5, K7, K9, K11, K13, K15, K17, K19, K21])
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
        let y2 = (SfpM64E16::one() - x.abs()).scalbn(-1);
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

fn atan_core(x: SfpM64E16) -> SfpM64E16 {
    #[inline]
    fn atan_poly(x: SfpM64E16, x2: SfpM64E16, x3: SfpM64E16) -> SfpM64E16 {
        // GENERATE: atan_poly SfpM64E16 18 -0.00001 1.0
        const K3: SfpM64E16 = Sfp::newn(-2, 0xAAAAAAAAAA10EB23); // -3.333333333330602217e-1
        const K5: SfpM64E16 = Sfp::newp(-3, 0xCCCCCCCC43752DC0); // 1.999999999687719831e-1
        const K7: SfpM64E16 = Sfp::newn(-3, 0x9249247AC919D230); // -1.428571414892532774e-1
        const K9: SfpM64E16 = Sfp::newp(-4, 0xE38E34932251218A); // 1.111110789682148741e-1
        const K11: SfpM64E16 = Sfp::newn(-4, 0xBA2E4C92261F9ADE); // -9.090862103472812314e-2
        const K13: SfpM64E16 = Sfp::newp(-4, 0x9D8768829B76B616); // 7.691842697698390520e-2
        const K15: SfpM64E16 = Sfp::newn(-4, 0x8877497FF44472AA); // -6.663377210364471633e-2
        const K17: SfpM64E16 = Sfp::newp(-5, 0xF03BD34859745C8E); // 5.865080387374374239e-2
        const K19: SfpM64E16 = Sfp::newn(-5, 0xD4BEB95630F48C2D); // -5.193970105306989717e-2
        const K21: SfpM64E16 = Sfp::newp(-5, 0xBA348D4756730C00); // 4.546027361848526890e-2
        const K23: SfpM64E16 = Sfp::newn(-5, 0x9C3A362A476F9F63); // -3.814145239126878697e-2
        const K25: SfpM64E16 = Sfp::newp(-6, 0xF0B523B175383017); // 2.938324900939106453e-2
        const K27: SfpM64E16 = Sfp::newn(-6, 0xA1D73736D4968245); // -1.975594314594070298e-2
        const K29: SfpM64E16 = Sfp::newp(-7, 0xB3D44F80A87D5B44); // 1.097591175005859546e-2
        const K31: SfpM64E16 = Sfp::newn(-8, 0x9B64503BF752F5AE); // -4.742182900310780705e-3
        const K33: SfpM64E16 = Sfp::newp(-10, 0xC136EE50BDA7664C); // 1.474110213503461439e-3
        const K35: SfpM64E16 = Sfp::newn(-12, 0x9868F0A119BC1D93); // -2.906988558038991261e-4
        const K37: SfpM64E16 = Sfp::newp(-16, 0xE3CB9E00DFE01C62); // 2.715532537589036658e-5

        x + horner!(
            x3,
            x2,
            [
                K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25, K27, K29, K31, K33, K35,
                K37
            ]
        )
    }

    if x.abs() <= SfpM64E16::one() {
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

fn atan2_core(mut n: SfpM128E16, mut d: SfpM128E16) -> SfpM128E16 {
    // GENERATE: consts SfpM128E16 FRAC_PI_2
    const FRAC_PI_2: SfpM128E16 = Sfp::newp(0, 0xC90FDAA22168C234C4C6628B80DC1CD1); // 1.5707963267948966192313216916397514421e0

    #[inline]
    fn atan_poly(x: SfpM128E16, x2: SfpM128E16, x3: SfpM128E16) -> SfpM128E16 {
        // GENERATE: atan_poly SfpM128E16 27 -0.00001 1.0
        const K3: SfpM128E16 = Sfp::newn(-2, 0xAAAAAAAAAAAAAAA809E99158EC7DE019); // -3.3333333333333333326210270192931234891e-1
        const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCC7D4D123AA3D4A0AC476); // 1.9999999999999998276141517897779467271e-1
        const K7: SfpM128E16 = Sfp::newn(-3, 0x924924924922C3908E498536E7518E7E); // -1.4285714285714125175674436216949937683e-1
        const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E38E38D830FA7FA506491D7D70AEF); // 1.1111111111103036337727785312099415922e-1
        const K11: SfpM128E16 = Sfp::newn(-4, 0xBA2E8BA2D24C732FC078D9A102BB708C); // -9.0909090906541067921236150148440099657e-2
        const K13: SfpM128E16 = Sfp::newp(-4, 0x9D89D89BA4D1A83AB473CD915314C2E2); // 7.6923076867935716703146537729951875414e-2
        const K15: SfpM128E16 = Sfp::newn(-4, 0x8888886AD54BE62256F36D78622A7DEE); // -6.6666665802278712398934559228847771749e-2
        const K17: SfpM128E16 = Sfp::newp(-5, 0xF0F0EE326B1499B6BD071B7DAAC3E200); // 5.8823519188711115389582082991741886088e-2
        const K19: SfpM128E16 = Sfp::newn(-5, 0xD7941CACA94809022A9DB7AF9F0498A4); // -5.2631484994508434025319342159506197252e-2
        const K21: SfpM128E16 = Sfp::newp(-5, 0xC30B788D0EA9BBEC27FEA5B91D2AFD2C); // 4.7618361379969827778470678937064103721e-2
        const K23: SfpM128E16 = Sfp::newn(-5, 0xB21202734D998AD7997EBC4EF9B780FD); // -4.3474206516157197189251496952614732901e-2
        const K25: SfpM128E16 = Sfp::newp(-5, 0xA3C26FBD6C5C2FC821FE8111AD64725B); // 3.9980350956077478718494249620695177255e-2
        const K27: SfpM128E16 = Sfp::newn(-5, 0x97614D16324B87B709AE22C8ED9E2348); // -3.6958027954045464843854192784430188947e-2
        const K29: SfpM128E16 = Sfp::newp(-5, 0x8C26C8CAAEAA11227ED36B8ADB1B92C8); // 3.4216675131498523339558967757802697230e-2
        const K31: SfpM128E16 = Sfp::newn(-5, 0x8107B71BA45C9E6FC073BF1F08180B9B); // -3.1501498475584059664788367145977706292e-2
        const K33: SfpM128E16 = Sfp::newp(-6, 0xE941DD4B20B6C948079DA989070E77DC); // 2.8473789418987792282959764913020183559e-2
        const K35: SfpM128E16 = Sfp::newn(-6, 0xCB091CC0658A28717B272A5572D9982A); // -2.4784618525857799784704444319963506491e-2
        const K37: SfpM128E16 = Sfp::newp(-6, 0xA60483F3BB66C4B9B59546399BEE8E91); // 2.0265825003531331256334443769123501467e-2
        const K39: SfpM128E16 = Sfp::newn(-7, 0xF808C43C8964613C679FA7B32F9EA5B0); // -1.5138808858088624855084624851532137976e-2
        const K41: SfpM128E16 = Sfp::newp(-7, 0xA477CBC648AE890C6D1A85DD2A444142); // 1.0038327215748406881271534450738258923e-2
        const K43: SfpM128E16 = Sfp::newn(-8, 0xBC0D5C0616B838CFAE396231E01F3D4E); // -5.7388972601779614768056610114238619639e-3
        const K45: SfpM128E16 = Sfp::newp(-9, 0xB3CA69B6AE9EC21EC6875E60628CF328); // 2.7433879937981236790969356774731841020e-3
        const K47: SfpM128E16 = Sfp::newn(-10, 0x8AC1E6BE26FD91D5AB9071F1AE56E717); // -1.0586351555286031116589896537240521124e-3
        const K49: SfpM128E16 = Sfp::newp(-12, 0xA533916C8B414B536F36DBF389645F1E); // 3.1509673641994198317928668171148100005e-4
        const K51: SfpM128E16 = Sfp::newn(-14, 0x8DCF5B775DEBC60D79BDF29114D1D183); // -6.7620272357285070662751283036112589324e-5
        const K53: SfpM128E16 = Sfp::newp(-17, 0x9BC4BD19DA4F357E3A4A3482B0250940); // 9.2845267326878340439996365004139261494e-6
        const K55: SfpM128E16 = Sfp::newn(-21, 0xA412D076990EFA211B3E413D960BD68D); // -6.1122139318700802899459070678938046521e-7

        x + horner!(
            x3,
            x2,
            [
                K3, K5, K7, K9, K11, K13, K15, K17, K19, K21, K23, K25, K27, K29, K31, K33, K35,
                K37, K39, K41, K43, K45, K47, K49, K51, K53, K55
            ]
        )
    }

    let ysgn = n.sign();
    let xsgn = d.sign();

    let mut off = SfpM128E16::ZERO;
    if xsgn {
        off = SfpM128E16::two().set_sign(ysgn);
    }
    if n.abs() > d.abs() {
        core::mem::swap(&mut n, &mut d);
        n = -n;
        off = off + SfpM128E16::one().set_sign(ysgn ^ xsgn);
    }

    let z = n / d;
    if off.is_zero() && z.exponent() <= -32 {
        // atan2(y, x) ~= y/x = n/d
        // Avoid falsely-tied roundings
        z.next_mant_down()
    } else {
        // atan2(y, x) = atan(n/d) + off * π/2
        let z2 = z.square();
        let z3 = z2 * z;
        atan_poly(z, z2, z3) + off * FRAC_PI_2
    }
}

impl crate::math::InvTrigonometric for F32 {
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
