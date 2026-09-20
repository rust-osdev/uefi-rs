# uefi-raw - [Unreleased]

## Added
- Added `MemoryAttribute::HOT_PLUGGABLE` (UEFI 2.11).
- Added `HttpStatusCode::STATUS_429_TOO_MANY_REQUESTS` (UEFI 2.11).
- Added `Clone` and `Copy` derives to `PciRootBridgeIoAccess`.
- Added HII Internal Forms Representation (IFR) types
- Added `EdidDiscoveredProtocol`.

## Changed
- **Breaking**: Use `PxeBaseCodeBootType` (newtype-enum) instead of `u16` for
`PxeBaseCodeSrvlist::server_type` and for the `server_type` parameter of
`PxeBaseCodeSrvlist::new`.
- **Breaking**: Changed `this` parameter of `SimplePointerProtocol::get_state`
from `*mut Self` to `*const Self`.
- Fixed undefined behavior in `IpAddress::new_v4` and the corresponding
  `From` impl, which left 12 of the 16 union bytes uninitialized.
- **Breaking**: Changed `this` parameter of `SimpleTextOutputProtocol::query_mode`
from `*mut Self` to `*const Self`.
- **Breaking**: Changed the `data` parameter of
  `Usb2HostControllerProtocol::{bulk_transfer, isochronous_transfer,
  async_isochronous_transfer}` from `*const *const c_void` to `*const *mut
  c_void`. The buffers are `IN OUT`; the controller writes received data into
  them.
- **Breaking**: Changed the callback parameter of
  `UsbIoProtocol::async_interrupt_transfer` and
  `Usb2HostControllerProtocol::async_interrupt_transfer` to
  `Option<AsyncUsbTransferCallback>`. The specification marks it `OPTIONAL`;
  NULL cancels the transfer.
- **Breaking**: Changed the `target` parameter of
  `ExtScsiPassThruProtocol::get_target_lun` (from `*mut *const u8`) and
  `ScsiIoProtocol::get_device_location` (from `*mut *mut u8`) to `*const *mut
  u8`. The firmware writes the target ID into the caller-provided array and
  only reads the pointer to it.
- **Breaking**: Changed `HttpRequestOrResponse::response` and
  `HttpAccessPoint::{ipv4_node, ipv6_node}` from `*const` to `*mut`. The
  driver writes the status code and the access point through these pointers.
- **Breaking**: Fixed the pointer mutability of several `ShellProtocol` items
  to match the EDK2 header: `free_file_list` and `remove_dup_in_file_list`
  take `*mut *mut ShellFileInfo` (the shell frees the list and clears the
  pointer), `write_file` takes a `*const` buffer, `get_guid_name` returns a
  `*const` name that points into the shell, and `ShellFileInfo::{full_name,
  file_name}` are `*const`.
- **Breaking**: Changed the read-only inputs of the USB protocols from `*mut`
  to `*const`: the `request` parameter of `UsbIoProtocol::control_transfer`,
  the `data` parameter of `AsyncUsbTransferCallback`, and the `context`
  parameters of the asynchronous transfer functions and the callback.
- **Breaking**: Changed the read-only inputs `BootServices::create_event_ex`
  (`event_group`), `BootServices::wait_for_event` (`events`),
  `BootServices::exit` (`exit_data`) and `BootServices::locate_protocol`
  (`registration`) from `*mut` to `*const`.
- **Breaking**: Changed the `virtual_map` parameter of
  `RuntimeServices::set_virtual_address_map` from `*mut` to `*const
  MemoryDescriptor`.
- **Breaking**: Changed the read-only inputs of `EdkiiIommuProtocol` from
  `*mut` to `*const c_void`: the `host_address` parameter of `map` and
  `free_buffer`, and the `mapping` parameter of `set_attribute` and `unmap`.
- **Breaking**: Changed the `buffer` parameter of
  `FirmwareVolumeBlock2Protocol::write` from `*mut u8` to `*const u8`.
- **Breaking**: Fixed the pointer mutability of `Dhcp4Protocol`: the
  `seed_packet` and `delete_list` parameters of `build` and the `packet`
  parameter of `parse` are `*const`, and the `new_packet` output of the DHCP4
  callback is `*mut *mut Dhcp4Packet` because the driver takes ownership of
  the returned packet.
