use std::fmt::Write as _;

use crate::RunError;

mod approx;
mod arg_utils;
mod consts;
mod julia;
mod reduce_pi_2;
mod sollya;

pub(super) fn proc_path(path: &std::path::Path) -> Result<(), RunError> {
    let meta = path.metadata().map_err(|e| {
        eprintln!("Failed to get metadata of {path:?}: {e}");
        RunError
    })?;

    if meta.is_dir() {
        proc_dir(path)
    } else {
        proc_file(path)
    }
}

fn proc_dir(path: &std::path::Path) -> Result<(), RunError> {
    let dir = path.read_dir().map_err(|e| {
        eprintln!("Failed to read directory {path:?}: {e}");
        RunError
    })?;
    for entry in dir {
        let entry = entry.map_err(|e| {
            eprintln!("Failed to read directory {path:?}: {e}");
            RunError
        })?;
        proc_path(&entry.path())?;
    }
    Ok(())
}

fn proc_file(path: &std::path::Path) -> Result<(), RunError> {
    if path.extension() != Some(std::ffi::OsStr::new("rs")) {
        return Ok(());
    }

    eprintln!("Processing file {path:?}");

    let file_data = std::fs::read_to_string(path).map_err(|e| {
        eprintln!("Failed to read file {path:?}: {e}");
        RunError
    })?;
    let mut result = String::new();

    let mut num_generates = 0usize;
    let mut lines = file_data.lines();
    while let Some(line) = lines.next() {
        result.push_str(line);
        result.push('\n');
        if let Some((ident, txt)) = check_comment(line, check_generate) {
            let end_line = lines
                .find_map(|line| {
                    let line_ident_len = line.find(|c| c != ' ').unwrap_or(0);
                    if line_ident_len < ident.len() || line.trim_matches(' ').is_empty() {
                        Some(Ok(line))
                    } else if check_comment(line, check_generate).is_some() {
                        eprintln!("Another \"GENERATE\" found before expected");
                        Some(Err(RunError))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    eprintln!("End line not found for \"GENERATE\" block");
                    RunError
                })??;
            eprintln!("GENERATE: {}", txt.trim());
            let generated = generate(txt)?;
            for generated_line in generated.lines() {
                result.push_str(ident);
                result.push_str(generated_line);
                result.push('\n');
            }
            result.push_str(end_line);
            result.push('\n');
            num_generates += 1;
        }
    }

    let result = pass_through_rustfmt(&result)?;
    std::fs::write(path, result).map_err(|e| {
        eprintln!("Failed to write file {path:?}: {e}");
        RunError
    })?;

    eprintln!("Processed {num_generates} \"GENERATE\" blocks");

    Ok(())
}

fn check_comment<'a, R: 'a>(
    line: &'a str,
    f: impl FnOnce(&'a str) -> Option<R>,
) -> Option<(&'a str, R)> {
    let no_sp_i = line.find(|c| c != ' ')?;
    let (ident, txt) = line.split_at(no_sp_i);
    let txt = txt.strip_prefix("//")?.trim_start_matches(' ');
    f(txt).map(|r| (ident, r))
}

fn check_generate(txt: &str) -> Option<&str> {
    txt.strip_prefix("GENERATE:")
}

fn pass_through_rustfmt(src: &str) -> Result<String, RunError> {
    use std::io::{Read as _, Write as _};

    let mut child = std::process::Command::new("rustfmt")
        .args(["--edition", "2024"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            eprintln!("Failed to spawn rustfmt: {e}");
            RunError
        })?;

    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let stdout_reader = std::thread::spawn(move || {
        let mut data = String::new();
        child_stdout.read_to_string(&mut data)?;
        Ok::<_, std::io::Error>(data)
    });
    child_stdin.write_all(src.as_bytes()).map_err(|e| {
        eprintln!("Failed to write to rustfmt stdin: {e}");
        RunError
    })?;
    child_stdin.flush().map_err(|e| {
        eprintln!("Failed to write to rustfmt stdin: {e}");
        RunError
    })?;
    drop(child_stdin);

    let formatted = stdout_reader
        .join()
        .expect("read thread panicked")
        .map_err(|e| {
            eprintln!("Failed to read from rustfmt stdout: {e}");
            RunError
        })?;

    let status = child.wait().map_err(|e| {
        eprintln!("Failed to wait for rustfmt: {e}");
        RunError
    })?;
    if !status.success() {
        eprintln!("rustfmt failed: {status}");
        return Err(RunError);
    }

    Ok(formatted)
}

fn generate(param: &str) -> Result<String, RunError> {
    let mut args = param.split_ascii_whitespace();
    let cmd = args.next().ok_or_else(|| {
        eprintln!("Empty generate parameter");
        RunError
    })?;
    let args = args.collect::<Vec<_>>();

    let r = match cmd {
        "consts" => consts::gen_consts(&args),

        "reduce_pi_2::medium_consts" => reduce_pi_2::gen_medium_consts(&args),
        "reduce_pi_2::frac_2_pi_large" => reduce_pi_2::gen_frac_2_pi_large(&args),
        "reduce_pi_2::frac_pi_2_medium" => reduce_pi_2::gen_frac_pi_2_medium(&args),

        "inv_cbrt_poly" => approx::gen_inv_cbrt_poly(&args),
        "exp_m1_poly" => approx::gen_exp_m1_poly(&args),
        "exp_table" => approx::gen_exp_table(&args),
        "ln_1p_poly" => approx::gen_ln_1p_poly(&args),
        "ln_table" => approx::gen_ln_table(&args),
        "ln_lo_scale_table" => approx::gen_ln_lo_scale_table(&args),
        "sin_poly" => approx::gen_sin_poly(&args),
        "cos_poly" => approx::gen_cos_poly(&args),
        "tan_poly" => approx::gen_tan_poly(&args),
        "asin_poly" => approx::gen_asin_poly(&args),
        "atan_poly" => approx::gen_atan_poly(&args),
        "asinh_poly" => approx::gen_asinh_poly(&args),
        "gamma_poly" => approx::gen_gamma_poly(&args),
        "ln_gamma_poly" => approx::gen_ln_gamma_poly(&args),
        "gamma_lanczos_poly" => approx::gen_gamma_lanczos_poly(&args),

        _ => {
            eprintln!("Invalid generate command: {cmd:?}");
            return Err(RunError);
        }
    };

    r.map_err(|e| {
        eprintln!("Error generating {cmd:?}: {e}");
        RunError
    })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AuxFloatKind {
    SfpM16E8,
    SfpM32E16,
    SfpM64E16,
    SfpM128E16,
    SfpM192E16,
}

impl std::str::FromStr for AuxFloatKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SfpM16E8" => Ok(Self::SfpM16E8),
            "SfpM32E16" => Ok(Self::SfpM32E16),
            "SfpM64E16" => Ok(Self::SfpM64E16),
            "SfpM128E16" => Ok(Self::SfpM128E16),
            "SfpM192E16" => Ok(Self::SfpM192E16),
            _ => Err("invalid float kind"),
        }
    }
}

