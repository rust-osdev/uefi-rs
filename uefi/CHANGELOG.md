# uefi - [Unreleased]

## Added
- Added `ConfigTableEntry::MEMORY_ATTRIBUTES_GUID` and `ConfigTableEntry::IMAGE_SECURITY_DATABASE_GUID`.
- Added `proto::usb::io::UsbIo`.
- Added `proto::pci::PciRootBridgeIo`.
- Added `proto::hii::config::ConfigKeywordHandler`.
- Added `proto::hii::config::HiiConfigAccess`.
- Added `proto::hii::config_str::ConfigurationString`.

## Changed
- **Breaking:** `boot::stall` now take `core::time::Duration` instead of `usize`.
- `table::cfg::*_GUID` constants now deprecated. Use `ConfigTableEntry::*_GUID` instead.
- `system::with_config_table`, `system::with_stdin`, `system::with_stdout` and `system::with_stderr`
  now take mutably closure.
- **Breaking:** The MSRV is now 1.85.1 and the crate uses the Rust 2024 edition.
- The documentation in `lib.rs` now provides guidance on how to select features
  tailored to your use case.
- Feature `log-debugcon` is no longer a default feature. You only need to add
  it in case you are also using the `logger` feature and if you run your UEFI
  image in QEMU or Cloud Hypervisor, when the debugcon/debug-console device is
  available.
- The documentation for UEFI protocols has been streamlined and improved.

# uefi - 0.35.0 (2025-05-04)

## Added
- Added `boot::signal_event`.
- Added conversions between `proto::network::IpAddress` and `core::net` types.
- Added conversions between `proto::network::MacAddress` and the `[u8; 6]` type that's more commonly used to represent MAC addresses.
- Added `proto::media::disk_info::DiskInfo`.
- Added `mem::AlignedBuffer`.
- Added `proto::device_path::DevicePath::append_path()`.
- Added `proto::device_path::DevicePath::append_node()`.
- Added `proto::scsi::pass_thru::ExtScsiPassThru`.
- Added `proto::nvme::pass_thru::NvmePassThru`.
- Added `proto::ata::pass_thru::AtaPassThru`.
- Added `boot::ScopedProtocol::open_params()`.
- Added `boot::TplGuard::old_tpl()`.
- Added `boot::calculate_crc32()`.

## Changed
- **Breaking:** Removed `BootPolicyError` as `BootPolicy` construction is no
  longer fallible. `BootPolicy` now tightly integrates the new `Boolean` type
  of `uefi-raw`.
- **Breaking:** The `pxe::BaseCode::tftp_read_dir` and
  `pxe::BaseCode::mtftp_read_dir` methods now take `&mut self` instead of
  `&self`.
- **Breaking:** The `pxe::Mode` struct is now opaque. Use method calls to access
  mode data instead of direct field access.
- **Breaking:** `PoolDevicePathNode` and `PoolDevicePath` moved from module
  `proto::device_path::text` to `proto::device_path`.
- **Breaking:** `exit_boot_services` now consumes a `Option<MemoryType>` which
  defaults to the recommended value of `MemoryType::LOADER_DATA`.
- **Breaking:** Removed duplication in `DevicePathHeader`. Instead of public fields,
  there is now a public constructor combined with public getters.
- `boot::memory_map()` will never return `Status::BUFFER_TOO_SMALL` from now on,
  as this is considered a hard internal error where users can't do anything
  about it anyway. It will panic instead.
- `SimpleNetwork::transmit` now passes the correct buffer size argument.
  Previously it incorrectly added the header size to the buffer length, which
  could cause the firmware to read past the end of the buffer.
- `boot::allocate_pages` no longer panics if the allocation is at address
  zero. The allocation is retried instead, and in all failure cases an error is
  returned rather than panicking.
- The `Display` impl for `CStr8` now excludes the trailing null character.
- `VariableKeys` initializes with a larger name buffer to work around firmware
  bugs on some devices.
- The UEFI `allocator::Allocator` has been optimized for page-aligned
  allocations.


# uefi - 0.34.1 (2025-02-07)

Trivial release to fix crate license documentation.


# uefi - 0.34.0 (2025-02-07)

As of this release, the project has been relicensed from MPL-2.0 to
Apache-2.0/MIT, to better align with the Rust crate ecosystem. (This does not
alter the license of previous releases.)
Details at <https://github.com/rust-osdev/uefi-rs/issues/1470>.

## Added
- Added `proto::device_path::PoolDevicePath` and
  `proto::device_path::PoolDevicePathNode`.

## Changed
- MSRV increased to 1.81.
- `core::error::Error` impls are no longer gated by the `unstable` feature.
- Fixed missing checks in the `TryFrom` conversion from `&DevicePathNode` to
  specific node types. The node type and subtype are now checked, and
  `NodeConversionError::DifferentType` is returned if they do not match.
