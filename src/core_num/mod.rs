// From Rust libcore, commit 2bd7a97871a74d4333bd3edb6564136167ac604b
// Check for changes with
// `git diff A..B -- library/core/src/num library/core/src/fmt/mod.rs library/coretests/tests/num`

// Some comments might not make much sense, since they have been copied verbatim.

#![allow(
    clippy::approx_constant,
    clippy::eq_op,
    clippy::excessive_precision,
    clippy::identity_op,
    clippy::needless_late_init,
    clippy::too_many_arguments,
    clippy::zero_divided_by_zero
)]

mod bignum;
pub(crate) mod dec2flt;
mod diy_float;
pub(crate) mod flt2dec;
mod fmt;
pub(crate) mod fmt_float;
mod numfmt;
