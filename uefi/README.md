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

## API and User Documentation

Please refer to [docs.rs](https://docs.rs/uefi) for comprehensive documentation
of the **latest stable release**. The latest not necessarily yet published
documentation can be found in [`src/lib.rs`](./src/lib.rs), which can also be
locally build by running `$ cargo xtask doc --open`.

For an introduction to the `uefi-rs` project and this repository, please refer
to our main [README](https://github.com/rust-osdev/uefi-rs/blob/main/README.md).
<!-- ^ This link can't be relative as it also should work in the packaged crate
     on crates.io. -->
