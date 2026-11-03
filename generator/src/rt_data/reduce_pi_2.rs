use std::fmt::Write as _;

use super::{AuxFloatKind, arg_utils, render_aux_const};

pub(super) fn gen_medium_consts(args: &[&str]) -> Result<String, String> {
    let (fkind, fkind_ex): (AuxFloatKind, AuxFloatKind) = arg_utils::parse_2_args(args)?;
    let rug_prec = fkind_ex.rug_prec();

    let mut out = String::new();

    // 2/π
    let tmp = 2 / rug::Float::with_val(rug_prec, rug::float::Constant::Pi);
    render_aux_const(fkind_ex, "FRAC_2_PI", tmp, &mut out);

    // π/2
    let tmp = rug::Float::with_val(rug_prec * 2, rug::float::Constant::Pi) / 2;

    let (p0base, p0ex) = split_ex(tmp, fkind.prec());
    render_aux_const(fkind, "FRAC_PI_2_P0", p0base, &mut out);
    render_aux_const(fkind_ex, "FRAC_PI_2_P0EX", p0ex.clone(), &mut out);

    Ok(out)
}

fn split_ex(tmp: rug::Float, base_prec: u32) -> (rug::Float, rug::Float) {
    let (base, _) = rug::Float::with_val_round(base_prec, &tmp, rug::float::Round::Zero);
    let ex = tmp - &base;
    (base, ex)
}

/// Bits of `2 / π`, in 64-bit words, most significant first:
/// `FRAC_2_PI_BITS[i]` holds bits `64*i + 1 ..= 64*i + 64` of the binary
/// expansion (i.e., `2 / π ~= sum(FRAC_2_PI_BITS[i] * 2^(-64 * (i + 1)))`).
pub(super) fn gen_frac_2_pi_bits(args: &[&str]) -> Result<String, String> {
    arg_utils::expect_0_args(args)?;

    let mut out = String::new();

    let num_words = 20;
    let mut tmp = 2u8 / rug::Float::with_val((num_words + 2) * 64, rug::float::Constant::Pi);

    writeln!(out, "static FRAC_2_PI_BITS: [u64; {num_words}] = [").unwrap();
    for _ in 0..num_words {
        tmp <<= 64;
        let word = tmp
            .to_integer_round(rug::float::Round::Zero)
            .unwrap()
            .0
            .to_u64()
            .unwrap();
        writeln!(out, "    0x{word:016X},").unwrap();
        tmp -= word;
    }
    out.push_str("];\n");

    Ok(out)
}
