# Building and running UEFI applications

## UEFI binaries

UEFI applications are simple COFF (Windows) executables, with the special
`EFI_Application` subsystem, and some limitations (such as no dynamic linking).

The Rust compiler supports building UEFI applications for the
[`aarch64-unknown-uefi`], [`i686-unknown-uefi`], and [`x86_64-unknown-uefi`]
targets.

[`aarch64-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/aarch64_unknown_uefi.rs
[`i686-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/i686_unknown_uefi.rs
[`x86_64-unknown-uefi`]: https://github.com/rust-lang/rust/blob/HEAD/compiler/rustc_target/src/spec/x86_64_unknown_uefi.rs

## Building for a UEFI target

For instructions on building the `uefi-rs` crates, see the
[README](README.md). This section is for building your own crates,
outside of the `uefi-rs` repo.

- Install a `nightly` version of the Rust [toolchain] and include the
  `rust-src` [component]. The easiest way to do this is with a
  `rust-toolchain.toml` file, for example:
  
  ```toml
  [toolchain]
  channel = "nightly"
  components = ["rust-src"]
  ```

- Build the crate:

  `cargo build --target x86_64-unknown-uefi`.

- The `target` directory will contain a `x86_64-unknown-uefi` subdirectory,
  where you will find a `<crate name>.efi` file - a normal UEFI executable.
  
[toolchain]: https://rust-lang.github.io/rustup/concepts/toolchains.html
[component]: https://rust-lang.github.io/rustup/concepts/components.html

## Running

- To run an `.efi` executable on a real computer:
  - Find a USB drive which is FAT12 / FAT16 / FAT32 formatted
  - Copy the file to the USB drive, to `/EFI/Boot/Bootx64.efi`
  - In the UEFI BIOS, choose "Boot from USB" or similar

- To run this in QEMU:
  - You will need a recent version of QEMU as well as OVMF to provide UEFI support
  - Check the [`qemu.rs`](xtask/src/qemu.rs) module for an idea of
    what arguments to pass to QEMU.

    In principle, you need to replicate the file structure described above for an USB drive,
    then [mount the directory as if it were a FAT drive][qemu-vvfat].

[qemu-vvfat]: https://en.wikibooks.org/wiki/QEMU/Devices/Storage#Virtual_FAT_filesystem_(VVFAT)

## Template

The [template](template) provides a quick way to get started building UEFI applications.