- **Breaking:** Fixed memory leaks in `DevicePathFromText` protocol. The methods
  now return wrapper objects that free the device path / device path node on
  drop.


# uefi - 0.33.0 (2024-10-23)

See [Deprecating SystemTable/BootServices/RuntimeServices][funcmigrate] for
details of the deprecated items that were removed in this release.

## Added
- Impl `PartialEq` and `Eq` for `GptPartitionEntry`.
- Added `CStr16::from_u16_until_nul` and `CStr16::from_char16_until_nul`.

## Changed
- **Breaking:** Deleted the deprecated `BootServices`, `RuntimeServices`, and
  `SystemTable` structs.
- **Breaking:** Deleted deprecated functions `allocator::init`,
  `allocator::exit_boot_services`, `helpers::system_table`,
  `table::system_table_boot`, and `table::system_table_runtime`.
- **Breaking:** `FileSystem` no longer has a lifetime parameter, and the
  deprecated conversion from `uefi::table::boot::ScopedProtocol` has been
  removed.
- Fixed `boot::open_protocol` to properly handle a null interface pointer.
- `VariableKey` now has a public `name` field. This `name` field always contains
  a valid string, so the `VariableKey::name()` method has been deprecated. Since
  all fields of `VariableKey` are now public, the type can be constructed by
  users.
- The `VariableKeys` iterator will now yield an error item if a variable name is
  not UCS-2.

# uefi - 0.32.0 (2024-09-09)

See [Deprecating SystemTable/BootServices/RuntimeServices][funcmigrate] for
details of the deprecations in this release.

We added documentation to `lib.rs` and the [uefi-rs book] about how
`uefi` compares to "standard Rust binaries" for UEFI (those using `std`), and
how to integrate the `uefi` crate into them.

## Added
- Added `Handle::new`.
- Added the `uefi::boot`, `uefi::runtime`, and `uefi::system` modules to the
  prelude.
- Added `runtime::variable_exists`.

## Changed
- The `BootServices`, `RuntimeServices`, and `SystemTable` structs have been
  deprecated (as well as related types `Boot`, `Runtime`, and
  `SystemTableView`). Use the `uefi::boot` and `uefi::runtime`, and
  `uefi::system` modules instead.
- In `uefi::table::boot`, `ScopedProtocol`, `TplGuard`, `ProtocolsPerHandle`,
  and `HandleBuffer` have been deprecated. Use the structs of the same name in
  `uefi::boot` instead.
- `uefi::table::system_table_boot` and `uefi::table::system_table_runtime` have
  been deprecated. Use the `uefi::runtime` and `uefi::boot` modules instead.
- **Breaking:** The conversion functions between device paths and text no longer
  take a `BootServices` argument. The global system table is used instead.
- **Breaking:** `GraphicsOutput::modes` no longer takes a `BootServices`
  argument. The global system table is used instead.
- **Breaking:** `ComponentName::open` no longer takes a `BootServices`
  argument. The global system table is used instead.
- `allocator::init` and `allocator::exit_boot_services` have been
  deprecated. These functions are now no-ops. The allocator now internally uses
  the global system table.
- `FileSystem::new` now accepts `boot::ScopedProtocol` in addition to
  `table::boot::ScopedProtocol`.


# uefi - 0.31.0 (2024-08-21)

See [Deprecating SystemTable/BootServices/RuntimeServices][funcmigrate] for
details of the new `system`/`boot`/`runtime` modules, and upcoming deprecations.

## Added
- `uefi::system` is a new module that provides freestanding functions for
  accessing fields of the global system table.
- `uefi::boot` is a new module that provides freestanding functions for
  boot services using the global system table.
- `uefi::runtime` is a new module that provides freestanding functions for
  runtime services using the global system table.
- `uefi::table::system_table_raw` is a new function to retrieve a raw pointer to
  the global system table.
