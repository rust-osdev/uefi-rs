# Boot Stages

A UEFI system goes through several distinct phases during the boot process.
1. **Platform Initialization.** This early-boot phase is mostly outside
   the scope of `uefi-rs`. It is described by the [UEFI Platform
   Initialization Specification], which is separate from the main UEFI
   Specification.
2. **Boot Services.** This is when UEFI drivers and applications are
   loaded. Both the [`BootServices`] and [`RuntimeServices`] tables are
   accessible. This stage typically culminates in running a bootloader
   that loads an operating system. The stage ends when
   [`SystemTable::exit_boot_services`] is called, putting the system in
   Runtime mode.
3. **Runtime.** This stage is typically active when running an operating
   system such as Linux or Windows. UEFI functionality is much more
   limited in the Runtime mode. The [`BootServices`] table is no longer
   accessible, but the [`RuntimeServices`] table is still
   available. Once the system is in Runtime mode, it cannot return to
   the Boot Services stage until after a system reset.

[UEFI Platform Initialization Specification]: https://uefi.org/specifications
[`BootServices`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html
[`RuntimeServices`]: https://docs.rs/uefi/latest/uefi/table/runtime/struct.RuntimeServices.html
[`SystemTable::exit_boot_services`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html#method.exit_boot_services
