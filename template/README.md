# UEFI application template

This directory contains a minimal example of a UEFI application.
Copy it to a new directory to get started.

Check out the [`BUILDING.md`](../BUILDING.md) document for more instructions on
how to build and run a UEFI application developed using `uefi-rs`.

## File structure

- [`.cargo/config`](./.cargo/config) sets some `build-std` options.
- [`rust-toolchain.toml`](rust-toolchain.toml) sets the nightly channel.
- [`Cargo.toml`](./Cargo.toml) shows the necessary dependencies.
- [`src/main.rs`](./src/main.rs) has a minimal entry point that
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
uses UEFI to set up an environment for it. See the [bootloader] crate for an example
of how `uefi-rs` can be used to construct a first-stage bootloader which then
loads and executes an ELF kernel binary.

[higher half kernel]: https://wiki.osdev.org/Higher_Half_Kernel
[linker scripts]: https://sourceware.org/binutils/docs/ld/Scripts.html
[EFI stub]: https://www.kernel.org/doc/Documentation/efi-stub.txt
[bootloader]: https://github.com/rust-osdev/bootloader
