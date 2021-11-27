# UEFI application template

This directory contains a minimal example of a UEFI application.
Copy it to a new directory to get started.

Check out the [`BUILDING.md`](../BUILDING.md) document for more instructions on
how to build and run a UEFI application developed using `uefi-rs`.

## File structure

- [`template/.cargo/config`](template/.cargo/config) file sets some `build-std` options.
- [`template/Cargo.toml`](template/Cargo.toml) shows the necessary
  dependencies. Note that when creating your project the
  [`uefi`](https://crates.io/crates/uefi) and
  [`uefi-services`](https://crates.io/crates/uefi-services) dependencies should
  be changed to the latest releases on [crates.io](https://crates.io).
- [`template/src/main.rs`](template/src/main.rs) has a minimal entry point that
  initializes the `uefi-services` crate and exits successfully.

## Building kernels which use UEFI

This template makes it easy to start building simple applications with UEFI.
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
