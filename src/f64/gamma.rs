use crate::math_aux::reduce_half_revs_sfp;
use crate::math_aux::sfpm128e16::log_core;
use crate::simple_fp::{Sfp, SfpM128E16};
use crate::traits::FloatImpExt as _;
use crate::{F64, Float as _};

// GENERATE: consts SfpM128E16 PI LN_2 LN_PI
const PI: SfpM128E16 = Sfp::newp(1, 0xC90FDAA22168C234C4C6628B80DC1CD1); // 3.1415926535897932384626433832795028842e0
const LN_2: SfpM128E16 = Sfp::newp(-1, 0xB17217F7D1CF79ABC9E3B39803F2F6AF); // 6.9314718055994530941723212145817656808e-1
const LN_PI: SfpM128E16 = Sfp::newp(0, 0x928682473D0DE85EAFCAB635421FA4CC); // 1.1447298858494001741434273513530587116e0

impl crate::generic::Gamma for F64 {
    fn gamma_finite(x: Self) -> Self {
        if x >= Self::from_int(172) {
            // Overflow
            return Self::INFINITY;
        } else if x <= Self::from_int(-185) {
            // Underflow
            let xi = (-x).to_uint(64).unwrap();
            return Self::ZERO.set_sign((xi & 1) == 0);
        }

        if x.exponent() < -500 {
            // Γ(x) ~= 1 / x
            Self::ONE / x
        } else {
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

            let x = SfpM128E16::from_ieee_float(x.0);
            let k = SfpM128E16::newp(1, 0x17 << 123); // 2.875

            let d = x - k;
            let i = d.to_int_round::<i16>().unwrap();
            let f = d - SfpM128E16::from_int(i);

            // gf = Γ(k + f)
            let gf = {
                // GENERATE: gamma_poly SfpM128E16 31 2.875 -0.5 0.5
                const K0: SfpM128E16 = Sfp::newp(0, 0xE4D3B5F2BB8916A7455B6EF6AA36C654); // 1.7877108988969403106484306138406836946e0
                const K1: SfpM128E16 = Sfp::newp(0, 0xC793AA6EE7C85FC2D5A731F0BE08550F); // 1.5591939012079505322581226465685041697e0
                const K2: SfpM128E16 = Sfp::newp(0, 0x8688C8CA4A0C4D136C19E3B29F520358); // 1.0510493266811828126891038101520684174e0
                const K3: SfpM128E16 = Sfp::newp(-2, 0xF0FA16785732E8C51975A3BCFE7030D8); // 4.7065801829339710106392216377505709515e-1
                const K4: SfpM128E16 = Sfp::newp(-3, 0xC159AC51D730B43CB7FB31007E9C19B7); // 1.8881863832011509889551834966643455972e-1
                const K5: SfpM128E16 = Sfp::newp(-5, 0xF0F95986589096EE4CE0CBBAF74DA22F); // 5.8831548410612686156023551749820606054e-2
                const K6: SfpM128E16 = Sfp::newp(-6, 0x9207B69EB1C4D928FDEE122F007A6B6A); // 1.7825943641178382068816717682585821813e-2
                const K7: SfpM128E16 = Sfp::newp(-8, 0x8A916506BAE8110FD0971ADF4CBFBBB8); // 4.2287581722668684160883141287022714699e-3
                const K8: SfpM128E16 = Sfp::newp(-10, 0x8FE80412F83B3F9803274ED54A166DD7); // 1.0979180310503825508205565342751658167e-3
                const K9: SfpM128E16 = Sfp::newp(-13, 0xCC044302FE2ED2FC85022C5597109998); // 1.9456543685651592909758679461629444307e-4
                const K10: SfpM128E16 = Sfp::newp(-15, 0xD9FA23238C608F0E9AEACC130742F3F7); // 5.1969790143123594399053334927709537090e-5
                const K11: SfpM128E16 = Sfp::newp(-18, 0xA4F14A8BBD1547AD12FFD7B9A0CF4BA0); // 4.9156708636719165582410666553079344983e-6
                const K12: SfpM128E16 = Sfp::newp(-19, 0xA40AA29ED42E1227ABF4083F738881DC); // 2.4444094880039095464650126300431836035e-6
                const K13: SfpM128E16 = Sfp::newn(-23, 0xA0C964A0772570FB567A01099A6EE655); // -1.4974427567179343975449051177778157135e-7
                const K14: SfpM128E16 = Sfp::newp(-23, 0xB128F354A5C45AA9F2DF94FD44EA0FC9); // 1.6499307279580085342749199446743450183e-7
                const K15: SfpM128E16 = Sfp::newn(-25, 0xADC82A4958ED35034926E9C25DFFD9AE); // -4.0461750524347771152108966906046604187e-8
                const K16: SfpM128E16 = Sfp::newp(-26, 0x8E58A4892B735F277F39100867605A21); // 1.6571285740630611760107088324090744946e-8
                const K17: SfpM128E16 = Sfp::newn(-28, 0xBAAE7AB9EC8B179EE9FC75EFC870CAC7); // -5.4331484761263547707033226159749353306e-9
                const K18: SfpM128E16 = Sfp::newp(-29, 0x8519DFCDD10D964561108498AFEFC772); // 1.9368755053086654265606336800902464446e-9
                const K19: SfpM128E16 = Sfp::newn(-31, 0xB7C9C17486F585E31FC846321CDCC2B4); // -6.6861724241553188970925633334086562293e-10
                const K20: SfpM128E16 = Sfp::newp(-32, 0x8050D71728FB64B1B8DB3B3D60E4CC35); // 2.3340504780225662032178947123598484123e-10
                const K21: SfpM128E16 = Sfp::newn(-34, 0xB26F784F5BE556CC3064520949149047); // -8.1143038885261652686556740414125745664e-11
                const K22: SfpM128E16 = Sfp::newp(-36, 0xF86DCC612F5E5F95D223EB927A611F38); // 2.8243096020974936158560325501983518132e-11
                const K23: SfpM128E16 = Sfp::newn(-37, 0xACD98862DCF2750F4409E8B79C8D9D61); // -9.8253700194852026009651376329282496110e-12
                const K24: SfpM128E16 = Sfp::newp(-39, 0xF084911CB9FAA5C7FE6A6A42DFEF4248); // 3.4179640698062847254671977894099978442e-12
                const K25: SfpM128E16 = Sfp::newn(-40, 0xA75248E83F6CB238946BE195735FA11E); // -1.1888902305362398481978455848479425502e-12
                const K26: SfpM128E16 = Sfp::newp(-42, 0xE95B646BDA0135B9086D875CA026F1D2); // 4.1452530485017544073349928337446922804e-13
                const K27: SfpM128E16 = Sfp::newn(-43, 0xA2A22FF3B862990DAC9817EC0DC43BF8); // -1.4444760426885594969445498970688371071e-13
                const K28: SfpM128E16 = Sfp::newp(-45, 0xDA55E4F1E40E268CB0C61FB157928AAE); // 4.8480225316832358806982293223031000345e-14
                const K29: SfpM128E16 = Sfp::newn(-46, 0x95473BFC640039EB6A0C2F732D1F4351); // -1.6573216028682273952058816714831124183e-14
                const K30: SfpM128E16 = Sfp::newp(-47, 0x8B2F2D5174822FD1083CB3F123E3B113); // 7.7262799075611842214521037424400510613e-15
                const K31: SfpM128E16 = Sfp::newn(-49, 0xCC7965F2A29EB34BF935A086A767F9AD); // -2.8376497242227662005852122989199246040e-15

                K0 + horner!(
                    f,
                    f,
                    [
                        K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16, K17,
                        K18, K19, K20, K21, K22, K23, K24, K25, K26, K27, K28, K29, K30, K31
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
                        v = v + SfpM128E16::one();
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
                        v = v + SfpM128E16::one();
                        gi = gi * v;
                    }
                    // Γ(x) = Γ(k + f) / prod(j = 0 to |i| - 1, x + j) = gf / gi
                    gf / gi
                }
            };

            Self(y.to_ieee_float())
        }
    }

