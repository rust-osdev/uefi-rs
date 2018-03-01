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
- [LLD](https://lld.llvm.org/): the LLVM linker is currently the only supported linker.
  Alternatively, you can use `link.exe` if you are on Windows.

### Steps

The following steps allow you to build a simple UEFI app.

- Create a new `#![no_std]` crate, and make sure you have an entry point function which matches the one below:

```rust
#[no_mangle]
pub extern "C" fn entry_point(handle: Handle, system_table: &'static table::SystemTable) -> Status;
```

- Copy the `tests/x86_64-uefi.json` target file to your project's root. You can create your own target file based on it.
- Build using `xargo build --target x86_64-uefi`.

- The generated static library needs to be linked with LLD, e.g.

```sh
lld-link /Machine:x64 /Subsystem:EFI_Application /Entry:entry_point uefi_app.lib /Out:uefi_app.efi
```

- You can run the `uefi_app.efi` file as a normal UEFI executable.

You can use the `tests` directory as sample code for building a simple UEFI app.

## License

The code in this repository is licensed under the Mozilla Public License 2. This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the `LICENSE` file.
