use std::io::BufRead as _;

pub(crate) trait LineData: Sized {
    fn parse_line(s: &str) -> Result<Self, String>;
}

impl<T: ItemData> LineData for T {
    fn parse_line(s: &str) -> Result<Self, String> {
        T::parse_item(s)
    }
}

pub(crate) trait ItemData: Sized {
    fn parse_item(s: &str) -> Result<Self, String>;
}

impl<T: ItemData, const N: usize> LineData for [T; N] {
    fn parse_line(s: &str) -> Result<Self, String> {
        let mut values = std::array::from_fn(|_| None);
        let mut split = s.split(',');

        // Use `std::array::try_from_fn` when stable.
        for item in values.iter_mut() {
            if let Some(part) = split.next() {
                *item = Some(T::parse_item(part.trim())?);
            } else {
                return Err(format!("too few values in {s:?}"));
            }
        }

        if split.next().is_some() {
            return Err(format!("too many values in {s:?}"));
        }

        Ok(values.map(Option::unwrap))
    }
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

    let value = if s == "nan" || s == "snan" {
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
            if i == s.len() {
                return Err("missing exponent digits");
            }
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
            if neg_exp {
                exp = -exp;
            }
        }
        exp = exp
            .checked_sub(num_frac_digits * 4)
            .ok_or("exponent overflow")?;

        if i != s.len() {
            return Err("extra characters");
        }

        let (value, status) = T::from_uint_ex(mant, swfp::Round::TowardZero);
        if status != swfp::FpStatus::Ok {
            return Err("from u128 not ok");
        }

        let (value, status) = value.scalbn_ex(exp, swfp::Round::NearestTiesToEven);
        if !matches!(
            status,
            swfp::FpStatus::Ok | swfp::FpStatus::Underflow | swfp::FpStatus::Inexact
        ) {
            return Err("scalbn not ok");
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

    use super::parse_float;

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
    }
}