    fn ln_gamma_finite(x: Self) -> (Self, i8) {
        fn ln(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: ln_1p_poly SfpM128E16 12 -0.00001 0.015625
            const K2: SfpM128E16 = Sfp::newn(-2, 0xFFFFFFFFFFFFFFFFFFFFFFF16F59479F); // -4.9999999999999999999999999990808164997e-1
            const K3: SfpM128E16 = Sfp::newp(-2, 0xAAAAAAAAAAAAAAAAAAA9B2804C4BE9A3); // 3.3333333333333333333333293240034046042e-1
            const K4: SfpM128E16 = Sfp::newn(-3, 0xFFFFFFFFFFFFFFFFF47A89CE0991E2F1); // -2.4999999999999999999939006561610015738e-1
            const K5: SfpM128E16 = Sfp::newp(-3, 0xCCCCCCCCCCCCCCAA7604D17E66919861); // 1.9999999999999999953461991936312423447e-1
            const K6: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAA6EAF5B63D262709B326B); // -1.6666666666666645856341530968105352731e-1
            const K7: SfpM128E16 = Sfp::newp(-3, 0x9249249248E1DBEE53B529687D44C830); // 1.4285714285708360469702707930960114499e-1
            const K8: SfpM128E16 = Sfp::newn(-4, 0xFFFFFFFF9D06F3C160D128880BF129D1); // -1.2499999998874809044900537166204302648e-1
            const K9: SfpM128E16 = Sfp::newp(-4, 0xE38E38B1722134EC8F475DBBADB02F1C); // 1.1111110965272583982904133178697643877e-1
            const K10: SfpM128E16 = Sfp::newn(-4, 0xCCCCBB68685ADA6FC8505986AC614DB3); // -9.9999870418327520335284407501121774889e-2
            const K11: SfpM128E16 = Sfp::newp(-4, 0xBA2A786AFBFA447E8610651E025C71F3); // 9.0901318325902321662289945050430200859e-2
            const K12: SfpM128E16 = Sfp::newn(-4, 0xAA0CA486768123C919139338D116F1E0); // -8.3031926492197323147364540646048877048e-2
            const K13: SfpM128E16 = Sfp::newp(-4, 0x8F7D6E780ADC16880CD29C46D668B3E0); // 7.0063460386661486018349089735095949981e-2

            let (k, lo, ln_hi) = log_core(x);
            let lo2 = lo.square();
            let ln_lo = lo
                + horner!(
                    lo2,
                    lo,
                    [K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13]
                );

            // ln(x) = ln(2^k * m) = k * ln(2) + ln(m)
            k * LN_2 + (ln_hi + ln_lo)
        }

        fn sin(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: sin_poly SfpM128E16 10
            const K3: SfpM128E16 = Sfp::newn(-3, 0xAAAAAAAAAAAAAAAAAAAAAAA302E445F0); // -1.6666666666666666666666666664251057510e-1
            const K5: SfpM128E16 = Sfp::newp(-7, 0x8888888888888888888862DF81A38506); // 8.3333333333333333333333314319760251295e-3
            const K7: SfpM128E16 = Sfp::newn(-13, 0xD00D00D00D00D00CFFCF78CDAD84D476); // -1.9841269841269841269836088271542832935e-4
            const K9: SfpM128E16 = Sfp::newp(-19, 0xB8EF1D2AB6399C79F4A053168CA9FD46); // 2.7557319223985890645566435686537375382e-6
            const K11: SfpM128E16 = Sfp::newn(-26, 0xD7322B3FAA270F6547AC9E934F5886E8); // -2.5052108385441713356457601535563778649e-8
            const K13: SfpM128E16 = Sfp::newp(-33, 0xB092309D4348E5D03D8DB46EFA2E4414); // 1.6059043836819017355175144448234015424e-10
            const K15: SfpM128E16 = Sfp::newn(-41, 0xD73F9F393D8F678EED5A6216D815762E); // -7.6471637310241215109672524400725762307e-13
            const K17: SfpM128E16 = Sfp::newp(-49, 0xCA963AC4C6F7AB3C743515EC00CE2EDE); // 2.8114570982201953214859113254910921913e-15
            const K19: SfpM128E16 = Sfp::newn(-57, 0x97A3F4AEB2696883FBBE83F27D9145B9); // -8.2204453914118296356866014032243851139e-18
            const K21: SfpM128E16 = Sfp::newp(-66, 0xB7A151C499B19C928347D759DF3A620B); // 1.9442598811033228252822928295304904515e-20

            let x2 = x.square();
            let x3 = x2 * x;

            x + horner!(x3, x2, [K3, K5, K7, K9, K11, K13, K15, K17, K19, K21])
        }

