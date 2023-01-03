# uefi-rs

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)

[UEFI] is the successor to the BIOS. It provides an early boot environment for
OS loaders, hypervisors and other low-level applications.

The `uefi` crate makes it easy to:
- Write UEFI applications in Rust (for `i686`, `x86_64`, or `aarch64`)
- Call UEFI functions from an OS (usually built with a [custom target][rustc-custom])

The objective is to provide **safe** and **performant** wrappers for UEFI interfaces,
and allow developers to write idiomatic Rust code.

Check out the [UEFI application template] for a quick start.

[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
[rustc-custom]: https://doc.rust-lang.org/rustc/targets/custom.html
[UEFI application template]: https://github.com/rust-osdev/uefi-rs/tree/HEAD/template

## Optional features

This crate's features are described in [`src/lib.rs`].

See also the [`uefi-services`] crate, which provides a panic handler and
initializes the `global_allocator` and `logger` features.

[`src/lib.rs`]: src/lib.rs
[`uefi-services`]: https://crates.io/crates/uefi-services

## Documentation

The [uefi-rs book] contains a tutorial, how-tos, and overviews of some
important UEFI concepts.

Reference documentation can be found on docs.rs:
- [docs.rs/uefi](https://docs.rs/uefi)
- [docs.rs/uefi-macros](https://docs.rs/uefi-macros)
- [docs.rs/uefi-services](https://docs.rs/uefi-services)

For additional information, refer to the [UEFI specification][spec].

[spec]: http://www.uefi.org/specifications
[uefi-rs book]: https://rust-osdev.github.io/uefi-rs/HEAD

## Building UEFI programs

For instructions on how to create your own UEFI apps, see the [tutorial].

The uefi-rs crates currently require some [unstable features].
The nightly MSRV is currently 2022-08-25.

[unstable features]: https://github.com/rust-osdev/uefi-rs/issues/452
[tutorial]: https://rust-osdev.github.io/uefi-rs/HEAD/tutorial/introduction.html

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).
