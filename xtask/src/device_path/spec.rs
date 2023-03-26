// See the README in this directory for details of what this file is.
//
// The nodes here are in the same order as in the UEFI Specification.

mod end {
    /// Node that terminates a [`DevicePathInstance`].
    ///
    /// [`DevicePathInstance`]: crate::proto::device_path::DevicePathInstance
    #[node(static_size = 4)]
    struct Instance;

    /// Node that terminates an entire [`DevicePath`].
    ///
    /// [`DevicePath`]: crate::proto::device_path::DevicePath
    #[node(static_size = 4)]
    struct Entire;
}

mod hardware {
    /// PCI hardware device path node.
    #[node(static_size = 6)]
    struct Pci {
        /// PCI function number.
        function: u8,

        /// PCI device number.
        device: u8,
    }

    /// PCCARD hardware device path node.
    #[node(static_size = 5)]
    struct Pccard {
        /// Function number starting from 0.
        function: u8,
    }

    /// Memory mapped hardware device path node.
    #[node(static_size = 24)]
    struct MemoryMapped {
        /// Memory type.
        memory_type: MemoryType,

        /// Starting memory address.
        start_address: u64,

        /// Ending memory address.
        end_address: u64,
    }

    /// Vendor-defined hardware device path node.
    #[node(static_size = 20)]
    struct Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        vendor_guid: Guid,

        /// Vendor-defined data.
        vendor_defined_data: [u8],
    }

    /// Controller hardware device path node.
    #[node(static_size = 8)]
    struct Controller {
        /// Controller number.
        controller_number: u32,
    }

    /// Baseboard Management Controller (BMC) host interface hardware
    /// device path node.
    #[node(static_size = 13)]
    struct Bmc {
        /// Host interface type.
        interface_type: crate::proto::device_path::hardware::BmcInterfaceType,

        /// Base address of the BMC. If the least-significant bit of the
        /// field is a 1 then the address is in I/O space, otherwise the
        /// address is memory-mapped.
        base_address: u64,
    }

    newtype_enum! {
        /// Baseboard Management Controller (BMC) host interface type.
        pub enum BmcInterfaceType: u8 => {
            /// Unknown.
            UNKNOWN = 0x00,

            /// Keyboard controller style.
            KEYBOARD_CONTROLLER_STYLE = 0x01,

            /// Server management interface chip.
            SERVER_MANAGEMENT_INTERFACE_CHIP = 0x02,

            /// Block transfer.
            BLOCK_TRANSFER = 0x03,
        }
    }
}

mod acpi {
    /// ACPI device path node.
    #[node(static_size = 12, sub_type = "ACPI")]
    struct Acpi {
        /// Device's PnP hardware ID stored in a numeric 32-bit
        /// compressed EISA-type ID.
        hid: u32,

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID.
        uid: u32,
    }

    /// Expanded ACPI device path node.
    #[node(static_size = 16)]
    struct Expanded {
        /// Device's PnP hardware ID stored in a numeric 32-bit compressed
        /// EISA-type ID.
        hid: u32,

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID.
        uid: u32,

        /// Device's compatible PnP hardware ID stored in a numeric 32-bit
        /// compressed EISA-type ID.
        cid: u32,

        /// Device's PnP hardware ID stored as a null-terminated ASCII
        /// string. This value must match the corresponding HID in the
        /// ACPI name space. If the length of this string not including
        /// the null-terminator is 0, then the numeric HID is used.
        #[node(custom_get_impl)]
        hid_str: [u8],

        /// Unique ID that is required by ACPI if two devices have the
        /// same HID. This value is stored as a null-terminated ASCII
        /// string. If the length of this string not including the
        /// null-terminator is 0, then the numeric UID is used.
        #[node(custom_get_impl)]
        uid_str: [u8],

        /// Device's compatible PnP hardware ID stored as a
        /// null-terminated ASCII string. If the length of this string
        /// not including the null-terminator is 0, then the numeric CID
        /// is used.
        #[node(custom_get_impl)]
        cid_str: [u8],
    }

