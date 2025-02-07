// SPDX-License-Identifier: MIT OR Apache-2.0

// DO NOT EDIT
//
// This file was automatically generated with:
// `cargo xtask gen-code`
//
// See `/xtask/src/device_path/README.md` for more details.
#![allow(clippy::missing_const_for_fn)]
#![allow(missing_debug_implementations)]
use crate::protocol::device_path;
use crate::table::boot::MemoryType;
use crate::{guid, Guid, IpAddress};
use bitflags::bitflags;
use device_path::DevicePathProtocol as DevicePathHeader;
#[cfg(doc)]
use device_path::DeviceType;
/// Device path nodes for [`DeviceType::END`].
pub mod end {
    use super::*;
    #[repr(C, packed)]
    pub struct Instance {
        pub header: DevicePathHeader,
    }

    #[repr(C, packed)]
    pub struct Entire {
        pub header: DevicePathHeader,
    }
}

/// Device path nodes for [`DeviceType::HARDWARE`].
pub mod hardware {
    use super::*;
    #[repr(C, packed)]
    pub struct Pci {
        pub header: DevicePathHeader,
        pub function: u8,
        pub device: u8,
    }

    #[repr(C, packed)]
    pub struct Pccard {
        pub header: DevicePathHeader,
        pub function: u8,
    }

    #[repr(C, packed)]
    pub struct MemoryMapped {
        pub header: DevicePathHeader,
        pub memory_type: MemoryType,
        pub start_address: u64,
        pub end_address: u64,
    }

    #[repr(C, packed)]
    pub struct Vendor {
        pub header: DevicePathHeader,
        pub vendor_guid: Guid,
        pub vendor_defined_data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct Controller {
        pub header: DevicePathHeader,
        pub controller_number: u32,
    }

    #[repr(C, packed)]
    pub struct Bmc {
        pub header: DevicePathHeader,
        pub interface_type: device_path::hardware::BmcInterfaceType,
        pub base_address: u64,
    }

    newtype_enum! { # [doc = " Baseboard Management Controller (BMC) host interface type."] pub enum BmcInterfaceType : u8 => { # [doc = " Unknown."] UNKNOWN = 0x00 , # [doc = " Keyboard controller style."] KEYBOARD_CONTROLLER_STYLE = 0x01 , # [doc = " Server management interface chip."] SERVER_MANAGEMENT_INTERFACE_CHIP = 0x02 , # [doc = " Block transfer."] BLOCK_TRANSFER = 0x03 , }

    }
}

/// Device path nodes for [`DeviceType::ACPI`].
pub mod acpi {
    use super::*;
    #[repr(C, packed)]
    pub struct Acpi {
        pub header: DevicePathHeader,
        pub hid: u32,
        pub uid: u32,
    }

    #[repr(C, packed)]
    pub struct Expanded {
        pub header: DevicePathHeader,
        pub hid: u32,
        pub uid: u32,
        pub cid: u32,
        pub data: [u8; 0],
    }

    #[repr(C, packed)]
    pub struct Adr {
        pub header: DevicePathHeader,
        pub adr: [u32; 0usize],
    }

    #[repr(C, packed)]
    pub struct Nvdimm {
        pub header: DevicePathHeader,
        pub nfit_device_handle: u32,
    }
}

/// Device path nodes for [`DeviceType::MESSAGING`].
pub mod messaging {
    use super::*;
    #[repr(C, packed)]
    pub struct Atapi {
        pub header: DevicePathHeader,
        pub primary_secondary: device_path::messaging::PrimarySecondary,
        pub master_slave: device_path::messaging::MasterSlave,
        pub logical_unit_number: u16,
    }

    #[repr(C, packed)]
    pub struct Scsi {
        pub header: DevicePathHeader,
        pub target_id: u16,
        pub logical_unit_number: u16,
    }

    #[repr(C, packed)]
    pub struct FibreChannel {
        pub header: DevicePathHeader,
        pub _reserved: u32,
        pub world_wide_name: u64,
        pub logical_unit_number: u64,
    }

    #[repr(C, packed)]
    pub struct FibreChannelEx {
        pub header: DevicePathHeader,
        pub _reserved: u32,
        pub world_wide_name: [u8; 8usize],
        pub logical_unit_number: [u8; 8usize],
    }

    #[repr(C, packed)]
    pub struct Ieee1394 {
        pub header: DevicePathHeader,
        pub _reserved: u32,
        pub guid: [u8; 8usize],
    }

    #[repr(C, packed)]
    pub struct Usb {
        pub header: DevicePathHeader,
        pub parent_port_number: u8,
        pub interface: u8,
    }

