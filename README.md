# uefi-rs

Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].

This crate makes it easy to develop Rust software that leverages **safe**,
**convenient**, and **performant** abstractions for [UEFI] functionality.

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)

## Description

[UEFI] started as the successor firmware to the BIOS in x86 space and developed
to a universal firmware specification for various platforms, such as ARM. It
provides an early boot environment with a variety of [specified][spec]
ready-to-use "high-level" functionality, such as accessing disks or the network.
EFI images, the files that can be loaded by an UEFI environment, can leverage
these abstractions to extend the functionality in form of additional drivers,
OS-specific bootloaders, or different kind of low-level applications.

Our mission is to provide **safe** and **performant** wrappers for UEFI
interfaces, and allow developers to write idiomatic Rust code.

This repository provides various crates:

- `uefi-raw`: Raw Rust UEFI bindings for basic structures and functions.
- `uefi`: High-level wrapper around various low-level UEFI APIs. \
  Offers various optional features for typical Rust convenience, such as a
  Logger and an Allocator. (_This is what you are usually looking for!_)
- `uefi-macros`: Helper macros. Used by `uefi`.


You can use the abstractions for example to:

- create OS-specific loaders and leverage UEFI boot service
- access UEFI runtime services from an OS

All crates are compatible with all platforms that both the Rust compiler and
UEFI support, such as `i686`, `x86_64`, and `aarch64`). Please note that we
can't test all possible hardware/firmware/platform combinations.

[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface

![UEFI App running in QEMU](https://imgur.com/SFPSVuO.png)
Screenshot of an application running in QEMU on an UEFI firmware that leverages
our Rust library.

## User Documentation

<!-- KEEP IN SYNC WITH uefi/README -->

For a quick start, please check out [the UEFI application template](template).

The [uefi-rs book] contains a tutorial, how-tos, and overviews of some important
UEFI concepts. Reference documentation for the various crates can be found on
[docs.rs]:

- [docs.rs/uefi](https://docs.rs/uefi)
- [docs.rs/uefi-macros](https://docs.rs/uefi-macros)
- [docs.rs/uefi-raw](https://docs.rs/uefi-raw)

For additional information, refer to the [UEFI specification][spec].

[spec]: https://uefi.org/specs/UEFI/2.10
[uefi-rs book]: https://rust-osdev.github.io/uefi-rs/HEAD
[docs.rs]: https://docs.rs

### MSRV

See the [uefi package's README](uefi/README.md#MSRV).

## Developer Guide

### Project structure

This project contains multiple sub-crates:

- `uefi`: defines the standard UEFI tables / interfaces.
  The objective is to stay unopinionated and safely wrap most interfaces.
  Additional opinionated features (such as a Logger) are feature-gated.

- `uefi-macros`: procedural macros that are used to derive some traits
  in `uefi`.

- `uefi-raw`: raw types that closely match the definitions in the UEFI
  Specification. Safe wrappers for these types are provided by the `uefi`
  crate. The raw types are suitable for implementing UEFI firmware.

- `uefi-test-runner`: a UEFI application that runs unit / integration tests.

[log]: https://github.com/rust-lang-nursery/log

### Building and testing uefi-rs

Use the `cargo xtask` command to build and test the crate.

Available commands:

- `build`: build all the UEFI packages
  - `--release`: build in release mode
  - `--target {x86_64,ia32,aarch64}`: choose target UEFI arch
- `clippy`: run clippy on all the packages
  - `--target {x86_64,ia32,aarch64}`: choose target UEFI arch
  - `--warnings-as-errors`: treat warnings as errors
- `doc`: build the docs for the UEFI packages
  - `--open`: open the docs in a browser
  - `--warnings-as-errors`: treat warnings as errors
- `run`: build `uefi-test-runner` and run it in QEMU
  - `--ci`: disable some tests that don't work in the CI
  - `--disable-kvm`: disable hardware accelerated virtualization support in
    QEMU.
    Especially useful if you want to run the tests under
    [WSL](https://docs.microsoft.com/en-us/windows/wsl) on Windows.
  - `--example <NAME>`: run an example instead of the main binary.
  - `--headless`: run QEMU without a GUI
  - `--ovmf-code <PATH>`: path of an OVMF code file
  - `--ovmf-vars <PATH>`: path of an OVMF vars file
  - `--release`: build in release mode
  - `--target {x86_64,ia32,aarch64}`: choose target UEFI arch
- `test`: run unit tests and doctests on the host

The `uefi-test-runner` directory contains a sample UEFI app which exercises
most of the library's functionality.

Check out the testing project's [`README.md`](uefi-test-runner/README.md) for
prerequisites for running the tests.

## Contributing

We welcome issues and pull requests! For instructions on how to set up a
development environment and how to add new protocols, check out
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any
modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).

[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