    impl Expanded {
        fn get_hid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 0)
        }

        fn get_uid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 1)
        }

        fn get_cid_str(&self) -> &[u8] {
            get_acpi_expanded_substr(&self.data, 2)
        }
    }

    /// Get the indices of the three nulls in the combined hid/uid/cid
    /// string. This never fails; if some nulls are missing then `None`
    /// is returned for those indices. If more than three nulls are
    /// present then the extra ones are ignored.
    fn acpi_expanded_null_indices(data: &[u8]) -> [Option<usize>; 3] {
        let mut iter = data
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, byte)| if byte == 0 { Some(index) } else { None })
            .fuse();

        [iter.next(), iter.next(), iter.next()]
    }

    /// Get the hid, uid, or cid string from the combined string. The
    /// returned string includes the trailing null if possible; if the
    /// substring was not properly null terminated then it ends at the
    /// end of `data`.
    ///
    /// This never fails; if there aren't enough nulls in the input
    /// string then an empty slice may be returned.
    fn get_acpi_expanded_substr(data: &[u8], string_index: usize) -> &[u8] {
        let [n0, n1, n2] = acpi_expanded_null_indices(data);

        let mut start = data.len();
        let mut end = start;

        match string_index {
            0 => {
                start = 0;
                if let Some(n0) = n0 {
                    end = n0 + 1;
                }
            }
            1 => {
                if let Some(n0) = n0 {
                    start = n0 + 1;
                    if let Some(n1) = n1 {
                        end = n1 + 1;
                    }
                }
            }
            2 => {
                if let Some(n1) = n1 {
                    start = n1 + 1;
                    if let Some(n2) = n2 {
                        end = n2 + 1;
                    }
                }
            }
            _ => {
                unreachable!("invalid string index")
            }
        }

        data.get(start..end).unwrap_or(&[])
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_get_acpi_expanded_substr() {
            // All valid strings.
            let s = b"ab\0cd\0ef\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"ef\0");

            // All valid empty strings.
            let s = b"\0\0\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"\0");

            // Invalid: missing third string.
            let s = b"ab\0cd\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");

            // Invalid: missing second and third strings.
            let s = b"ab\0";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");

            // Invalid: missing third null.
            let s = b"ab\0cd\0ef";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd\0");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"ef");

            // Invalid: missing second null.
            let s = b"ab\0cd";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab\0");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"cd");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");

            // Invalid: missing first null.
            let s = b"ab";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"ab");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");

            // Invalid: empty data.
            let s = b"";
            assert_eq!(get_acpi_expanded_substr(s, 0), b"");
            assert_eq!(get_acpi_expanded_substr(s, 1), b"");
            assert_eq!(get_acpi_expanded_substr(s, 2), b"");
        }
    }

    /// ADR ACPI device path node.
    #[node(static_size = 4)]
    struct Adr {
        /// ADR values. For video output devices the value of this field
        /// comes from Table B-2 ACPI 3.0 specification. At least one
        /// ADR value is required.
        #[node(build_type = "&'a AdrSlice")]
        adr: [u32],
    }

    /// Wrapper for [`u32`] ADR values that enforces at least one
    /// element is present.
    #[build]
    #[repr(transparent)]
    #[derive(Debug)]
    pub struct AdrSlice([u32]);

    #[build]
    impl AdrSlice {
        /// Create a new `AdrSlice`. Returns `None` if the input slice
        /// is empty.
        #[must_use]
        pub fn new(slice: &[u32]) -> Option<&Self> {
            if slice.is_empty() {
                None
            } else {
                // Safety: `AdrSlice` has the same repr as `[u32]`.
                let adr_slice: &Self = unsafe { core::mem::transmute(slice) };
                Some(adr_slice)
            }
        }

        fn as_ptr(&self) -> *const u32 {
            self.0.as_ptr()
        }
    }

    /// NVDIMM ACPI device path node.
    #[node(static_size = 8)]
    struct Nvdimm {
        /// NFIT device handle.
        nfit_device_handle: u32,
    }
}

mod messaging {
    /// ATAPI messaging device path node.
    #[node(static_size = 8)]
    struct Atapi {
        /// Whether the ATAPI device is primary or secondary.
        primary_secondary: crate::proto::device_path::messaging::PrimarySecondary,

        /// Whether the ATAPI device is master or slave.
        master_slave: crate::proto::device_path::messaging::MasterSlave,

