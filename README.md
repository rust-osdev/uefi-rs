# uefi-rs

Rusty wrapper for the [Unified Extensible Firmware Interface][UEFI].

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)

## TL;DR

Develop Rust software that leverages **safe**, **convenient**, and
**performant** abstractions for [UEFI] functionality.

## API and User Documentation

The main contribution of this project is the `uefi` crate.
Please refer to [docs.rs](https://docs.rs/uefi) for comprehensive documentation
of the **latest stable release**. The latest not necessarily yet published
documentation can be found in [`src/lib.rs`](./uefi/src/lib.rs), which can also
be locally build by running `$ cargo xtask doc --open`.

![UEFI App running in QEMU](https://imgur.com/SFPSVuO.png)
Screenshot of an application running in QEMU on an UEFI firmware that leverages
our Rust library.

## Developer Guide

### Repository Structure

This repository provides various crates:

- [`uefi-raw`](/uefi-raw/README.md): Raw Rust UEFI bindings for basic structures and functions.
- [`uefi`](/uefi/README.md): High-level wrapper around various low-level UEFI APIs. \
  Offers various optional features for typical Rust convenience, such as a
  Logger and an Allocator.
  This is the **main contribution** of this project.
- [`uefi-macros`](/uefi-macros/README.md): Helper macros used by `uefi`.
- [`uefi-test-runner`](/uefi-test-runner/README.md): a UEFI application that runs our unit / integration tests.


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

## Discuss and Contribute

For general discussions, feel free to join us in our [Zulip] and ask
your questions there.

Further, you can submit bugs and also ask questions in our [issue tracker].
Contributions in form of a PR are also highly welcome. Check our
[contributing guide](./CONTRIBUTING.md) for details.

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any
modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).

[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
[Zulip]: https://rust-osdev.zulipchat.com