        fn cos(x: SfpM128E16) -> SfpM128E16 {
            // GENERATE: cos_poly SfpM128E16 9
            const K4: SfpM128E16 = Sfp::newp(-5, 0xAAAAAAAAAAAAAAAAAAAA603F6D55E4F8); // 4.1666666666666666666666651637888570130e-2
            const K6: SfpM128E16 = Sfp::newn(-10, 0xB60B60B60B60B60B5F27C7FCD8E61BA7); // -1.3888888888888888888882454599183204521e-3
            const K8: SfpM128E16 = Sfp::newp(-16, 0xD00D00D00D00D006846F9324760BF104); // 2.4801587301587301576571653335441852770e-5
            const K10: SfpM128E16 = Sfp::newn(-22, 0x93F27DBBC4FAD5563C2F5A0D99C893A5); // -2.7557319223985881219508969029157079895e-7
            const K12: SfpM128E16 = Sfp::newp(-29, 0x8F76C77FC69F93D899A6EF0541AB761B); // 2.0876756987863180450294734833704070271e-9
            const K14: SfpM128E16 = Sfp::newn(-37, 0xC9CBA5458AF1B5661D978DFC0598787F); // -1.1470745596128964706828629395704329557e-11
            const K16: SfpM128E16 = Sfp::newp(-45, 0xD73F9E4137E64A126F9B3033318D1AA7); // 4.7794770036355442352920835772684642509e-14
            const K18: SfpM128E16 = Sfp::newn(-53, 0xB4128A12862C9BAB466751CD2A87B6F1); // -1.5618792658258192839355388342083817717e-16
            const K20: SfpM128E16 = Sfp::newp(-62, 0xF0E6FDB00E554F887D17749FAA55131A); // 4.0810438468299167211411951751110131309e-19

            let x2 = x.square();
            let x4 = x2.square();

            let t = horner!(x4, x2, [K4, K6, K8, K10, K12, K14, K16, K18, K20]);
            SfpM128E16::one() + (t - x2.scalbn(-1))
        }

