// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Boolean, Char8, Guid, IpAddress, MacAddress, Status, guid, newtype_enum};
use bitflags::bitflags;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
#[repr(C)]
pub struct PxeBaseCodeProtocol {
    pub revision: u64,
    pub start: unsafe extern "efiapi" fn(this: *mut Self, use_ipv6: Boolean) -> Status,
    pub stop: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub dhcp: unsafe extern "efiapi" fn(this: *mut Self, sort_offers: Boolean) -> Status,
    pub discover: unsafe extern "efiapi" fn(
        this: *mut Self,
        ty: PxeBaseCodeBootType,
        layer: *mut u16,
        use_bis: Boolean,
        info: *const PxeBaseCodeDiscoverInfo,
    ) -> Status,
    pub mtftp: unsafe extern "efiapi" fn(
        this: *mut Self,
        operation: PxeBaseCodeTftpOpcode,
        buffer: *mut c_void,
        overwrite: Boolean,
        buffer_size: *mut u64,
        block_size: *const usize,
        server_ip: *const IpAddress,
        filename: *const Char8,
        info: *const PxeBaseCodeMtftpInfo,
        dont_use_buffer: Boolean,
    ) -> Status,
    pub udp_write: unsafe extern "efiapi" fn(
        this: *mut Self,
        op_flags: PxeBaseCodeUdpOpFlags,
        dest_ip: *const IpAddress,
        dest_port: *const PxeBaseCodeUdpPort,
        gateway_ip: *const IpAddress,
        src_ip: *const IpAddress,
        src_port: *mut PxeBaseCodeUdpPort,
        header_size: *const usize,
        header_ptr: *const c_void,
        buffer_size: *const usize,
        buffer_ptr: *const c_void,
    ) -> Status,
    pub udp_read: unsafe extern "efiapi" fn(
        this: *mut Self,
        op_flags: PxeBaseCodeUdpOpFlags,
        dest_ip: *mut IpAddress,
        dest_port: *mut PxeBaseCodeUdpPort,
        src_ip: *mut IpAddress,
        src_port: *mut PxeBaseCodeUdpPort,
        header_size: *const usize,
        header_ptr: *mut c_void,
        buffer_size: *mut usize,
        buffer_ptr: *mut c_void,
    ) -> Status,
    pub set_ip_filter: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_filter: *const PxeBaseCodeIpFilter,
    ) -> Status,
    pub arp: unsafe extern "efiapi" fn(
        this: *mut Self,
        ip_addr: *const IpAddress,
        mac_addr: *mut MacAddress,
    ) -> Status,
    pub set_parameters: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_auto_arp: *const Boolean,
        new_send_guid: *const Boolean,
        new_ttl: *const u8,
        new_tos: *const u8,
        new_make_callback: *const Boolean,
    ) -> Status,
    pub set_station_ip: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_station_ip: *const IpAddress,
        new_subnet_mask: *const IpAddress,
    ) -> Status,
    pub set_packets: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_dhcp_discover_valid: *const Boolean,
        new_dhcp_ack_received: *const Boolean,
        new_proxy_offer_received: *const Boolean,
        new_pxe_discover_valid: *const Boolean,
        new_pxe_reply_received: *const Boolean,
        new_pxe_bis_reply_received: *const Boolean,
        new_dhcp_discover: *const PxeBaseCodePacket,
        new_dhcp_ack: *const PxeBaseCodePacket,
        new_proxy_offer: *const PxeBaseCodePacket,
        new_pxe_discover: *const PxeBaseCodePacket,
        new_pxe_reply: *const PxeBaseCodePacket,
        new_pxe_bis_reply: *const PxeBaseCodePacket,
    ) -> Status,
    pub mode: *const PxeBaseCodeMode,
}

impl PxeBaseCodeProtocol {
    pub const GUID: Guid = guid!("03c4e603-ac28-11d3-9a2d-0090273fc14d");
}

newtype_enum! {
    pub enum PxeBaseCodeBootType: u16 => {
        BOOTSTRAP = 0,
        MS_WINNT_RIS = 1,
        INTEL_LCM = 2,
        DOS_UNDI = 3,
        NEC_ESMPRO = 4,
        IBM_WSOD = 5,
        IBM_LCCM = 6,
        CA_UNICENTER_TNG = 7,
        HP_OPENVIEW = 8,
        ALTIRIS_9 = 9,
        ALTIRIS_10 = 10,
        ALTIRIS_11 = 11,
        NOT_USED_12 = 12,
        REDHAT_INSTALL = 13,
        REDHAT_BOOT = 14,
        REMBO = 15,
        BEOBOOT = 16,
        //    17..=32767: reserved.
        // 32768..=65279: reserved for vendor use.
        // 65280..=65534: reserved.
        PXETEST = 65535,
    }
}

