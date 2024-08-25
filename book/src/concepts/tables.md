# Tables

UEFI has a few table structures. These tables are how you get access to UEFI
services. In the specification and C API, `EFI_SYSTEM_TABLE` is the top-level
table that provides access to the other tables, `EFI_BOOT_SERVICES` and
`EFI_RUNTIME_SERVICES`.

In the `uefi` crate, these tables are modeled as modules rather than structs. The
functions in each module make use of a global pointer to the system table that
is set automatically by the [`entry`] macro.

* [`uefi::system`] (`EFI_SYSTEM_TABLE` in the specification) provides access to
system information such as the firmware vendor and version. It can also be used
to access stdout/stderr/stdin.

* [`uefi::boot`] (`EFI_BOOT_SERVICES` in the specification) provides access to a
wide array of services such as memory allocation, executable loading, and
optional extension interfaces called protocols. Functions in this module can
only be used while in the Boot Services stage. After [`exit_boot_services`] has
been called, these functions will panic.

* [`uefi::runtime`] (`EFI_RUNTIME_SERVICES` in the specification) provides access
to a fairly limited set of services, including variable storage, system time,
and virtual-memory mapping. Functions in this module are accessible during both
the Boot Services and Runtime stages.

[`entry`]: https://docs.rs/uefi/latest/uefi/attr.entry.html
[`exit_boot_services`]: https://docs.rs/uefi/latest/uefi/boot/fn.exit_boot_services.html
[`uefi::boot`]: https://docs.rs/uefi/latest/uefi/boot/index.html
[`uefi::runtime`]: https://docs.rs/uefi/latest/uefi/runtime/index.html
[`uefi::system`]: https://docs.rs/uefi/latest/uefi/system/index.html
