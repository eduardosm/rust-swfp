use std::ffi::OsStr;

use rayon::iter::{IntoParallelIterator as _, ParallelExtend as _, ParallelIterator as _};
use rayon::slice::ParallelSliceMut as _;

use crate::RunError;

pub(crate) fn run(arg: &OsStr) -> Result<(), RunError> {
    match arg.to_str() {
        Some("f16::pow") => {
            gen_f16_pow();
            Ok(())
        }
        Some("f16::atan2") => {
            gen_f16_atan2();
            Ok(())
        }
        Some("f16::atan2d") => {
            gen_f16_atan2d();
            Ok(())
        }
        Some("f16::atan2pi") => {
            gen_f16_atan2pi();
            Ok(())
        }
        Some("f32::exp") => {
            gen_f32_exp();
            Ok(())
        }
        Some("f32::exp_m1") => {
            gen_f32_exp_m1();
            Ok(())
        }
        Some("f32::exp2") => {
            gen_f32_exp2();
            Ok(())
        }
        Some("f32::exp2_m1") => {
            gen_f32_exp2_m1();
            Ok(())
        }
        Some("f32::exp10") => {
            gen_f32_exp10();
            Ok(())
        }
        Some("f32::exp10_m1") => {
            gen_f32_exp10_m1();
            Ok(())
        }
        Some("f32::ln") => {
            gen_f32_ln();
            Ok(())
        }
        Some("f32::ln_1p") => {
            gen_f32_ln_1p();
            Ok(())
        }
        Some("f32::log2") => {
            gen_f32_log2();
            Ok(())
        }
        Some("f32::log2_1p") => {
            gen_f32_log2_1p();
            Ok(())
        }
        Some("f32::log10") => {
            gen_f32_log10();
            Ok(())
        }
        Some("f32::log10_1p") => {
            gen_f32_log10_1p();
            Ok(())
        }
        Some("f32::sin") => {
            gen_f32_sin();
            Ok(())
        }
        Some("f32::cos") => {
            gen_f32_cos();
            Ok(())
        }
        Some("f32::tan") => {
            gen_f32_tan();
            Ok(())
        }
        Some("f32::sind") => {
            gen_f32_sind();
            Ok(())
        }
        Some("f32::cosd") => {
            gen_f32_cosd();
            Ok(())
        }
        Some("f32::tand") => {
            gen_f32_tand();
            Ok(())
        }
        Some("f32::sinpi") => {
            gen_f32_sinpi();
            Ok(())
        }
        Some("f32::cospi") => {
            gen_f32_cospi();
            Ok(())
        }
        Some("f32::tanpi") => {
            gen_f32_tanpi();
            Ok(())
        }
        Some("f32::asin") => {
            gen_f32_asin();
            Ok(())
        }
        Some("f32::acos") => {
            gen_f32_acos();
            Ok(())
        }
        Some("f32::atan") => {
            gen_f32_atan();
            Ok(())
        }
        Some("f32::asind") => {
            gen_f32_asind();
            Ok(())
        }
        Some("f32::acosd") => {
            gen_f32_acosd();
            Ok(())
        }
        Some("f32::atand") => {
            gen_f32_atand();
            Ok(())
        }
        Some("f32::asinpi") => {
            gen_f32_asinpi();
            Ok(())
        }
        Some("f32::acospi") => {
            gen_f32_acospi();
            Ok(())
        }
        Some("f32::atanpi") => {
            gen_f32_atanpi();
            Ok(())
        }
        Some("f32::sinh") => {
            gen_f32_sinh();
            Ok(())
        }
        Some("f32::cosh") => {
            gen_f32_cosh();
            Ok(())
        }
        Some("f32::tanh") => {
            gen_f32_tanh();
            Ok(())
        }
        Some("f32::asinh") => {
            gen_f32_asinh();
            Ok(())
        }
        Some("f32::acosh") => {
            gen_f32_acosh();
            Ok(())
        }
        Some("f32::atanh") => {
            gen_f32_atanh();
            Ok(())
        }
        Some("f32::gamma") => {
            gen_f32_gamma();
            Ok(())
        }
        Some("f32::ln_gamma") => {
            gen_f32_ln_gamma();
            Ok(())
        }
        _ => {
            eprintln!("Invalid function: {arg:?}");
            Err(RunError)
        }
    }
}

fn gen_f16_pow() {
    gen_2xf16(-14..=15, [false, true], -14..=15, [false, true], |x, y| {
        use rug::ops::PowAssignRound;
        let mut v = rug::Float::with_val(24, x);
        v.pow_assign_round(y, rug::float::Round::Zero);
        check_hard_round_f16(v, 13)
    });
}

fn gen_f16_atan2() {
    gen_2xf16(-14..=15, [false], -14..=15, [false, true], |y, x| {
        let (v, _) = rug::Float::with_val_round(26, y.atan2_ref(x), rug::float::Round::Zero);
        check_hard_round_f16(v, 15)
    });
}

