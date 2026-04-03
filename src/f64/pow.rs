use crate::F64;
use crate::math_aux::sfpm192e16::log_core;
use crate::simple_fp::{Sfp, SfpM192E16};
use crate::uint192::u192;

// GENERATE: consts SfpM192E16 LN_2 LOG2_E
const LN_2: SfpM192E16 = Sfp::newp(
    -1,
    u192(0xB17217F7D1CF79AB, 0xC9E3B39803F2F6AF40F343267298B62E),
); // 6.93147180559945309417232121458176568075500134360255254121e-1
const LOG2_E: SfpM192E16 = Sfp::newp(
    0,
    u192(0xB8AA3B295C17F0BB, 0xBE87FED0691D3E88EB577AA8DD695A59),
); // 1.44269504088896340735992468100189213742664595415298593414e0

impl crate::generic::Pow for F64 {
    fn pow_finite(x: Self, y: Self, sign: bool) -> Self {
        let x = SfpM192E16::from_ieee_float(x.0);
        let y = SfpM192E16::from_ieee_float(y.0);
        pow_core(x, y, sign)
    }

    fn powi_finite(x: Self, y: i32) -> Self {
        let x = SfpM192E16::from_ieee_float(x.0);
        let sign = x.sign() && (y & 1) != 0;
        let y = SfpM192E16::from_int(y);
        pow_core(x, y, sign)
    }
}