        /// Logical Unit Number (LUN).
        logical_unit_number: u16,
    }

    newtype_enum! {
        /// Whether the ATAPI device is primary or secondary.
        pub enum PrimarySecondary: u8 => {
            /// Primary.
            PRIMARY = 0x00,
            /// Secondary.
            SECONDARY = 0x01,
        }
    }

    newtype_enum! {
        /// Whether the ATAPI device is master or slave.
        pub enum MasterSlave: u8 => {
            /// Master mode.
            MASTER = 0x00,
            /// Slave mode.
            SLAVE = 0x01,
        }
    }

    /// SCSI messaging device path node.
    #[node(static_size = 8)]
    struct Scsi {
        /// Target ID on the SCSI bus.
        target_id: u16,

        /// Logical Unit Number.
        logical_unit_number: u16,
    }

    /// Fibre channel messaging device path node.
    #[node(static_size = 24)]
    struct FibreChannel {
        _reserved: u32,

        /// Fibre Channel World Wide Name.
        world_wide_name: u64,

        /// Fibre Channel Logical Unit Number.
        logical_unit_number: u64,
    }

    /// Fibre channel extended messaging device path node.
    #[node(static_size = 24)]
    struct FibreChannelEx {
        _reserved: u32,

        /// Fibre Channel end device port name (aka World Wide Name).
        world_wide_name: [u8; 8],

        /// Fibre Channel Logical Unit Number.
        logical_unit_number: [u8; 8],
    }

    /// 1394 messaging device path node.
    #[node(static_size = 16, sub_type = "MESSAGING_1394")]
    struct Ieee1394 {
        _reserved: u32,

        /// 1394 Global Unique ID. Note that this is not the same as a
        /// UEFI GUID.
        guid: [u8; 8],
    }

    /// USB messaging device path node.
    #[node(static_size = 6)]
    struct Usb {
        /// USB parent port number.
        parent_port_number: u8,

        /// USB interface number.
        interface: u8,
    }

    /// SATA messaging device path node.
    #[node(static_size = 10)]
    struct Sata {
        /// The HBA port number that facilitates the connection to the
        /// device or a port multiplier. The value 0xffff is reserved.
        hba_port_number: u16,

        /// the port multiplier port number that facilitates the
        /// connection to the device. Must be set to 0xffff if the
        /// device is directly connected to the HBA.
        port_multiplier_port_number: u16,

        /// Logical unit number.
        logical_unit_number: u16,
    }

    /// USB World Wide ID (WWID) messaging device path node.
    #[node(static_size = 10)]
    struct UsbWwid {
        /// USB interface number.
        interface_number: u16,

        /// USB vendor ID.
        device_vendor_id: u16,

        /// USB product ID.
        device_product_id: u16,

        /// Last 64 (or fewer) characters of the USB Serial number.
        serial_number: [u16],
    }

    /// Device logical unit messaging device path node.
    #[node(static_size = 5)]
    struct DeviceLogicalUnit {
        /// Logical Unit Number.
        logical_unit_number: u8,
    }

    /// USB class messaging device path node.
    #[node(static_size = 11)]
    struct UsbClass {
        /// USB vendor ID.
        vendor_id: u16,

        /// USB product ID.
        product_id: u16,

        /// USB device class.
        device_class: u8,

        /// USB device subclass.
        device_subclass: u8,

        /// USB device protocol.
        device_protocol: u8,
    }

    /// I2O messaging device path node.
    #[node(static_size = 8)]
    struct I2o {
        /// Target ID (TID).
        target_id: u32,
    }

    /// MAC address messaging device path node.
    #[node(static_size = 37)]
    struct MacAddress {
        /// MAC address for a network interface, padded with zeros.
        mac_address: [u8; 32],

        /// Network interface type. See
        /// <https://www.iana.org/assignments/smi-numbers/smi-numbers.xhtml#smi-numbers-5>
        interface_type: u8,
    }

    /// IPv4 messaging device path node.
    #[node(static_size = 27)]
    struct Ipv4 {
        /// Local IPv4 address.
        local_ip_address: [u8; 4],

        /// Remote IPv4 address.
        remote_ip_address: [u8; 4],