        let x = SfpM128E16::from_ieee_float(x.0);
        let (y, sign) = if x.exponent() < -500 {
            // Γ(x) ~= 1 / x
            // ln(abs(Γ(x))) ~= -ln(x)
            let y = -ln(x.abs());
            let sign = if x.sign() { -1 } else { 1 };
            (y, sign)
        } else if let Some((y, sign)) = ln_gamma_near_root(x) {
            (y, sign)
        } else if x.abs() <= SfpM128E16::from_int(50) {
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

            let k = SfpM128E16::two();

            let d = x - k;
            let i = d.to_int_round::<i8>().unwrap();
            let f = d - SfpM128E16::from_int(i);

            // lgf = ln(Γ(k + f))
            let lgf = {
                // GENERATE: ln_gamma_poly SfpM128E16 34 2 -0.5 0.50001
                const K1: SfpM128E16 = Sfp::newp(-2, 0xD8773039049E70B65C8380FDFCFECA03); // 4.2278433509846713939348790991758874605e-1
                const K2: SfpM128E16 = Sfp::newp(-2, 0xA51A6625307D3230E7B122441394DA95); // 3.2246703342411321823620758332345933732e-1
                const K3: SfpM128E16 = Sfp::newn(-4, 0x89F000D2ABB034092E83A0E2B4A70398); // -6.7352301053198095133246053816593271259e-2
                const K4: SfpM128E16 = Sfp::newp(-6, 0xA8991563EC241B5F91210E4EC1E7C3A2); // 2.0580808427784547879000923803916110699e-2
                const K5: SfpM128E16 = Sfp::newn(-8, 0xF2027E10C7AF8C36B3B19C75D03C7B38); // -7.3855510286739852662731052355530451214e-3
                const K6: SfpM128E16 = Sfp::newp(-9, 0xBD6EB756DB617EA487EDECD1EC3E483F); // 2.8905103307415232857530616664666485855e-3
                const K7: SfpM128E16 = Sfp::newn(-10, 0x9C562E15FC703E75B0F3BDE77223A280); // -1.1927539117032609771127219547025020612e-3
                const K8: SfpM128E16 = Sfp::newp(-11, 0x859B57C31CB745F2A95F0A2AFCE61236); // 5.0966952474304242232801357652009349784e-4
                const K9: SfpM128E16 = Sfp::newn(-13, 0xE9FEA63B697E3E3F9860C4A89C8A5D08); // -2.2315475845357937985890934037240889883e-4
                const K10: SfpM128E16 = Sfp::newp(-14, 0xD093D878BEB2D1E1FA97181CB4AEC779); // 9.9457512781808534169803361595344140501e-5
                const K11: SfpM128E16 = Sfp::newn(-15, 0xBC6F2DEBE40F71FD3FE83D73FEDA673C); // -4.4926236738133136954688115056740163710e-5
                const K12: SfpM128E16 = Sfp::newp(-16, 0xAC06E7733757E85FA453C012C4F3774F); // 2.0507212775670674284316356829701977056e-5
                const K13: SfpM128E16 = Sfp::newn(-17, 0x9E5E4B1E7114E2A35E2E72CAE05A317E); // -9.4394882752685480457018544829979591999e-6
                const K14: SfpM128E16 = Sfp::newp(-18, 0x92CBD1CF9A65CD60B237A6CAA5A5B7FD); // 4.3748667899079334360858002680953046887e-6
                const K15: SfpM128E16 = Sfp::newn(-19, 0x88D975BB3BB0290C0F152F693A3DC756); // -2.0392157537979798074029258006734817009e-6
                const K16: SfpM128E16 = Sfp::newp(-20, 0x803266F58CC3A2F7E442E7B6801F85A8); // 9.5514121303257767505119905429308571718e-7
                const K17: SfpM128E16 = Sfp::newn(-22, 0xF13006CA64C95A1615D4E493A78BAB08); // -4.4924691993061231911196464558139233479e-7
                const K18: SfpM128E16 = Sfp::newp(-23, 0xE3B5DDA17B6E54249BC442B475F5FAA4); // 2.1207184816474945643963768930475852029e-7
                const K19: SfpM128E16 = Sfp::newn(-24, 0xD7AD36471367FD74B520979AEA3D2599); // -1.0043224760380457839586006622777218087e-7
                const K20: SfpM128E16 = Sfp::newp(-25, 0xCCDC9DC20838C7316610044828946F6E); // 4.7698100608563559002680237788825957495e-8
                const K21: SfpM128E16 = Sfp::newn(-26, 0xC3163CC6846F26EDA02725C9A03ACF24); // -2.2711100156872783869317354183805405217e-8
                const K22: SfpM128E16 = Sfp::newp(-27, 0xBA34F67F7B6934AA60FB1ED8F3AB6AAE); // 1.0838667295211525325041384729367796899e-8
                const K23: SfpM128E16 = Sfp::newn(-28, 0xB21A02DDDA5951730A5B57D483779A76); // -5.1834389521112588430011097572030591817e-9
                const K24: SfpM128E16 = Sfp::newp(-29, 0xAAAC7878D563DFF98B9E21A8FCC21E0D); // 2.4836294070254172524279125828364393536e-9
                const K25: SfpM128E16 = Sfp::newn(-30, 0xA3DED39CD09ECAF7F3684B8BEB0856B3); // -1.1923142043188997747565325222125270235e-9
                const K26: SfpM128E16 = Sfp::newp(-31, 0x9D981E73120552D2EC9BD1559CFB75DF); // 5.7332441291656075001404706404365602265e-10
                const K27: SfpM128E16 = Sfp::newn(-32, 0x975EAF1821B664D6ED9626DD738A5831); // -2.7534016994913456008320178306072165884e-10
                const K28: SfpM128E16 = Sfp::newp(-33, 0x91A913564141285D50DF9741C6588435); // 1.3247740872264775187457061315827206585e-10
                const K29: SfpM128E16 = Sfp::newn(-34, 0x9094A4107CCB20E9DD54D4A07A2E2DF2); // -6.5747657765376202667607875278984901340e-11
                const K30: SfpM128E16 = Sfp::newp(-35, 0x8DE855C88468E058C6AC326E4C88182D); // 3.2266043251372206448036108575254894490e-11
                const K31: SfpM128E16 = Sfp::newn(-37, 0xDC713E18CE22DB99EF70F81FE639C443); // -1.2530697050205399432485919143555355155e-11
                const K32: SfpM128E16 = Sfp::newp(-38, 0xC33A4CA350B8839770E3EBDBFB5938AA); // 5.5487058688845568898746404993801254520e-12
                const K33: SfpM128E16 = Sfp::newn(-38, 0xCF16B057F64DB776F27F6045A525279C); // -5.8858128195943810592098321938168825547e-12
                const K34: SfpM128E16 = Sfp::newp(-39, 0xDAC64EC2E5A6C63871A441D578CA090B); // 3.1089746144959613497411047820506805208e-12

                horner!(
                    f,
                    f,
                    [
                        K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11, K12, K13, K14, K15, K16, K17,
                        K18, K19, K20, K21, K22, K23, K24, K25, K26, K27, K28, K29, K30, K31, K32,
                        K33, K34
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
                        v = v + SfpM128E16::one();
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
                        v = v + SfpM128E16::one();
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
            let nx = if reflect { SfpM128E16::one() - x } else { x };

            // p = P(1 / nx)
            let p = {
                // GENERATE: gamma_lanczos_poly SfpM128E16 11 0.5 5.56e-309 0.0201
                const K0: SfpM128E16 = Sfp::newp(1, 0xA06C98FFB1382CB2BE520FD734B3D042); // 2.5066282746310005024157652848101803936e0
                const K1: SfpM128E16 = Sfp::newp(-3, 0xD5E62154EC4AE643A86D798B88A44441); // 2.0888568955258337520131378592447850259e-1
                const K2: SfpM128E16 = Sfp::newp(-7, 0x8E996B8DF2DC998267B18300C3347D4E); // 8.7035703980243073000263135778513065930e-3
                const K3: SfpM128E16 = Sfp::newn(-8, 0xDC3C97E3C3E2ECD9089D2E602E850F4B); // -6.7210904740298817224472732820344847058e-3
                const K4: SfpM128E16 = Sfp::newn(-11, 0x96C91A0674047B1976F8A743170502B7); // -5.7520123811018342450018445411448727689e-4
                const K5: SfpM128E16 = Sfp::newp(-9, 0x80CC2D3E5026D922BD043A9DCEC1BB6A); // 1.9652948815865718427452908753852656239e-3
                const K6: SfpM128E16 = Sfp::newp(-13, 0xB745D35C0526B696D469CEBFAF934D2E); // 1.7478252061778914407725036710996436502e-4
                const K7: SfpM128E16 = Sfp::newn(-10, 0xC28E38B0D3E7FD3F506FCCE6A8FC5EC1); // -1.4843410685115298406299430075500434454e-3
                const K8: SfpM128E16 = Sfp::newn(-13, 0x87F0A797AAB727027C938EF23D1B4F8B); // -1.2964254117758043682661915472922350278e-4
                const K9: SfpM128E16 = Sfp::newp(-9, 0x89EC75B38CB78785150D55A269DCBAF0); // 2.1045482022142782043475604460859449355e-3
                const K10: SfpM128E16 = Sfp::newp(-13, 0xB6656DCB7832C9B6620689669D6F6861); // 1.7394657763074547815739695706698831708e-4
                const K11: SfpM128E16 = Sfp::newn(-8, 0x9A8D1E38839E4411C3FA34E246450B48); // -4.7165296137138897318874679474084324592e-3

                let inv_nx = nx.recip();
                K0 + horner!(
                    inv_nx,
                    inv_nx,
                    [K1, K2, K3, K4, K5, K6, K7, K8, K9, K10, K11]
                )
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
                let lgx = LN_PI - ln(sinpix.abs() * p) - (nx - SfpM128E16::half()) * ln_nx + nx;
                let sign = if sinpix.sign() { -1 } else { 1 };

                (lgx, sign)
            } else {
                // ln(abs(Γ(x))) = ln(Γ(nx))
                let lgx = (nx - SfpM128E16::half()) * ln_nx - nx + ln(p);
                (lgx, 1)
            }
        };

        (Self(y.to_ieee_float()), sign)
    }
}

impl crate::math::Gamma for F64 {
    fn gamma(self) -> Self {
        crate::generic::gamma(self)
    }

    fn ln_gamma(self) -> (Self, i8) {
        crate::generic::ln_gamma(self)
    }
}

/// Calculates ln(|Γ(x)|) with an expansion around a negative root of
/// ln(|Γ(x)|), if `x` is close enough to one of them.
///
/// Close to those roots, the general algorithm computes the result as a
/// difference of two almost equal values, which loses too much accuracy.
fn ln_gamma_near_root(x: SfpM128E16) -> Option<(SfpM128E16, i8)> {
    // All the roots in the table are in (-16, -2)
    if !x.sign() || !matches!(x.exponent(), 1..=3) {
        return None;
    }

    // The roots in the table are sorted in decreasing order, so only the
    // nearest ones above and below x can be close enough.
    let i = LN_GAMMA_ROOTS.partition_point(|root| root.x0_hi > x);
    for root in LN_GAMMA_ROOTS[i.saturating_sub(1)..].iter().take(2) {
        // When x is close to x0, both are in the same binade, so the
        // subtraction is exact.
        let h = x - root.x0_hi;
        if h.abs() <= root.radius {
            let h = h - root.x0_lo;
            let (&k_last, k_rest) = root.k.split_last().unwrap();
            let p = k_rest.iter().rev().fold(k_last, |p, &k| k + h * p);
            return Some((h * p, root.sign));
        }
    }

    None
}

/// Expansion of ln(|Γ(x)|) around one of its negative roots x0.
struct LnGammaRoot {
    /// x0 = x0_hi + x0_lo
    x0_hi: SfpM128E16,
    x0_lo: SfpM128E16,
    /// The expansion is used when |x - x0| <= radius.
    radius: SfpM128E16,
    /// Sign of Γ(x) around x0.
    sign: i8,
    /// ln(|Γ(x0 + h)|) ~= h * (k[0] + h * (k[1] + h * (k[2] + ...)))
    k: &'static [SfpM128E16],
}

// The radii are chosen so that, outside them, the error of the general
// algorithm stays below 2^-47 ulp. Roots below -8 are not included, since the
// general algorithm only loses that much accuracy within a few thousand ulps
// of them.
static LN_GAMMA_ROOTS: [LnGammaRoot; 13] = [
    ROOT_2_457, ROOT_2_747, ROOT_3_143, ROOT_3_955, ROOT_4_039, ROOT_4_991, ROOT_5_008, ROOT_5_998,
    ROOT_6_001, ROOT_6_999, ROOT_7_000, ROOT_7_999, ROOT_8_000,
];

// GENERATE: ln_gamma_root SfpM128E16 ROOT_2_457 -2.4570247382208006 -8 15
const ROOT_2_457: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(1, 0x9D3FE4B007C360AAFB27CC57C681C4B1), // -2.4570247382208006230394541476511795432e0
    x0_lo: Sfp::newp(-129, 0xCB7FB2658634A2B9F6B2BA80C4A82023), // 2.3360518403245786342169193832187667030e-39
    radius: Sfp::newp(-8, 0x80000000000000000000000000000000), // 3.9062500000000000000000000000000000000e-3
    sign: -1,
    k: &[
        Sfp::newp(0, 0xC1FF4B357A9AF688A6F65C9595C7C5CB), // 1.5156034480216573216370580047112015408e0
        Sfp::newp(2, 0x9B775D8017AAE4E569BDB6B87A9856FE), // 4.8583209516339961204939002695085430624e0
        Sfp::newp(0, 0xB4A5302C53C2BEE27374668A97E97B47), // 1.4112911430779799505534410562978283272e0
        Sfp::newp(3, 0x8B8C6BE504F2DB0632B61FBB4BF58DD3), // 8.7217825838153462179505692489932411043e0
        Sfp::newp(2, 0xB99CFF02593B1D6F36C9C5CE546AFF03), // 5.8004145665998724717337954535376048595e0
        Sfp::newp(4, 0xC6997B415505E66F6A76B8E455ECD4E2), // 2.4824942121894071094353798850747677816e1
        Sfp::newp(4, 0xC04F82773707CD3E725596C519467060), // 2.4038823062292954288535586227743094017e1
        Sfp::newp(6, 0xA475540B2B9B084B8B0B4FAA1622DD79), // 8.2229156827042530677634594623081946462e1
        Sfp::newp(6, 0xC94BD6D96F7ACD4F2270EC4C0B424DF3), // 1.0064812354551159023173894902486818987e2
        Sfp::newp(8, 0x9437DC65D7F79E4AD78FE7DD2B10328D), // 2.9643641350789333651755240664492692892e2
        Sfp::newp(8, 0xD497052EF303A6D1A986B341887EBA81), // 4.2517984568468027944016550712724509093e2
        Sfp::newp(10, 0x8D4EA693CA7B8963E03BDA2AC09CC3F3), // 1.1304578341440904236532186990872656522e3
        Sfp::newp(10, 0xE266928396A0EF5551E3E65D5025D578), // 1.8112053850118604524732517064746361789e3
        Sfp::newp(12, 0x8C4225C32EA957C4CCD9D5CC61532EFD), // 4.4882684386868886313949106983586767492e3
        Sfp::newp(12, 0xF30F1FDD54CBF2168904DBB604386A61), // 7.7778905588745822215091997890790015913e3
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_2_747 -2.7476826467274127 -8 17
const ROOT_2_747: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(1, 0xAFDA0850DEC8065E65692D1907ABAFD3), // -2.7476826467274126013914884826914996959e0
    x0_lo: Sfp::newp(-129, 0xFF4B7D6069E1BD7ACC1952FB4BCDEDC7), // 2.9306415176077171516863797061407352237e-39
    radius: Sfp::newp(-8, 0x80000000000000000000000000000000), // 3.9062500000000000000000000000000000000e-3
    sign: -1,
    k: &[
        Sfp::newn(0, 0xF5096D48258C62440261F33659D5FC91), // -1.9143501856115988164947315746096805315e0
        Sfp::newp(3, 0x9933F9E132D28DC739E01CC47959097F), // 9.5751894757096666706501462395944657910e0
        Sfp::newn(4, 0xA0C2D618645F8E0E9ED9C94A09DE788A), // -2.0095134916842602582459140657039988276e1
        Sfp::newp(5, 0xFA8256664F8CD61533FDE03F33EDB0B8), // 6.2627282713513713192931735943658205130e1
        Sfp::newn(7, 0xC2C422C103F55D695D35D16F5048B762), // -1.9476615530344621664787938610114273350e2
        Sfp::newp(9, 0xA1B9FBE6384D96DB8AEF56223BBC49FA), // 6.4690599971292827572126396454587955409e2
        Sfp::newn(11, 0x8911CDEEB600975C8371F81893F4A564), // -2.1931127764806237429059305245800864100e3
        Sfp::newp(12, 0xEDD32F13A10E213E0283820CE85AB05B), // 7.6103979866583287323721670038950919355e3
        Sfp::newn(14, 0xD1692826BF4C37D487560CE8A65F6443), // -2.6804578420618108716639683520247559749e4
        Sfp::newp(16, 0xBAC0B9CF7302D6E5AB3E54C14A4521D6), // 9.5617451643349062391542644311328292678e4
        Sfp::newn(18, 0xA83632FD6B0AFA7FBCF765B3ADE67F0B), // -3.4449759343483108749916681241536498701e5
        Sfp::newp(20, 0x98C77B92A5619FA79AF3CD00CC5A5B8D), // 1.2515674466045023772955212967736115830e6
        Sfp::newn(22, 0x8BBB39308DFB9B83F8F0E8028D9EE321), // -4.5787165948332431162145557838116712499e6
        Sfp::newp(24, 0x808F175566F3F43AC288813710A04990), // 1.6850478667204374554348570012975838466e7
        Sfp::newn(25, 0xEDC5CA6B0C08A4070D8DB84C8D090776), // -6.2330665672609481986193509235220884586e7
        Sfp::newp(27, 0xDD109205368F049C1237174C0175C615), // 2.3180316832581998634954147742248786617e8
        Sfp::newn(29, 0xCE24D73A198F72A9250930BAB6557357), // -8.6463022252496127278687615666985762395e8
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_3_143 -3.14358088834998 -11 13
const ROOT_3_143: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(1, 0xC9306DE4F2CD7BEE2F9A66B4F66E9951), // -3.1435808883499800586943587818202278996e0
    x0_lo: Sfp::newn(-128, 0xA65BFD63071F8B6CEBC20616FBBAF07A), // -3.8194230204559065727616362603766721461e-39
    radius: Sfp::newp(-11, 0x80000000000000000000000000000000), // 4.8828125000000000000000000000000000000e-4
    sign: 1,
    k: &[
        Sfp::newp(2, 0xF90532F97D62A6E2DA71B4F417BF2008), // 7.7818846581313508721395525959964643990e0
        Sfp::newp(4, 0xCEA694BB8A877B408112AF118311D276), // 2.5831338372387958935747275217113276474e1
        Sfp::newp(6, 0xE089B8926AE2D9D6C922CBB9E517E966), // 1.1226898629717600257828030157934042442e2
        Sfp::newp(9, 0x933901EBBB586CAB3EFDD41FBA93BA09), // 5.8889074223800134673368126365683137731e2
        Sfp::newp(11, 0xCCD319BED1CEE949B005FEEB79D7DE77), // 3.2771937854953417449816103187101529154e3
        Sfp::newp(14, 0x949E1FBC69DED87B6023A6449A90E173), // 1.9023061984356358371740370159772625568e4
        Sfp::newp(16, 0xDDCBD505B8F227C0020979CCB1E5F83F), // 1.1355966423713516223875755236550467378e5
        Sfp::newp(19, 0xA8F519A2FA9AB13E96FF43DF7DD61902), // 6.9204960228977610305434112301314931139e5
        Sfp::newp(22, 0x82BFB2E325914A6AF4AD19CD12C33FD8), // 4.2843774436459925977741324042523403494e6
        Sfp::newp(24, 0xCCE43285C78F83F58D2DCC90C46FD827), // 2.6855525045152606423564939338687723063e7
        Sfp::newp(27, 0xA229061BDE30277A8114434A075E59CB), // 1.7003734574174514218618009816460281211e8
        Sfp::newp(30, 0x816A22081F0369954FC1130C1EB5F5F2), // 1.0856081960605729098095954199786312240e9
        Sfp::newp(32, 0xD0004A3A8EB507D358D09AEF4D25F4AD), // 6.9793598611148996145217367741079000670e9
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_3_955 -3.955294284858598 -15 11
const ROOT_3_955: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(1, 0xFD238AA3E17F285C351584A448DB30C8), // -3.9552942848585979285327972832472084236e0
    x0_lo: Sfp::newn(-129, 0xA4887D1CB4C711828A75D5758F21D1F7), // -1.8887480370856199769505958214273408774e-39
    radius: Sfp::newp(-15, 0x80000000000000000000000000000000), // 3.0517578125000000000000000000000000000e-5
    sign: 1,
    k: &[
        Sfp::newn(4, 0xA5CCECB362B234C68BB75EA00192137D), // -2.0725060845803705678300499820071070971e1
        Sfp::newp(7, 0xFBB6F57021B5ED4A0CCBCA151D1480D0), // 2.5171468258688940014614044664622882533e2
        Sfp::newn(11, 0xE929ACEA5979BEEF3FD448176312D931), // -3.7306047156806125107418498987900980103e3
        Sfp::newp(15, 0xF47C14F8A0D528A59FFB8760869143CB), // 6.2588081918766060559366805816082512389e4
        Sfp::newn(20, 0x88B7BC03693698D17881A5EABDA5A9B4), // -1.1199915016655221353938405028668858115e6
        Sfp::newp(24, 0x9F479D5CFE0FA3C7D2B17C0CF8F50E27), // 2.0877114726503329249359050117455640556e7
        Sfp::newn(28, 0xBEDDF0317FECF2457F7CD4AD30187D65), // -4.0027699818746365809050617146864203055e8
        Sfp::newp(32, 0xE97BB6F3DE1B04C7EA63E682285D6AB0), // 7.8343981837351995445394827329973602602e9
        Sfp::newn(37, 0x9112FF27C1FB9D332B8EF8E607EDDDC2), // -1.5577224241649571685746536155875085102e11
        Sfp::newp(41, 0xB68963D2F71CF92723526D0AEC7745A0), // 3.1359562741404527070851269889242116077e12
        Sfp::newn(45, 0xE7FE2700081780456B71AA5D95EC9C72), // -6.3769690505733875264815151778195493692e13
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_4_039 -4.039361839740537 -15 11
const ROOT_4_039: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0x814273C2CCAC061873E161BC30225583), // -4.0393618397405368742345770963753546197e0
    x0_lo: Sfp::newp(-130, 0x805282DC248400D6567A39E057C3F592), // 7.3653393371299958181891405780041123626e-40
    radius: Sfp::newp(-15, 0x80000000000000000000000000000000), // 3.0517578125000000000000000000000000000e-5
    sign: -1,
    k: &[
        Sfp::newp(4, 0xD652E7A490B211A46A2E0D8FE1032500), // 2.6790480886140593260622633936281014950e1
        Sfp::newp(8, 0xA220AE6C09FC706BF7099C9C84C668C8), // 3.2425532293784715720433271573720740608e2
        Sfp::newp(12, 0xAACD88D954E3E117B8ADA88B734B2E10), // 5.4656918207771342500406851778988818045e3
        Sfp::newp(16, 0xCB68C710D75ED78F434AC7EED4D0AB4D), // 1.0414555520145541151509050207353498302e5
        Sfp::newp(21, 0x8130F5AB9972027221EBDC7A19A31489), // 2.1166694175775350563827726561794846376e6
        Sfp::newp(25, 0xAAF1EDFCCF59E43BEDA794E5822905ED), // 4.4812215950155709156855076226556636209e7
        Sfp::newp(29, 0xE8A7F24E2721B45C7126F6719EDC43EE), // 9.7583016353821451009961315274595604171e8
        Sfp::newp(34, 0xA19EE714165DB764AE2AE08036D130ED), // 2.1692364960698939987794201356234462904e10
        Sfp::newp(38, 0xE41CCE3AC39400C0E786C4D4DAC9F773), // 4.8986791049778906824899912356745704429e11
        Sfp::newp(43, 0xA2FE01EA5A492E71875303F14201133C), // 1.1200739845540573838738069977262818734e13
        Sfp::newp(47, 0xEB46D41D3ED7EA4AB71584BFEFBAD8C1), // 2.5868943892450391520256305079760908258e14
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_4_991 -4.991544640560048 -19 9
const ROOT_4_991: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0x9FBABBD37757E6A21A15480ED310A03B), // -4.9915446405600477223452601228064657217e0
    x0_lo: Sfp::newp(-131, 0xA1FB6B5E421CB86BF83ADA661AEF3D34), // 4.6486584907652567931342094966113110008e-40
    radius: Sfp::newp(-19, 0x80000000000000000000000000000000), // 1.9073486328125000000000000000000000000e-6
    sign: -1,
    k: &[
        Sfp::newn(6, 0xE91251F7CF20FB37C4E542775785B9E6), // -1.1653578161624363079668877957442926358e2
        Sfp::newp(12, 0xDA99E33C51CAAEC82595D3DBD96F7CB2), // 6.9952359548940638412458958571962364113e3
        Sfp::newn(19, 0x869FBFF732E99C32E8120330FEA2EE48), // -5.5141999785128835645233066591257060749e5
        Sfp::newp(25, 0xBA9537AD61392FBA439DB565184ECDB8), // 4.8911582709059044465044735959460084611e7
        Sfp::newn(32, 0x89EAE8B1DE9FBB026FCD8462AE9B7D0C), // -4.6277471717392495881937116815462384437e9
        Sfp::newp(38, 0xD462E29A2C648D429B8731D0182D0A2E), // 4.5609604226219639022968803809886554660e11
        Sfp::newn(45, 0xA83459F6B49FA31CAF5AE3831DDD74A8), // -4.6235700276519909289112017066184865751e13
        Sfp::newp(52, 0x87FD2C804D5124AF704545203298C56E), // 4.7846861786097005856633578987609375422e15
        Sfp::newn(58, 0xDF60986D4B208F5AEA3C7E4CFD0BBAAF), // -5.0300102073129484284109329860794576397e17
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_5_008 -5.0082181683225935 -19 9
const ROOT_5_008: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xA04352BF85B6C865498ADCB2A729BD99), // -5.0082181683225935215523681373913697096e0
    x0_lo: Sfp::newp(-128, 0xA87DE79580D45C82C6E6654874D9EE97), // 3.8683823559250804618918455963182012338e-39
    radius: Sfp::newp(-19, 0x80000000000000000000000000000000), // 1.9073486328125000000000000000000000000e-6
    sign: 1,
    k: &[
        Sfp::newp(6, 0xF6B970414D700F011F3D328ADC6AFD1B), // 1.2336218456335339333730679831563451668e2
        Sfp::newp(12, 0xE7661976117CD9B6ECC778E445F8E4D1), // 7.4047624322286821682781082575219130083e3
        Sfp::newp(19, 0x929EC2B1FB931D465B185315733B8296), // 6.0055616845281092209665075149116905704e5
        Sfp::newp(25, 0xD112EF96D3731402C658FD19D45724D8), // 5.4807486356655854741892818708680412744e7
        Sfp::newp(32, 0x9F00BB9BB133841E2FF196E9B104BB42), // 5.3352507433843846461676434209902380368e9
        Sfp::newp(38, 0xFBEC6ADEE58B5BFDB9AC6E373A252B75), // 5.4100160907477218621150515470171700240e11
        Sfp::newp(45, 0xCD468063BD487B1CECE795487D0BB4F8), // 5.6425671356242120227529177391979384633e13
        Sfp::newp(52, 0xAABFED7809BB0BAB9BA66F77B67AA620), // 6.0077215853586894587929728285999782399e15
        Sfp::newp(59, 0x904911DB823A92F38AC7FDFF9E3292A0), // 6.4980380307761796722138213365714122743e17
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_5_998 -5.998607480080875 -23 8
const ROOT_5_998: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xBFF497AC8FA06AFBA9DAC597A63DBF1C), // -5.9986074800808756294424079113719197389e0
    x0_lo: Sfp::newn(-127, 0xFE3977AE8B59F6F5C3765B383A5FFCA6), // -1.1673415740631564324947803015512936183e-38
    radius: Sfp::newp(-23, 0x80000000000000000000000000000000), // 1.1920928955078125000000000000000000000e-7
    sign: 1,
    k: &[
        Sfp::newn(9, 0xB30FB521D2F0921886F707781CCD72F7), // -7.1624543042754729208083690850643731716e2
        Sfp::newp(17, 0xFBCEE5BCA793779495FF9CA7811DF939), // 2.5785158963956261715944054314115406974e5
        Sfp::newn(26, 0xEB7404450CFFF12C08708DC0667F1392), // -1.2344528215783688644320476620797574897e8
        Sfp::newp(35, 0xF7AE9846DFE4B861F3709517A1D5FE0B), // 6.6486633581993339903469892669354994736e10
        Sfp::newn(45, 0x8AF535855A94EE8941943A20272D818D), // -3.8196442388133232945465728818271101124e13
        Sfp::newp(54, 0xA26AA74FF11CB68EB2EE15BF223E539D), // 2.2858106530205275278708877710930102480e16
        Sfp::newn(63, 0xC342722BCE3A9649A2BCA826AC2CAF89), // -1.4069933718327957065635691174928228569e19
        Sfp::newp(72, 0xEFA261206B0B724B7AABE39F41B35D09), // 8.8409450051351874495893428839751056574e21
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_6_001 -6.001385294453155 -23 8
const ROOT_6_001: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xC00B592C4BE4676C0F85B2DA2C70B971), // -6.0013852944531550972619816537417720798e0
    x0_lo: Sfp::newp(-128, 0xA9D4B65F9870D06D9042AA428577AAED), // 3.8991264201373662261817731233347794959e-39
    radius: Sfp::newp(-23, 0x80000000000000000000000000000000), // 1.1920928955078125000000000000000000000e-7
    sign: -1,
    k: &[
        Sfp::newp(9, 0xB4EF24F1D79550CAA5B484A1D999E209), // 7.2373662992528012397870838742930394051e2
        Sfp::newp(17, 0xFE711A4267E88304CDCEAF73C6CB4ADE), // 2.6054841030309396962826348013658214462e5
        Sfp::newp(26, 0xEF281D1E1BE2019FA745A4D064BFEAEF), // 1.2538698494090366665953047362656405925e8
        Sfp::newp(35, 0xFCE3DA92AC55DB6D12E9CE9FA61ECE72), // 6.7884656938770961214132160371465772906e10
        Sfp::newp(45, 0x8E9EA838A20B4FCEA7C55B5355694923), // 3.9203019565186827936765118312342115720e13
        Sfp::newp(54, 0xA790F17DCF0300B000AA99C7DCF5A518), // 2.3582843991458176343755084297237373559e16
        Sfp::newp(63, 0xCA804CCD8EF9BC56E6074DB2D0C17643), // 1.4591747238431145046898548942714801988e19
        Sfp::newp(72, 0xF9D161553EBE2651A1A6AD6DFA102502), // 9.2166534166166254805153021676512728746e21
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_6_999 -6.999801507890638 -30 6
const ROOT_6_999: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xDFFE5FBB5C377FE7AC4D6CB847FA43E8), // -6.9998015078906376978920974118680798505e0
    x0_lo: Sfp::newn(-131, 0xC028EEA050C2CB149A62FA64411F0D86), // -5.5147184203693398638822551370506879869e-40
    radius: Sfp::newp(-30, 0x80000000000000000000000000000000), // 9.3132257461547851562500000000000000000e-10
    sign: -1,
    k: &[
        Sfp::newn(12, 0x9D5FBD2E7548DB04AF7157A0A2275E8F), // -5.0359673737681254145061352361561097594e3
        Sfp::newp(23, 0xC1A4D12A821164721A217DD8C247DBBA), // 1.2690641166047179243925778374512243060e7
        Sfp::newn(35, 0x9EC8ED6E4C213EB99F48444FC5AA3916), // -4.2623489764758116459929184818646836225e10
        Sfp::newp(47, 0x9279EB1B7999CAB2CCC820E72D7951A1), // 1.6105233314447379179077039671304885512e14
        Sfp::newn(59, 0x90213EF8E83E7F31F878E2BC8E520444), // -6.4910321590389349112316216057742624216e17
        Sfp::newp(71, 0x93BAF42C0FAA2469832BF31223A4CA7C), // 2.7251428196664667283231716777169247425e21
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_7_000 -7.000198333407325 -30 6
const ROOT_7_000: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xE0019FEF6FF0F5BE8905B9B730F05565), // -7.0001983334073247516069810464441723783e0
    x0_lo: Sfp::newp(-128, 0xCF1AD3A6B203DC01496F8CCFA1548355), // 4.7548928334298778928083966298664982601e-39
    radius: Sfp::newp(-30, 0x80000000000000000000000000000000), // 9.3132257461547851562500000000000000000e-10
    sign: 1,
    k: &[
        Sfp::newp(12, 0x9DA03D51C3DE89DA1E57343B1EED9E76), // 5.0440299411108291965348895970389543693e3
        Sfp::newp(23, 0xC1F42ED57DD6AC19158E4C4768204B82), // 1.2710958833951394096479091371862119388e7
        Sfp::newp(35, 0x9F2A95AF1E112DF09CFDCAFED66D7550), // 4.2725890801879194202323435146752973486e10
        Sfp::newp(47, 0x92F21522F482D8B0D29BBE71ADB9C220), // 1.6156843435328284644810011236045929087e14
        Sfp::newp(59, 0x90B51AB048CE5B07A8CDF9D1ED0105AF), // 6.5170436604276881647871205888696999824e17
        Sfp::newp(71, 0x9470E3878CABFB66780BCAD65E95CA33), // 2.7382526172992254378160460638028069205e21
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_7_999 -7.999975197095821 -32 7
const ROOT_7_999: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(2, 0xFFFFCBFC0ACE787953D056A4561953A6), // -7.9999751970958206641543361471668073725e0
    x0_lo: Sfp::newp(-128, 0xBBE4197075A1617D7FE8D8A2BF5C47A9), // 4.3137661045887351677068180260831709871e-39
    radius: Sfp::newp(-32, 0x80000000000000000000000000000000), // 2.3283064365386962890625000000000000000e-10
    sign: 1,
    k: &[
        Sfp::newn(15, 0x9D7BB7F2617D597CC92F0B996A4F648E), // -4.0315718542187788887026563971207362892e4
        Sfp::newp(29, 0xC1C73B6557891A525C2BEA526EECA8BD), // 8.1276488933548394324956724272398062285e8
        Sfp::newn(44, 0x9EF345992B292D5752B6F8602EC62395), // -2.1845960238437147139211853797827838469e13
        Sfp::newp(59, 0x92AE02929863874C76A752E68B41D866), // 6.6058676275814206877896816618950067597e17
        Sfp::newn(74, 0x90615420C0934F7141F09F8CC881F171), // -2.1306755305163003234831519476310364139e22
        Sfp::newp(89, 0x9409C9A1B10194125A9CAC1471796DE7), // 7.1586896819186082696884702431942592366e26
        Sfp::newn(104, 0x9C203361EB8D0E5662A36C2DBE41D4B9), // -2.4739117980342035809895919934332514304e31
    ],
};

// GENERATE: ln_gamma_root SfpM128E16 ROOT_8_000 -8.000024800270682 -32 7
const ROOT_8_000: LnGammaRoot = LnGammaRoot {
    x0_hi: Sfp::newn(3, 0x80001A01459FC9F60CB3CEC1CEC85766), // -8.0000248002706819596977101037947886773e0
    x0_lo: Sfp::newn(-126, 0xEF94A71B10108621307701CF0B46A1C2), // -2.2002010142956471912831492801644337251e-38
    radius: Sfp::newp(-32, 0x80000000000000000000000000000000), // 2.3283064365386962890625000000000000000e-10
    sign: -1,
    k: &[
        Sfp::newp(15, 0x9D8447F6B3B8BD5F53D88BACB9DA843E), // 4.0324281108124353861397263804985255342e4
        Sfp::newp(29, 0xC1D1C49AA876E3EA5C764868514C5509), // 8.1293751066451603048255084157759125957e8
        Sfp::newp(44, 0x9F003C6DC56D22A0CAE7F7DAF688ECD1), // 2.1852920330413641908250049693302153569e13
        Sfp::newp(59, 0x92BDF64A321288630A708B463A199CFB), // 6.6086739366649255019004873661072565776e17
        Sfp::newp(74, 0x9074F503AAD234D0D45784939F34DF75), // 2.1318070343392601474722734933672857535e22
        Sfp::newp(89, 0x9421F098E7DB6A912E046DF3CBA567DD), // 7.1632519053397508229956037580930457251e26
        Sfp::newp(104, 0x9C3DEB541D351D51B37A2FA95A09550E), // 2.4757512865219031191837049049780072908e31
    ],
};

#[cfg(test)]
mod tests {
    use super::LN_GAMMA_ROOTS;

    #[test]
    fn test_ln_gamma_roots() {
        // `ln_gamma_near_root` requires the roots to be in (-16, -2), sorted
        // in decreasing order and with non-overlapping ranges.
        for root in LN_GAMMA_ROOTS.iter() {
            assert!(root.x0_hi.sign());
            assert!(matches!(root.x0_hi.exponent(), 1..=3));
        }
        for pair in LN_GAMMA_ROOTS.windows(2) {
            assert!(pair[0].x0_hi - pair[0].radius > pair[1].x0_hi + pair[1].radius);
        }
    }
}
