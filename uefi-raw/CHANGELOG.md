# uefi-raw - [Unreleased]


# uefi-raw - 0.11.0 (2025-05-04)

## Added
- MSRV increased to 1.77.
- Added `Boolean` type
- Added `protocol::network::pxe` module.
- Added conversions between `MacAddress` and the `[u8; 6]` type that's more commonly used to represent MAC addresses.
- Implemented `From` conversions between the `core::net` and `uefi_raw` IP
  address types.
- Added `DiskInfoProtocol`.
- Added `ExtScsiPassThruProtocol`.
- Added `NvmExpressPassThruProtocol`.
- Added `AtaPassThruProtocol`.
- Added `DevicePathUtilitiesProtocol`.
- Added `UsbIoProtocol`.
- Added `Usb2HostControllerProtocol`.
- Added  `DevicePathProtocol::length()` properly constructing the `u16` value

## Changed
- `DevicePathProtocol` now derives
  `Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash`


# uefi-raw - 0.10.0 (2025-02-07)

As of this release, the project has been relicensed from MPL-2.0 to
Apache-2.0/MIT, to better align with the Rust crate ecosystem. (This does not
alter the license of previous releases.)
Details at <https://github.com/rust-osdev/uefi-rs/issues/1470>.

## Added

- Added `protocol::string::UnicodeCollationProtocol`.
- Added `protocol::tcg` module, containing the TCG v1 and v2 protocols.
- Added `DriverBindingProtocol`.
- Added `FirmwareVolume2Protocol`.
- Added `FirmwareVolumeBlock2Protocol`.
- Added `HiiDatabaseProtocol`.
- Added `ScsiIoProtocol`.
- Added `Default` and other common impls for HTTP types.
- Added `boot::TimerDelay`.

## Changed
- The definition of `BootServices::set_timer` now uses `TimerDelay` rather than
  a plain integer.


# uefi-raw - 0.9.0 (2024-10-23)

## Added

- Added `DeviceType` and `DeviceSubType` enums.
- Added device path node types in the `protocol::device_path` module.


# uefi-raw - 0.8.0 (2024-09-09)

## Added

- Added `PAGE_SIZE` constant.


# uefi-raw - 0.7.0 (2024-08-20)

## Added
- New `MemoryType` constants: `UNACCEPTED`, `MAX`, `RESERVED_FOR_OEM`, and
  `RESERVED_FOR_OS_LOADER`.


# uefi-raw - 0.6.0 (2024-07-02)

## Added
- Added `ResetNotificationProtocol`.

## Changed
- `maximum_capsule_size` of `query_capsule_capabilities` now takes a *mut u64 instead of a *mut usize.
- `ResetType` now derives the `Default` trait.


# uefi-raw - 0.5.2 (2024-04-19)

## Added
- Added `TimestampProtocol`.
- Added `DevicePathToTextProtocol` and `DevicePathFromTextProtocol`.


# uefi-raw - 0.5.1 (2024-03-17)

## Added
- Added `IpAddress`, `Ipv4Address`, `Ipv6Address`, and `MacAddress` types.
- Added `ServiceBindingProtocol`, `Dhcp4Protocol`, `HttpProtocol`,
  `Ip4Config2Protocol`, `TlsConfigurationProtocol`, and related types.
- Added `LoadFileProtocol` and `LoadFile2Protocol`.
- Added `firmware_storage` module.


# uefi-raw - 0.5.0 (2023-11-12)

## Added
- Added `AbsolutePointerProtocol`.
- Added `SimpleFileSystemProtocol` and related types.

## Changed
- `{install,reinstall,uninstall}_protocol_interface` now take `const` interface pointers.
- `{un}install_multiple_protocol_interfaces` are now defined as c-variadic
  function pointers. The ABI is `extern "C"` until such time as
  [`extended_varargs_abi_support`](https://github.com/rust-lang/rust/issues/100189)
  is stabilized.
