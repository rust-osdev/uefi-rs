# uefi-services - [Unreleased]

## Changed
- The implicit `qemu-exit` crate feature has been removed. (Note that this is
  different from the `qemu` crate feature, which is unchanged.)

# uefi-services - 0.23.0 (2023-11-12)

## Changed
- `uefi_services::system_table` now returns `SystemTable<Boot>` directly, rather
  than wrapped in a `NonNull` pointer.

# uefi-services - 0.22.0 (2023-10-11)

## Changed
- Updated to latest version of `uefi`.

# uefi-services - 0.21.0 (2023-06-20)

## Changed
- Updated to latest version of `uefi`.

# uefi-services - 0.20.0 (2023-06-04)

## Changed
- Updated to latest version of `uefi`.

# uefi-services - 0.19.0 (2023-06-01)

## Changed
- Internal updates for changes in `uefi`.

# uefi-services - 0.18.0 (2023-05-15)

## Changed
- Internal updates for changes in `uefi`.

# uefi-services - 0.17.0 (2023-03-19)

## Changed
- Drop use of unstable `alloc_error_handler` feature. As of Rust 1.68 we can use
  [`default_alloc_error_handler`](https://github.com/rust-lang/rust/pull/102318)
  instead.

# uefi-services - 0.16.0 (2023-01-16)

## Changed
- Bumped `uefi` dependency to latest version.

# uefi-services - 0.15.0 (2022-11-15)

## Changed
- Changed the panic handler log message to use `println!` instead of
  `error!`. This removes an extraneous file name and line number from
  the log message.
- Added a `logger` feature which reflects the same feature in `uefi`.
  This allows using both crates while disabling `logger` in `uefi`,
  which was previously impossible.

# uefi-services - 0.14.0 (2022-09-09)

## Added
- Added `print!` and `println!` macros.

## Changed
- The `no_panic_handler` feature has been replaced with an additive
  `panic_handler` feature. The new feature is enabled by default.

# uefi-services - 0.13.1 (2022-08-26)

## Changed
- Relaxed the version requirements for the `log` dependency to allow
  earlier patch versions.

# uefi-services - 0.13.0 (2022-05-16)

## Changed
- Bumped `uefi` dependency to latest version.

# uefi-services - 0.12.1 (2022-03-15)

## Changed
- Updated to the 2021 edition.