    #[repr(C, packed)]
    pub struct Sata {
        pub header: DevicePathHeader,
        pub hba_port_number: u16,
        pub port_multiplier_port_number: u16,
        pub logical_unit_number: u16,
    }

    #[repr(C, packed)]
    pub struct UsbWwid {
        pub header: DevicePathHeader,
        pub interface_number: u16,
        pub device_vendor_id: u16,
        pub device_product_id: u16,
        pub serial_number: [u16; 0usize],
    }

    #[repr(C, packed)]
    pub struct DeviceLogicalUnit {
        pub header: DevicePathHeader,
        pub logical_unit_number: u8,
    }

    #[repr(C, packed)]
    pub struct UsbClass {
        pub header: DevicePathHeader,
        pub vendor_id: u16,
        pub product_id: u16,
        pub device_class: u8,
        pub device_subclass: u8,
        pub device_protocol: u8,
    }

    #[repr(C, packed)]
    pub struct I2o {
        pub header: DevicePathHeader,
        pub target_id: u32,
    }

    #[repr(C, packed)]
    pub struct MacAddress {
        pub header: DevicePathHeader,
        pub mac_address: [u8; 32usize],
        pub interface_type: u8,
    }

    #[repr(C, packed)]
    pub struct Ipv4 {
        pub header: DevicePathHeader,
        pub local_ip_address: [u8; 4usize],
        pub remote_ip_address: [u8; 4usize],
        pub local_port: u16,
        pub remote_port: u16,
        pub protocol: u16,
        pub ip_address_origin: device_path::messaging::Ipv4AddressOrigin,
        pub gateway_ip_address: [u8; 4usize],
        pub subnet_mask: [u8; 4usize],
    }

    #[repr(C, packed)]
    pub struct Ipv6 {
        pub header: DevicePathHeader,
        pub local_ip_address: [u8; 16usize],
        pub remote_ip_address: [u8; 16usize],
        pub local_port: u16,
        pub remote_port: u16,
        pub protocol: u16,
        pub ip_address_origin: device_path::messaging::Ipv6AddressOrigin,
        pub prefix_length: u8,
        pub gateway_ip_address: [u8; 16usize],
    }

    #[repr(C, packed)]
    pub struct Vlan {
        pub header: DevicePathHeader,
        pub vlan_id: u16,
    }

    #[repr(C, packed)]
    pub struct Infiniband {
        pub header: DevicePathHeader,
        pub resource_flags: device_path::messaging::InfinibandResourceFlags,
        pub port_gid: [u8; 16usize],
        pub ioc_guid_or_service_id: u64,
        pub target_port_id: u64,
        pub device_id: u64,
    }

    #[repr(C, packed)]
    pub struct Uart {
        pub header: DevicePathHeader,
        pub _reserved: u32,
        pub baud_rate: u64,
        pub data_bits: u8,
        pub parity: device_path::messaging::Parity,
        pub stop_bits: device_path::messaging::StopBits,
    }

    #[repr(C, packed)]
    pub struct Vendor {
        pub header: DevicePathHeader,
        pub vendor_guid: Guid,
        pub vendor_defined_data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct SasEx {
        pub header: DevicePathHeader,
        pub sas_address: [u8; 8usize],
        pub logical_unit_number: [u8; 8usize],
        pub info: u16,
        pub relative_target_port: u16,
    }

    #[repr(C, packed)]
    pub struct Iscsi {
        pub header: DevicePathHeader,
        pub protocol: device_path::messaging::IscsiProtocol,
        pub options: device_path::messaging::IscsiLoginOptions,
        pub logical_unit_number: [u8; 8usize],
        pub target_portal_group_tag: u16,
        pub iscsi_target_name: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct NvmeNamespace {
        pub header: DevicePathHeader,
        pub namespace_identifier: u32,
        pub ieee_extended_unique_identifier: u64,
    }

    #[repr(C, packed)]
    pub struct Uri {
        pub header: DevicePathHeader,
        pub value: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct Ufs {
        pub header: DevicePathHeader,
        pub target_id: u8,
        pub logical_unit_number: u8,
    }

    #[repr(C, packed)]
    pub struct Sd {
        pub header: DevicePathHeader,
        pub slot_number: u8,
    }

    #[repr(C, packed)]
    pub struct Bluetooth {
        pub header: DevicePathHeader,
        pub device_address: [u8; 6usize],
    }

    #[repr(C, packed)]
    pub struct Wifi {
        pub header: DevicePathHeader,
        pub ssid: [u8; 32usize],
    }

    #[repr(C, packed)]
    pub struct Emmc {
        pub header: DevicePathHeader,
        pub slot_number: u8,
    }

