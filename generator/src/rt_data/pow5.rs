use std::fmt::Write as _;

use super::arg_utils;

pub(super) fn gen_pow5_table(args: &[&str]) -> Result<String, String> {
    let (step, min_i, max_i): (u32, i32, i32) = arg_utils::parse_3_args(args)?;
    if min_i > max_i {
        return Err("empty range".into());
    }

    let mut out = String::new();

    let num = max_i - min_i + 1;
    let offset = -min_i;
    let q_desc = if offset >= 0 {
        format!("{step} * (i - {offset})")
    } else {
        format!("{step} * (i + {})", -offset)
    };
    writeln!(
        out,
        "// POW5_TABLE[i] = floor(5^q * 2^(127 - floor(q * log2(5)))), where q = {q_desc}",
    )
    .unwrap();
    writeln!(out, "static POW5_TABLE: [u128; {num}] = [").unwrap();
    for i in min_i..=max_i {
        let q = i64::from(step) * i64::from(i);
        let n = u32::try_from(q.unsigned_abs()).unwrap();
        let pow5 = rug::Integer::from(rug::Integer::u_pow_u(5, n));
        let pow5_bits = pow5.significant_bits();
        let t = if q >= 0 {
            // floor(q * log2(5)) = pow5_bits - 1
            if pow5_bits >= 128 {
                pow5 >> (pow5_bits - 128)
            } else {
                pow5 << (128 - pow5_bits)
            }
        } else {
            // floor(q * log2(5)) = -pow5_bits, since 5^-q is not a power of two
            (rug::Integer::from(1u8) << (127 + pow5_bits)) / pow5
        };
        assert_eq!(t.significant_bits(), 128);
        let t = t.to_u128().unwrap();
        writeln!(out, "    0x{t:032X}, // 5^{q}").unwrap();
    }
    writeln!(out, "];").unwrap();

    Ok(out)
}
