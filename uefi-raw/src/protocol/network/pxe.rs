// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Boolean, Char8, IpAddress, MacAddress};
use bitflags::bitflags;
use core::fmt::{self, Debug, Formatter};

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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeDhcpV4Packet {
    pub bootp_opcode: u8,
    pub bootp_hw_type: u8,
    pub bootp_hw_addr_len: u8,
    pub bootp_gate_hops: u8,
    pub bootp_ident: u32,
    pub bootp_seconds: u16,
    pub bootp_flags: u16,
    pub bootp_ci_addr: [u8; 4],
    pub bootp_yi_addr: [u8; 4],
    pub bootp_si_addr: [u8; 4],
    pub bootp_gi_addr: [u8; 4],
    pub bootp_hw_addr: [u8; 16],
    pub bootp_srv_name: [u8; 64],
    pub bootp_boot_file: [u8; 128],
    pub dhcp_magik: u32,
    pub dhcp_options: [u8; 56],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeDhcpV6Packet {
    pub message_type: u8,
    pub transaction_id: [u8; 3],
    pub dhcp_options: [u8; 1024],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIpFilter {
    pub filters: PxeBaseCodeIpFilterFlags,
    pub ip_cnt: u8,
    pub reserved: u16,
    pub ip_list: [IpAddress; 8],
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct PxeBaseCodeIpFilterFlags: u8 {
        const STATION_IP = 0x01;
        const BROADCAST = 0x02;
        const PROMISCUOUS = 0x04;
        const PROMISCUOUS_MULTICAST = 0x08;
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeArpEntry {
    pub ip_addr: IpAddress,
    pub mac_addr: MacAddress,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeRouteEntry {
    pub ip_addr: IpAddress,
    pub subnet_mask: IpAddress,
    pub gw_addr: IpAddress,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIcmpError {
    pub ty: u8,
    pub code: u8,
    pub checksum: u16,
    pub u: PxeBaseCodeIcmpErrorUnion,
    pub data: [u8; 494],
}

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

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeTftpError {
    pub error_code: u8,
    pub error_string: [Char8; 127],
}
