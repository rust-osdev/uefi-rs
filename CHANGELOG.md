# Changelog

## uefi - [Unreleased]

### Added

- There is a new `fs` module that provides a high-level API for file-system
  access. The API is close to the `std::fs` module.
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

### Changed

- The `global_allocator` module has been renamed to `allocator`, and is now
  available regardless of whether the `global_allocator` feature is enabled. The
  `global_allocator` feature now only controls whether `allocator::Allocator` is
  set as Rust's global allocator.
- `Error::new` and `Error::from` now panic if the status is `SUCCESS`.
- `Image::get_image_file_system` now returns a `fs::FileSystem` instead of the
  protocol.
- `CString16::default` now always contains a null character.

## uefi-macros - [Unreleased]

## uefi-services - [Unreleased]

## uefi - 0.20.0 (2023-03-19)

As of this release, the UEFI crates work on the stable channel. This requires
Rust 1.68 or higher.

### Added

- Added the `ComponentName1` and `ComponentName2` protocols. The `ComponentName`
  wrapper will automatically select `ComponentName2` if available, and fall back
  to `ComponentName1` otherwise.
- `FileType`, `FileHandle`, `RegularFile`, and `Directory` now implement `Debug`.
- Added `RuntimeServices::delete_variable()` helper method.
- Implement `Borrow` for `CString16` and `ToOwned` for `CStr16`.
- Every public struct now implements `Debug`. Exceptions are cases when there
  is no sensible way of presenting a useful Debug representation, such as for
  Unions.

### Changed

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

## uefi-macros - 0.11.0 (2023-03-19)

### Changed

- Errors produced by the `entry` macro have been improved.

## uefi-services - 0.17.0 (2023-03-19)

### Changed