- Add standard derives for `ConfigTableEntry`.
- `PcrEvent`/`PcrEventInputs` impl `Align`, `Eq`, and `PartialEq`.
- Added `PcrEvent::new_in_box` and `PcrEventInputs::new_in_box`.
- `VariableKey` impls `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, and `Hash`.
- The traits `MemoryMap` and `MemoryMapMut` have been introduced together with
  the implementations `MemoryMapRef`, `MemoryMapRefMut`, and `MemoryMapOwned`.
  This comes with some changes. Read below. We recommend to directly use the
  implementations instead of the traits.
- Added `LoadFile` and `LoadFile2` which abstracts over the `LOAD_FILE` and
  `LOAD_FILE2` protocols. The UEFI test runner includes an integration test
  that shows how Linux loaders can use this to implement the initrd loading
  mechanism used in Linux.

## Changed
- **Breaking:** `uefi::helpers::init` no longer takes an argument.
- The lifetime of the `SearchType` returned from
  `BootServices::register_protocol_notify` is now tied to the protocol GUID.
  The old `MemoryMap` was renamed to `MemoryMapOwned`.
  - `pub fn memory_map(&self, mt: MemoryType) -> Result<MemoryMap>` now returns
     a `MemoryMapOwned`.
- **Breaking:** `PcrEvent::new_in_buffer` and `PcrEventInputs::new_in_buffer`
  now take an initialized buffer (`[u8`] instead of `[MaybeUninit<u8>]`), and if
  the buffer is too small the required size is returned in the error data.
- **Breaking:** The type `MemoryMap` was renamed to `MemoryMapOwned`. `MemoryMap`
  is now a trait. Read the [documentation](https://docs.rs/uefi/latest/uefi/) of the
  `uefi > mem > memory_map` module to learn more.
- **Breaking:** Exports of Memory Map-related types from `uefi::table::boot` are
  now removed. Use `uefi::mem::memory_map` instead. The patch you have to apply
  to the `use` statements of your code might look as follows:
  ```diff
  < use uefi::table::boot::{BootServices, MemoryMap, MemoryMapMut, MemoryType};
  ---
  > use uefi::mem::memory_map::{MemoryMap, MemoryMapMut, MemoryType};
  > use uefi::table::boot::BootServices;
  ```
- **Breaking:** Added a new `BootPolicy` type which breaks existing usages
  of `LoadImageSource`.

[funcmigrate]: ../docs/funcs_migration.md

# uefi - 0.30.0 (2024-08-02)

## Changed
- **Breaking:**: Fixed a bug in the impls of `TryFrom<&[u8]>` for
  `&DevicePathHeader`, `&DevicePathNode` and `&DevicePath` that could lead to
  memory unsafety. See <https://github.com/rust-osdev/uefi-rs/issues/1281>.


# uefi - 0.29.0 (2024-07-02)

## Added
- Added `RuntimeServices::update_capsule`.
- Added `RuntimeServices::query_capsule_capabilities`.
- The logger from `uefi::helpers` now also logs to the [debugcon](https://phip1611.de/blog/how-to-use-qemus-debugcon-feature/)
  device (QEMU) respectively the debug-console (cloud-hypervisor). This only
  works on x86. It is activated by default (only on x86) and can be deactivated
  by removing the `log-debugcon` cargo feature. The major benefit is that one
  can get log messages even after one exited the boot services.
- Added `table::{set_system_table, system_table_boot, system_table_runtime}`.
  This provides an initial API for global tables that do not require passing
  around a reference.
- Added `ResetNotification` protocol.
- Added `TryFrom<&[u8]>` for `DevicePathHeader`, `DevicePathNode` and `DevicePath`.
- Added `ByteConversionError`.
- Re-exported `CapsuleFlags`.
- One can now specify in `TimeError` what fields of `Time` are outside its valid
  range. `Time::is_valid` has been updated accordingly.
- `MemoryMap::as_raw` which provides raw access to the memory map. This is for
  example useful if you create your own Multiboot2 bootloader that embeds the
  EFI mmap in a Multiboot2 boot information structure.
- `Mode` is now `Copy` and `Clone`.
- Added `TryFrom<&[u8]>` for `Time`.

## Changed
- `SystemTable::exit_boot_services` is now `unsafe`. See that method's
  documentation for details of obligations for callers.
- `BootServices::allocate_pool` now returns `NonZero<u8>` instead of
  `*mut u8`.
- `helpers::system_table` is deprecated, use `table::system_table_boot` instead.
- `BootServices::memory_map` changed its signature from \
  `pub fn memory_map<'buf>(&self, buffer: &'buf mut [u8]) -> Result<MemoryMap<'buf>> {` \
  to \
  `pub fn memory_map(&self, mt: MemoryType) -> Result<MemoryMap>`
  - Allocations now happen automatically internally on the UEFI heap. Also, the
    returned type is automatically freed on the UEFI heap, as long as boot
    services are not excited. By removing the need for that explicit buffer and
    the lifetime, the API is simpler.
- `GraphicsOutput::query_mode` is now private. Use `GraphicsOutput::modes`
  instead.

## Removed
- Removed the `panic-on-logger-errors` feature of the `uefi` crate. Logger
  errors are now silently ignored.


# uefi - 0.28.0 (2024-04-19)

## Added
- Added `Timestamp` protocol.
- Added `UnalignedSlice::as_ptr`.
- Added common derives for `Event` and `Handle`.
- `uefi::helpers::init` with the functionality that used to be in
`uefi::services`. With that, new features were added:
- `global_allocator`
- `panic_handler`
- `qemu`


