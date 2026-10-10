use std::io::BufRead as _;

pub(crate) trait LineData: Sized {
    fn parse_line(s: &str) -> Result<Self, String>;
}

impl<T: ItemData> LineData for T {
    fn parse_line(s: &str) -> Result<Self, String> {
        <[T; 1]>::parse_line(s).map(|[value]| value)
    }
}

pub(crate) trait ItemData: Sized {
    fn parse_item(s: &str) -> Result<Self, String>;
}

impl<T: ItemData, const N: usize> LineData for [T; N] {
    fn parse_line(line: &str) -> Result<Self, String> {
        let mut values = std::array::from_fn(|_| None);
        let mut s = line;

        // Use `std::array::try_from_fn` when stable.
        for (i, item) in values.iter_mut().enumerate() {
            if i != 0 {
                s = s
                    .trim_start()
                    .strip_prefix(',')
                    .ok_or_else(|| format!("too few values in {line:?}"))?;
            }
            let (token, rest) = split_token(s.trim_start());
            *item = Some(T::parse_item(token)?);
            s = rest;
        }

        // Like core-math's own `sscanf`-based reader, ignore anything after
        // the last value (e.g., `0x1.4f1d73be27a31p+1 44` or `-0x8p-972,0x4p-128):
        // Exception` in core-math data), but still reject extra values.
        if s.trim_start().starts_with(',') {
            return Err(format!("too many values in {line:?}"));
        }

        Ok(values.map(Option::unwrap))
    }
}

/// Splits `s` into the leading token that can form a number and the rest.
fn split_token(s: &str) -> (&str, &str) {
    let end = s
        .find(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '.' | '+' | '-'))
        .unwrap_or(s.len());
    s.split_at(end)
}

impl ItemData for swfp::F16 {
    fn parse_item(s: &str) -> Result<Self, String> {
        parse_float(s).map_err(|e| format!("failed to parse float {s:?}: {e}"))
    }
}

impl ItemData for swfp::F32 {
    fn parse_item(s: &str) -> Result<Self, String> {
        parse_float(s).map_err(|e| format!("failed to parse float {s:?}: {e}"))
    }
}

impl ItemData for swfp::F64 {
    fn parse_item(s: &str) -> Result<Self, String> {
        parse_float(s).map_err(|e| format!("failed to parse float {s:?}: {e}"))
    }
}

pub(crate) fn read_data_file<T: LineData>(path: &str) -> impl Iterator<Item = T> {
    read_lines(path).map(|line| {
        T::parse_line(&line).unwrap_or_else(|e| {
            panic!("failed to parse line {line:?}: {e}");
        })
    })
}

fn read_lines(data_path: &str) -> impl Iterator<Item = String> {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test-data");
    path.push(data_path);

    let file =
        std::fs::File::open(&path).unwrap_or_else(|e| panic!("failed to open {path:?}: {e}"));
    let mut file = std::io::BufReader::new(file);
    let mut finished = false;

    std::iter::from_fn(move || {
        let mut line = String::new();
        loop {
            if finished {
                return None;
            }

            line.clear();
            file.read_line(&mut line).unwrap_or_else(|e| {
                panic!("failed to read line from {path:?}: {e}");
            });

            if !line.ends_with('\n') {
                finished = true;
            }

            if let Some(hash_pos) = line.find('#') {
                line.truncate(hash_pos);
            }
            let line = line.trim();
            if !line.is_empty() {
                return Some(line.into());
            }
        }
    })
}

