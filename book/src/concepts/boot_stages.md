# Boot Stages

A UEFI system goes through several distinct phases during the boot process.
1. **Platform Initialization.** This early-boot phase is mostly outside
   the scope of the `uefi` crate. It is described by the [UEFI Platform
   Initialization Specification], which is separate from the main UEFI
   Specification.
2. **Boot Services.** This is when UEFI drivers and applications are loaded.
   Functions in both the [`boot`] module and [`runtime`] module can be used.
   This stage typically culminates in running a bootloader that loads an
   operating system. The stage ends when [`boot::exit_boot_services`] is called,
   putting the system in Runtime mode.
3. **Runtime.** This stage is typically active when running an operating system
   such as Linux or Windows. UEFI functionality is much more limited in the
   Runtime mode. Functions in the [`boot`] module can no longer be used, but
   functions in the [`runtime`] module are still available. Once the system is
   in Runtime mode, it cannot return to the Boot Services stage until after a
   system reset.

[UEFI Platform Initialization Specification]: https://uefi.org/specifications
[`boot`]: https://docs.rs/uefi/latest/uefi/boot/index.html
[`runtime`]: https://docs.rs/uefi/latest/uefi/runtime/index.html
[`boot::exit_boot_services`]: https://docs.rs/uefi/latest/uefi/boot/fn.exit_boot_services.html
