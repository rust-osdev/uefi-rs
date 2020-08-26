# uefi-rs

[![Crates.io](https://img.shields.io/crates/v/uefi)](https://crates.io/crates/uefi)
[![Docs.rs](https://docs.rs/uefi/badge.svg)](https://docs.rs/uefi)
![Stars](https://img.shields.io/github/stars/rust-osdev/uefi-rs)
![License](https://img.shields.io/github/license/rust-osdev/uefi-rs)
![Build status](https://github.com/rust-osdev/uefi-rs/workflows/Rust/badge.svg)

## Description

[UEFI] is the successor to the BIOS. It provides an early boot environment for
OS loaders, hypervisors and other low-level applications. While it started out
as x86-specific, it has been adopted on other platforms, such as ARM.

This crate makes it easy to both:
  - Write UEFI applications in Rust (for `x86_64` or `aarch64`)
  - Call UEFI functions from an OS (usually built with a [custom target][rustc-custom])

The objective is to provide **safe** and **performant** wrappers for UEFI interfaces,
and allow developers to write idiomatic Rust code.

Check out @gil0mendes [blog post on getting started with UEFI in Rust][gm-blog].

**Note**: this crate currently has only been tested with **64-bit** UEFI on x86/ARM.

[UEFI]: https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface
[gm-blog]: https://gil0mendes.io/blog/an-efi-app-a-bit-rusty/
[rustc-custom]: https://doc.rust-lang.org/rustc/targets/custom.html

![uefi-rs running in QEMU](https://imgur.com/SFPSVuO.png)

## Project structure

This project contains multiple sub-crates:

- `uefi` (top directory): defines the standard UEFI tables / interfaces.
  The objective is to stay unopionated and safely wrap most interfaces.

  Optional features:
  - `alloc`: implements a global allocator using UEFI functions.
    - This allows you to allocate objects on the heap.
    - There's no guarantee of the efficiency of UEFI's allocator.
  - `logger`: logging implementation for the standard [log] crate.
    - Prints output to console.
    - No buffering is done: this is not a high-performance logger.
  - `exts`: extensions providing utility functions for common patterns.
    - Requires the `alloc` crate (either enable the `alloc` optional feature or your own custom allocator).

- `uefi-macros`: procedural macros that are used to derive some traits in `uefi`.

- `uefi-services`: provides a panic handler, and initializes the `alloc` / `logger` features.

- `uefi-test-runner`: a UEFI application that runs unit / integration tests.

[log]: https://github.com/rust-lang-nursery/log

## Building kernels which use UEFI

This crate makes it easy to start building simple applications with UEFI.
However, there are some limitations you should be aware of:

- The global logger / allocator **can only be set once** per binary.
  It is useful when just starting out, but if you're building a real OS you will
  want to write your own specific kernel logger and memory allocator.

- To support advanced features such as [higher half kernel] and [linker scripts]
  you will want to build your kernel as an ELF binary.

In other words, the best way to use this crate is to create a small binary which
wraps your actual kernel, and then use UEFI's convenient functions for loading
it from disk and booting it.

This is similar to what the Linux kernel's [EFI stub] does: the compressed kernel
is an ELF binary which has little knowledge of how it's booted, and the boot loader
uses UEFI to set up an environment for it.

[higher half kernel]: https://wiki.osdev.org/Higher_Half_Kernel
[linker scripts]: https://sourceware.org/binutils/docs/ld/Scripts.html
[EFI stub]: https://www.kernel.org/doc/Documentation/efi-stub.txt

## Documentation

The docs for the latest published crate version can be found at
[docs.rs/uefi/](https://docs.rs/uefi/)

This crate's documentation is fairly minimal, and you are encouraged to refer to
the [UEFI specification][spec] for detailed information.

[spec]: http://www.uefi.org/specifications

## Sample code

An example UEFI app is built in the `uefi-test-runner` directory.

Check out the testing [README.md](uefi-test-runner/README.md) for instructions on how to run the crate's tests.

## Building UEFI programs

For instructions on how to create your own UEFI apps, see the [BUILDING.md](BUILDING.md) file.

## Contributing

We welcome issues and pull requests! For instructions on how to set up a development
environment and how to add new protocols, check out [CONTRIBUTING.md](CONTRIBUTING.md).

## License

The code in this repository is licensed under the Mozilla Public License 2.
This license allows you to use the crate in proprietary programs, but any modifications to the files must be open-sourced.

The full text of the license is available in the [license file](LICENSE).
