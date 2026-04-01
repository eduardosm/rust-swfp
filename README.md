# swfp

swfp is a pure-Rust soft-float library. It provides arithmetic and
correctly-rounded math functions.

## Development

The code contains some generated constants. They can be identified with a
comment that starts with `// GENERATE:`.

They can be regenerated with:

```sh
cargo run -p generator -- rt-data src
```

The generator needs [Sollya](https://www.sollya.org/) and
[Julia](https://julialang.org/) to be installed.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