        /// Local port number.
        local_port: u16,

        /// Remote port number.
        remote_port: u16,

        /// Network protocol. See
        /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
        protocol: u16,

        /// Whether the source IP address is static or assigned via DHCP.
        ip_address_origin: crate::proto::device_path::messaging::Ipv4AddressOrigin,

        /// Gateway IP address.
        gateway_ip_address: [u8; 4],

        /// Subnet mask.
        subnet_mask: [u8; 4],
    }

    newtype_enum! {
        /// Origin of the source IP address.
        pub enum Ipv4AddressOrigin: u8 => {
            /// Source IP address was assigned through DHCP.
            DHCP = 0x00,

            /// Source IP address is statically bound.
            STATIC = 0x01,
        }
    }

    /// IPv6 messaging device path node.
    #[node(static_size = 60)]
    struct Ipv6 {
        /// Local Ipv6 address.
        local_ip_address: [u8; 16],

        /// Remote Ipv6 address.
        remote_ip_address: [u8; 16],

        /// Local port number.
        local_port: u16,

        /// Remote port number.
        remote_port: u16,

        /// Network protocol. See
        /// <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>
        protocol: u16,

        /// Origin of the local IP address.
        ip_address_origin: crate::proto::device_path::messaging::Ipv6AddressOrigin,

        /// Prefix length.
        prefix_length: u8,

        /// Gateway IP address.
        gateway_ip_address: [u8; 16],
    }

    newtype_enum! {
        /// Origin of the local IP address.
        pub enum Ipv6AddressOrigin: u8 => {
            /// Local IP address was manually configured.
            MANUAL = 0x00,

            /// Local IP address assigned through IPv6 stateless
            /// auto-configuration.
            STATELESS_AUTO_CONFIGURATION = 0x01,

            /// Local IP address assigned through IPv6 stateful
            /// configuration.
            STATEFUL_CONFIGURATION = 0x02,
        }
    }

    /// VLAN messaging device path node.
    #[node(static_size = 6)]
    struct Vlan {
        /// VLAN identifier (0-4094).
        vlan_id: u16,
    }

    /// InfiniBand messaging device path node.
    #[node(static_size = 48)]
    struct Infiniband {
        /// Flags to identify/manage InfiniBand elements.
        resource_flags: crate::proto::device_path::messaging::InfinibandResourceFlags,

        /// 128-bit Global Identifier for remote fabric port. Note that
        /// this is not the same as a UEFI GUID.
        port_gid: [u8; 16],

        /// IOC GUID if bit 0 of `resource_flags` is unset, or Service
        /// ID if bit 0 of `resource_flags` is set.
        ioc_guid_or_service_id: u64,

        /// 64-bit persistent ID of remote IOC port.
        target_port_id: u64,

        /// 64-bit persistent ID of remote device..
        device_id: u64,
    }

