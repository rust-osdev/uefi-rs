# `uefi`

Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)

## TL;DR

Develop Rust software that leverages **safe**, **convenient**, and
**performant** abstractions for [UEFI] functionality.

## About

With `uefi`, you have the flexibility to integrate selected types and
abstractions into your project or to conveniently create EFI images, addressing
the entire spectrum of your development needs.

`uefi` works with stable Rust, but additional nightly-only features are
gated behind an `unstable` Cargo feature flag.

_Note that for producing EFI images, you also need to use a corresponding `uefi`
compiler target of Rust, such as `x86_64-unknown-uefi`._

For an introduction to the `uefi-rs` project and documentation, please refer to
our main [README].

[README]: https://github.com/rust-osdev/uefi-rs/blob/main/README.md

## Optional features

This crate's features are described in [`src/lib.rs`].

[`src/lib.rs`]: src/lib.rs

## User Documentation

<!-- KEEP IN SYNC WITH MAIN README -->

For a quick start, please check out [the UEFI application template](template).

The [uefi-rs book] contains a tutorial, how-tos, and overviews of some important
UEFI concepts. Reference documentation for the various crates can be found on
[docs.rs]:

- [docs.rs/uefi](https://docs.rs/uefi)
- [docs.rs/uefi-macros](https://docs.rs/uefi-macros)
- [docs.rs/uefi-raw](https://docs.rs/uefi-raw)

[spec]: http://www.uefi.org/specifications
[uefi-rs book]: https://rust-osdev.github.io/uefi-rs/HEAD

## MSRV

The minimum supported Rust version is currently 1.70.

Our policy is to support at least the past two stable releases.

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).