newtype_enum! {
    pub enum PxeBaseCodeTftpOpcode: i32 => {
        TFTP_FIRST = 0,
        TFTP_GET_FILE_SIZE = 1,
        TFTP_READ_FILE = 2,
        TFTP_WRITE_FILE = 3,
        TFTP_READ_DIRECTORY = 4,
        MTFTP_GET_FILE_SIZE = 5,
        MTFTP_READ_FILE = 6,
        MTFTP_READ_DIRECTORY = 7,
        MTFTP_LAST = 8,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PxeBaseCodeDiscoverInfo {
    pub use_m_cast: Boolean,
    pub use_b_cast: Boolean,
    pub use_u_cast: Boolean,
    pub must_use_list: Boolean,
    pub server_m_cast_ip: IpAddress,
    pub ip_cnt: u16,

    /// Note that this field is actually a variable-length array.
    pub srv_list: [PxeBaseCodeSrvlist; 0],
}

/// An entry in the boot server list.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_SRVLIST` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeSrvlist {
    pub server_type: u16,
    pub accept_any_response: Boolean,
    pub reserved: u8,
    pub ip_addr: IpAddress,
}

impl PxeBaseCodeSrvlist {
    /// Construct a [`PxeBaseCodeSrvlist`] for a boot server reply type. If `ip_addr` is not `None`,
    /// only boot server replies matching the provided IP address will be accepted.
    #[must_use]
    pub fn new(server_type: u16, ip_addr: Option<IpAddress>) -> Self {
        Self {
            server_type,
            accept_any_response: Boolean::from(ip_addr.is_none()),
            reserved: 0,
            ip_addr: ip_addr.unwrap_or_default(),
        }
    }

    /// Returns `None` if any response should be accepted, or otherwise the IP
    /// address of a boot server whose responses should be accepted.
    #[must_use]
    pub fn ip_addr(&self) -> Option<&IpAddress> {
        if self.accept_any_response.into() {
            None
        } else {
            Some(&self.ip_addr)
        }
    }
}

pub type PxeBaseCodeUdpPort = u16;

/// MTFTP connection parameters.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_MTFTP_INFO` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeMtftpInfo {
    /// We need a low level type and a high-level type with `IpAddr`
    /// File multicast IP address. This is the IP address to which the server
    /// will send the requested file.
    pub m_cast_ip: IpAddress,
    /// Client multicast listening port. This is the UDP port to which the
    /// server will send the requested file.
    pub c_port: PxeBaseCodeUdpPort,
    /// Server multicast listening port. This is the UDP port on which the
    /// server listens for multicast open requests and data acks.
    pub s_port: PxeBaseCodeUdpPort,
    /// The number of seconds a client should listen for an active multicast
    /// session before requesting a new multicast session.
    pub listen_timeout: u16,
    /// The number of seconds a client should wait for a packet from the server
    /// before retransmitting the previous open request or data ack packet.
    pub transmit_timeout: u16,
}

bitflags! {
    /// Flags for UDP read and write operations.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct PxeBaseCodeUdpOpFlags: u16 {
        /// Receive a packet sent from any IP address in UDP read operations.
        const ANY_SRC_IP = 0x0001;

        /// Receive a packet sent from any UDP port in UDP read operations. If
        /// the source port is not specified in UDP write operations, the
        /// source port will be automatically selected.
        const ANY_SRC_PORT = 0x0002;

        /// Receive a packet sent to any IP address in UDP read operations.
        const ANY_DEST_IP = 0x0004;

        /// Receive a packet sent to any UDP port in UDP read operations.
        const ANY_DEST_PORT = 0x0008;

        /// The software filter is used in UDP read operations.
        const USE_FILTER = 0x0010;

        /// If required, a UDP write operation may be broken up across multiple packets.
        const MAY_FRAGMENT = 0x0020;
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeMode {
    pub started: Boolean,
    pub ipv6_available: Boolean,
    pub ipv6_supported: Boolean,
    pub using_ipv6: Boolean,
    pub bis_supported: Boolean,
    pub bis_detected: Boolean,
    pub auto_arp: Boolean,
    pub send_guid: Boolean,
    pub dhcp_discover_valid: Boolean,
    pub dhcp_ack_received: Boolean,
    pub proxy_offer_received: Boolean,
    pub pxe_discover_valid: Boolean,
    pub pxe_reply_received: Boolean,
    pub pxe_bis_reply_received: Boolean,
    pub icmp_error_received: Boolean,
    pub tftp_error_received: Boolean,
    pub make_callbacks: Boolean,
    pub ttl: u8,
    pub tos: u8,
    pub station_ip: IpAddress,
    pub subnet_mask: IpAddress,
    pub dhcp_discover: PxeBaseCodePacket,
    pub dhcp_ack: PxeBaseCodePacket,
    pub proxy_offer: PxeBaseCodePacket,
    pub pxe_discover: PxeBaseCodePacket,
    pub pxe_reply: PxeBaseCodePacket,
    pub pxe_bis_reply: PxeBaseCodePacket,
    pub ip_filter: PxeBaseCodeIpFilter,
    pub arp_cache_entries: u32,
    pub arp_cache: [PxeBaseCodeArpEntry; 8],
    pub route_table_entries: u32,
    pub route_table: [PxeBaseCodeRouteEntry; 8],
    pub icmp_error: PxeBaseCodeIcmpError,
    pub tftp_error: PxeBaseCodeTftpError,
}

/// A network packet.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_PACKET` type.
#[derive(Clone, Copy)]
#[repr(C)]
pub union PxeBaseCodePacket {
    pub raw: [u8; 1472],
    pub dhcpv4: PxeBaseCodeDhcpV4Packet,
    pub dhcpv6: PxeBaseCodeDhcpV6Packet,
}

impl Debug for PxeBaseCodePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PxeBaseCodePacket").finish()
    }
}

