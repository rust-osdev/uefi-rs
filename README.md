# uefi-rs

This library allows you to write [UEFI][uefi] applications in Rust.

UEFI is the successor to the BIOS. It provides an early boot environment for OS loaders
and other low-level applications.

The objective of this library is to provide **safe** and **performant** wrappers for UEFI
interfaces, and allow developers to write idiomatic Rust code.

[uefi]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface

## Crates

This project contains multiple sub-crates:

- `uefi` (top directory): contains wrappers around the UEFI interfaces.

- `uefi-services`: initializes many convenience crates:
  - `uefi-logger`: wrapper for the standard [logging](https://github.com/rust-lang-nursery/log) crate.
  - `uefi-alloc`: wrapper for the memory allocation functions. This allows you to allocate objects on the heap.

- `uefi-utils`: building on top of `uefi-services`, this crate provides a higher-level access to UEFI functions.
  Provides utility functions for common API usage.

- `tests`: a sample UEFI applications that runs unit tests.

## Documentation

This crate's documentation is fairly minimal, and you are encouraged to refer to
the [UEFI specification][spec] for detailed information.

You can find some example code in the `tests` directory,
as well as use the `build.py` script to generate the documentation.

This repo also contains a `x86_64-uefi.json` file, which is
a custom Rust target for 64-bit UEFI applications.

[spec]: http://www.uefi.org/specifications

## Building UEFI programs

### Prerequisites

- [Xargo](https://github.com/japaric/xargo): this is essential if you plan to do any sort of cross-platform / bare-bones Rust programming.
- [LLD](https://lld.llvm.org/): this linker is now [shipped](https://github.com/rust-lang/rust/pull/48125) with the latest nightly!

### Steps

The following steps allow you to build a simple UEFI app.

- Create a new `#![no_std]` binary, add `#![no_main]` to use a custom entry point,
  and make sure you have an entry point function which matches the one below:

```rust
#[no_mangle]
pub extern "C" fn uefi_start(handle: Handle, system_table: &'static table::SystemTable) -> Status;
```

- Copy the `tests/x86_64-uefi.json` target file to your project's root.
  You can customize it.

- Build using `xargo build --target x86_64-uefi`.

- The `target` directory will contain a `x86_64-uefi` subdirectory,
  where you will find the `uefi_app.efi` file - a normal UEFI executable.

You can use the `tests` directory as sample code for building a simple UEFI app.

## License

The code in this repository is licensed under the Mozilla Public License 2. This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the `LICENSE` file.