impl AuxFloatKind {
    fn rug_prec(self) -> u32 {
        match self {
            Self::SfpM16E8 => 64,
            Self::SfpM32E16 => 128,
            Self::SfpM64E16 => 256,
            Self::SfpM128E16 => 512,
            Self::SfpM192E16 => 768,
        }
    }

    fn prec(self) -> u32 {
        match self {
            Self::SfpM16E8 => 16,
            Self::SfpM32E16 => 32,
            Self::SfpM64E16 => 64,
            Self::SfpM128E16 => 128,
            Self::SfpM192E16 => 192,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::SfpM16E8 => "SfpM16E8",
            Self::SfpM32E16 => "SfpM32E16",
            Self::SfpM64E16 => "SfpM64E16",
            Self::SfpM128E16 => "SfpM128E16",
            Self::SfpM192E16 => "SfpM192E16",
        }
    }
}

fn render_aux_const(fkind: AuxFloatKind, name: &str, val: rug::Float, out: &mut String) {
    let type_ = fkind.name();

    write!(out, "const {name}: {type_} = ").unwrap();
    render_aux_const_value(fkind, &val, out);
    out.push_str("; // ");
    render_aux_const_dec_value(fkind, &val, out);
    out.push('\n');
}

fn render_aux_const_value(fkind: AuxFloatKind, val: &rug::Float, out: &mut String) {
    assert!(val.is_finite());
    if val.is_zero() {
        out.push_str("Sfp::ZERO");
    } else {
        let new_fn = if val.is_sign_positive() {
            "newp"
        } else {
            "newn"
        };
        let exp = val.get_exp().unwrap() - 1;
        write!(out, "Sfp::{new_fn}({exp}, ").unwrap();

        if fkind != AuxFloatKind::SfpM192E16 {
            let prec = fkind.prec();
            let tmp = val.clone().abs() >> (exp - (prec - 1) as i32);
            let mant = tmp.to_integer().unwrap().to_u128().unwrap();
            let hex_width = (prec / 4) as usize;
            write!(out, "0x{mant:0hex_width$X}").unwrap();
        } else {
            let prec_hi = 64;
            let prec_lo = 128;

            let mut tmp = val.clone().abs() >> (exp - (prec_hi - 1));
            let mant_hi = tmp
                .to_integer_round(rug::float::Round::Zero)
                .unwrap()
                .0
                .to_u64()
                .unwrap();
            tmp -= &mant_hi;
            tmp <<= prec_lo;
            let mant_lo = tmp.to_integer().unwrap().to_u128().unwrap();

            write!(out, "u192(0x{mant_hi:016X}, 0x{mant_lo:032X})").unwrap();
        }

        out.push(')');
    }
}

fn render_aux_const_dec_value(fkind: AuxFloatKind, val: &rug::Float, out: &mut String) {
    assert!(val.is_finite());
    if val.is_zero() {
        out.push_str("0.0");
    } else {
        let prec = fkind.prec();
        let dec_prec = ((prec as f64) * std::f64::consts::LOG10_2) as usize;
        write!(out, "{val:.dec_prec$e}").unwrap();
    }
}