impl AsRef<[u8; 1472]> for PxeBaseCodePacket {
    fn as_ref(&self) -> &[u8; 1472] {
        // SAFETY: The packet union is defined with `raw` as the byte view for
        // the full storage, and any byte pattern is valid here.
        unsafe { &self.raw }
    }
}

impl AsRef<PxeBaseCodeDhcpV4Packet> for PxeBaseCodePacket {
    fn as_ref(&self) -> &PxeBaseCodeDhcpV4Packet {
        // SAFETY: The caller chooses this view; the union stores the packet
        // bytes in a layout compatible with the DHCPv4 packet type.
        unsafe { &self.dhcpv4 }
    }
}

impl AsRef<PxeBaseCodeDhcpV6Packet> for PxeBaseCodePacket {
    fn as_ref(&self) -> &PxeBaseCodeDhcpV6Packet {
        // SAFETY: The caller chooses this view; the union stores the packet
        // bytes in a layout compatible with the DHCPv6 packet type.
        unsafe { &self.dhcpv6 }
    }
}

/// A DHCPv4 packet.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_DHCPV4_PACKET` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeDhcpV4Packet {
    /// Packet op code / message type.
    pub bootp_opcode: u8,
    /// Hardware address type.
    pub bootp_hw_type: u8,
    /// Hardware address length.
    pub bootp_hw_addr_len: u8,
    /// Client sets to zero, optionally used by gateways in cross-gateway booting.
    pub bootp_gate_hops: u8,
    pub bootp_ident: u32,
    pub bootp_seconds: u16,
    pub bootp_flags: u16,
    /// Client IP address, filled in by client in bootrequest if known.
    pub bootp_ci_addr: [u8; 4],
    /// 'your' (client) IP address; filled by server if client doesn't know its own address (`bootp_ci_addr` was 0).
    pub bootp_yi_addr: [u8; 4],
    /// Server IP address, returned in bootreply by server.
    pub bootp_si_addr: [u8; 4],
    /// Gateway IP address, used in optional cross-gateway booting.
    pub bootp_gi_addr: [u8; 4],
    /// Client hardware address, filled in by client.
    pub bootp_hw_addr: [u8; 16],
    /// Optional server host name, null terminated string.
    pub bootp_srv_name: [u8; 64],
    /// Boot file name, null terminated string, 'generic' name or null in
    /// bootrequest, fully qualified directory-path name in bootreply.
    pub bootp_boot_file: [u8; 128],
    /// Validation magic number.
    pub dhcp_magik: u32,
    /// Optional vendor-specific area, e.g. could be hardware type/serial on request, or 'capability' / remote file system handle on reply.  This info may be set aside for use by a third phase bootstrap or kernel.
    pub dhcp_options: [u8; 56],
}

impl PxeBaseCodeDhcpV4Packet {
    /// The expected value for [`Self::dhcp_magik`].
    pub const DHCP_MAGIK: u32 = 0x63825363;

    /// Transaction ID, a random number, used to match this boot request with the responses it generates.
    #[must_use]
    pub const fn bootp_ident(&self) -> u32 {
        u32::from_be(self.bootp_ident)
    }

    /// Filled in by client, seconds elapsed since client started trying to boot.
    #[must_use]
    pub const fn bootp_seconds(&self) -> u16 {
        u16::from_be(self.bootp_seconds)
    }

    /// The flags.
    #[must_use]
    pub const fn bootp_flags(&self) -> PxeBaseCodeDhcpV4Flags {
        PxeBaseCodeDhcpV4Flags::from_bits_truncate(u16::from_be(self.bootp_flags))
    }