    #[repr(C, packed)]
    pub struct BluetoothLe {
        pub header: DevicePathHeader,
        pub device_address: [u8; 6usize],
        pub address_type: device_path::messaging::BluetoothLeAddressType,
    }

    #[repr(C, packed)]
    pub struct Dns {
        pub header: DevicePathHeader,
        pub address_type: device_path::messaging::DnsAddressType,
        pub addresses: [IpAddress; 0usize],
    }

    #[repr(C, packed)]
    pub struct NvdimmNamespace {
        pub header: DevicePathHeader,
        pub uuid: [u8; 16usize],
    }

    #[repr(C, packed)]
    pub struct RestService {
        pub header: DevicePathHeader,
        pub service_type: device_path::messaging::RestServiceType,
        pub access_mode: device_path::messaging::RestServiceAccessMode,
        pub vendor_guid_and_data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct NvmeOfNamespace {
        pub header: DevicePathHeader,
        pub nidt: u8,
        pub nid: [u8; 16usize],
        pub subsystem_nqn: [u8; 0usize],
    }

    newtype_enum! { # [doc = " Whether the ATAPI device is primary or secondary."] pub enum PrimarySecondary : u8 => { # [doc = " Primary."] PRIMARY = 0x00 , # [doc = " Secondary."] SECONDARY = 0x01 , }

    }

    newtype_enum! { # [doc = " Whether the ATAPI device is master or slave."] pub enum MasterSlave : u8 => { # [doc = " Master mode."] MASTER = 0x00 , # [doc = " Slave mode."] SLAVE = 0x01 , }

    }

    newtype_enum! { # [doc = " Origin of the source IP address."] pub enum Ipv4AddressOrigin : u8 => { # [doc = " Source IP address was assigned through DHCP."] DHCP = 0x00 , # [doc = " Source IP address is statically bound."] STATIC = 0x01 , }

    }

    newtype_enum! { # [doc = " Origin of the local IP address."] pub enum Ipv6AddressOrigin : u8 => { # [doc = " Local IP address was manually configured."] MANUAL = 0x00 , # [doc = " Local IP address assigned through IPv6 stateless"] # [doc = " auto-configuration."] STATELESS_AUTO_CONFIGURATION = 0x01 , # [doc = " Local IP address assigned through IPv6 stateful"] # [doc = " configuration."] STATEFUL_CONFIGURATION = 0x02 , }

    }

    bitflags! { # [doc = " Flags to identify/manage InfiniBand elements."] # [derive (Clone , Copy , Debug , Default , PartialEq , Eq , PartialOrd , Ord)] # [repr (transparent)] pub struct InfinibandResourceFlags : u32 { # [doc = " Set = service, unset = IOC."] const SERVICE = 0x0000_0001 ; # [doc = " Extended boot environment."] const EXTENDED_BOOT_ENVIRONMENT = 0x0000_0002 ; # [doc = " Console protocol."] const CONSOLE_PROTOCOL = 0x0000_0004 ; # [doc = " Storage protocol."] const STORAGE_PROTOCOL = 0x0000_0008 ; # [doc = " Network protocol."] const NETWORK_PROTOCOL = 0x0000_0010 ; }

    }

    newtype_enum! { # [doc = " UART parity setting."] pub enum Parity : u8 => { # [doc = " Default parity."] DEFAULT = 0x00 , # [doc = " No parity."] NO = 0x01 , # [doc = " Even parity."] EVEN = 0x02 , # [doc = " Odd parity."] ODD = 0x03 , # [doc = " Mark parity."] MARK = 0x04 , # [doc = " Space parity."] SPACE = 0x05 , }

    }

    newtype_enum! { # [doc = " UART number of stop bits."] pub enum StopBits : u8 => { # [doc = " Default number of stop bits."] DEFAULT = 0x00 , # [doc = " 1 stop bit."] ONE = 0x01 , # [doc = " 1.5 stop bits."] ONE_POINT_FIVE = 0x02 , # [doc = " 2 stop bits."] TWO = 0x03 , }

    }

    newtype_enum! { # [doc = " iSCSI network protocol."] pub enum IscsiProtocol : u16 => { # [doc = " TCP."] TCP = 0x0000 , }

    }

    bitflags! { # [doc = " iSCSI login options."] # [derive (Clone , Copy , Debug , Default , PartialEq , Eq , PartialOrd , Ord)] # [repr (transparent)] pub struct IscsiLoginOptions : u16 { # [doc = " Header digest using CRC32. If not set, no header digest."] const HEADER_DIGEST_USING_CRC32 = 0x0002 ; # [doc = " Data digest using CRC32. If not set, no data digest."] const DATA_DIGEST_USING_CRC32 = 0x0008 ; # [doc = " Auth method none. If not set, auth method CHAP."] const AUTH_METHOD_NONE = 0x0800 ; # [doc = " CHAP UNI. If not set, CHAP BI."] const CHAP_UNI = 0x1000 ; }

    }