    bitflags! {
        /// Flags to identify/manage InfiniBand elements.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct InfinibandResourceFlags: u32 {
            /// Set = service, unset = IOC.
            const SERVICE = 0x0000_0001;

            /// Extended boot environment.
            const EXTENDED_BOOT_ENVIRONMENT = 0x0000_0002;

            /// Console protocol.
            const CONSOLE_PROTOCOL = 0x0000_0004;

            /// Storage protocol.
            const STORAGE_PROTOCOL = 0x0000_0008;

            /// Network protocol.
            const NETWORK_PROTOCOL = 0x0000_0010;
        }
    }

    /// UART messaging device path node.
    #[node(static_size = 19)]
    struct Uart {
        _reserved: u32,

        /// Baud rate setting, or 0 to use the device's default.
        baud_rate: u64,

        /// Number of data bits, or 0 to use the device's default.
        data_bits: u8,

        /// Parity setting.
        parity: crate::proto::device_path::messaging::Parity,

        /// Number of stop bits.
        stop_bits: crate::proto::device_path::messaging::StopBits,
    }

    newtype_enum! {
        /// UART parity setting.
        pub enum Parity: u8 => {
            /// Default parity.
            DEFAULT = 0x00,

            /// No parity.
            NO = 0x01,

            /// Even parity.
            EVEN = 0x02,

            /// Odd parity.
            ODD = 0x03,

            /// Mark parity.
            MARK = 0x04,

            /// Space parity.
            SPACE = 0x05,
        }
    }

    newtype_enum! {
        /// UART number of stop bits.
        pub enum StopBits: u8 => {
            /// Default number of stop bits.
            DEFAULT = 0x00,

            /// 1 stop bit.
            ONE = 0x01,

            /// 1.5 stop bits.
            ONE_POINT_FIVE = 0x02,

            /// 2 stop bits.
            TWO = 0x03,
        }
    }

    /// Vendor-defined messaging device path node.
    #[node(static_size = 20)]
    struct Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        vendor_guid: Guid,

        /// Vendor-defined data.
        vendor_defined_data: [u8],
    }

    // The spec defines a couple specific messaging-vendor types here,
    // one for UART and one for SAS. These are sort of subclasses of
    // `messaging::Vendor` so they don't quite fit the usual pattern of
    // other nodes. Leave them out for now, but we could potentially
    // provide a convenient way to construct them in the future.

    /// Serial Attached SCSI (SAS) extended messaging device path node.
    // The spec says 32, but it seems to be wrong.
    #[node(static_size = 24, sub_type = "MESSAGING_SCSI_SAS_EX")]
    struct SasEx {
        /// SAS address.
        sas_address: [u8; 8],

        /// Logical Unit Number.
        logical_unit_number: [u8; 8],

        /// Information about the device and its interconnect.
        info: u16,

        /// Relative Target Port (RTP).
        relative_target_port: u16,
    }

    /// iSCSI messaging device path node.
    #[node(static_size = 18)]
    struct Iscsi {
        /// Network protocol.
        protocol: crate::proto::device_path::messaging::IscsiProtocol,

        /// iSCSI login options (bitfield).
        options: crate::proto::device_path::messaging::IscsiLoginOptions,

        /// iSCSI Logical Unit Number.
        logical_unit_number: [u8; 8],

        /// iSCSI Target Portal group tag the initiator intends to
        /// establish a session with.
        target_portal_group_tag: u16,

        /// iSCSI Node Target name.
        ///
        /// The UEFI Specification does not specify how the string is
        /// encoded, but gives one example that appears to be
        /// null-terminated ASCII.
        iscsi_target_name: [u8],
    }

    newtype_enum! {
        /// iSCSI network protocol.
        pub enum IscsiProtocol: u16 => {
            /// TCP.
            TCP = 0x0000,
        }
    }

    bitflags! {
        /// iSCSI login options.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct IscsiLoginOptions: u16 {
            /// Header digest using CRC32. If not set, no header digest.
            const HEADER_DIGEST_USING_CRC32 = 0x0002;

            /// Data digest using CRC32. If not set, no data digest.
            const DATA_DIGEST_USING_CRC32 = 0x0008;

            /// Auth method none. If not set, auth method CHAP.
            const AUTH_METHOD_NONE = 0x0800;

            /// CHAP UNI. If not set, CHAP BI.
            const CHAP_UNI = 0x1000;
        }
    }

    /// NVM Express namespace messaging device path node.
    #[node(static_size = 16)]
    struct NvmeNamespace {
        /// Namespace identifier (NSID). The values 0 and 0xffff_ffff
        /// are invalid.
        namespace_identifier: u32,

        /// IEEE Extended Unique Identifier (EUI-64), or 0 if the device
        /// does not have a EUI-64.
        ieee_extended_unique_identifier: u64,
    }

    /// Uniform Resource Identifier (URI) messaging device path node.
    #[node(static_size = 4)]
    struct Uri {
        /// URI as defined by [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986).
        value: [u8],
    }

    /// Universal Flash Storage (UFS) messaging device path node.
    #[node(static_size = 6)]
    struct Ufs {
        /// Target ID on the UFS interface (PUN).
        target_id: u8,

        /// Logical Unit Number (LUN).
        logical_unit_number: u8,
    }

    /// Secure Digital (SD) messaging device path node.
    #[node(static_size = 5)]
    struct Sd {
        /// Slot number.
        slot_number: u8,
    }

    /// Bluetooth messaging device path node.
    #[node(static_size = 10)]
    struct Bluetooth {
        /// 48-bit bluetooth device address.
        device_address: [u8; 6],
    }

    /// Wi-Fi messaging device path node.
    #[node(static_size = 36)]
    struct Wifi {
        /// Service set identifier (SSID).
        ssid: [u8; 32],
    }

    /// Embedded Multi-Media Card (eMMC) messaging device path node.
    #[node(static_size = 5)]
    struct Emmc {
        /// Slot number.
        slot_number: u8,
    }

    /// BluetoothLE messaging device path node.
    #[node(static_size = 11)]
    struct BluetoothLe {
        /// 48-bit bluetooth device address.
        device_address: [u8; 6],

        /// Address type.
        address_type: crate::proto::device_path::messaging::BluetoothLeAddressType,
    }

    newtype_enum! {
        /// BluetoothLE address type.
        pub enum BluetoothLeAddressType: u8 => {
            /// Public device address.
            PUBLIC = 0x00,

            /// Random device address.
            RANDOM = 0x01,
        }
    }

    /// DNS messaging device path node.
    #[node(static_size = 5)]
    struct Dns {
        /// Whether the addresses are IPv4 or IPv6.
        address_type: crate::proto::device_path::messaging::DnsAddressType,

        /// One or more instances of the DNS server address.
        addresses: [IpAddress],
    }

    newtype_enum! {
        /// Whether the address is IPv4 or IPv6.
        pub enum DnsAddressType: u8 => {
            /// DNS server address is IPv4.
            IPV4 = 0x00,

            /// DNS server address is IPv6.
            IPV6 = 0x01,
        }
    }

    /// NVDIMM namespace messaging device path node.
    #[node(static_size = 20)]
    struct NvdimmNamespace {
        /// Namespace unique label identifier.
        uuid: [u8; 16],
    }

    // The `RestService` node is a bit weird. The specification defines
    // it as a fixed-size 6-byte node, but then also specifies a
    // variable-length vendor-specific variation with the same type and
    // subtype. Since we want a single type to represent both
    // variations, add the vendor-specific guid and data as a single
    // `[u8]` field. The custom getters and builders provide a
    // convenient API around this.

    /// REST service messaging device path node.
    #[node(static_size = 6)]
    struct RestService {
        /// Type of REST service.
        service_type: crate::proto::device_path::messaging::RestServiceType,

        /// Whether the service is in-band or out-of-band.
        access_mode: crate::proto::device_path::messaging::RestServiceAccessMode,

        /// Vendor-specific data. Only used if the service type is [`VENDOR`].
        ///
        /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
        #[node(
            no_get_func,
            custom_build_impl,
            custom_build_size_impl,
            build_type = "Option<RestServiceVendorData<'a>>"
        )]
        vendor_guid_and_data: [u8],
    }

    impl RestService {
        /// Get the vendor GUID and vendor data. Only used if the
        /// service type is [`VENDOR`], otherwise returns None.
        ///
        /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
        #[must_use]
        pub fn vendor_guid_and_data(&self) -> Option<(Guid, &[u8])> {
            if self.service_type == RestServiceType::VENDOR
                && self.vendor_guid_and_data.len() >= size_of::<Guid>()
            {
                let (guid, data) = self.vendor_guid_and_data.split_at(size_of::<Guid>());
                // OK to unwrap, we just verified the length.
                let guid: [u8; 16] = guid.try_into().unwrap();
                Some((Guid::from_bytes(guid), data))
            } else {
                None
            }
        }
    }

    newtype_enum! {
        /// Type of REST service.
        pub enum RestServiceType: u8 => {
            /// Redfish REST service.
            REDFISH = 0x01,

            /// OData REST service.
            ODATA = 0x02,

            /// Vendor-specific REST service.
            VENDOR = 0xff,
        }
    }

    newtype_enum! {
        /// Whether the service is in-band or out-of-band.
        pub enum RestServiceAccessMode: u8 => {
            /// In-band REST service.
            IN_BAND = 0x01,

            /// Out-of-band REST service.
            OUT_OF_BAND = 0x02,
        }
    }

    /// Vendor-specific REST service data. Only used for service type [`VENDOR`].
    ///
    /// [`VENDOR`]: uefi::proto::device_path::messaging::RestServiceType
    #[build]
    #[derive(Debug)]
    pub struct RestServiceVendorData<'a> {
        /// Vendor GUID.
        pub vendor_guid: Guid,

        /// Vendor-defined data.
        pub vendor_defined_data: &'a [u8],
    }

    #[build]
    impl<'a> RestService<'a> {
        fn build_size_vendor_guid_and_data(&self) -> usize {
            if let Some(src) = &self.vendor_guid_and_data {
                assert!(
                    self.service_type
                        == crate::proto::device_path::messaging::RestServiceType::VENDOR
                );

                size_of::<Guid>() + size_of_val(src.vendor_defined_data)
            } else {
                0
            }
        }

        fn build_vendor_guid_and_data(&self, out: &mut [MaybeUninit<u8>]) {
            if let Some(src) = &self.vendor_guid_and_data {
                assert!(
                    self.service_type
                        == crate::proto::device_path::messaging::RestServiceType::VENDOR
                );

                let (guid_out, data_out) = out.split_at_mut(size_of::<Guid>());

                let guid_out_ptr: *mut Guid = maybe_uninit_slice_as_mut_ptr(guid_out).cast();
                unsafe {
                    guid_out_ptr.write_unaligned(src.vendor_guid);
                }

                let data_out_ptr = maybe_uninit_slice_as_mut_ptr(data_out);
                unsafe {
                    src.vendor_defined_data
                        .as_ptr()
                        .copy_to_nonoverlapping(data_out_ptr, data_out.len());
                }
            }
        }
    }

    /// NVME over Fabric (NVMe-oF) namespace messaging device path node.
    // The spec says 20, but it seems to be wrong.
    #[node(static_size = 21)]
    struct NvmeOfNamespace {
        /// Namespace Identifier Type (NIDT).
        nidt: u8,

        /// Namespace Identifier (NID).
        nid: [u8; 16],

        /// Unique identifier of an NVM subsystem stored as a
        /// null-terminated UTF-8 string. Maximum length of 224 bytes.
        subsystem_nqn: [u8],
    }
}

