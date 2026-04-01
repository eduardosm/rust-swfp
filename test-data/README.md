# Test data

Data in `core-math` comes from the [core-math project](https://gitlab.inria.fr/core-math/core-math/),
commit 93d9f3bab7561cfb62f746f7e70c0888bb5c9a00

Remaining test data, is generated with:

```sh
cargo run -p generator --release -- test-data f16::pow > test-data/f16/pow.txt

cargo run -p generator --release -- test-data f16::atan2 > test-data/f16/atan2.txt
cargo run -p generator --release -- test-data f16::atan2d > test-data/f16/atan2d.txt
cargo run -p generator --release -- test-data f16::atan2pi > test-data/f16/atan2pi.txt

cargo run -p generator --release -- test-data f32::exp > test-data/f32/exp.txt
cargo run -p generator --release -- test-data f32::exp_m1 > test-data/f32/exp_m1.txt
cargo run -p generator --release -- test-data f32::exp2 > test-data/f32/exp2.txt
cargo run -p generator --release -- test-data f32::exp2_m1 > test-data/f32/exp2_m1.txt
cargo run -p generator --release -- test-data f32::exp10 > test-data/f32/exp10.txt
cargo run -p generator --release -- test-data f32::exp10_m1 > test-data/f32/exp10_m1.txt

cargo run -p generator --release -- test-data f32::ln > test-data/f32/ln.txt
cargo run -p generator --release -- test-data f32::ln_1p > test-data/f32/ln_1p.txt
cargo run -p generator --release -- test-data f32::log2 > test-data/f32/log2.txt
cargo run -p generator --release -- test-data f32::log2_1p > test-data/f32/log2_1p.txt
cargo run -p generator --release -- test-data f32::log10 > test-data/f32/log10.txt
cargo run -p generator --release -- test-data f32::log10_1p > test-data/f32/log10_1p.txt

cargo run -p generator --release -- test-data f32::sin > test-data/f32/sin.txt
cargo run -p generator --release -- test-data f32::cos > test-data/f32/cos.txt
cargo run -p generator --release -- test-data f32::tan > test-data/f32/tan.txt

cargo run -p generator --release -- test-data f32::sind > test-data/f32/sind.txt
cargo run -p generator --release -- test-data f32::cosd > test-data/f32/cosd.txt
cargo run -p generator --release -- test-data f32::tand > test-data/f32/tand.txt

cargo run -p generator --release -- test-data f32::sinpi > test-data/f32/sinpi.txt
cargo run -p generator --release -- test-data f32::cospi > test-data/f32/cospi.txt
cargo run -p generator --release -- test-data f32::tanpi > test-data/f32/tanpi.txt

cargo run -p generator --release -- test-data f32::asin > test-data/f32/asin.txt
cargo run -p generator --release -- test-data f32::acos > test-data/f32/acos.txt
cargo run -p generator --release -- test-data f32::atan > test-data/f32/atan.txt

cargo run -p generator --release -- test-data f32::asind > test-data/f32/asind.txt
cargo run -p generator --release -- test-data f32::acosd > test-data/f32/acosd.txt
cargo run -p generator --release -- test-data f32::atand > test-data/f32/atand.txt

cargo run -p generator --release -- test-data f32::asinpi > test-data/f32/asinpi.txt
cargo run -p generator --release -- test-data f32::acospi > test-data/f32/acospi.txt
cargo run -p generator --release -- test-data f32::atanpi > test-data/f32/atanpi.txt

cargo run -p generator --release -- test-data f32::sinh > test-data/f32/sinh.txt
cargo run -p generator --release -- test-data f32::cosh > test-data/f32/cosh.txt
cargo run -p generator --release -- test-data f32::tanh > test-data/f32/tanh.txt

cargo run -p generator --release -- test-data f32::asinh > test-data/f32/asinh.txt
cargo run -p generator --release -- test-data f32::acosh > test-data/f32/acosh.txt
cargo run -p generator --release -- test-data f32::atanh > test-data/f32/atanh.txt

cargo run -p generator --release -- test-data f32::gamma > test-data/f32/gamma.txt
cargo run -p generator --release -- test-data f32::ln_gamma > test-data/f32/ln_gamma.txt
```
