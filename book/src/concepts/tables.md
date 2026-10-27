# Tables

UEFI has a few table structures. These tables are how you get access to
UEFI services.

[`SystemTable`] (`EFI_SYSTEM_TABLE` in the specification) is the
top-level table that provides access to the other tables.

[`BootServices`] (`EFI_BOOT_SERVICES` in the specification) provides
access to a wide array of services such as memory allocation, executable
loading, and optional extension interfaces called protocols. This table
is only accessible while in the Boot Services stage.

[`RuntimeServices`] (`EFI_RUNTIME_SERVICES` in the specification)
provides access to a fairly limited set of services, including variable
storage, system time, and virtual-memory mapping. This table is
accessible during both the Boot Services and Runtime stages.

When writing a UEFI application, you get access to the system table from
one of the arguments to the `main` entry point:

```rust,ignore
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status;
```

Then use [`SystemTable::boot_services`] and
[`SystemTable::runtime_services`] to get access to the other
tables. Once [`SystemTable::exit_boot_services`] is called, the original
system table is consumed and a new system table is returned that only
provides access to the [`RuntimeServices`] table.

[`BootServices`]: https://docs.rs/uefi/latest/uefi/table/boot/struct.BootServices.html
[`RuntimeServices`]: https://docs.rs/uefi/latest/uefi/table/runtime/struct.RuntimeServices.html
[`SystemTable::boot_services`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html#method.boot_services
[`SystemTable::exit_boot_services`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html#method.exit_boot_services
[`SystemTable::runtime_services`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html#method.runtime_services
[`SystemTable`]: https://docs.rs/uefi/latest/uefi/table/struct.SystemTable.html
