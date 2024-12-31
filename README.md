# nostd-bv: bit-vectors and bit-slices for Rust

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/nostd-rs/bv/ci.yml?event=push)](https://github.com/nostd-rs/bv)
[![Crates.io Version](https://img.shields.io/crates/v/nostd-bv)](https://crates.io/crates/nostd-bv)
[![GitHub License](https://img.shields.io/badge/license-MIT%2FApache2-blue)](#LICENSE)

The main type exported by the library, `BitVec`, is a packed, growable
bit-vector. Its API mirrors that of `Vec` where reasonable. The library
also defines slice operations that return `BitSlice` or `BitSliceMut`,
akin to Rust’s array slices but for bit-vectors. A common API to
bit-vectors and bit-slices is provided by the `Bits`, `BitsMut`, and
`BitsPush` traits, which also allow treating as bit-vectors all primitive 
unsigned integer types (`uN`), and vectors and slices thereof, as well
as unpacked vectors and slices of `bool`.

## Usage

It’s [on crates.io](https://crates.io/crates/nostd-bv), so you can add

```toml
[dependencies]
nostd-bv = "0.11"
```

to your `Cargo.toml`.

## License

Licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
