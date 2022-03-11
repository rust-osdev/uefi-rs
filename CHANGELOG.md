# Changelog

## uefi - [Unreleased]

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

## uefi-macros - [Unreleased]

### Changed

- Updated to the 2021 edition.

## uefi-services - [Unreleased]

### Changed

- Updated to the 2021 edition.