# uefi - 0.27.0 (2024-03-17)

## Added
- Implemented `PartialEq<char>` for `Char8` and `Char16`.
- Added `CStr16::from_char16_with_nul` and `Char16::from_char16_with_nul_unchecked`.
- Added terminal GUID constants to `device_path::messaging::Vendor`.
- Added `MemoryMap::from_raw`.
- Implemented `Hash` for all char and string types.

## Changed
- `DevicePath::to_string` and `DevicePathNode::to_string` now return
  out-of-memory errors as part of the error type rather than with an `Option`.


# uefi - 0.26.0 (2023-11-12)

## Added
- Implemented `Index`, `IndexMut`, `get`, and `get_mut` on `MemoryMap`.
- Added `SystemTable::as_ptr`.

## Changed
- We fixed a memory leak in `GraphicsOutput::query_mode`. As a consequence, we
  had to add `&BootServices` as additional parameter.
- `BootServices::free_pages` and `BootServices::free_pool` are now `unsafe` to
  call, since it is possible to trigger UB by freeing memory that is still in use.
- `Logger` no longer requires exterior mutability. `Logger::new` is now `const`,
  takes no arguments, and creates the logger in a disabled state. Call
  `Logger::set_output` to enable it.
- `uefi::allocator::init` now takes a `&mut SystemTable<Boot>` instead of
  `&BootServices`.
- `BootServices::{install,reinstall,uninstall}_protocol_interface` now take
  `const` interface pointers.


# uefi - 0.25.0 (2023-10-10)

## Changed
- MSRV bumped to 1.70.
- `Input::wait_for_key_event` now returns an `Option<Event>`, and is no longer `const`.
- `Protocol::wait_for_input_event` now returns an `Option<Event>`, and is no longer `const`.
- `LoadedImage::device` now returns an `Option<Handle>` and is no longer `const`.
- `BootServices::get_image_file_system` now returns
  `ScopedProtocol<SimpleFileSystem>` instead of `fs::FileSystem`.
- `uefi::proto::shim` is now available on 32-bit x86 targets.
- `Parity` and `StopBits` are now a newtype-enums instead of Rust enums. Their
  members now have upper-case names.
- `FileSystem::try_exists` now returns `FileSystemResult<bool>`.
- `FileSystem::copy` is now more efficient for large files.
- `MpService::startup_all_aps` and `MpService::startup_this_ap` now accept an
    optional `event` parameter to allow non-blocking operation.
- Added `core::error::Error` implementations to all error types.
- `SystemTable::exit_boot_services` now takes one param `memory_type` to ensure
  the memory type of memory map.
- Added the `ShellParams` protocol

## Removed
- `BootServices::memmove` and `BootServices::set_mem` have been removed, use
  standard functions like `core::ptr::copy` and `core::ptr::write_bytes` instead.


# uefi - 0.24.0 (2023-06-20)

## Added
- `DevicePath::to_boxed`, `DevicePath::to_owned`, and `DevicePath::as_bytes`
- `DevicePathInstance::to_boxed`, `DevicePathInstance::to_owned`, and `DevicePathInstance::as_bytes`
- `DevicePathNode::data`
- Added `Event::from_ptr`, `Event::as_ptr`, and `Handle::as_ptr`.
- Added `ScopedProtocol::get` and `ScopedProtocol::get_mut` to access
  potentially-null interfaces without panicking.
- `DevicePath::to_string` and `DevicePathNode::to_string`

## Changed
- Renamed `LoadImageSource::FromFilePath` to `LoadImageSource::FromDevicePath`
- The `Deref` and `DerefMut` impls for `ScopedProtocol` will now panic if the
  interface pointer is null.


# uefi - 0.23.0 (2023-06-04)

## Changed
- Fixed function signature bug in `BootServices::install_configuration_table`.


# uefi - 0.22.0 (2023-06-01)

## Added
- Added `BootServices::install_configuration_table`.

## Changed
- Renamed `FileSystemIOErrorContext` to `IoErrorContext`.
- `ResetType` is now a newtype-enum instead of a Rust enum. Its members now have
  upper-case names.
- `PointerMode` and `PointerState` now contain arrays rather than tuples, as
  tuples are not FFI safe.
- `RegularFile::read` no longer returns `Option<usize>` in error data. A
  `BUFFER_TOO_SMALL` error can only occur when reading a directory, not a file.
- `RegularFile::read` now reads in 1 MiB chunks to avoid a bug in some
  firmware. This fix also applies to `fs::FileSystem::read`.


# uefi - 0.21.0 (2023-05-15)

## Added
- There is a new `fs` module that provides a high-level API for file-system
  access. The API is close to the `std::fs` module. The module also provides a
  `Path` and a `PathBuf` abstraction that is similar to the ones from
  `std::path`. However, they are adapted for UEFI.
