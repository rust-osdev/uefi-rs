# Changelog

## uefi - [Unreleased]

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

### Changed

- `Time::new` now takes a single `TimeParams` argument so that date and
  time fields can be explicitly named at the call site.
  
### Fixed

- Fixed undefined behavior in `proto::media::file::File::get_boxed_info`.

## uefi-macros - [Unreleased]

## uefi-services - [Unreleased]

## uefi - 0.15.2

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

## uefi-macros - 0.6.1

### Changed

- Updated to the 2021 edition.

## uefi-services - 0.12.1

### Changed

- Updated to the 2021 edition.