    newtype_enum! { # [doc = " BluetoothLE address type."] pub enum BluetoothLeAddressType : u8 => { # [doc = " Public device address."] PUBLIC = 0x00 , # [doc = " Random device address."] RANDOM = 0x01 , }

    }

    newtype_enum! { # [doc = " Whether the address is IPv4 or IPv6."] pub enum DnsAddressType : u8 => { # [doc = " DNS server address is IPv4."] IPV4 = 0x00 , # [doc = " DNS server address is IPv6."] IPV6 = 0x01 , }

    }

    newtype_enum! { # [doc = " Type of REST service."] pub enum RestServiceType : u8 => { # [doc = " Redfish REST service."] REDFISH = 0x01 , # [doc = " OData REST service."] ODATA = 0x02 , # [doc = " Vendor-specific REST service."] VENDOR = 0xff , }

    }

    newtype_enum! { # [doc = " Whether the service is in-band or out-of-band."] pub enum RestServiceAccessMode : u8 => { # [doc = " In-band REST service."] IN_BAND = 0x01 , # [doc = " Out-of-band REST service."] OUT_OF_BAND = 0x02 , }

    }
}

/// Device path nodes for [`DeviceType::MEDIA`].
pub mod media {
    use super::*;
    #[repr(C, packed)]
    pub struct HardDrive {
        pub header: DevicePathHeader,
        pub partition_number: u32,
        pub partition_start: u64,
        pub partition_size: u64,
        pub partition_signature: [u8; 16usize],
        pub partition_format: device_path::media::PartitionFormat,
        pub signature_type: u8,
    }

    #[repr(C, packed)]
    pub struct CdRom {
        pub header: DevicePathHeader,
        pub boot_entry: u32,
        pub partition_start: u64,
        pub partition_size: u64,
    }

    #[repr(C, packed)]
    pub struct Vendor {
        pub header: DevicePathHeader,
        pub vendor_guid: Guid,
        pub vendor_defined_data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct FilePath {
        pub header: DevicePathHeader,
        pub path_name: [u16; 0usize],
    }

    #[repr(C, packed)]
    pub struct Protocol {
        pub header: DevicePathHeader,
        pub protocol_guid: Guid,
    }

    #[repr(C, packed)]
    pub struct PiwgFirmwareFile {
        pub header: DevicePathHeader,
        pub data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct PiwgFirmwareVolume {
        pub header: DevicePathHeader,
        pub data: [u8; 0usize],
    }

    #[repr(C, packed)]
    pub struct RelativeOffsetRange {
        pub header: DevicePathHeader,
        pub _reserved: u32,
        pub starting_offset: u64,
        pub ending_offset: u64,
    }

    #[repr(C, packed)]
    pub struct RamDisk {
        pub header: DevicePathHeader,
        pub starting_address: u64,
        pub ending_address: u64,
        pub disk_type: device_path::media::RamDiskType,
        pub disk_instance: u16,
    }

    newtype_enum! { # [doc = " Hard drive partition format."] pub enum PartitionFormat : u8 => { # [doc = " MBR (PC-AT compatible Master Boot Record) format."] MBR = 0x01 , # [doc = " GPT (GUID Partition Table) format."] GPT = 0x02 , }

    }

    newtype_enum! { # [doc = " RAM disk type."] pub enum RamDiskType : Guid => { # [doc = " RAM disk with a raw disk format in volatile memory."] VIRTUAL_DISK = guid ! ("77ab535a-45fc-624b-5560-f7b281d1f96e") , # [doc = " RAM disk of an ISO image in volatile memory."] VIRTUAL_CD = guid ! ("3d5abd30-4175-87ce-6d64-d2ade523c4bb") , # [doc = " RAM disk with a raw disk format in persistent memory."] PERSISTENT_VIRTUAL_DISK = guid ! ("5cea02c9-4d07-69d3-269f-4496fbe096f9") , # [doc = " RAM disk of an ISO image in persistent memory."] PERSISTENT_VIRTUAL_CD = guid ! ("08018188-42cd-bb48-100f-5387d53ded3d") , }

    }
}

/// Device path nodes for [`DeviceType::BIOS_BOOT_SPEC`].
pub mod bios_boot_spec {
    use super::*;
    #[repr(C, packed)]
    pub struct BootSpecification {
        pub header: DevicePathHeader,
        pub device_type: u16,
        pub status_flag: u16,
        pub description_string: [u8; 0usize],
    }
}
