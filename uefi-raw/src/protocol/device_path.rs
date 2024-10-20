/*
 * Device Path definitions were derived from TianoCore under the following license:
 *
 * Copyright (c) 2019, TianoCore and contributors.  All rights reserved.
 *
 * SPDX-License-Identifier: BSD-2-Clause-Patent
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * Subject to the terms and conditions of this license, each copyright holder
 * and contributor hereby grants to those receiving rights under this license
 * a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
 * (except for failure to satisfy the conditions of this license) patent
 * license to make, have made, use, offer to sell, sell, import, and otherwise
 * transfer this software, where such license applies only to those patent
 * claims, already acquired or hereafter acquired, licensable by such copyright
 * holder or contributor that are necessarily infringed by:
 *
 * (a) their Contribution(s) (the licensed copyrights of copyright holders and
 *     non-copyrightable additions of contributors, in source or binary form)
 *     alone; or
 *
 * (b) combination of their Contribution(s) with the work of authorship to
 *     which such Contribution(s) was added by such copyright holder or
 *     contributor, if, at the time the Contribution is added, such addition
 *     causes such combination to be necessarily infringed. The patent license
 *     shall not apply to any other combinations which include the
 *     Contribution.
 *
 * Except as expressly stated above, no rights or licenses from any copyright
 * holder or contributor is granted under this license, whether expressly, by
 * implication, estoppel or otherwise.
 *
 * DISCLAIMER
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use crate::{guid, BluetoothAddress, BluetoothLeAddress, Char16, Guid, Ipv4Address, Ipv6Address, MacAddress, PhysicalAddress};

/// Device path protocol.
///
/// A device path contains one or more device path instances made of up
/// variable-length nodes.
///
/// Note that the fields in this struct define the header at the start of each
/// node; a device path is typically larger than these four bytes.
#[derive(Copy,Clone,Debug)]
#[repr(C,packed)]
pub struct DevicePathProtocol {
    pub major_type: u8,
    pub sub_type: u8,
    pub length: [u8; 2],
    // followed by payload (dynamically sized)
}

impl DevicePathProtocol {
    pub const GUID: Guid = guid!("09576e91-6d3f-11d2-8e39-00a0c969723b");
}

/// Hardware Device Path Type.
pub const HARDWARE_DEVICE_PATH: u8 = 0x01;

/// PCI Device Path SubType.
pub const HW_PCI_DP: u8 = 0x01;

/// PCI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct PciDevicePath {
    pub header: DevicePathProtocol,
    /// PCI Function Number.
    pub function: u8,
    /// PCI Device Number.
    pub device: u8,
}

/// PCCARD Device Path SubType.
pub const HW_PCCARD_DP: u8 = 0x02;

/// PCCARD Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct PccardDevicePath {
    pub header: DevicePathProtocol,
    /// Function Number (0 = First Function).
    pub function_number: u8,
}

/// Memory Mapped Device Path SubType.
pub const HW_MEMMAP_DP: u8 = 0x03;

/// Memory Mapped Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MemmapDevicePath {
    pub header: DevicePathProtocol,
    /// EFI_MEMORY_TYPE
    pub memory_type: u32,
    /// Starting Memory Address.
    pub starting_address: PhysicalAddress,
    /// Ending Memory Address.
    pub ending_address: PhysicalAddress,
}

/// Hardware Vendor Device Path SubType.
pub const HW_VENDOR_DP: u8 = 0x04;

/// The Vendor Device Path allows the creation of vendor-defined Device Paths. A vendor must
/// allocate a Vendor GUID for a Device Path. The Vendor GUID can then be used to define the
/// contents on the n bytes that follow in the Vendor Device Path node.
#[derive(Debug)]
#[repr(C,packed)]
pub struct VendorDevicePath {
    pub header: DevicePathProtocol,
    /// Vendor-assigned GUID that defines the data that follows.
    pub guid: Guid,
    // Vendor-defined variable size data.
}

/// Controller Device Path SubType.
pub const HW_CONTROLLER_DP: u8 = 0x05;

/// Controller Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct ControllerDevicePath {
    pub header: DevicePathProtocol,
    /// Controller number.
    pub controller_number: u32,
}

/// BMC Device Path SubType.
pub const HW_BMC_DP: u8 = 0x06;

/// BMC Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct BmcDevicePath {
    pub header: DevicePathProtocol,
    /// Interface Type.
    pub interface_type: u8,
    /// Base Address.
    pub base_address: [u8; 8],
}

/// ACPI Device Path Type.
pub const ACPI_DEVICE_PATH: u8 = 0x02;

/// ACPI Device Path SubType.
pub const ACPI_DP: u8 = 0x01;

/// ACPI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct AcpiHidDevicePath {
    pub header: DevicePathProtocol,
    /// Device's PnP hardware ID stored in a numeric 32-bit
    /// compressed EISA-type ID. This value must match the
    /// corresponding _HID in the ACPI name space.
    pub hid: u32,
    /// Unique ID that is required by ACPI if two devices have the
    /// same _HID. This value must also match the corresponding
    /// _UID/_HID pair in the ACPI name space. Only the 32-bit
    /// numeric value type of _UID is supported. Thus, strings must
    /// not be used for the _UID in the ACPI name space.
    pub uid: u32,
}

/// Expanded ACPI Device Path SubType.
pub const ACPI_EXTENDED_DP: u8 = 0x02;

/// Expanded ACPI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct AcpiExtendedHidDevicePath {
    pub header: DevicePathProtocol,
    /// Device's PnP hardware ID stored in a numeric 32-bit
    /// compressed EISA-type ID. This value must match the
    /// corresponding _HID in the ACPI name space.
    pub hid: u32,
    /// Unique ID that is required by ACPI if two devices have the
    /// same _HID. This value must also match the corresponding
    /// _UID/_HID pair in the ACPI name space.
    pub uid: u32,
    /// Device's compatible PnP hardware ID stored in a numeric
    /// 32-bit compressed EISA-type ID. This value must match at
    /// least one of the compatible device IDs returned by the
    /// corresponding _CID in the ACPI name space.
    pub cid: u32,
    // Optional variable length _HIDSTR.
    // Optional variable length _UIDSTR.
    // Optional variable length _CIDSTR.
}

/// ACPI _ADR Device Path SubType.
pub const ACPI_ADR_DP: u8 = 0x03;

/// ACPI _ADR Device Path.
/// The _ADR device path is used to contain video output device attributes to support the Graphics
/// Output Protocol. The device path can contain multiple _ADR entries if multiple video output
/// devices are displaying the same output.
#[derive(Debug)]
#[repr(C,packed)]
pub struct AcpiAdrDevicePath {
    pub header: DevicePathProtocol,
    /// _ADR value. For video output devices the value of this
    /// field comes from Table B-2 of the ACPI 3.0 specification. At
    /// least one _ADR value is required.
    pub adr: u32,
    // This device path may optionally contain more than one _ADR entry.
}

/// ACPI NVDIMM Device Path SubType.
pub const ACPI_NVDIMM_DP: u8 = 0x04;

/// ACPI NVDIMM Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct AcpiNvdimmDevicePath {
    pub header: DevicePathProtocol,
    /// NFIT Device Handle, the _ADR of the NVDIMM device.
    /// The value of this field comes from Section 9.20.3 of the ACPI 6.2A specification.
    pub nfit_device_handle: u32,
}

/// Messaging Device Path Type.
pub const MESSAGING_DEVICE_PATH: u8 = 0x03;

/// ATAPI Device Path SubType.
pub const MSG_ATAPI_DP: u8 = 0x01;

/// ATAPI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct AtapiDevicePath {
    pub header: DevicePathProtocol,
    /// Set to zero for primary, or one for secondary.
    pub primary_secondary: u8,
    /// Set to zero for master, or one for slave mode.
    pub slave_master: u8,
    /// Logical Unit Number.
    pub lun: u16,
}

/// SCSI Device Path SubType.
pub const MSG_SCSI_DP: u8 = 0x02;

/// SCSI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct ScsiDevicePath {
    pub header: DevicePathProtocol,
    /// Target ID on the SCSI bus (PUN).
    pub pun: u16,
    /// Logical Unit Number (LUN).
    pub lun: u16,
}

/// Fibre Channel Device Path SubType.
pub const MSG_FIBRECHANNEL_DP: u8 = 0x03;

/// Fibre Channel Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct FibreChannelDevicePath {
    pub header: DevicePathProtocol,
    /// Reserved for the future.
    pub reserved: u32,
    /// Fibre Channel World Wide Number.
    pub wwn: u64,
    /// Fibre Channel Logical Unit Number.
    pub lun: u64,
}

/// Fibre Channel Ex Device Path SubType.
pub const MSG_FIBRECHANNELEX_DP: u8 = 0x15;

/// Fibre Channel Ex Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct FibreChannelExDevicePath {
    pub header: DevicePathProtocol,
    /// Reserved for the future.
    pub reserved: u32,
    /// 8 byte array containing Fibre Channel End Device Port Name.
    pub wwn: [u8; 8],
    /// 8 byte array containing Fibre Channel Logical Unit Number.
    pub lun: [u8; 8],
}

/// 1394 Device Path SubType.
pub const MSG_1394_DP: u8 = 0x04;


/// 1394 Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct F1394DevicePath {
    pub header: DevicePathProtocol,
    /// Reserved for the future.
    pub reserved: u32,
    /// 1394 Global Unique ID (GUID).
    pub guid: u64,
}

/// USB Device Path SubType.
pub const MSG_USB_DP: u8 = 0x05;

/// USB Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UsbDevicePath {
    pub header: DevicePathProtocol,
    /// USB Parent Port Number.
    pub parent_port_number: u8,
    /// USB Interface Number.
    pub interface_number: u8,
}

/// USB Class Device Path SubType.
pub const MSG_USB_CLASS_DP: u8 = 0x0f;

/// USB Class Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UsbClassDevicePath {
    pub header: DevicePathProtocol,
    /// Vendor ID assigned by USB-IF. A value of 0xFFFF will
    /// match any Vendor ID.
    pub vendor_id: u16,
    /// Product ID assigned by USB-IF. A value of 0xFFFF will
    /// match any Product ID.
    pub product_id: u16,
    /// The class code assigned by the USB-IF. A value of 0xFF
    /// will match any class code.
    pub device_class: u8,
    /// The subclass code assigned by the USB-IF. A value of
    /// 0xFF will match any subclass code.
    pub device_sub_class: u8,
    /// The protocol code assigned by the USB-IF. A value of
    /// 0xFF will match any protocol code.
    pub device_protocol: u8,
}

/// USB WWID Device Path SubType.
pub const MSG_USB_WWID_DP: u8 = 0x10;

/// USB WWID Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UsbWwidDevicePath {
    pub header: DevicePathProtocol,
    /// USB interface number.
    pub interface_number: u16,
    /// USB vendor id of the device.
    pub vendor_id: u16,
    /// USB product id of the device.
    pub product_id: u16,
    // Last 64-or-fewer UTF-16 characters of the USB
    // serial number. The length of the string is
    // determined by the Length field less the offset of the
    // Serial Number field (10)
    // pub serial_number: [Char16; _],
}

/// Device Logical Unit Device Path SubType.
pub const MSG_DEVICE_LOGICAL_UNIT_DP: u8 = 0x11;

/// Device Logical Unit Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct DeviceLogicalUnitDevicePath {
    pub header: DevicePathProtocol,
    /// Logical Unit Number for the interface.
    pub lun: u8,
}

/// SATA Device Path SubType.
pub const MSG_SATA_DP: u8 = 0x12;

/// SATA Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct SataDevicePath {
    pub header: DevicePathProtocol,
    /// The HBA port number that facilitates the connection to the
    /// device or a port multiplier. The value 0xFFFF is reserved.
    pub hba_port_number: u16,
    /// The Port multiplier port number that facilitates the connection
    /// to the device. Must be set to 0xFFFF if the device is directly
    /// connected to the HBA.
    pub port_multiplier_port_number: u16,
    /// Logical Unit Number.
    pub lun: u16,
}

/// Flag for if the device is directly connected to the HBA.
pub const SATA_HBA_DIRECT_CONNECT_FLAG: u16 = 0x8000;

/// I2O Device Path SubType.
pub const MSG_I2O_DP: u8 = 0x06;

/// I2O Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct I2oDevicePath {
    pub header: DevicePathProtocol,
    /// Target ID (TID) for a device.
    pub tid: u32,
}

/// MAC Address Device Path SubType.
pub const MSG_MAC_ADDR_DP: u8 = 0x0b;

/// MAC Address Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MacAddr {
    pub header: DevicePathProtocol,
    /// The MAC address for a network interface padded with 0s.
    pub mac_address: MacAddress,
    /// Network interface type(i.e. 802.3, FDDI).
    pub if_type: u8,
}

/// IPv4 Device Path SubType.
pub const MSG_IPV4_DP: u8 = 0x0c;

/// IPv4 Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct Ipv4DevicePath {
    pub header: DevicePathProtocol,
    /// The local IPv4 address.
    pub local_ip_address: Ipv4Address,
    /// The remote IPv4 address.
    pub remote_ip_address: Ipv4Address,
    /// The local port number.
    pub local_port: u16,
    /// The remote port number.
    pub remote_port: u16,
    /// The network protocol (i.e. UDP, TCP).
    pub protocol: u16,
    /// false - The Source IP Address was assigned though DHCP.
    /// true  - The Source IP Address is statically bound.
    pub static_ip_address: bool,
    /// The gateway IP address
    pub gateway_ip_address: Ipv4Address,
    /// The subnet mask
    pub subnet_mask: Ipv4Address,
}

/// IPv6 Device Path SubType.
pub const MSG_IPV6_DP: u8 = 0x0d;

/// IPv6 Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct Ipv6DevicePath {
    pub header: DevicePathProtocol,
    /// The local IPv6 address.
    pub local_ip_address: Ipv6Address,
    /// The remote IPv6 address.
    pub remote_ip_address: Ipv6Address,
    /// The local port number.
    pub local_port: u16,
    /// The remote port number.
    pub remote_port: u16,
    /// The network protocol(i.e. UDP, TCP).
    pub protocol: u16,
    /// 0x00 - The Local IP Address was manually configured.
    /// 0x01 - The Local IP Address is assigned through IPv6
    /// stateless auto-configuration.
    /// 0x02 - The Local IP Address is assigned through IPv6
    /// stateful configuration.
    pub ip_address_origin: u8,
    /// The prefix length
    pub prefix_length: u8,
    /// The gateway IP address
    pub gateway_ip_address: Ipv6Address,
}

/// InfiniBand Device Path SubType.
pub const MSG_INFINIBAND_DP: u8 = 0x09;

/// InfiniBand Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct InfinibandDevicePath {
    pub header: DevicePathProtocol,
    /// Flags to help identify/manage InfiniBand device path elements:
    /// Bit 0 - IOC/Service (0b = IOC, 1b = Service).
    /// Bit 1 - Extend Boot Environment.
    /// Bit 2 - Console Protocol.
    /// Bit 3 - Storage Protocol.
    /// Bit 4 - Network Protocol.
    /// All other bits are reserved.
    pub resource_flags: u32,
    /// 128-bit Global Identifier for remote fabric port.
    pub port_gid: [u8; 16],
    /// 64-bit unique identifier to remote IOC or server process.
    /// Interpretation of field specified by Resource Flags (bit 0).
    pub service_id: u64,
    /// 64-bit persistent ID of remote IOC port.
    pub target_port_id: u64,
    /// 64-bit persistent ID of remote device.
    pub device_id: u64,
}

pub const INFINIBAND_RESOURCE_FLAG_IOC_SERVICE: u8 = 0x01;
pub const INFINIBAND_RESOURCE_FLAG_EXTENDED_BOOT_ENVIRONMENT: u8 = 0x02;
pub const INFINIBAND_RESOURCE_FLAG_CONSOLE_PROTOCOL: u8 = 0x04;
pub const INFINIBAND_RESOURCE_FLAG_STORAGE_PROTOCOL: u8 = 0x08;
pub const INFINIBAND_RESOURCE_FLAG_NETWORK_PROTOCOL: u8 = 0x10;

/// UART Device Path SubType.
pub const MSG_UART_DP: u8 = 0x0e;

/// UART Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UartDevicePath {
    pub header: DevicePathProtocol,
    /// Reserved.
    pub reserved: u32,
    /// The baud rate setting for the UART style device. A value of 0
    /// means that the device's default baud rate will be used.
    pub baud_rate: u64,
    /// The number of data bits for the UART style device. A value
    /// of 0 means that the device's default number of data bits will be used.
    pub data_bits: u8,
    /// The parity setting for the UART style device.
    /// Parity 0x00 - Default Parity.
    /// Parity 0x01 - No Parity.
    /// Parity 0x02 - Even Parity.
    /// Parity 0x03 - Odd Parity.
    /// Parity 0x04 - Mark Parity.
    /// Parity 0x05 - Space Parity.
    pub parity: u8,
    /// The number of stop bits for the UART style device.
    /// Stop Bits 0x00 - Default Stop Bits.
    /// Stop Bits 0x01 - 1 Stop Bit.
    /// Stop Bits 0x02 - 1.5 Stop Bits.
    /// Stop Bits 0x03 - 2 Stop Bits.
    pub stop_bits: u8,
}

/// NVDIMM Namespace Device Path SubType.
pub const NVDIMM_NAMESPACE_DP: u8 = 0x20;

/// NVDIMM Namespace Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct NvdimmNamespaceDevicePath {
    pub header: DevicePathProtocol,
    /// Namespace unique label identifier UUID.
    pub uuid: Guid,
}

/// Messaging Vendor Device Path SubType.
/// Use VENDOR_DEVICE_PATH struct
pub const MSG_VENDOR_DP: u8 = 0x0a;

pub const DEVICE_PATH_MESSAGING_PC_ANSI: Guid = guid!("e0c14753-f9be-11d2-9a0c-0090273fc14d");
pub const DEVICE_PATH_MESSAGING_VT_100: Guid = guid!("dfa66065-b419-11d3-9a2d-0090273fc14d");
pub const DEVICE_PATH_MESSAGING_VT_100_PLUS: Guid = guid!("7baec70b-57e0-4c76-8e87-2f9e28088343");
pub const DEVICE_PATH_MESSAGING_VT_UTF8: Guid = guid!("ad15a0d6-8bec-4acf-a073-d01de77e2d88");

pub const DEVICE_PATH_MESSAGING_UART_FLOW_CONTROL: Guid = guid!("37499a9d-542f-4c89-a026-35da142094e4");

pub const UART_FLOW_CONTROL_HARDWARE: u8 = 0x00000001;
pub const UART_FLOW_CONTROL_XON_XOFF: u8 = 0x00000010;

/// A new device path node is defined to declare flow control characteristics.
/// UART Flow Control Messaging Device Path
#[derive(Debug)]
#[repr(C,packed)]
pub struct UartFlowControlDevicePath {
    pub header: DevicePathProtocol,
    /// DEVICE_PATH_MESSAGING_UART_FLOW_CONTROL GUID.
    pub guid: Guid,
    /// Bitmap of supported flow control types.
    /// Bit 0 set indicates hardware flow control.
    /// Bit 1 set indicates Xon/Xoff flow control.
    /// All other bits are reserved and are clear.
    pub flow_control_map: u32,
}

pub const DEVICE_PATH_MESSAGING_SAS: Guid = guid!("d487ddb4-008b-11d9-afdc-001083ffca4d");

/// Serial Attached SCSI (SAS) Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct SasDevicePath {
    pub header: DevicePathProtocol,
    /// DEVICE_PATH_MESSAGING_SAS GUID.
    pub guid: Guid,
    /// Reserved for future use.
    pub reserved: u32,
    /// SAS Address for Serial Attached SCSI Target.
    pub sas_address: u64,
    /// SAS Logical Unit Number.
    pub lun: u64,
    /// More Information about the device and its interconnect.
    pub device_topology: u16,
    /// Relative Target Port (RTP).
    pub relative_target_port: u16,
}

/// Serial Attached SCSI (SAS) Ex Device Path SubType.
pub const MSG_SASEX_DP: u8 = 0x16;

/// Serial Attached SCSI (SAS) Ex Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct SasExDevicePath {
    pub header: DevicePathProtocol,
    /// 8-byte array of the SAS Address for Serial Attached SCSI Target Port.
    pub sas_address: [u8; 8],
    /// 8-byte array of the SAS Logical Unit Number.
    pub lun: [u8; 8],
    /// More Information about the device and its interconnect.
    pub device_topology: u16,
    /// Relative Target Port (RTP).
    pub relative_target_port: u16,
}

/// NvmExpress Namespace Device Path SubType.
pub const MSG_NVME_NAMESPACE_DP: u8 = 0x17;

/// NvmExpress Namespace Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct NvmeNamespaceDevicePath {
    pub header: DevicePathProtocol,
    pub namespace_id: u32,
    pub namespace_uuid: u64,
}

/// NVMe over Fabric (NVMe-oF) Namespace Device Path SubType.
pub const MSG_NVME_OF_NAMESPACE_DP: u8 = 0x22;

/// NVMe over Fabric (NVMe-oF) Namespace Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct NvmeOfNamespaceDevicePath {
    pub header: DevicePathProtocol,
    /// Namespace Identifier Type (NIDT)
    pub namespace_id_type: u8,
    /// Namespace Identifier (NID)
    pub namespace_id: [u8; 16],
    // Unique identifier of an NVM subsystem
    // pub subsystem_nqn: [Char8; _],
}

/// DNS Device Path SubType.
pub const MSG_DNS_DP: u8 = 0x1F;

/// DNS Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct DnsDevicePath {
    pub header: DevicePathProtocol,
    /// Indicates the DNS server address is IPv4 or IPv6 address.
    pub is_ipv6: bool,
    // Instance of the DNS server address.
    // pub dns_server_ip: [IpAddress; _],
}

/// Uniform Resource Identifiers (URI) Device Path SubType.
pub const MSG_URI_DP: u8 = 0x18;

/// Uniform Resource Identifiers (URI) Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UriDevicePath {
    pub header: DevicePathProtocol,
    // Instance of the URI pursuant to RFC 3986.
    // pub uri: [Char8; _],
}

/// Universal Flash Storage (UFS) Device Path SubType.
pub const MSG_UFS_DP: u8 = 0x19;

/// Universal Flash Storage (UFS) Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct UfsDevicePath {
    pub header: DevicePathProtocol,
    /// Target ID on the UFS bus (PUN).
    pub pun: u8,
    /// Logical Unit Number (LUN).
    pub lun: u8,
}

/// SD (Secure Digital) Device Path SubType.
pub const MSG_SD_DP: u8 = 0x1A;

/// SD (Secure Digital) Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct SdDevicePath {
    pub header: DevicePathProtocol,
    pub slot_number: u8,
}

/// EMMC (Embedded MMC) Device Path SubType.
pub const MSG_EMMC_DP: u8 = 0x1D;

/// EMMC (Embedded MMC) Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct EmmcDevicePath {
    pub header: DevicePathProtocol,
    pub slot_number: u8,
}

/// iSCSI Device Path SubType.
pub const MSG_ISCSI_DP: u8 = 0x13;

/// iSCSI Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct IScsiDevicePath {
    pub header: DevicePathProtocol,
    /// Network Protocol (0 = TCP, 1+ = reserved).
    pub network_protocol: u16,
    /// iSCSI Login Options.
    pub login_pption: u16,
    /// iSCSI Logical Unit Number.
    pub lun: u64,
    /// iSCSI Target Portal group tag the initiator intends
    /// to establish a session with.
    pub target_portal_group_tag: u16,
    // iSCSI NodeTarget Name. The length of the name
    // is determined by subtracting the offset of this field from Length.
    // pub target_name: [Char8; _],
}

pub const ISCSI_LOGIN_OPTION_NO_HEADER_DIGEST           : u16 = 0x0000;
pub const ISCSI_LOGIN_OPTION_HEADER_DIGEST_USING_CRC32C : u16 = 0x0002;
pub const ISCSI_LOGIN_OPTION_NO_DATA_DIGEST             : u16 = 0x0000;
pub const ISCSI_LOGIN_OPTION_DATA_DIGEST_USING_CRC32C   : u16 = 0x0008;
pub const ISCSI_LOGIN_OPTION_AUTHMETHOD_CHAP            : u16 = 0x0000;
pub const ISCSI_LOGIN_OPTION_AUTHMETHOD_NON             : u16 = 0x1000;
pub const ISCSI_LOGIN_OPTION_CHAP_BI                    : u16 = 0x0000;
pub const ISCSI_LOGIN_OPTION_CHAP_UNI                   : u16 = 0x2000;

/// VLAN Device Path SubType.
pub const MSG_VLAN_DP: u8 = 0x14;

/// VLAN Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct VlanDevicePath {
    pub header: DevicePathProtocol,
    /// VLAN identifier (0-4094).
    pub vlan_id: u16,
}

/// Bluetooth Device Path SubType.
pub const MSG_BLUETOOTH_DP: u8 = 0x1b;

/// Bluetooth Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct BluetoothDevicePath {
    pub header: DevicePathProtocol,
    /// Bluetooth address.
    pub address: BluetoothAddress,
}

/// Wi-Fi Device Path SubType.
pub const MSG_WIFI_DP: u8 = 0x1C;

/// Wi-Fi Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct WifiDevicePath {
    pub header: DevicePathProtocol,
    /// Service set identifier. A 32-byte octets string.
    pub ssid: [u8; 32],
}

/// Bluetooth LE Device Path SubType.
pub const MSG_BLUETOOTH_LE_DP: u8 = 0x1E;

/// Bluetooth LE Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct BluetoothLeDevicePath {
    pub header: DevicePathProtocol,
    /// Bluetooth Low Energy address.
    pub address: BluetoothLeAddress,
}

/// Media Device Path Type.
pub const MEDIA_DEVICE_PATH: u8 = 0x04;

/// Hard Drive Media Device Path SubType.
pub const MEDIA_HARDDRIVE_DP: u8 = 0x01;

/// Hard Drive Media Device Path.
/// The Hard Drive Media Device Path is used to represent a partition on a hard drive.
#[derive(Debug)]
#[repr(C,packed)]
pub struct HardDriveDevicePath {
    pub header: DevicePathProtocol,
    /// Describes the entry in a partition table, starting with entry 1.
    /// Partition number zero represents the entire device. Valid
    /// partition numbers for a MBR partition are [1, 4]. Valid
    /// partition numbers for a GPT partition are [1, NumberOfPartitionEntries].
    pub partition_number: u32,
    /// Starting LBA of the partition on the hard drive.
    pub partition_start: u64,
    /// Size of the partition in units of Logical Blocks.
    pub partition_size: u64,
    /// Signature unique to this partition:
    /// If SignatureType is 0, this field has to be initialized with 16 zeros.
    /// If SignatureType is 1, the MBR signature is stored in the first 4 bytes of this field.
    /// The other 12 bytes are initialized with zeros.
    /// If SignatureType is 2, this field contains a 16 byte signature.
    pub signature: [u8; 16],
    /// Partition Format: (Unused values reserved).
    /// 0x01 - PC-AT compatible legacy MBR.
    /// 0x02 - GUID Partition Table.
    pub mbr_type: u8,
    /// Type of Disk Signature: (Unused values reserved).
    /// 0x00 - No Disk Signature.
    /// 0x01 - 32-bit signature from address 0x1b8 of the type 0x01 MBR.
    /// 0x02 - GUID signature.
    pub signature_type: u8,
}

pub const MBR_TYPE_PCAT: u8 = 0x01;
pub const MBR_TYPE_EFI_PARTITION_TABLE_HEADER: u8 = 0x02;

pub const NO_DISK_SIGNATURE: u8 = 0x00;
pub const SIGNATURE_TYPE_MBR: u8 = 0x01;
pub const SIGNATURE_TYPE_GUID: u8 = 0x02;

/// CD-ROM Media Device Path SubType.
pub const MEDIA_CDROM_DP: u8 = 0x02;

/// CD-ROM Media Device Path.
/// The CD-ROM Media Device Path is used to define a system partition that exists on a CD-ROM.
#[derive(Debug)]
#[repr(C,packed)]
pub struct CdRomDevicePath {
    pub header: DevicePathProtocol,
    /// Boot Entry number from the Boot Catalog. The Initial/Default entry is defined as zero.
    pub boot_entry: u32,
    /// Starting RBA of the partition on the medium. CD-ROMs use Relative logical Block Addressing.
    pub partition_start: u64,
    /// Size of the partition in units of Blocks, also called Sectors.
    pub partition_size: u64,
}

/// Media Vendor Device Path SubType.
/// Use VENDOR_DEVICE_PATH struct
pub const MEDIA_VENDOR_DP: u8 = 0x03;

/// File Path Media Device Path SubType.
pub const MEDIA_FILEPATH_DP: u8 = 0x04;

/// File Path Media Device Path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct FilePathDevicePath {
    pub header: DevicePathProtocol,
    // A NULL-terminated Path string including directory and file names.
    // pub path_name: [Char16; _],
}

/// Media Protocol Device Path SubType.
pub const MEDIA_PROTOCOL_DP: u8 = 0x05;

/// Media Protocol Device Path.
/// The Media Protocol Device Path is used to denote the protocol that is being
/// used in a device path at the location of the path specified.
/// Many protocols are inherent to the style of device path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MediaProtocolDevicePath {
    pub header: DevicePathProtocol,
    /// The ID of the protocol.
    pub protocol: Guid,
}

/// PIWG Firmware File Device Path SubType.
pub const MEDIA_PIWG_FW_FILE_DP: u8 = 0x06;

/// PIWG Firmware File Device Path.
/// This device path is used by systems implementing the UEFI PI Specification 1.0 to describe a firmware file.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MediaFwVolFilePathDevicePath {
    pub header: DevicePathProtocol,
    /// Firmware file name
    pub fv_file_name: Guid,
}

/// PIWG Firmware Volume Device Path SubType.
pub const MEDIA_PIWG_FW_VOL_DP: u8 = 0x07;

/// PIWG Firmware Volume Device Path.
/// This device path is used by systems implementing the UEFI PI Specification 1.0 to describe a firmware volume.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MediaFwVolDevicePath {
    pub header: DevicePathProtocol,
    /// Firmware volume name.
    pub fv_name: Guid,
}

/// Media relative offset range device path SubType.
pub const MEDIA_RELATIVE_OFFSET_RANGE_DP: u8 = 0x08;

/// Media relative offset range device path.
/// Used to describe the offset range of media relative.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MediaRelativeOffsetRangeDevicePath {
    pub header: DevicePathProtocol,
    pub reserved: u32,
    pub starting_offset: u64,
    pub ending_offset: u64,
}

/// This GUID defines a RAM Disk supporting a raw disk format in volatile memory.
pub const EFI_VIRTUAL_DISK_GUID: Guid = guid!("77AB535A-45FC-624B-5560-F7B281D1F96E");

/// This GUID defines a RAM Disk supporting an ISO image in volatile memory.
pub const EFI_VIRTUAL_CD_GUID: Guid = guid!("3D5ABD30-4175-87CE-6D64-D2ADE523C4BB");

/// This GUID defines a RAM Disk supporting a raw disk format in persistent memory.
pub const EFI_PERSISTENT_VIRTUAL_DISK_GUID: Guid = guid!("5CEA02C9-4D07-69D3-269F-4496FBE096F9");

/// This GUID defines a RAM Disk supporting an ISO image in persistent memory.
pub const EFI_PERSISTENT_VIRTUAL_CD_GUID: Guid = guid!("08018188-42CD-BB48-100F-5387D53DED3D");

/// Media ram disk device path SubType.
pub const MEDIA_RAM_DISK_DP: u8 = 0x09;

/// Media ram disk device path.
/// Used to describe the ram disk device path.
#[derive(Debug)]
#[repr(C,packed)]
pub struct MediaRamDiskDevicePath {
    pub header: DevicePathProtocol,
    /// Starting Memory Address.
    pub starting_addr: [u32; 2],
    /// Ending Memory Address.
    pub ending_addr: [u32; 2],
    /// GUID that defines the type of the RAM Disk.
    pub type_guid: Guid,
    /// RAM Disk instance number, if supported. The default value is zero.
    pub instance: u16,
}

/// BIOS Boot Specification Device Path Type.
pub const BBS_DEVICE_PATH: u8 = 0x05;

/// BIOS Boot Specification Device Path SubType.
pub const BBS_BBS_DP: u8 = 0x01;

/// BIOS Boot Specification Device Path.
/// This Device Path is used to describe the booting of non-EFI-aware operating systems.
#[derive(Debug)]
#[repr(C,packed)]
pub struct BbsBbsDevicePath {
    pub header: DevicePathProtocol,
    /// Device Type as defined by the BIOS Boot Specification.
    pub device_type: u16,
    /// Status Flags as defined by the BIOS Boot Specification.
    pub status_flag: u16,
    // Null-terminated ASCII string that describes the boot device to a user.
    // pub string: [Char8; _],
}

// DeviceType definitions - from BBS specification
pub const BBS_TYPE_FLOPPY: u8 = 0x01;
pub const BBS_TYPE_HARDDRIVE: u8 = 0x02;
pub const BBS_TYPE_CDROM: u8 = 0x03;
pub const BBS_TYPE_PCMCIA: u8 = 0x04;
pub const BBS_TYPE_USB: u8 = 0x05;
pub const BBS_TYPE_EMBEDDED_NETWORK: u8 = 0x06;
pub const BBS_TYPE_BEV: u8 = 0x80;
pub const BBS_TYPE_UNKNOWN: u8 = 0xFF;

/// Device Path terminator Type.
pub const END_DEVICE_PATH_TYPE: u8 = 0x7f;

/// Entire Device Path terminator SubType.
pub const END_ENTIRE_DEVICE_PATH_SUBTYPE: u8 = 0xFF;

/// Device Path Instance terminator SubType.
pub const END_INSTANCE_DEVICE_PATH_SUBTYPE: u8 = 0x01;

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathToTextProtocol {
    pub convert_device_node_to_text: unsafe extern "efiapi" fn(
        device_node: *const DevicePathProtocol,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
    pub convert_device_path_to_text: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        display_only: bool,
        allow_shortcuts: bool,
    ) -> *const Char16,
}

impl DevicePathToTextProtocol {
    pub const GUID: Guid = guid!("8b843e20-8132-4852-90cc-551a4e4a7f1c");
}

#[derive(Debug)]
#[repr(C)]
pub struct DevicePathFromTextProtocol {
    pub convert_text_to_device_node:
        unsafe extern "efiapi" fn(text_device_node: *const Char16) -> *const DevicePathProtocol,
    pub convert_text_to_device_path:
        unsafe extern "efiapi" fn(text_device_path: *const Char16) -> *const DevicePathProtocol,
}

impl DevicePathFromTextProtocol {
    pub const GUID: Guid = guid!("05c99a21-c70f-4ad2-8a5f-35df3343f51e");
}
