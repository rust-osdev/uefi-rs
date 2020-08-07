# Creating UEFI applications

UEFI applications are simple COFF (Windows) executables, with the special
`EFI_Application` subsystem, and some limitations (such as no dynamic linking).
[Rust supports building UEFI applications](https://github.com/rust-lang/rust/pull/56769)
though the `x86_64-unknown-uefi` target.

## Steps

The following steps allow you to build a simple UEFI app.

- Create a new `#![no_std]` binary, add `#![no_main]` to use a custom entry point,
  and make sure you have an entry point function which matches the one below:
  ```rust
  #![feature(abi_efiapi)]
  use uefi::prelude::*;

  #[entry]
  fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> Status;
  ```
  You will also want to add a dependency to the [`rlibc`](https://docs.rs/rlibc/) crate,
  to avoid linking errors.

- Build using a `nightly` version of the compiler and activate the
  [`build-std`](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std)
  Cargo feature: `cargo build -Z build-std --target x86_64-unknown-uefi`.

- The `target` directory will contain a `x86_64-unknown-uefi` subdirectory,
  where you will find the `uefi_app.efi` file - a normal UEFI executable.

- To run this on a real computer:
  - Find a USB drive which is FAT12 / FAT16 / FAT32 formatted
  - Copy the file to the USB drive, to `/EFI/Boot/Bootx64.efi`
  - In the UEFI BIOS, choose "Boot from USB" or similar

- To run this in QEMU:
  - You will need a recent version of QEMU as well as OVMF to provide UEFI support
  - Check the `build.py` script for an idea of what arguments to pass to QEMU

You can use the `uefi-test-runner` directory as sample code for building a simple UEFI app.
