# Creating UEFI applications

UEFI applications are simple COFF (Windows) executables, with the special
`EFI_Application` subsystem, and some limitations (such as no dynamic linking).
Rust supports building UEFI applications for the
[`aarch64-unknown-uefi`], [`i686-unknown-uefi`], and [`x86_64-unknown-uefi`]
targets.

## Template

The [template](template) subdirectory contains a minimal example of a UEFI
application. Copy it to a new directory to get started.

- [template/.cargo/config](template/.cargo/config) file sets some `build-std` options.
- [template/Cargo.toml](template/Cargo.toml) shows the necessary
  dependencies. Note that when creating your project the
  [`uefi`](https://crates.io/crates/uefi) and
  [`uefi-services`](https://crates.io/crates/uefi-services) dependencies should
  be changed to the latest releases on [crates.io](https://crates.io).
- [template/src/main.rs](template/src/main.rs) has a minimal entry point that
  initializes services and exits successfully.

## Building and running

- Build using a `nightly` version of the compiler and activate the
  [`build-std`](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std)
  Cargo features: `cargo +nightly build -Z build-std -Z build-std-features=compiler-builtins-mem --target x86_64-unknown-uefi`.

- The `target` directory will contain a `x86_64-unknown-uefi` subdirectory,
  where you will find the `uefi_app.efi` file - a normal UEFI executable.

- To run this on a real computer:
  - Find a USB drive which is FAT12 / FAT16 / FAT32 formatted
  - Copy the file to the USB drive, to `/EFI/Boot/Bootx64.efi`
  - In the UEFI BIOS, choose "Boot from USB" or similar

- To run this in QEMU:
  - You will need a recent version of QEMU as well as OVMF to provide UEFI support
  - Check the [`build.py`](uefi-test-runner/build.py) script for an idea of
    what arguments to pass to QEMU

[`aarch64-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/aarch64_unknown_uefi.rs
[`i686-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/i686_unknown_uefi.rs
[`x86_64-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/x86_64_unknown_uefi.rs
