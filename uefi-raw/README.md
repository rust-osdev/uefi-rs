# uefi-raw

[![Crates.io](https://img.shields.io/crates/v/uefi-raw)](https://crates.io/crates/uefi-raw)
[![Docs.rs](https://docs.rs/uefi-raw/badge.svg)](https://docs.rs/uefi-raw)


Raw UEFI types and bindings for protocols, boot, and runtime services of the
[UEFI specification]. This can serve as base for an UEFI firmware implementation
or a high-level wrapper to access UEFI functionality from an UEFI image.

For creating UEFI applications and drivers, consider using the `uefi` crate
instead of `uefi-raw`.

## Modeling of UEFI Types in Rust

Please check the module documentation in `lib.rs`.

[UEFI Specification]: https://uefi.org/specifications