- Drop use of unstable `alloc_error_handler` feature. As of Rust 1.68 we can use
  [`default_alloc_error_handler`](https://github.com/rust-lang/rust/pull/102318)
  instead.

## uefi - 0.19.1 (2023-02-04)

### Added

- Added `table::boot::PAGE_SIZE` constant.

### Changed

- Fixed several protocol functions so that they work with unsized protocols
  (like `DevicePath`): `BootServices::locate_device_path`,
  `BootServices::get_handle_for_protocol`, `BootServices::test_protocol`,
  `BootServices::find_handles`, and `SearchType::from_proto`.
- Fixed a warning printed when using `uefi` as a dependency: "the following
  packages contain code that will be rejected by a future version".

## uefi - 0.19.0 (2023-01-16)

### Added

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

### Changed

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

## uefi-macros - 0.10.0 (2023-01-16)

### Added

- Added the `unsafe_protocol` macro to provide a slightly nicer way to
  implement protocols.

### Removed

- The `unsafe_guid` attribute macro and `Protocol` derive macro have
  been removed. For implementing protocols, use the `unsafe_protocol`
  macro instead. For any other implementations of the `Identify` trait,
  implement it directly.

## uefi-services - 0.16.0 (2023-01-16)

No changes in this release except depending on a newer version of `uefi`.

## uefi - 0.18.0 (2022-11-15)

### Added

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

### Changed

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

### Removed

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

## uefi-macros - 0.9.0 (2022-11-15)

### Added

- Added a `guid!` macro. This is similar to `Guid::from_values`, but
  takes a more convenient string argument like the `unsafe_guid!`
  attribute macro.

## uefi-services - 0.15.0 (2022-11-15)

### Changed

- Changed the panic handler log message to use `println!` instead of
  `error!`. This removes an extraneous file name and line number from
  the log message.

- Added a `logger` feature which reflects the same feature in `uefi`.
  This allows using both crates while disabling `logger` in `uefi`,
  which was previously impossible.

## uefi - 0.17.0 (2022-09-09)


### Added

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

### Changed

- Marked `BootServices::handle_protocol` as `unsafe`. (This method is
  also deprecated -- use `open_protocol_exclusive` or `open_protocol` instead.)
- Deprecated `BootServices::locate_protocol` and marked it `unsafe`. Use
  `BootServices::get_handle_for_protocol` and
  `BootServices::open_protocol_exclusive` (or
  `BootServices::open_protocol`) instead.
- Renamed feature `ignore-logger-errors` to `panic-on-logger-errors` so that it is
  additive. It is now a default feature.
- Corrected the name of `BlockIOMedia::is_media_preset` to `is_media_present`.

### Removed

- Removed the `exts::allocate_buffer` function. This function could
  cause undefined behavior when called with a `Layout` with an alignment
  other than 1. A safe alternative is to use
  [`Vec::into_boxed_slice`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_boxed_slice).
- Removed `From` conversions from `ucs2::Error` to `Status` and `Error`.
- Removed use of the unstable `try_trait_v2` feature, which allowed `?`
  to be used with `Status` in a function returning `uefi::Result`. This
  can be replaced by calling `status.into()`, or `Result::from(status)`
  in cases where the compiler needs a type hint.

## uefi-macros - 0.8.0 (2022-09-09)

### Changed

- The `#[entry]` macro now calls `BootServices::set_image_handle` to set
  the global image handle. Due to this change, the two arguments to main
  must both be named (e.g. `image: Handle` and `_image: Handle` are both
  OK, but not `_: Handle`).

## uefi-services - 0.14.0 (2022-09-09)

### Added

- Added `print!` and `println!` macros.

### Changed

- The `no_panic_handler` feature has been replaced with an additive
  `panic_handler` feature. The new feature is enabled by default.
## uefi - 0.16.1

### Added

- Added EFI revision constants to `Revision`.

### Fixed

- The table `Header` struct's `Debug` impl now prints the correct signature.
- The `BootServices::create_event_ex` and
  `RuntimeServices::query_variable_info` methods now check the table
  version to make sure it's 2.0 or higher before calling the associated
  function pointers. This prevents potential invalid pointer access.
- Fixed an incorrect pointer cast in the `Rng` protocol that could cause
  undefined behavior.

### Changed

- Relaxed the version requirements for the `bitflags` and `log`
  dependencies to allow earlier patch versions.
- Enabled `doc_auto_cfg` on docs.rs to show badges on items that are
  gated behind a feature.

## uefi-macros - 0.7.1 (2022-08-26)

### Changed

- Relaxed the version requirements for the `proc-macro2`, `quote`, and
  `sync` dependencies to allow earlier patch versions.

## uefi-services - 0.13.1 (2022-08-26)

### Changed

- Relaxed the version requirements for the `log` dependency to allow
  earlier patch versions.

## uefi - 0.16.0 (2022-05-16)

### Added

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

### Changed

- `Time::new` now takes a single `TimeParams` argument so that date and
  time fields can be explicitly named at the call site.
- The file info types now derive `PartialEq` and `Eq`.
- The `FileAttributes` type is now `repr(transparent)`.
- `DevicePath` is now a DST that represents an entire device path. The
  `DevicePathInstance` and `DevicePathNode` provide views of path
  instances and nodes, respectively.
- The methods of `Revision` are now `const`.

### Fixed

- Fixed undefined behavior in `proto::media::file::File::get_boxed_info`.

## uefi-macros - 0.7.0 (2022-05-16)

### Added

- Added `cstr8` and `cstr16` macros for creating `CStr8`/`CStr16` string literals
  at compile time.

## uefi-services - 0.13.0 (2022-05-16)

### Changed

- Bumped `uefi` dependency to latest version.

## uefi - 0.15.2 (2022-03-15)

### Added

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

### Changed

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

### Removed

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

### Fixed

- Fixed compilation with Rust 1.60 by no longer enabling the
  `vec_spare_capacity` feature, which has been stabilized.
- Fixed the header size calculated by `FileInfo::new` and
  `FileSystemInfo::new`.
- Fixed incorrect alignment of the volume label field in
  `FileSystemInfo`. This caused the beginning of the string to be
  truncated and could result in out-of-bounds reads.
- Fixed size check for file info types so that alignment padding is
  taken into account. This fixes potential out-of-bounds writes.

## uefi-macros - 0.6.1 (2022-03-15)

### Changed

- Updated to the 2021 edition.

## uefi-services - 0.12.1 (2022-03-15)

### Changed

- Updated to the 2021 edition.
