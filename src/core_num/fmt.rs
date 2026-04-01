//! Utilities for formatting and printing strings.

use core::fmt::Write as _;

use super::numfmt;

/// Takes the formatted parts and applies the padding.
///
/// Assumes that the caller already has rendered the parts with required precision,
/// so that `self.precision` can be ignored.
pub(super) fn pad_formatted_parts(
    fmt: &mut core::fmt::Formatter<'_>,
    formatted: &numfmt::Formatted<'_>,
) -> core::fmt::Result {
    if let Some(mut width) = fmt.width() {
        // for the sign-aware zero padding, we render the sign first and
        // behave as if we had no sign from the beginning.
        let mut formatted = formatted.clone();
        let mut fill = fmt.fill();
        let mut align = fmt.align();
        if fmt.sign_aware_zero_pad() {
            // a sign always goes first
            let sign = formatted.sign;
            fmt.write_str(sign)?;

            // remove the sign from the formatted parts
            formatted.sign = "";
            width = width.saturating_sub(sign.len());
            fill = '0';
            align = Some(core::fmt::Alignment::Right);
        }

        // remaining parts go through the ordinary padding process.
        let len = formatted.len();
        if width <= len {
            // no padding
            write_formatted_parts(fmt, &formatted)
        } else {
            let post_padding = padding(fmt, width - len, core::fmt::Alignment::Right, align, fill)?;
            write_formatted_parts(fmt, &formatted)?;
            post_padding.write(fmt)
        }
    } else {
        // this is the common case and we take a shortcut
        write_formatted_parts(fmt, formatted)
    }
}

pub(super) fn write_formatted_parts(
    fmt: &mut core::fmt::Formatter<'_>,
    formatted: &numfmt::Formatted<'_>,
) -> core::fmt::Result {
    if !formatted.sign.is_empty() {
        fmt.write_str(formatted.sign)?;
    }
    for part in formatted.parts {
        match *part {
            numfmt::Part::Zero(mut nzeroes) => {
                const ZEROES: &str = // 64 zeroes
                    "0000000000000000000000000000000000000000000000000000000000000000";
                while nzeroes > ZEROES.len() {
                    fmt.write_str(ZEROES)?;
                    nzeroes -= ZEROES.len();
                }
                if nzeroes > 0 {
                    fmt.write_str(&ZEROES[..nzeroes])?;
                }
            }
            numfmt::Part::Num(mut v) => {
                let mut s = [0; 5];
                let len = part.len();
                for c in s[..len].iter_mut().rev() {
                    *c = b'0' + (v % 10) as u8;
                    v /= 10;
                }
                fmt.write_str(core::str::from_utf8(&s[..len]).unwrap())?;
            }
            numfmt::Part::Copy(buf) => fmt.write_str(buf)?,
        }
    }
    Ok(())
}

/// Padding after the end of something. Returned by `Formatter::padding`.
#[must_use = "don't forget to write the post padding"]
pub(crate) struct PostPadding {
    fill: char,
    padding: usize,
}

impl PostPadding {
    fn new(fill: char, padding: usize) -> PostPadding {
        PostPadding { fill, padding }
    }

    /// Writes this post padding.
    pub(crate) fn write(self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for _ in 0..self.padding {
            f.write_char(self.fill)?;
        }
        Ok(())
    }
}

/// Writes the pre-padding and returns the unwritten post-padding.
///
/// Callers are responsible for ensuring post-padding is written after the
/// thing that is being padded.
pub(crate) fn padding(
    fmt: &mut core::fmt::Formatter<'_>,
    padding: usize,
    default: core::fmt::Alignment,
    align: Option<core::fmt::Alignment>,
    fill: char,
) -> Result<PostPadding, core::fmt::Error> {
    let align = align.unwrap_or(default);

    let padding_left = match align {
        core::fmt::Alignment::Left => 0,
        core::fmt::Alignment::Right => padding,
        core::fmt::Alignment::Center => padding / 2,
    };

    for _ in 0..padding_left {
        fmt.write_char(fill)?;
    }

    Ok(PostPadding::new(fill, padding - padding_left))
}
