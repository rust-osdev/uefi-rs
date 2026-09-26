# uefi-raw - [Unreleased]

## Added

## Changed
- **Breaking**: The MSRV is now 1.91 for `extended_varargs_abi_support`.
- **Breaking**: Changed ABI of c-variadic function pointers from `extern "C"`
  to `extern "efiapi"`.
- **Breaking**: `udp_read()`'s `header_size` parameter is now mutable. The spec
  declares it as in but EDK2 may shrink if the payload is too small to fill in a
  whole header.

## Removed


# uefi-raw - v0.17.0 (2026-09-21)

Most of this release is the outcome of an LLM-assisted audit of the crate
against UEFI 2.11 and the EDK2 headers. Accordingly, the bulk of the entries
below are pointer declarations that now express who writes through a pointer and
who owns the pointee, as described in `api_guidelines.md`. None of them change
the ABI or the memory that the firmware sees, so adapting to them usually means
adding or dropping a `cast_mut()`/`cast_const()`. Everything under `## Changed`
is breaking.

We use LLMs to find problems, not to write code we do not understand. Nothing
unreviewed lands in this crate.

## Added
- HII Internal Forms Representation (IFR) bindings in `protocol::hii::ifr`.
- `EdidDiscoveredProtocol`.
- `MemoryAttribute::HOT_PLUGGABLE` (UEFI 2.11) and
  `HttpStatusCode::STATUS_429_TOO_MANY_REQUESTS`.
- `Clone` and `Copy` derives for `PciRootBridgeIoAccess`.

## Fixed
- `IpAddress::new_v4` and the corresponding `From` impl left 12 of the 16 union
  bytes uninitialized, which is undefined behavior.
- `EdkiiIommuAttribute::INVALID_FOR_ALLOCATE_BUFFER` was always `0`, because
  `from_bits_truncate` drops every bit that is not a named attribute.

## Changed

### Callee-allocated results are `*mut`, as the caller must free them
- The pool-allocating functions of `DevicePathUtilitiesProtocol`,
  `DevicePathToTextProtocol` and `DevicePathFromTextProtocol`.
- `device_path` of `build_device_path` of `ExtScsiPassThruProtocol`,
  `AtaPassThruProtocol` and `NvmExpressPassThruProtocol`.
- `info` of `GraphicsOutputProtocol::query_mode`.
- The result strings of `ConfigKeywordHandlerProtocol::get_data`,
  `HiiConfigAccessProtocol::extract_config` and
  `HiiConfigRoutingProtocol::{extract_config, export_config, block_to_config,
  get_alt_cfg}`.
- `ShellProtocol::{get_device_path_from_file_path,
  get_file_path_from_device_path, get_file_info}`, plus `ShellFileHandle`,
  which is now `*mut c_void` like the other opaque handles.
- `host_addr` of `PciRootBridgeIoProtocol::allocate_buffer`, which is writable
  memory owned by the caller.

### Pointers whose pointee the firmware writes are `*mut`
- `data` of `Usb2HostControllerProtocol::{bulk_transfer, isochronous_transfer,
  async_isochronous_transfer}` is `*const *mut c_void`. The buffers are
  `IN OUT`; the controller writes received data into them.
- `target` of `ExtScsiPassThruProtocol::get_target_lun` and
  `ScsiIoProtocol::get_device_location` is `*const *mut u8`. The firmware fills
  the caller's array and only reads the pointer to it.
- `HttpRequestOrResponse::response` and `HttpAccessPoint::{ipv4_node,
  ipv6_node}`.
- The inner pointer of the outputs `registration` of
  `BootServices::register_protocol_notify`, `entry_buffer` of
  `BootServices::open_protocol_information` and `address` of
  `RuntimeServices::convert_pointer`.
- `ShellProtocol::{free_file_list, remove_dup_in_file_list}` take
  `*mut *mut ShellFileInfo`, as the shell frees the list and clears the pointer.
- The `new_packet` output of the DHCP4 callback is `*mut *mut Dhcp4Packet`, as
  the driver takes ownership of the returned packet.
- `this` of `AbsolutePointerProtocol::get_state`. The call consumes the state
  it reports: it returns `EFI_NOT_READY` unless the state changed since the
  last call, and clears the change flag. `SimplePointerProtocol::get_state`
  already declares `this` this way.

### Inputs that the firmware only reads are `*const`
- USB: `request` of `UsbIoProtocol::control_transfer`, `data` of
  `AsyncUsbTransferCallback`, and the `context` parameters of the asynchronous
  transfer functions and the callback.
- Boot services: `event_group` and `notify_ctx` of `create_event_ex`, `events`
  of `wait_for_event`, and `registration` of `locate_protocol`.
- `virtual_map` of `RuntimeServices::set_virtual_address_map`.
- `EdkiiIommuProtocol`: `host_address` of `free_buffer`, and `mapping` of
  `set_attribute` and `unmap`.
- `buffer` of `FirmwareVolumeBlock2Protocol::write`.
- DHCP4: `seed_packet` and `delete_list` of `build`, and `packet` of `parse`.
- `notify_handle` of `SimpleTextInputExProtocol::unregister_key_notify`.
- The `write_file` buffer of `ShellProtocol`.
- `this` of `SimpleTextOutputProtocol::query_mode`, which only reads the
  protocol.

### Firmware-owned memory that consumers only read is `*const`
- The mode and info pointers `SimpleNetworkProtocol::mode`,
  `AbsolutePointerProtocol::mode`, `SimpleTextOutputProtocol::mode`,
  `GraphicsOutputProtocol::mode` and `GraphicsOutputProtocolMode::info`,
  matching the other mode pointers in the crate.
- The name returned by `ShellProtocol::get_guid_name` and
  `ShellFileInfo::{full_name, file_name}`, which point into the shell.

### Types, layout and module paths
- `PxeBaseCodeSrvlist::server_type` and the `server_type` parameter of
  `PxeBaseCodeSrvlist::new` use the newtype-enum `PxeBaseCodeBootType` instead
  of `u16`.
- The callback of `UsbIoProtocol::async_interrupt_transfer` and
  `Usb2HostControllerProtocol::async_interrupt_transfer` is
  `Option<AsyncUsbTransferCallback>`. The specification marks it `OPTIONAL`,
  and NULL cancels the transfer.
- `HiiPackageListHeader` and `HiiPackageHeader` are packed, as the
  specification mandates.
- `HiiRef`, `HiiTime` and `HiiDate` moved from `protocol::hii::config` to
  `protocol::hii`, and `IfrTypeValue` moved to `protocol::hii::ifr`.

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
