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
                let lgx = LN_PI - ln(sinpix.abs() * p) - (nx - SfpM128E16::half()) * ln(nx) + nx;
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