- **Breaking**: Changed the `notify_handle` parameter of
  `SimpleTextInputExProtocol::unregister_key_notify` from `*mut` to `*const
  c_void`.
- **Breaking**: Changed the event notification context from `*mut c_void` to
  `*const c_void` in `EventNotifyFn` and in the `notify_ctx` parameter of
  `BootServices::{create_event, create_event_ex}`. The firmware passes the
  pointer through unchanged.
- **Breaking**: Changed the firmware-owned, read-only mode and info pointers
  `SimpleNetworkProtocol::mode`, `AbsolutePointerProtocol::mode`,
  `SimpleTextOutputProtocol::mode`, `GraphicsOutputProtocol::mode` and
  `GraphicsOutputProtocolMode::info` from `*mut` to `*const`, matching the
  other mode pointers in the crate.
- **Breaking**: Changed the `host_addr` parameter of
  `PciRootBridgeIoProtocol::allocate_buffer` from `*mut *const c_void` to
  `*mut *mut c_void`. The allocated buffer is writable memory owned by the
  caller.
- **Breaking**: Changed the return type of the pool-allocating functions of
  `DevicePathUtilitiesProtocol`, `DevicePathToTextProtocol` and
  `DevicePathFromTextProtocol` from `*const` to `*mut`. The caller owns and
  must free the result.
- **Breaking**: Changed the `device_path` output of
  `ExtScsiPassThruProtocol::build_device_path`,
  `AtaPassThruProtocol::build_device_path` and
  `NvmExpressPassThruProtocol::build_device_path` from `*mut *const` to `*mut
  *mut DevicePathProtocol`. The caller owns and must free the result.
- **Breaking**: Changed the `info` output of
  `GraphicsOutputProtocol::query_mode` from `*mut *const` to `*mut *mut
  GraphicsOutputModeInformation`. The caller owns and must free the buffer.
- **Breaking**: Changed the callee-allocated result strings of the HII
  configuration protocols (`ConfigKeywordHandlerProtocol::get_data`,
  `HiiConfigAccessProtocol::extract_config`,
  `HiiConfigRoutingProtocol::{extract_config, export_config, block_to_config,
  get_alt_cfg}`) from `*mut *const` to `*mut *mut Char16`. The caller owns and
  must free them.
- **Breaking**: Changed the inner pointer of the outputs
  `BootServices::register_protocol_notify` (`registration`),
  `BootServices::open_protocol_information` (`entry_buffer`) and
  `RuntimeServices::convert_pointer` (`address`) from `*const` to `*mut`,
  matching the non-const C declarations.
- **Breaking**: Changed the callee-allocated results of
  `ShellProtocol::{get_device_path_from_file_path,
  get_file_path_from_device_path, get_file_info}` from `*const` to `*mut` (the
  caller must free them), and `ShellFileHandle` from `*const c_void` to `*mut
  c_void` like the other opaque handles.
- **Breaking**: `HiiRef`, `HiiTime`, and `HiiDate` are moved from
  `protocol::hii::config` to `protocol::hii`.
- **Breaking**: `HiiPackageListHeader` and `HiiPackageHeader` are now packed
  to match the layout mandated by the UEFI specification.
- **Breaking**: `IfrTypeValue` is moved from `protocol::hii::config` to
  `protocol::hii::ifr`.

# uefi-raw - v0.16.0 (2026-08-25)

## Added
- Added the revision constants `BlockIoProtocol::{REVISION, REVISION_2,
  REVISION_3}`.
- Added `Boolean::is_true()` and  `Boolean::is_false()` for a quick conversion
  of an EFI boolean to a Rust boolean.

## Changed
- **Breaking**: `MemoryDescriptor` now has a new member to ensure correct
  layout on all non-UEFI 32-bit targets.
- **Breaking**: Corrected the `volatile` parameter of
  `ShellProtocol::get_alias` from `Boolean` to `*mut Boolean`.
- **Breaking**: The USB descriptor types in `protocol::usb` are now packed to
  match their layout in the USB specification. `ConfigDescriptor` and
  `EndpointDescriptor` previously had a too-large `size_of`.
- **Breaking**: `HiiKeyboardLayout` and `KeyDescriptor` are now packed to
  match the layout mandated by the UEFI specification. Previously, all
  `HiiKeyboardLayout` fields after `layout_length` were at wrong offsets.