- Multiple convenience methods for `CString16` and `CStr16`, including:
  - `CStr16::as_slice()`
  - `CStr16::num_chars()`
  - `CStr16::is_empty()`
  - `CString16::new()`
  - `CString16::is_empty()`
  - `CString16::num_chars()`
  - `CString16::replace_char()`
  - `CString16::push()`
  - `CString16::push_str()`
  - `From<&CStr16>` for `CString16`
  - `From<&CStr16>` for `String`
  - `From<&CString16>` for `String`
- Added `RuntimeServices::get_variable_boxed` (requires the `alloc` feature).
- Added `CStr16::as_bytes`
- Added `AsRef<[u8]>` and `Borrow<[u8]>` for `Cstr8` and `CStr16`.
- Added `LoadedImageDevicePath` protocol.
- Added `FileAttribute::is_directory(&self)` and
  `FileAttribute::is_regular_file(&self)`
- Added `LoadedImage::code_type()` and `LoadedImage::data_type()`
- `Allocator` will now use the memory type of the running UEFI binary:
  - `MemoryType::LOADER_DATA` for UEFI applications
  - `MemoryType::BOOT_SERVICES_DATA` for UEFI boot drivers
  - `MemoryType::RUNTIME_SERVICES_DATA` for UEFI runtime drivers

## Changed
- The `global_allocator` module has been renamed to `allocator`, and is now
  available regardless of whether the `global_allocator` feature is enabled. The
  `global_allocator` feature now only controls whether `allocator::Allocator` is
  set as Rust's global allocator.
- `Error::new` and `Error::from` now panic if the status is `SUCCESS`.
- `Image::get_image_file_system` now returns a `fs::FileSystem` instead of the
  protocol.
- `CString16::default` now always contains a null character.
- Conversion from `Status` to `Result` has been reworked. The `into_with`,
  `into_with_val`, and `into_with_err` methods have been removed from
  `Status`. `impl From<Status> for Result` has also been removed. A new
  `StatusExt` trait has been added that provides conversion methods to replace
  the ones that have been removed. `StatusExt` has been added to the prelude.