fn pow_core(x: SfpM192E16, y: SfpM192E16, sign: bool) -> F64 {
    #[inline]
    fn ln(x: SfpM192E16) -> SfpM192E16 {
        let xm1 = x - SfpM192E16::one();
        if xm1.exponent() <= -13 {
            // GENERATE: ln_1p_poly SfpM192E16 13 -0.0002442 0.0002442
            const K2: SfpM192E16 = Sfp::newn(
                -1,
                u192(0x8000000000000000, 0x000000000000000000000000000004F5),
            ); // -5.00000000000000000000000000000000000000000000000000000202e-1
            const K3: SfpM192E16 = Sfp::newp(
                -2,
                u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAAAAAAAAAAAA62117CC940),
            ); // 3.33333333333333333333333333333333333333333333308496427757e-1
            const K4: SfpM192E16 = Sfp::newn(
                -3,
                u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFFFFFFFCF37C2172F4),
            ); // -2.49999999999999999999999999999999999999999999866487471743e-1
            const K5: SfpM192E16 = Sfp::newp(
                -3,
                u192(0xCCCCCCCCCCCCCCCC, 0xCCCCCCCCCCCCCCD7D269302BB61C095C),
            ); // 2.00000000000000000000000000000000000008097626130163761978e-1
            const K6: SfpM192E16 = Sfp::newn(
                -3,
                u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAAAACA605EC6939553148C),
            ); // -1.66666666666666666666666666666666666689963333330968338666e-1
            const K7: SfpM192E16 = Sfp::newp(
                -3,
                u192(0x9249249249249249, 0x249249244AF8469B90ED0E4DB18E3084),
            ); // 1.42857142857142857142857142856263820969497630444984636990e-1
            const K8: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFEE2E172112CB1780AADB13A33),
            ); // -1.24999999999999999999999999998242816319682815737394660060e-1
            const K9: SfpM192E16 = Sfp::newp(
                -4,
                u192(0xE38E38E38E38E38E, 0x3A8BD73E2CF1EBB5E789CB6213C3AB8E),
            ); // 1.11111111111111111111154981176382521560251225104688292996e-1
            const K10: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xCCCCCCCCCCCCCCCC, 0xCF5C111DB621DA405132620D043C5BD5),
            ); // -1.00000000000000000000067753005289821083038470348564676606e-1
            const K11: SfpM192E16 = Sfp::newp(
                -4,
                u192(0xBA2E8BA2E8B7B078, 0x0038ABFB47DA2CD8057B3E5FA8464043),
            ); // 9.09090909090898022042761017191234278391572714912502314336e-2
            const K12: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xAAAAAAAAAAA7839A, 0x1783714D0B857A3C2A7CB63692D1D300),
            ); // -8.33333333333319332991728895246964665231113816861261509576e-2
            const K13: SfpM192E16 = Sfp::newp(
                -4,
                u192(0x9D89DA7591719C5B, 0x6719DF902593E57B92822C21AF2E9C18),
            ); // 7.69230906609486501189532546595761094890615743716855008692e-2
            const K14: SfpM192E16 = Sfp::newn(
                -4,
                u192(0x9249268DB9F0B237, 0x4437D46DBF9AA4BE23B07D519FE1567A),
            ); // -7.14285861970370455490628783447744666960242088934292572832e-2

            let xm1_2 = xm1.square();
            xm1 + horner!(
                xm1_2,
                xm1,
                [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14]
            )
        } else {
            // GENERATE: ln_1p_poly SfpM192E16 21 -0.00001 0.015625
            const K2: SfpM192E16 = Sfp::newn(
                -2,
                u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFFFFFFFFFFE8C4FF0C),
            ); // -4.99999999999999999999999999999999999999999999999968955192e-1
            const K3: SfpM192E16 = Sfp::newp(
                -2,
                u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAAAAAAAAAAA63919F9C6AE),
            ); // 3.33333333333333333333333333333333333333333332944156486014e-1
            const K4: SfpM192E16 = Sfp::newn(
                -3,
                u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFFFF666E4C436E60FA),
            ); // -2.49999999999999999999999999999999999999998278430300807403e-1
            const K5: SfpM192E16 = Sfp::newp(
                -3,
                u192(0xCCCCCCCCCCCCCCCC, 0xCCCCCCCCCCCCCCC77EB9D1A7F831D2CC),
            ); // 1.99999999999999999999999999999999999996102518347081444709e-1
            const K6: SfpM192E16 = Sfp::newn(
                -3,
                u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAA8E5791557CC7407A77F0),
            ); // -1.66666666666666666666666666666666661339400504160106449748e-1
            const K7: SfpM192E16 = Sfp::newp(
                -3,
                u192(0x9249249249249249, 0x2492492491E4D1F8F47E0993DA9AA35E),
            ); // 1.42857142857142857142857142857138026782860709593412315283e-1
            const K8: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFE0B4A9570F00F94C46C2E9050),
            ); // -1.24999999999999999999999999996914144651736376199794639346e-1
            const K9: SfpM192E16 = Sfp::newp(
                -4,
                u192(0xE38E38E38E38E38E, 0x38E38AA394A53480E6DCD2CD81C89272),
            ); // 1.11111111111111111111111109663853889138295522043162180252e-1
            const K10: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xCCCCCCCCCCCCCCCC, 0xCCC7D6B7DC2D2C7D4D6AF434701702F0),
            ); // -9.99999999999999999999994870180475958218588182746320127160e-2
            const K11: SfpM192E16 = Sfp::newp(
                -4,
                u192(0xBA2E8BA2E8BA2E8B, 0x9D9BD1FFA2FFD00F11E193BBC129E330),
            ); // 9.09090909090909090907687900172004650542384699251221220449e-2
            const K12: SfpM192E16 = Sfp::newn(
                -4,
                u192(0xAAAAAAAAAAAAAAA6, 0x3B94E393ADC5E0447DC5371154273060),
            ); // -8.33333333333333333032878817245710476551644976024325107914e-2
            const K13: SfpM192E16 = Sfp::newp(
                -4,
                u192(0x9D89D89D89D89A9B, 0x1180003B759935AF535E977CB575B947),
            ); // 7.69230769230769179894548531450088944772307098307400882945e-2
            const K14: SfpM192E16 = Sfp::newn(
                -4,
                u192(0x9249249249230770, 0xDB39EA6F6A4DE676F48E979963566043),
            ); // -7.14285714285707436247646725613715897405843508258831940671e-2
            const K15: SfpM192E16 = Sfp::newp(
                -4,
                u192(0x8888888887E325F6, 0xD4704004BFF4A5B42343136F052DF12F),
            ); // 6.66666666665932209568154706342858401715108560071108856490e-2
            const K16: SfpM192E16 = Sfp::newn(
                -5,
                u192(0xFFFFFFFF91EB2D72, 0xDEC2C84324C3FB095FDEA1D87C2E0559),
            ); // -6.24999999937426004090811416763457863986772287513118796227e-2
            const K17: SfpM192E16 = Sfp::newp(
                -5,
                u192(0xF0F0F0D4093CF87C, 0x09840F5EED2EF3707A2D6D1917FD5BB0),
            ); // 5.88235289911402882451556613347083456983911575567106884354e-2
            const K18: SfpM192E16 = Sfp::newn(
                -5,
                u192(0xE38E32FA1CAE803D, 0xD2A1C043CDE56D0FF3F261ED6E1C5B40),
            ); // -5.55555335320537804085418473604889843758658124212256704971e-2
            const K19: SfpM192E16 = Sfp::newp(
                -5,
                u192(0xD79349CE702B57FB, 0xC05D8597663C89589A97FAA6CF34917F),
            ); // 5.26306994497741306571585265216903176057403505778594089519e-2
            const K20: SfpM192E16 = Sfp::newn(
                -5,
                u192(0xCCB1A4FFB9EC02DF, 0xF6942355E2AED273C440C231F9E9E42B),
            ); // -4.99741025229196747470055572082232472062169550310550856245e-2
            const K21: SfpM192E16 = Sfp::newp(
                -5,
                u192(0xC0DEF73463FFEA53, 0xFA870B4CD3948CB22C68C96408923528),
            ); // 4.70876366073298225565029397444994149997146308253551428287e-2
            const K22: SfpM192E16 = Sfp::newn(
                -5,
                u192(0x9E249017CAC38177, 0x39B7141C15AA578B6FE72A5DD19103AC),
            ); // -3.86090878134134353695219287166199539990796606316105070356e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo
                + horner!(
                    lo2,
                    lo,
                    [
                        K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16, K17,
                        K18, K19, K20, K21, K22
                    ]
                );

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }
    }

    #[inline]
    fn exp(x: SfpM192E16) -> SfpM192E16 {
        // x = k*ln(2) + r
        // k is an integer
        // |r| <= 0.5*ln(2)
        let Some(k) = (x * LOG2_E).to_int_round() else {
            if x.sign() {
                return SfpM192E16::ZERO;
            } else {
                return SfpM192E16::INFINITY;
            }
        };
        let r = x - SfpM192E16::from_int(k) * LN_2;

        // exp(r) = exp(r / s)^s
        // s = 2^5

        let rs = r.scalbn(-5);

        // GENERATE: exp_m1_poly SfpM192E16 16 -0.0108305 0.0108305
        const K2: SfpM192E16 = Sfp::newp(
            -2,
            u192(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFFFFFFFFFFFEE9EA10),
        ); // 4.99999999999999999999999999999999999999999999999998548325e-1
        const K3: SfpM192E16 = Sfp::newp(
            -3,
            u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAAAAAAAAAAAAAAA9E060FA),
        ); // 1.66666666666666666666666666666666666666666666666666138671e-1
        const K4: SfpM192E16 = Sfp::newp(
            -5,
            u192(0xAAAAAAAAAAAAAAAA, 0xAAAAAAAAAAAAAAAAAAAAE0EFCD3DC63A),
        ); // 4.16666666666666666666666666666666666666666672607959641609e-2
        const K5: SfpM192E16 = Sfp::newp(
            -7,
            u192(0x8888888888888888, 0x88888888888888888888B5D5515920E5),
        ); // 8.33333333333333333333333333333333333333333345731523806464e-3
        const K6: SfpM192E16 = Sfp::newp(
            -10,
            u192(0xB60B60B60B60B60B, 0x60B60B60B60B60A9B07436FCA03C0870),
        ); // 1.38888888888888888888888888888888888881797368436124169758e-3
        const K7: SfpM192E16 = Sfp::newp(
            -13,
            u192(0xD00D00D00D00D00D, 0x00D00D00D00D00C1A4DDAE03EAB7CAB8),
        ); // 1.98412698412698412698412698412698412688076336316787873222e-4
        const K8: SfpM192E16 = Sfp::newp(
            -16,
            u192(0xD00D00D00D00D00D, 0x00D00D00D29394D95A207D13AB80A997),
        ); // 2.48015873015873015873015873015911018328022673938394923546e-5
        const K9: SfpM192E16 = Sfp::newp(
            -19,
            u192(0xB8EF1D2AB6399C7D, 0x560E4472824EF285AAF823E283209B4B),
        ); // 2.75573192239858906525573192239901473304740086396611775643e-6
        const K10: SfpM192E16 = Sfp::newp(
            -22,
            u192(0x93F27DBBC4FAE397, 0x780B23DD804CAA31EA110DD6F3959DF8),
        ); // 2.75573192239858906525573084245149025710887316068428879199e-7
        const K11: SfpM192E16 = Sfp::newp(
            -26,
            u192(0xD7322B3FAA271C7F, 0x3A3EBFA2D0C60B35FDF62D7B0244361B),
        ); // 2.50521083854417187750520985515643776548264784237551504772e-8
        const K12: SfpM192E16 = Sfp::newp(
            -29,
            u192(0x8F76C77FC6C4BDB2, 0xC5BA1F02286D9912F2136C95523049D7),
        ); // 2.08767569878680989966194294719489632125873128015978739341e-9
        const K13: SfpM192E16 = Sfp::newp(
            -33,
            u192(0xB092309D43684BEF, 0xB47BB0E91D99449031A1D660167DC26A),
        ); // 1.60590438368216146127654595085195425567928860697412622461e-10
        const K14: SfpM192E16 = Sfp::newp(
            -37,
            u192(0xC9CBA54602AFBE1B, 0xD47EBCDAB70E1C1C34DCD4D5763D196F),
        ); // 1.14707455977137411218382325949085628736907430915740789563e-11
        const K15: SfpM192E16 = Sfp::newp(
            -41,
            u192(0xD73F9F399C77F07A, 0xD4727ADD4D068FBC05884561D3252ED0),
        ); // 7.64716373180918489299067443247228931447455048962646080316e-13
        const K16: SfpM192E16 = Sfp::newp(
            -45,
            u192(0xD73FB634EEFED294, 0x7BC79CF8EB256C3B733899CB7A560165),
        ); // 4.77948511889306440580236339706308561202418839016865565242e-14
        const K17: SfpM192E16 = Sfp::newp(
            -49,
            u192(0xCA9651199599B049, 0x724A117C8822C675877A2B753BB554A3),
        ); // 2.81146182705224355024349704922355440456861641083700627986e-15

        let rs2 = rs.square();
        let mut t = rs
            + horner!(
                rs2,
                rs,
                [
                    K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16, K17
                ]
            );

        // exp(r) - 1 = (t + 1)^s - 1
        // (t + 1)^2 - 1 = t^2 + 2*t
        for _ in 0..5 {
            t = t.square() + t.scalbn(1);
        }

        // exp(x) = exp(r) * 2^k = (t + 1) * 2^k
        (t + SfpM192E16::one()).scalbn(k)
    }

    // |z| = |x|^y = exp(y * ln(|x|))
    let absz = exp(y * ln(x.abs()));
    let z = absz.set_sign(sign);

    F64(z.round_prec(175).to_ieee_float())
}

impl crate::math::Pow for F64 {
    fn pow(self, other: Self) -> Self {
        crate::generic::pow(self, other)
    }

    fn powi(self, n: i32) -> Self {
        crate::generic::powi(self, n)
    }
}