fn gen_f16_atan2d() {
    gen_2xf16(-14..=15, [false], -14..=15, [false, true], |y, x| {
        let de = x.get_exp().unwrap() - y.get_exp().unwrap();
        if (x.is_sign_negative() && de >= 24) || de <= -25 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(26, y.atan2_u_ref(x, 360), rug::float::Round::Zero);
        check_hard_round_f16(v, 15)
    });
}

fn gen_f16_atan2pi() {
    gen_2xf16(-14..=15, [false], -14..=15, [false, true], |y, x| {
        let de = x.get_exp().unwrap() - y.get_exp().unwrap();
        if (x.is_sign_negative() && de >= 24) || de <= -24 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(26, y.atan2_pi_ref(x), rug::float::Round::Zero);
        check_hard_round_f16(v, 15)
    });
}

fn gen_f32_exp() {
    gen_f32(-39..=7, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.exp_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_exp_m1() {
    gen_f32(-38..=7, [false, true], |x| {
        if x.is_sign_negative() && x.get_exp().unwrap() > 4 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.exp_m1_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_exp2() {
    gen_f32(-38..=8, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.exp2_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_exp2_m1() {
    gen_f32(-126..=8, [false, true], |x| {
        if x.is_sign_negative() && x.get_exp().unwrap() > 5 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.exp2_m1_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_exp10() {
    gen_f32(-40..=6, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.exp10_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_exp10_m1() {
    gen_f32(-126..=6, [false, true], |x| {
        if x.is_sign_negative() && x.get_exp().unwrap() > 3 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.exp10_m1_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_ln() {
    gen_f32(-126..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.ln_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_ln_1p() {
    gen_f32(-38..=127, [false, true], |x| {
        if *x <= -1 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.ln_1p_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_log2() {
    gen_f32(-126..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.log2_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_log2_1p() {
    gen_f32(-126..=127, [false, true], |x| {
        if *x <= -1 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.log2_1p_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_log10() {
    gen_f32(-126..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.log10_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_log10_1p() {
    gen_f32(-126..=127, [false, true], |x| {
        if *x <= -1 {
            return false;
        }
        let (v, _) = rug::Float::with_val_round(40, x.log10_1p_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_sin() {
    gen_f32(-18..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.sin_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_cos() {
    gen_f32(-19..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.cos_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_tan() {
    gen_f32(-18..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.tan_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_sind() {
    gen_f32(-126..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.sin_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_cosd() {
    gen_f32(-13..=24, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.cos_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_tand() {
    gen_f32(-126..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.tan_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_sinpi() {
    gen_f32(-126..=18, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.sin_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_cospi() {
    gen_f32(-21..=18, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.cos_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_tanpi() {
    gen_f32(-126..=16, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.tan_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_asin() {
    gen_f32(-18..=-1, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.asin_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_acos() {
    gen_f32(-126..=-1, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.acos_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_atan() {
    gen_f32(-18..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.atan_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_asind() {
    gen_f32(-126..=-1, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.asin_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_acosd() {
    gen_f32(-38..=-1, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.acos_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_atand() {
    gen_f32(-126..=37, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.atan_u_ref(360), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_asinpi() {
    gen_f32(-126..=-1, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.asin_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_acospi() {
    gen_f32(-38..=-1, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.acos_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_atanpi() {
    gen_f32(-126..=38, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.atan_pi_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_sinh() {
    gen_f32(-18..=9, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.sinh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_cosh() {
    gen_f32(-19..=9, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.cosh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_tanh() {
    gen_f32(-18..=2, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.tanh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_asinh() {
    gen_f32(-18..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(41, x.asinh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 17)
    });
}

fn gen_f32_acosh() {
    gen_f32(0..=127, [false], |x| {
        let (v, _) = rug::Float::with_val_round(41, x.acosh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 17)
    });
}

fn gen_f32_atanh() {
    gen_f32(-18..=-1, [false], |x| {
        let (v, _) = rug::Float::with_val_round(40, x.atanh_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 16)
    });
}

fn gen_f32_gamma() {
    gen_f32(-126..=7, [false, true], |x| {
        let (v, _) = rug::Float::with_val_round(44, x.gamma_ref(), rug::float::Round::Zero);
        check_hard_round_f32(v, 20)
    });
}

fn gen_f32_ln_gamma() {
    gen_f32(-126..=127, [false, true], |x| {
        use rug::ops::AssignRound as _;
        let mut v = rug::Float::with_val(44, 0);
        (&mut v, &mut std::cmp::Ordering::Equal)
            .assign_round(x.ln_abs_gamma_ref(), rug::float::Round::Zero);
        v.get_exp().is_some_and(|e| e < -8) || check_hard_round_f32(v, 20)
    });
}

fn gen_2xf16(
    exps1: impl IntoIterator<Item = i32>,
    signs1: impl IntoIterator<Item = bool> + Clone,
    exps2: impl IntoIterator<Item = i32> + Clone,
    signs2: impl IntoIterator<Item = bool> + Clone,
    f: impl Fn(&rug::Float, &rug::Float) -> bool + Sync,
) {
    let mut results = Vec::new();
    for e1 in exps1 {
        for s1 in signs1.clone() {
            for e2 in exps2.clone() {
                for s2 in signs2.clone() {
                    eprintln!(
                        "Generating for x: e={e1}, {}; y: e={e2}, {}",
                        if s1 { "negative" } else { "positive" },
                        if s2 { "negative" } else { "positive" },
                    );

                    for m1 in 0u16..(1 << 10) {
                        results.par_extend((0u16..(1 << 10)).into_par_iter().filter_map(|m2| {
                            let m1 = m1 | (1 << 10);
                            let mut x = rug::Float::with_val(11, m1) << (e1 - 10);
                            if s1 {
                                x = -x;
                            }
                            let m2 = m2 | (1 << 10);
                            let mut y = rug::Float::with_val(11, m2) << (e2 - 10);
                            if s2 {
                                y = -y;
                            }
                            if f(&x, &y) {
                                Some(((s1, e1, m1), (s2, e2, m2)))
                            } else {
                                None
                            }
                        }));
                    }

                    eprintln!("{} value(s) so far", results.len());
                }
            }
        }
    }

    results.par_sort_by_key(|&((s1, e1, m1), (s2, e2, m2))| ((e1, m1, s1), (e2, m2, s2)));

    println!("# Generated file. Do not edit manually.");
    print_2xf16_list(results.iter().copied());
}

fn gen_f32(
    exps: impl IntoIterator<Item = i32>,
    signs: impl IntoIterator<Item = bool> + Clone,
    f: impl Fn(&rug::Float) -> bool + Sync,
) {
    let mut results = Vec::new();
    for e in exps {
        for s in signs.clone() {
            eprintln!(
                "Generating for e={e}, {}",
                if s { "negative" } else { "positive" }
            );

            results.par_extend((0u32..(1 << 23)).into_par_iter().filter_map(|m| {
                let m = m | (1 << 23);
                let mut x = rug::Float::with_val(24, m) << (e - 23);
                if s {
                    x = -x;
                }
                if f(&x) { Some((s, e, m)) } else { None }
            }));

            eprintln!("{} value(s) so far", results.len());
        }
    }

    results.par_sort_by_key(|&(s, e, m)| (e, m, s));

    println!("# Generated file. Do not edit manually.");
    print_f32_list(results.iter().copied());
}

fn check_hard_round_f16(v: rug::Float, n: u16) -> bool {
    assert!(v.prec() >= 11 + u32::from(n));

    let Some(e) = v.get_exp() else {
        return false;
    };
    let e = e - 1;
    if !matches!(e, -26..=15) {
        return false;
    }

    let v = v.abs() >> (e.max(-14) - 10);

    check_hard_round_common(v, n)
}

fn check_hard_round_f32(v: rug::Float, n: u16) -> bool {
    assert!(v.prec() >= 24 + u32::from(n));

    let Some(e) = v.get_exp() else {
        return false;
    };
    let e = e - 1;
    if !matches!(e, -151..=127) {
        return false;
    }

    let v = v.abs() >> (e.max(-126) - 23);

    check_hard_round_common(v, n)
}

fn check_hard_round_common(v: rug::Float, n: u16) -> bool {
    let mut v = v.fract();

    v <<= 2;
    let bits01 = v.to_u32_saturating_round(rug::float::Round::Zero).unwrap();
    //let bit0 = (bits01 & 0b10) != 0;
    let bit1 = (bits01 & 0b01) != 0;

    v = v.fract();

    for _ in 2..n {
        v <<= 1;
        let bit = v.to_u32_saturating_round(rug::float::Round::Zero).unwrap() != 0;
        if bit != bit1 {
            return false;
        }
        v = v.fract();
    }

    true
}

fn print_2xf16_list(values: impl Iterator<Item = ((bool, i32, u16), (bool, i32, u16))>) {
    for ((s1, e1, m1), (s2, e2, m2)) in values {
        print_f16(s1, e1, m1);
        print!(",");
        print_f16(s2, e2, m2);
        println!();
    }
}

fn print_f16(s: bool, e: i32, m: u16) {
    let mf = (m & ((1 << 10) - 1)) << 2;
    let sgn = if s { "-" } else { "+" };
    print!("{sgn}0x1.{mf:03x}p{e:+}");
}

fn print_f32_list(values: impl Iterator<Item = (bool, i32, u32)>) {
    for (s, e, m) in values {
        print_f32(s, e, m);
        println!();
    }
}

fn print_f32(s: bool, e: i32, m: u32) {
    let mf = (m & ((1 << 23) - 1)) << 1;
    let sgn = if s { "-" } else { "+" };
    print!("{sgn}0x1.{mf:06x}p{e:+}");
}