fn parse_float<T: swfp::Float>(s: &str) -> Result<T, &'static str> {
    let (s, neg) = if let Some(s) = s.strip_prefix('-') {
        (s, true)
    } else if let Some(s) = s.strip_prefix('+') {
        (s, false)
    } else {
        (s, false)
    };

    let value = if s == "nan" || s == "qnan" || s == "snan" {
        T::NAN
    } else if s == "inf" {
        T::INFINITY
    } else if let Some(s) = s.strip_prefix("0x") {
        let s = s.as_bytes();
        let mut i = 0;

        if i == s.len() {
            return Err("missing digits");
        }

        // Parse digits before the decimal point
        let mut mant = 0u128;
        while i < s.len() {
            if let Some(digit) = char::from(s[i]).to_digit(16) {
                mant = mant.checked_mul(16).ok_or("mantissa overflow")? | u128::from(digit);
                i += 1;
            } else {
                break;
            }
        }

        // Parse digits after the decimal point
        let mut num_frac_digits = 0;
        if i < s.len() && s[i] == b'.' {
            i += 1;
            while i < s.len() {
                if let Some(digit) = char::from(s[i]).to_digit(16) {
                    mant = mant.checked_mul(16).ok_or("mantissa overflow")? | u128::from(digit);
                    num_frac_digits += 1;
                    i += 1;
                } else {
                    break;
                }
            }
        }

        // Parse exponent
        let mut exp = 0i32;
        if i < s.len() && s[i] == b'p' {
            i += 1;
            let mut neg_exp = false;
            if i < s.len() {
                if s[i] == b'-' {
                    neg_exp = true;
                    i += 1;
                } else if s[i] == b'+' {
                    i += 1;
                }
            }
            let exp_start = i;
            while i < s.len() {
                if let Some(digit) = char::from(s[i]).to_digit(10) {
                    exp = exp
                        .checked_mul(10)
                        .ok_or("exponent overflow")?
                        .checked_add(digit as i32)
                        .ok_or("exponent overflow")?;
                    i += 1;
                } else {
                    break;
                }
            }
            if i == exp_start {
                return Err("missing exponent digits");
            }
            if neg_exp {
                exp = -exp;
            }

            // Allow C `float` literal suffix (e.g., `0x1.ffff36p-1f`)
            if i < s.len() && matches!(s[i], b'f' | b'F') {
                i += 1;
            }
        }
        exp = exp
            .checked_sub(num_frac_digits * 4)
            .ok_or("exponent overflow")?;

        if i != s.len() {
            return Err("extra characters");
        }

        // Values that are not exactly representable (e.g., `0x11.e44c76af19d67p+1`
        // in binary64) are rounded to nearest. This is done in two steps, so fail
        // if both of them round to avoid double rounding.
        let (value, status1) = T::from_uint_ex(mant, swfp::Round::NearestTiesToEven);
        if !matches!(status1, swfp::FpStatus::OK | swfp::FpStatus::INEXACT) {
            return Err("from u128 not ok");
        }

        let (value, status2) = value.scalbn_ex(exp, swfp::Round::NearestTiesToEven);
        if status2.contains(swfp::FpStatus::OVERFLOW) {
            return Err("scalbn not ok");
        } else if status2 != swfp::FpStatus::OK && status1 != swfp::FpStatus::OK {
            return Err("double rounding");
        }
        value
    } else {
        s.parse().map_err(|_| "invalid value")?
    };

    Ok(if neg { -value } else { value })
}

#[cfg(test)]
mod tests {
    use swfp::Float as _;

    use super::{LineData as _, parse_float};

    #[test]
    fn test_parse_line() {
        use swfp::F64;

        assert_eq!(
            F64::parse_line("0x1p0"),
            Ok(F64::from_bits(0x3FF0000000000000)),
        );
        assert_eq!(
            F64::parse_line("0x1.4f1d73be27a31p+1 44"),
            Ok(F64::from_bits(0x4004F1D73BE27A31)),
        );
        assert_eq!(
            <[F64; 2]>::parse_line("0x1p0,-0x1p1"),
            Ok([
                F64::from_bits(0x3FF0000000000000),
                F64::from_bits(0xC000000000000000),
            ]),
        );
        assert_eq!(
            <[F64; 2]>::parse_line("0x1p0 , 1.25"),
            Ok([
                F64::from_bits(0x3FF0000000000000),
                F64::from_bits(0x3FF4000000000000),
            ]),
        );
        assert_eq!(
            <[F64; 2]>::parse_line("-0x8p-972,0x4p-128): Exception"),
            Ok([
                F64::from_bits(0x8360000000000000),
                F64::from_bits(0x3810000000000000),
            ]),
        );
        assert!(<[F64; 2]>::parse_line("0x1p0").is_err());
        assert!(<[F64; 2]>::parse_line("0x1p0 0x1p1").is_err());
        assert!(<[F64; 2]>::parse_line("0x1p0,0x1p1,0x1p2").is_err());
        assert!(<[F64; 2]>::parse_line("0x1p0,").is_err());
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(
            parse_float("0x1.3ff4db2640b7fp12"),
            Ok(swfp::F64::from_bits(0x40B3FF4DB2640B7F)),
        );
        assert_eq!(
            parse_float("0x1.3ff4db2640b7fp+12"),
            Ok(swfp::F64::from_bits(0x40B3FF4DB2640B7F)),
        );
        assert_eq!(
            parse_float("0x1.a1cb9d879986bp-15"),
            Ok(swfp::F64::from_bits(0x3F0A1CB9D879986B)),
        );
        assert_eq!(
            parse_float("0x1.ffff36p-1f"),
            Ok(swfp::F32::from_bits(0x3F7FFF9B)),
        );
        assert_eq!(
            parse_float("0x11.e44c76af19d67p+1"),
            Ok(swfp::F64::from_bits(0x4041E44C76AF19D6)),
        );
        assert_eq!(
            parse_float("0x1.8p-1074"),
            Ok(swfp::F64::from_bits(0x0000000000000002)),
        );
        assert!(parse_float::<swfp::F64>("0x1.00000000000018p-1060").is_err());
        assert!(parse_float::<swfp::F64>("-qnan").unwrap().is_nan());
        assert!(parse_float::<swfp::F64>("0x1p").is_err());
        assert!(parse_float::<swfp::F64>("0x1pf").is_err());
    }
}
