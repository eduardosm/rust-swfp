use super::{flt2dec, fmt, numfmt};

pub(crate) trait GeneralFormat: PartialOrd {
    /// Determines if a value should use exponential based on its magnitude, given the precondition
    /// that it will not be rounded any further before it is displayed.
    fn already_rounded_value_should_use_exponential(&self) -> bool;
}

// Don't inline this so callers don't use the stack space this function
// requires unless they have to.
#[inline(never)]
fn float_to_decimal_common_exact<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
    sign: flt2dec::Sign,
    precision: usize,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    let mut buf: [u8; 1024] = [0; 1024]; // enough for f32 and f64
    let mut parts: [numfmt::Part<'_>; 4] = [numfmt::Part::Zero(0); 4];
    let formatted = flt2dec::to_exact_fixed_str(
        if T::IS_LARGE {
            flt2dec::strategy::dragon_large::format_exact
        } else {
            flt2dec::strategy::grisu::format_exact
        },
        *num,
        sign,
        precision,
        &mut buf,
        &mut parts,
    );
    fmt::pad_formatted_parts(fmt, &formatted)
}

// Don't inline this so callers that call both this and the above won't wind
// up using the combined stack space of both functions in some cases.
#[inline(never)]
fn float_to_decimal_common_shortest<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
    sign: flt2dec::Sign,
    precision: u16,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    // enough for f32 and f64
    let mut buf: [u8; flt2dec::MAX_SIG_DIGITS] = [0; flt2dec::MAX_SIG_DIGITS];
    let mut parts: [numfmt::Part<'_>; 4] = [numfmt::Part::Zero(0); 4];
    let formatted = flt2dec::to_shortest_str(
        if T::IS_LARGE {
            flt2dec::strategy::dragon_large::format_shortest
        } else {
            flt2dec::strategy::grisu::format_shortest
        },
        *num,
        sign,
        precision.into(),
        &mut buf,
        &mut parts,
    );
    fmt::pad_formatted_parts(fmt, &formatted)
}

pub(crate) fn float_to_decimal_display<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    let force_sign = fmt.sign_plus();
    let sign = match force_sign {
        false => flt2dec::Sign::Minus,
        true => flt2dec::Sign::MinusPlus,
    };

    if let Some(precision) = fmt.precision() {
        float_to_decimal_common_exact(fmt, num, sign, precision)
    } else {
        let min_precision = 0;
        float_to_decimal_common_shortest(fmt, num, sign, min_precision)
    }
}

// Don't inline this so callers don't use the stack space this function
// requires unless they have to.
#[inline(never)]
fn float_to_exponential_common_exact<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
    sign: flt2dec::Sign,
    precision: usize,
    upper: bool,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    let mut buf: [u8; 1024] = [0; 1024]; // enough for f32 and f64
    let mut parts: [numfmt::Part<'_>; 6] = [numfmt::Part::Zero(0); 6];
    let formatted = flt2dec::to_exact_exp_str(
        if T::IS_LARGE {
            flt2dec::strategy::dragon_large::format_exact
        } else {
            flt2dec::strategy::grisu::format_exact
        },
        *num,
        sign,
        precision,
        upper,
        &mut buf,
        &mut parts,
    );
    fmt::pad_formatted_parts(fmt, &formatted)
}

// Don't inline this so callers that call both this and the above won't wind
// up using the combined stack space of both functions in some cases.
#[inline(never)]
fn float_to_exponential_common_shortest<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
    sign: flt2dec::Sign,
    upper: bool,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    // enough for f32 and f64
    let mut buf: [u8; flt2dec::MAX_SIG_DIGITS] = [0; flt2dec::MAX_SIG_DIGITS];
    let mut parts: [numfmt::Part<'_>; 6] = [numfmt::Part::Zero(0); 6];
    let formatted = flt2dec::to_shortest_exp_str(
        if T::IS_LARGE {
            flt2dec::strategy::dragon_large::format_shortest
        } else {
            flt2dec::strategy::grisu::format_shortest
        },
        *num,
        sign,
        (0, 0),
        upper,
        &mut buf,
        &mut parts,
    );
    fmt::pad_formatted_parts(fmt, &formatted)
}

// Common code of floating point LowerExp and UpperExp.
pub(crate) fn float_to_exponential_common<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
    upper: bool,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat,
{
    let force_sign = fmt.sign_plus();
    let sign = match force_sign {
        false => flt2dec::Sign::Minus,
        true => flt2dec::Sign::MinusPlus,
    };

    if let Some(precision) = fmt.precision() {
        // 1 integral digit + `precision` fractional digits = `precision + 1` total digits
        float_to_exponential_common_exact(fmt, num, sign, precision + 1, upper)
    } else {
        float_to_exponential_common_shortest(fmt, num, sign, upper)
    }
}

pub(crate) fn float_to_general_debug<T>(
    fmt: &mut core::fmt::Formatter<'_>,
    num: &T,
) -> core::fmt::Result
where
    T: flt2dec::DecodableFloat + GeneralFormat,
{
    let force_sign = fmt.sign_plus();
    let sign = match force_sign {
        false => flt2dec::Sign::Minus,
        true => flt2dec::Sign::MinusPlus,
    };

    if let Some(precision) = fmt.precision() {
        // this behavior of {:.PREC?} predates exponential formatting for {:?}
        float_to_decimal_common_exact(fmt, num, sign, precision)
    } else {
        // since there is no precision, there will be no rounding
        if num.already_rounded_value_should_use_exponential() {
            let upper = false;
            float_to_exponential_common_shortest(fmt, num, sign, upper)
        } else {
            let min_precision = 1;
            float_to_decimal_common_shortest(fmt, num, sign, min_precision)
        }
    }
}
