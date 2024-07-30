# `uefi`

Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].

This crate makes it easy to develop Rust software that leverages **safe**,
**convenient**, and **performant** abstractions for [UEFI] functionality.

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)

## Value-add and Use Cases

`uefi` supports writing code for both pre- and post-exit boot services
epochs, but its true strength shines when you create UEFI images that heavily
interact with UEFI boot services. Still, you have the flexibility to just
integrate selected types and abstractions into your project, for example to
parse the UEFI memory map.

_Note that for producing UEFI images, you also need to use a corresponding
`uefi` compiler target of Rust, such as `x86_64-unknown-uefi`._

For an introduction to the `uefi-rs` project and documentation, please refer to
our main [README].

[README]: https://github.com/rust-osdev/uefi-rs/blob/main/README.md

## Optional features

This crate's features are described in [`src/lib.rs`].

[`src/lib.rs`]: src/lib.rs

## MSRV

The minimum supported Rust version is currently 1.70.

Our policy is to support at least the past two stable releases.

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).


[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
