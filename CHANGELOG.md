# Changelog

## [Unreleased]

### Added

- Added `PartialEq` impls for `CStr16 == CStr16`, `&CStr16 == CString`,
  anded `CString == &CStr16`.
- Added `Display` impl for `CString16`.
- Added `Handle::from_ptr` and `SystemTable<View>::from_ptr`, which are
  `unsafe` methods for initializing from a raw pointer.
  
### Changed

- `File::open` now takes the filename as `&CStr16` instead of `&str`,
  avoiding an implicit string conversion.

### Removed

- Removed `CStr16::as_string` method. Use
  [`ToString`](https://doc.rust-lang.org/alloc/string/trait.ToString.html)
  instead.
  
### Fixed

- Fixed compilation with Rust 1.60 by no longer enabling the
  `vec_spare_capacity` feature, which has been stabilized.
- Fixed the header size calculated by `FileInfo::new` and
  `FileSystemInfo::new`.
