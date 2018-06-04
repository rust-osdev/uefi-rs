# uefi-rs

This library allows you to write [UEFI][uefi] applications in Rust.

UEFI is the successor to the BIOS. It provides an early boot environment for OS loaders
and other low-level applications.

The objective of this library is to provide **safe** and **performant** wrappers for UEFI
interfaces, and allow developers to write idiomatic Rust code.

[uefi]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface

<p align="center">
  <img width="552px" height="389px" alt="uefi-rs running in QEMU" src="https://i.imgur.com/AtFQLUO.png"/>
</p>

## Project structure

This project contains multiple sub-crates:

- `uefi` (top directory): defines the standard UEFI tables / interfaces.

- `uefi-services`: initializes many convenience crates:
  - `uefi-logger`: wrapper for the standard [logging](https://github.com/rust-lang-nursery/log) crate. Prints log output to console.
  - `uefi-alloc`: implements a global allocator using UEFI functions. This allows you to allocate objects on the heap.

- `uefi-utils`: building on top of `uefi-services`, this crate provides a higher-level access to UEFI functions.
  Provides utility functions for common API usage.

- `uefi-test-runner` a UEFI application that runs unit / integration tests.

## Documentation

This crate's documentation is fairly minimal, and you are encouraged to refer to
the [UEFI specification][spec] for detailed information.

[spec]: http://www.uefi.org/specifications

### rustdoc

Use the `build.py` script in the `uefi-test-runner` directory to generate the documentation:

```sh
./build.py doc
```

## Sample code

An example UEFI app is built in the `uefi-test-runner` directory.

Check out the testing [README.md](uefi-test-runner/README.md) for instructions on how to run the crate's tests.

This repo also contains a `x86_64-uefi.json` file, which is a custom Rust target for 64-bit UEFI applications.

## Building UEFI programs

For instructions on how to create your own UEFI apps, see the [BUILDING.md](BUILDING.md) file.

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the `LICENSE` file.
