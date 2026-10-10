# Changelog

## 0.2.0 (unreleased)

### Breaking

- `FpStatus` is now a set of flags instead of an enum, so more than one flag
  can be reported by an operation. The enum variants have been replaced with
  the associated constants `OK`, `INVALID`, `DIV_BY_ZERO`, `OVERFLOW`,
  `UNDERFLOW` and `INEXACT`.
- Status flags now follow IEEE 754 semantics:
  - `OVERFLOW` and `UNDERFLOW` are always reported together with `INEXACT`.
  - Tininess is detected after rounding, so `UNDERFLOW` is also reported for
    some inexact results that are rounded to the smallest normal value.
  - Conversions to integer report `INVALID` (instead of overflow) when the
    input is infinity or out of range.
  - Converting negative zero to integer does not report `INEXACT` anymore.

### Changed

- Performance of floating point parsing has been improved.

### Fixed

- Fixed `from_str` returning infinity or zero for numbers with a very large
  number of leading or trailing zeros that are cancelled out by the exponent.

## 0.1.1 (2026-10-05)

### Fixed

- Fixed some exponent overflow cases resulting in a panic (in debug builds) or
  zero (release builds)
- Fixed rounding functions returning a malformed value when rounding a value
  whose magnitude is between 0.5 and 1
- Fixed panic when formatting `F128` and `X87F80` with a fixed number of digits
- Fixed `F128` being formatted with lowest digits replaced with zeros in some
  cases.
- `inf - (-inf)` now returns `FpStatus::Ok` instead of `FpStatus::Invalid`
- Fixed `from_int` not rounding correctly when the input is negative and rounding
  direction is `TowardPositive` or `TowardNegative`
- Poles and zeros of `tand` and `tanpi` now have alternating signs
- Fixed integer overflow reachable by `F32::pow` with some values
- Fixed `F32::atan2d` and `F32::atan2pi` producing incorrectly rounded results
  for some small `y/x` ratios
- Fixed `F64::ln_gamma` producing incorrectly rounded results for some inputs
  close to the negative roots

## 0.1.0 (2026-04-02)

- Initial release