mod media {
    /// Hard drive media device path node.
    #[node(static_size = 42)]
    struct HardDrive {
        /// Index of the partition, starting from 1.
        partition_number: u32,

        /// Starting LBA (logical block address) of the partition.
        partition_start: u64,

        /// Size of the partition in blocks.
        partition_size: u64,

        /// Partition signature.
        #[node(
            no_get_func,
            custom_build_impl,
            build_type = "crate::proto::device_path::media::PartitionSignature"
        )]
        partition_signature: [u8; 16],

        /// Partition format.
        partition_format: crate::proto::device_path::media::PartitionFormat,

        #[node(no_get_func, custom_build_impl, build_type = false)]
        signature_type: u8,
    }

    impl HardDrive {
        /// Signature unique to this partition.
        #[must_use]
        pub fn partition_signature(&self) -> PartitionSignature {
            match self.signature_type {
                0 => PartitionSignature::None,
                1 => PartitionSignature::Mbr([
                    self.partition_signature[0],
                    self.partition_signature[1],
                    self.partition_signature[2],
                    self.partition_signature[3],
                ]),
                2 => PartitionSignature::Guid(Guid::from_bytes(self.partition_signature)),
                unknown => PartitionSignature::Unknown {
                    signature_type: unknown,
                    signature: self.partition_signature,
                },
            }
        }
    }

    #[build]
    impl HardDrive {
        fn build_partition_signature(&self) -> [u8; 16] {
            use crate::proto::device_path::media::PartitionSignature::*;
            match self.partition_signature {
                None => [0u8; 16],
                Mbr(mbr) => {
                    let mut sig = [0u8; 16];
                    sig[0..4].copy_from_slice(&mbr);
                    sig
                }
                Guid(guid) => guid.to_bytes(),
                Unknown { signature, .. } => signature,
            }
        }

        fn build_signature_type(&self) -> u8 {
            use crate::proto::device_path::media::PartitionSignature::*;
            match self.partition_signature {
                None => 0,
                Mbr(_) => 1,
                Guid(_) => 2,
                Unknown { signature_type, .. } => signature_type,
            }
        }
    }

    /// Hard drive partition signature.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum PartitionSignature {
        /// No disk signature.
        None,

        /// 32-bit MBR partition signature.
        Mbr([u8; 4]),

        /// 128-bit GUID partition signature.
        Guid(Guid),

        /// Unknown signature type not listed in the UEFI Specification.
        Unknown {
            /// Signature type.
            signature_type: u8,

            /// Signature data.
            signature: [u8; 16],
        },
    }

    newtype_enum! {
        /// Hard drive partition format.
        pub enum PartitionFormat: u8 => {
            /// MBR (PC-AT compatible Master Boot Record) format.
            MBR = 0x01,
            /// GPT (GUID Partition Table) format.
            GPT = 0x02,
        }
    }

    /// CD-ROM media device path node.
    #[node(static_size = 24)]
    struct CdRom {
        /// Boot entry number from the boot catalog, or 0 for the
        /// default entry.
        boot_entry: u32,

        /// Starting RBA (Relative logical Block Address).
        partition_start: u64,

        /// Size of the partition in blocks.
        partition_size: u64,
    }

    /// Vendor-defined media device path node.
    #[node(static_size = 20)]
    struct Vendor {
        /// Vendor-assigned GUID that defines the data that follows.
        vendor_guid: Guid,

        /// Vendor-defined data.
        vendor_defined_data: [u8],
    }

    /// File path media device path node.
    #[node(static_size = 4)]
    struct FilePath {
        /// Null-terminated path.
        #[node(build_type = "&'a CStr16")]
        path_name: [u16],
    }

    /// Media protocol media device path node.
    #[node(static_size = 20)]
    struct Protocol {
        /// The ID of the protocol.
        protocol_guid: Guid,
    }

    /// PIWG firmware file media device path node.
    #[node(static_size = 4)]
    struct PiwgFirmwareFile {
        /// Contents are defined in the UEFI PI Specification.
        data: [u8],
    }

    /// PIWG firmware volume media device path node.
    #[node(static_size = 4)]
    struct PiwgFirmwareVolume {
        /// Contents are defined in the UEFI PI Specification.
        data: [u8],
    }

    /// Relative offset range media device path node.
    #[node(static_size = 24)]
    struct RelativeOffsetRange {
        _reserved: u32,

        /// Offset of the first byte, relative to the parent device node.
        starting_offset: u64,

        /// Offset of the last byte, relative to the parent device node.
        ending_offset: u64,
    }

    /// RAM disk media device path node.
    #[node(static_size = 38)]
    struct RamDisk {
        /// Starting memory address.
        starting_address: u64,

        /// Ending memory address.
        ending_address: u64,

        /// Type of RAM disk.
        disk_type: crate::proto::device_path::media::RamDiskType,

        /// RAM disk instance number if supported, otherwise 0.
        disk_instance: u16,
    }

    newtype_enum! {
        /// RAM disk type.
        pub enum RamDiskType: Guid => {
            /// RAM disk with a raw disk format in volatile memory.
            VIRTUAL_DISK = guid!("77ab535a-45fc-624b-5560-f7b281d1f96e"),

            /// RAM disk of an ISO image in volatile memory.
            VIRTUAL_CD = guid!("3d5abd30-4175-87ce-6d64-d2ade523c4bb"),

            /// RAM disk with a raw disk format in persistent memory.
            PERSISTENT_VIRTUAL_DISK = guid!("5cea02c9-4d07-69d3-269f-4496fbe096f9"),

            /// RAM disk of an ISO image in persistent memory.
            PERSISTENT_VIRTUAL_CD = guid!("08018188-42cd-bb48-100f-5387d53ded3d"),
        }
    }
}

mod bios_boot_spec {
    /// BIOS Boot Specification device path node.
    #[node(static_size = 8, sub_type = "BIOS_BOOT_SPECIFICATION")]
    struct BootSpecification {
        /// Device type as defined by the BIOS Boot Specification.
        device_type: u16,

        /// Status flags as defined by the BIOS Boot Specification.
        status_flag: u16,

        /// Description of the boot device encoded as a null-terminated
        /// ASCII string.
        description_string: [u8],
    }
}