- The `Guid` struct and `guid!` macro implementations have been replaced with
  re-exports from the [`uguid`](https://docs.rs/uguid) crate. The `from_values`
  method has been removed; usually the `guid!` macro is a more convenient
  choice, but `new` or `from_bytes` can also be used if needed. There are also a
  number of new `Guid` methods.
- The `MEMORY_DESCRIPTOR_VERSION` constant has been moved to
  `MemoryDescriptor::VERSION`.
- The `Revision` struct's one field is now public.
- Renamed `CStr8::to_bytes` to `CStr8::as_bytes` and changed the semantics:
  The trailing null character is now always included in the returned slice.
- `DevicePathBuilder::with_vec` now clears the `Vec` before use.
- `bitflags` bumped from `1.3` to `2.1`
  - `GptPartitionAttributes` now has 16 additional `TYPE_SPECIFIC_BIT_<N>`
    constants.


# uefi - 0.20.0 (2023-03-19)

As of this release, the UEFI crates work on the stable channel. This requires
Rust 1.68 or higher.

## Added
- Added the `ComponentName1` and `ComponentName2` protocols. The `ComponentName`
  wrapper will automatically select `ComponentName2` if available, and fall back
  to `ComponentName1` otherwise.
- `FileType`, `FileHandle`, `RegularFile`, and `Directory` now implement `Debug`.
- Added `RuntimeServices::delete_variable()` helper method.
- Implement `Borrow` for `CString16` and `ToOwned` for `CStr16`.
- Every public struct now implements `Debug`. Exceptions are cases when there
  is no sensible way of presenting a useful Debug representation, such as for
  Unions.

## Changed
- `SystemTable::exit_boot_services` now takes no parameters and handles
  the memory map allocation itself. Errors are now treated as
  unrecoverable and will cause the system to reset.
- Re-export the `cstr8`, `cstr16`, and `entry` macros from the root of the
  `uefi` crate.
- `HandleBuffer` and `ProtocolsPerHandle` now implement `Deref`. The
  `HandleBuffer::handles` and `ProtocolsPerHandle::protocols` methods have been
  deprecated.
- Removed `'boot` lifetime from the `GraphicsOutput`, `Output`, `Pointer`, and
  `Serial` protocols.
- The generic type `Data` of `uefi::Error<Data: Debug>` doesn't need to be
  `Display` to be compatible with `core::error::Error`. Note that the error
  Trait requires the `unstable` feature.
- deprecation removals:
  - interfaces `BootServices::locate_protocol` and
    `BootServices::handle_protocol` were removed. `BootServices::open_protocol`
    and `BootServices::open_protocol_exclusive` are better variants and
    available since EFI 1.10 (2002).
  - `ScopedProtocol::interface` is not public anymore. Use the `Deref` trait.


# uefi - 0.19.1 (2023-02-04)

## Added
- Added `table::boot::PAGE_SIZE` constant.

## Changed
- Fixed several protocol functions so that they work with unsized protocols
  (like `DevicePath`): `BootServices::locate_device_path`,
  `BootServices::get_handle_for_protocol`, `BootServices::test_protocol`,
  `BootServices::find_handles`, and `SearchType::from_proto`.
- Fixed a warning printed when using `uefi` as a dependency: "the following
  packages contain code that will be rejected by a future version".


# uefi - 0.19.0 (2023-01-16)

## Added
- Implementations for the trait `EqStrUntilNul` now allow `?Sized` inputs. This means that
  you can write `some_cstr16.eq_str_until_nul("test")` instead of
  `some_cstr16.eq_str_until_nul(&"test")` now.
- Added `TryFrom<core::ffi::CStr>` implementation for `CStr8`.
- Added `Directory::read_entry_boxed` which works similar to `File::get_boxed_info`. This allows
  easier iteration over the entries in a directory. (requires the **alloc** feature)
- Added `Directory::read_entry_boxed_in` and `File::get_boxed_info_in` that use the `allocator_api`
  feature. (requires the **unstable** and **alloc** features)
- Added an `core::error::Error` implementation for `Error` to ease
  integration with error-handling crates. (requires the **unstable** feature)
- Added partial support for the TCG protocols for TPM devices under `uefi::proto::tcg`.

## Changed
- `UnalignedSlice` now implements `Clone`, and the `Debug` impl now
  prints the elements instead of the internal fields.
- The unstable `negative_impls` feature is no longer required to use this library.
- `BootServices::memory_map()` now returns `MemoryMapIter` instead of
  `impl Iterator` which simplifies usage.
- `BootServices::exit_boot_services()` now returns `MemoryMapIter` instead of
  `impl Iterator` which simplifies usage.
- `GraphicsOutput::modes()` now returns `ModesIter` instead of `impl Iterator`
   which simplifies usage.
- Use of the unstable `ptr_metadata` feature has been replaced with a dependency
  on the [`ptr_meta`](https://docs.rs/ptr_meta) crate.
- `pxe::DiscoverInfo` is now a DST. Create with `new_in_buffer` by supplying a
  `MaybeUninit<u8>` slice of appropriate length.
- Redundant private field used for padding in `MemoryDescriptor` structure was removed. Now all
  fields of this struct are public.


# uefi - 0.18.0 (2022-11-15)

## Added
- Added `PhysicalAddress` and `VirtualAddress` type aliases.
- Added `Guid::from_bytes` and `Guid::to_bytes`.
- Added `UnalignedSlice` for representing a reference to an unaligned
  slice.
- Added `DeviceSubType::MESSAGING_REST_SERVICE` and
  `DeviceSubType::MESSAGING_NVME_OF_NAMESPACE`.
- Added `MemoryAttribute::SPECIAL_PURPOSE`, `MemoryAttribute::CPU_CRYPTO`,
  `MemoryAttribute::ISA_VALID`, and `MemoryAttribute::ISA_MASK`.
- Added the `UnicodeCollation` protocol
- Added structs to represent each type of device path node. All node
  types specified in the UEFI 2.10 Specification are now supported.
- Added `DevicePathBuilder` for building new device paths.
- Added `BootServices::install_protocol_interface`,
  `BootServices::uninstall_protocol_interface`, and
  `BootServices::reinstall_protocol_interface`.
- Added `BootServices::register_protocol_notify`.
- Added `SearchType::ByRegisterNotify`and `ProtocolSearchKey`.

## Changed
- Renamed crate feature `alloc` to `global_allocator`.
- Renamed crate feature `exts` to `alloc`.
- Fixed the definition of `AllocateType` so that `MaxAddress` and
  `Address` always take a 64-bit value, regardless of target platform.
- The conversion methods on `DevicePathToText` and `DevicePathFromText`
  now return a `uefi::Result` instead of an `Option`.
- `Event` is now a newtype around `NonNull<c_void>` instead of `*mut c_void`.
- Changed `SystemTable::firmware_revision` to return a `u32` instead of
  a `Revision`. The firmware revision's format is vendor specific and
  may not have the same semantics as the UEFI revision.
- Changed `Revision` to `repr(transparent)`.
- Add `Revision::EFI_2_100` constant.
- The `Revision` type now implements `Display` with correct formatting
  for all UEFI versions. The custom `Debug` impl has been removed and
  replaced with a derived `Debug` impl.
- `CStr16::from_u16_with_nul_unchecked` and `cstr16!` are now allowed in
  `const` contexts.

## Removed
- Removed `UnalignedCStr16`; use `UnalignedSlice` instead. An
  `UnalignedSlice<u16>` can be converted to a string with `to_cstr16` or
  `to_cstring16`.
- Removed `as_file_path_media_device_path` and
  `as_hard_drive_media_device_path` from `DevicePathNode`. Use
  `DevicePathNode::as_enum` instead. Alternatively, convert with `TryInto`,
  e.g. `let node: &proto::device_path::media::HardDrive = node.try_into()?`.
- Removed `AcpiDevicePath` and `HardDriveMediaDevicePath`. Use
  `proto::device_path::acpi::Acpi` and
  `proto::device_path::media::HardDrive` instead.  `


# uefi - 0.17.0 (2022-09-09)

## Added
- Added `Deref` and `DerefMut` trait implementations to `ScopedProtocol`.
  This eliminates the need to explicitly access the `interface` field,
  which is now marked as deprecated.
- Implemented `core::fmt::Write` for the `Serial` protocol.
- Added the `MemoryProtection` protocol.
- Added `BootServices::get_handle_for_protocol`.
- Added trait `EqStrUntilNul` and implemented it for `CStr8`, `CStr16`, and `CString16`
  (CString8 doesn't exist yet). Now you can compare everything that is `AsRef<str>`
  (such as `String` and `str` from the standard library) to UEFI strings. Please head to the
  documentation of `EqStrUntilNul` to find out limitations and further information.
- Added `BootServices::image_handle` to get the handle of the executing
  image. The image is set automatically by the `#[entry]` macro; if a
  program does not use that macro then it should call
  `BootServices::set_image_handle`.
- Added `BootServices::open_protocol_exclusive`. This provides a safe
  and convenient subset of `open_protocol` that can be used whenever a
  resource doesn't need to be shared. In same cases sharing is useful
  (e.g. you might want to draw to the screen using the graphics
  protocol, but still allow stdout output to go to the screen as
  well), and in those cases `open_protocol` can still be used.
- Added `DiskIo` and `DiskIo2` protocols.
- Added `HardDriveMediaDevicePath` and related types.
- Added `PartialOrd` and `Ord` to the traits derived by `Guid`.
- The `File` trait now knows the methods `is_regular_file` and `is_directory`.
  Developers profit from this on the struct `FileHandle`, for example.

## Changed
- Marked `BootServices::handle_protocol` as `unsafe`. (This method is
  also deprecated -- use `open_protocol_exclusive` or `open_protocol` instead.)
- Deprecated `BootServices::locate_protocol` and marked it `unsafe`. Use
  `BootServices::get_handle_for_protocol` and
  `BootServices::open_protocol_exclusive` (or
  `BootServices::open_protocol`) instead.
- Renamed feature `ignore-logger-errors` to `panic-on-logger-errors` so that it is
  additive. It is now a default feature.
- Corrected the name of `BlockIOMedia::is_media_preset` to `is_media_present`.

## Removed
- Removed the `exts::allocate_buffer` function. This function could
  cause undefined behavior when called with a `Layout` with an alignment
  other than 1. A safe alternative is to use
  [`Vec::into_boxed_slice`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_boxed_slice).
- Removed `From` conversions from `ucs2::Error` to `Status` and `Error`.
- Removed use of the unstable `try_trait_v2` feature, which allowed `?`
  to be used with `Status` in a function returning `uefi::Result`. This
  can be replaced by calling `status.into()`, or `Result::from(status)`
  in cases where the compiler needs a type hint.


# uefi - 0.16.1

## Added
- Added EFI revision constants to `Revision`.

## Fixed
- The table `Header` struct's `Debug` impl now prints the correct signature.
- The `BootServices::create_event_ex` and
  `RuntimeServices::query_variable_info` methods now check the table
  version to make sure it's 2.0 or higher before calling the associated
  function pointers. This prevents potential invalid pointer access.
- Fixed an incorrect pointer cast in the `Rng` protocol that could cause
  undefined behavior.

## Changed
- Relaxed the version requirements for the `bitflags` and `log`
  dependencies to allow earlier patch versions.
- Enabled `doc_auto_cfg` on docs.rs to show badges on items that are
  gated behind a feature.


# uefi - 0.16.0 (2022-05-16)

## Added
- Added `FileHandle::into_directory` and `FileHandle::into_regular_file`.
- Added `TimeParams`, `Time::invalid`, and `Time::is_invalid`.
- Added `RuntimeServices::query_variable_info` and `VariableStorageInfo`.
- Added `DevicePathToText` and `DevicePathFromText`.
- Added `LoadedImage::file_path`
- Implemented `TryFrom<Vec<u16>> for CString16`.
- Added `UnalignedCStr16`.
- Added `FilePathMediaDevicePath`.
- Added `DevicePath::as_acpi_device_path` and
  `DevicePath::as_file_path_media_device_path`.
- Included `cstr8` and `cstr16` macros from `uefi-macros` in the prelude.
- Added `DevicePathInstance`, `DevicePathNode`, and `FfiDevicePath`.

## Changed
- `Time::new` now takes a single `TimeParams` argument so that date and
  time fields can be explicitly named at the call site.
- The file info types now derive `PartialEq` and `Eq`.
- The `FileAttributes` type is now `repr(transparent)`.
- `DevicePath` is now a DST that represents an entire device path. The
  `DevicePathInstance` and `DevicePathNode` provide views of path
  instances and nodes, respectively.
- The methods of `Revision` are now `const`.

## Fixed

- Fixed undefined behavior in `proto::media::file::File::get_boxed_info`.


# uefi - 0.15.2 (2022-03-15)

## Added
- Added `PartialEq` impls for `CStr16 == CStr16`, `&CStr16 == CString`,
  and `CString == &CStr16`.
- Added `Display` impl for `CString16`.
- Added `Handle::from_ptr` and `SystemTable<View>::from_ptr`, which are
  `unsafe` methods for initializing from a raw pointer.
- Added `CStr16::as_slice_with_nul` to provide immutable access to the
  underlying slice.
- Added `LoadedImage::load_options_as_bytes` and
  `LoadedImage::load_options_as_cstr16`.
- Added `Align::offset_up_to_alignment`, `Align::round_up_to_alignment`,
  and `Align::align_buf`.
- Added `BootServices::connect_controller` and
  `BootServices::disconnect_controller`.
- Added `BootServices::load_image` and `LoadImageSource`. Together these
  replace `BootServices::load_image_from_buffer` and also allow an image
  to be loaded via the `SimpleFileSystem` protocol.
- Added `Rng` protocol.
- Added `GptPartitionAttributes` struct and associated constants.
- Added `Output::output_string_lossy`.
- Added `ResultExt::handle_warning`.

## Changed
- Updated to the 2021 edition.
- `File::open` now takes the filename as `&CStr16` instead of `&str`,
  avoiding an implicit string conversion.
- `FileInfo::new`, `FileSystemInfo::new`, and
  `FileSystemVolumeLabel::new` now take their `name` parameter as
  `&CStr16` instead of `&str`, avoiding an implicit string
  conversion. Additionally, an unaligned storage buffer is now allowed
  as long as it is big enough to provide an aligned subslice.
- `LoadImage::set_load_options` now takes a `u8` pointer instead of
  `Char16`.
- The `Error` type is now public.
- The type of `GptPartitionEntry.attributes` is now
  `GptPartitionAttributes`.
- The `uefi::Result` type now treats UEFI warnings as errors by
  default. The `uefi::Result::Ok` variant no longer contains a
  `Completion`, so the type behaves more like a regular Rust `Result`
  type.

## Removed
- Removed `CStr16::as_string` method. Use
  [`ToString`](https://doc.rust-lang.org/alloc/string/trait.ToString.html)
  instead.
- Removed `FileInfoCreationError::InvalidChar`. This error type is no
  longer needed due to the removal of implicit string conversions in
  file info types.
- Removed `LoadedImage::load_options`, use
  `LoadedImage::load_options_as_bytes` or
  `LoadedImage::load_options_as_cstr16` instead.
- Removed `NamedFileProtocolInfo`, `FileInfoHeader`,
  `FileSystemInfoHeader`, and `FileSystemVolumeLabelHeader`. Use
  `FileInfo`, `FileSystemInfo`, and `FileSystemVolumeLabel` instead.
- Removed `BootServices::load_image_from_buffer`. Use
  `BootServices::load_image` instead.
- Removed `Completion` type. Warnings are now treated as errors.
- Removed many `ResultExt` methods, for most of them the standard
  `Result` methods can be used instead. Use `unwrap` instead of
  `unwrap_success`, `expect` instead of `expect_success`, `expect_err`
  instead of `expect_error`, and `map` instead of `map_inner`. The
  `log_warning` method has also been removed, use the new
  `ResultExt::handle_warning` method instead.

## Fixed
- Fixed compilation with Rust 1.60 by no longer enabling the
  `vec_spare_capacity` feature, which has been stabilized.
- Fixed the header size calculated by `FileInfo::new` and
  `FileSystemInfo::new`.
- Fixed incorrect alignment of the volume label field in
  `FileSystemInfo`. This caused the beginning of the string to be
  truncated and could result in out-of-bounds reads.
- Fixed size check for file info types so that alignment padding is
  taken into account. This fixes potential out-of-bounds writes.


[uefi-rs book]: https://rust-osdev.github.io/uefi-rs/HEAD