- **Breaking**: `IfrTypeValue`, `HiiRef`, `HiiTime`, and `HiiDate` are now
  packed to match the size and alignment mandated by the UEFI specification.

## Removed


# uefi-raw - v0.15.1 (2026-07-11)

## Added
- Added additional derives to IOMMU types.

# uefi-raw - v0.15.0 (2026-06-21)

## Added
- Added `SimpleTextInputExProtocol`.
- Added `PciRootBridgeIoProtocolAttributes`

## Changed
- Corrected the type of the `driver_image` parameter in
  `BootServices::connect_controller` from `Handle` to `*const Handle`.
- Corrected signature of `BootServices::exit` from `!` to `Status`.
- We made changes to the `Boolean` type, especially switching to manual
  implementations of `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash` to match
  the type's logical properties. If you rely on the specific bit pattern,
  please use `.0` to access the underlying value.
- Added `PxeBaseCodeDhcpV4Flags`.
- Added `PxeBaseCodeDhcpV4Packet::{DHCP_MAGIK, bootp_ident(), bootp_seconds(),
  bootp_flags(), dhcp_magik()}`.
- Added `PxeBaseCodeDhcpV6Packet::transaction_id()`.
- Added `AsRef` implementations to `PxeBaseCodePacket`.
- Added `PxeBaseCodeIpFilter::{new(), ip_list()}`.
- Added `PxeBaseCodeSrvlist::{new(), ip_addr()}`.
- `PxeBaseCodeIcmpError` and `PxeBaseCodeTftpError` now implement `Display` and
  `core::error::Error`.


# uefi-raw - v0.14.0 (2026-03-22)

## Added
- Added `Tcpv4Protocol`.
- Added `StorageSecurityCommandProtocol`.
- Added `FirmwareManagementProtocol`.
- Added `HiiFontProtocol`, `HiiFontExProtocol`.
- Added `HiiImageProtocol`, `HiiImageExProtocol`.
- Added `HiiStringProtocol`.
- Added `HiiPopupProtocol`.
- Added `FormBrowser2Protocol`.
- Added new type `SerialIoProtocolRevision`
- Added new type `SerialIoProtocol_1_1` as companion for `SerialIoProtocol`
  that includes the  `device_type_guid` parameter

## Changed
- Switched `*const Self` to `*mut Self` in `SerialIoProtocol::set_attributes()`
- Switched field `revision` in `SerialIoProtocol` from `u32` to new type
  `SerialIoProtocolRevision`


# uefi-raw - v0.13.0 (2025-11-05)

## Changed
- **Breaking:** Various uses of `bool` have been replaced with `Boolean`.
- Fixing build on <https://docs.rs/uefi>


# uefi-raw - v0.12 (2025-10-21)

## Added
- Added `AllocateType`.
- Added `PciRootBridgeIoProtocol`.
- Added `ConfigKeywordHandlerProtocol`.
- Added `HiiConfigAccessProtocol`.
- Added `::octets()` for `Ipv4Address`, `Ipv6Address`, and
  `MacAddress` to streamline the API with `core::net`.
- Added `::into_core_addr()` for `IpAddress`
- Added `::into_ethernet_addr()` for `MacAddress`
- Added `::ZERO` constant for `IpAddress`
- `Ipv4Address` and `Ipv6Address` now implement `Display`. They
  use the same formatting as `core::net::{Ipv4Addr, Ipv6Addr}`
- Added comprehensive integration with `core::net::{IpAddr, Ipv4Addr, Ipv6Addr}`
  via `From` impls to better integrate uefi-raw types `IpAddress`,
  `Ipv4Address`, and `Ipv6Address` with the Rust ecosystem.
- Added convenient `From` impls:
  - `[u8; 6]`  <--> `MacAddress`
  - `[u8; 32]`  --> `MacAddress`
  - `[u8; 4]`   --> `Ipv4Address`, `IpAddress`
  - `[u8; 16]`  --> `Ipv6Address`, `IpAddress`
- Added `HiiConfigRoutingProtocol`.

## Changed
- **Breaking:** The MSRV is now 1.85.1 and the crate uses the Rust 2024 edition.
- The documentation for UEFI protocols has been streamlined and improved.


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