    /// A magic cookie, should be [`Self::DHCP_MAGIK`].
    #[must_use]
    pub const fn dhcp_magik(&self) -> u32 {
        u32::from_be(self.dhcp_magik)
    }
}

bitflags! {
    /// Represents the 'flags' field for a [`PxeBaseCodeDhcpV4Packet`].
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PxeBaseCodeDhcpV4Flags: u16 {
        /// Should be set when the client cannot receive unicast IP datagrams
        /// until its protocol software has been configured with an IP address.
        const BROADCAST = 1;
    }
}

/// A DHCPv6 packet.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_DHCPV6_PACKET` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeDhcpV6Packet {
    /// The message type.
    pub message_type: u8,
    /// The transaction id.
    pub transaction_id: [u8; 3],
    /// A byte array containing DHCP options.
    pub dhcp_options: [u8; 1024],
}

impl PxeBaseCodeDhcpV6Packet {
    /// The transaction id.
    #[must_use]
    pub fn transaction_id(&self) -> u32 {
        (u32::from(self.transaction_id[0]) << 16)
            | (u32::from(self.transaction_id[1]) << 8)
            | u32::from(self.transaction_id[2])
    }
}

/// IP receive filter settings.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_IP_FILTER` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIpFilter {
    pub filters: PxeBaseCodeIpFilterFlags,
    pub ip_cnt: u8,
    pub reserved: u16,
    pub ip_list: [IpAddress; 8],
}

impl PxeBaseCodeIpFilter {
    #[must_use]
    pub fn new(filters: PxeBaseCodeIpFilterFlags, ip_list: &[core::net::IpAddr]) -> Self {
        assert!(ip_list.len() <= 8);

        let ip_cnt = ip_list.len() as u8;
        let mut buffer = [IpAddress::default(); 8];
        for (index, ip_address) in ip_list
            .iter()
            .cloned()
            .map(|value| value.into())
            .enumerate()
        {
            buffer[index] = ip_address;
        }

        Self {
            filters,
            ip_cnt,
            reserved: 0,
            ip_list: buffer,
        }
    }

    /// A list of IP addresses other than the station IP that should be enabled.
    ///
    /// May be multicast or unicast.
    #[must_use]
    pub fn ip_list(&self) -> &[IpAddress] {
        &self.ip_list[..usize::from(self.ip_cnt)]
    }
}

bitflags! {
    /// IP receive filters.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct PxeBaseCodeIpFilterFlags: u8 {
        /// Enable the Station IP address.
        const STATION_IP = 0x01;

        /// Enable IPv4 broadcast addresses.
        const BROADCAST = 0x02;

        /// Enable all addresses.
        const PROMISCUOUS = 0x04;

        /// Enable all multicast addresses.
        const PROMISCUOUS_MULTICAST = 0x08;
    }
}

/// An entry for the ARP cache found in [`PxeBaseCodeMode::arp_cache`].
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_ARP_ENTRY` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeArpEntry {
    pub ip_addr: IpAddress,
    pub mac_addr: MacAddress,
}

/// An entry for the route table found in [`PxeBaseCodeMode::route_table`].
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_ROUTE_ENTRY` type.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeRouteEntry {
    pub ip_addr: IpAddress,
    pub subnet_mask: IpAddress,
    pub gw_addr: IpAddress,
}

/// An ICMP error packet.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_ICMP_ERROR` type.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIcmpError {
    pub ty: u8,
    pub code: u8,
    pub checksum: u16,
    pub u: PxeBaseCodeIcmpErrorUnion,
    pub data: [u8; 494],
}

impl Display for PxeBaseCodeIcmpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for PxeBaseCodeIcmpError {}

/// In the C API, this is an anonymous union inside the definition of
/// `EFI_PXE_BASE_CODE_ICMP_ERROR`.
#[derive(Clone, Copy)]
#[repr(C)]
pub union PxeBaseCodeIcmpErrorUnion {
    pub reserved: u32,
    pub mtu: u32,
    pub pointer: u32,
    pub echo: PxeBaseCodeIcmpErrorEcho,
}

impl Debug for PxeBaseCodeIcmpErrorUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PxeBaseCodeIcmpErrorUnion").finish()
    }
}

/// In the C API, this is an anonymous struct inside the definition of
/// `EFI_PXE_BASE_CODE_ICMP_ERROR`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIcmpErrorEcho {
    pub identifier: u16,
    pub sequence: u16,
}

/// A TFTP error packet.
///
/// In the C API, this corresponds to the `EFI_PXE_BASE_CODE_TFTP_ERROR` type.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeTftpError {
    pub error_code: u8,
    pub error_string: [Char8; 127],
}

impl Display for PxeBaseCodeTftpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for PxeBaseCodeTftpError {}
