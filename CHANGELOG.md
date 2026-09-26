# Changelog

## 0.1.1 (unreleased)

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
- Poles and zeros of `tand` and `tanpi` now have alternating signs.

## 0.1.0 (2026-04-02)

- Initial release
