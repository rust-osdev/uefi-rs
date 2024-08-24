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

## API and User Documentation
<!-- This section is duplicated with /README.md -->

Please refer to [docs.rs](https://docs.rs/uefi) for comprehensive documentation
of the **latest stable release**. The latest not necessarily yet published
documentation can be found in [`src/lib.rs`](./src/lib.rs), which can also be
locally viewed by running `$ cargo xtask doc --open`.

For an introduction to the `uefi-rs` project and this repository, please refer
to our main [README](https://github.com/rust-osdev/uefi-rs/blob/main/README.md).
<!-- ^ This link can't be relative as it also should work in the packaged crate
     on crates.io. -->


[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
