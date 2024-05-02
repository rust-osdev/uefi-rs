# uefi-macros - [Unreleased]

## Changed
- The `entry` macro now sets the global system table pointer with `uefi::set_system_table`.

## Removed
- Removed the `cstr8` and `cstr16` macros. Use the declarative macros of the
  same names exported by the `uefi` crate as a replacement.

# uefi-macros - 0.13.0 (2023-11-12)

## Changed
- The dev-dependency on `uefi` is now path-only.

# uefi-macros - 0.12.0 (2023-05-15)

## Changed
- The `unsafe_protocol` macro no longer makes protocols `!Send` and
  `!Sync`. Protocols can only be used while boot services are active, and that's
  already a single-threaded environment, so these negative traits do not have
  any effect.
- The `unsafe_protocol` macro now accepts the path of a `Guid` constant in
  addition to a string literal.
- The `cstr8` and the `cstr16` macros now both accept `(nothing)` and `""`
  (empty inputs) to create valid empty strings. They include the null-byte.
- The `entry` macro now works correctly with docstrings.

# uefi-macros - 0.11.0 (2023-03-19)

## Changed
- Errors produced by the `entry` macro have been improved.

# uefi-macros - 0.10.0 (2023-01-16)

## Added
- Added the `unsafe_protocol` macro to provide a slightly nicer way to
  implement protocols.

## Removed
- The `unsafe_guid` attribute macro and `Protocol` derive macro have
  been removed. For implementing protocols, use the `unsafe_protocol`
  macro instead. For any other implementations of the `Identify` trait,
  implement it directly.

# uefi-macros - 0.9.0 (2022-11-15)

## Added
- Added a `guid!` macro. This is similar to `Guid::from_values`, but
  takes a more convenient string argument like the `unsafe_guid!`
  attribute macro.

# uefi-macros - 0.8.0 (2022-09-09)

## Changed
- The `#[entry]` macro now calls `BootServices::set_image_handle` to set
  the global image handle. Due to this change, the two arguments to main
  must both be named (e.g. `image: Handle` and `_image: Handle` are both
  OK, but not `_: Handle`).

# uefi-macros - 0.7.1 (2022-08-26)

## Changed
- Relaxed the version requirements for the `proc-macro2`, `quote`, and
  `sync` dependencies to allow earlier patch versions.

# uefi-macros - 0.7.0 (2022-05-16)

## Added
- Added `cstr8` and `cstr16` macros for creating `CStr8`/`CStr16` string literals
  at compile time.

# uefi-macros - 0.6.1 (2022-03-15)

## Changed
- Updated to the 2021 edition.
