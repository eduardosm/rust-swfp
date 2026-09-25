// Copyright (c) The Rust Project Contributors
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

// The above copyright notice and licensee applies to all files of the
// `core_num` module.

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
