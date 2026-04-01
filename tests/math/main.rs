#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]
#![allow(clippy::identity_op)]

use swfp::Float as _;

macro_rules! assert_is_nan {
    ($value:expr) => {{
        let value = $value;
        if !swfp::Float::is_nan(value) {
            panic!("assertion failed: `({value:?}).is_nan()`");
        }
    }};
}

macro_rules! assert_total_eq {
    ($lhs:expr, $rhs:expr) => {
        let lhs = $lhs;
        let rhs = $rhs;
        if !$crate::TotalEq::total_eq(&lhs, &rhs) {
            panic!(
                "assertion `left == right` (using totalOrder) failed\n  left: {lhs:?}\n right: {rhs:?}",
            );
        }
    };
    ($lhs:expr, $rhs:expr, $($fmt:tt)+) => {
        let lhs = $lhs;
        let rhs = $rhs;
        if !$crate::TotalEq::total_eq(&lhs, &rhs) {
            panic!(
                "assertion `left == right` (using totalOrder) failed: {}\n  left: {lhs:?}\n right: {rhs:?}",
                format_args!($($fmt)+),
            );
        }
    };
}

mod data;
mod f128;
mod f16;
mod f32;
mod f64;
mod generic;
mod x87f80;

trait TotalEq {
    fn total_eq(&self, other: &Self) -> bool;
}

impl TotalEq for swfp::F16 {
    fn total_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl TotalEq for swfp::F32 {
    fn total_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl TotalEq for swfp::F64 {
    fn total_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl TotalEq for swfp::F128 {
    fn total_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl TotalEq for swfp::X87F80 {
    fn total_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}

impl<T: TotalEq> TotalEq for (T, T) {
    fn total_eq(&self, other: &Self) -> bool {
        self.0.total_eq(&other.0) && self.1.total_eq(&other.1)
    }
}

impl<T: TotalEq> TotalEq for (T, swfp::FpStatus) {
    fn total_eq(&self, other: &Self) -> bool {
        self.0.total_eq(&other.0) && self.1 == other.1
    }
}

trait TestFloat: swfp::Float + TotalEq {}

impl TestFloat for swfp::F16 {}
impl TestFloat for swfp::F32 {}
impl TestFloat for swfp::F64 {}
impl TestFloat for swfp::F128 {}
impl TestFloat for swfp::X87F80 {}

fn create_prng() -> impl rand::Rng {
    use rand::SeedableRng as _;
    rand_pcg::Pcg64::seed_from_u64(0xB05C_3028_6B30_9158)
}
