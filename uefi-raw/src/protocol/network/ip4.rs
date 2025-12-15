// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Boolean, Ipv4Address};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct Ip4RouteTable {
    pub subnet_addr: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway_addr: Ipv4Address,
}

/// Defined in [UEFI Specification, Section 28.3.5](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-ip4-protocol-getmodedata)
#[derive(Debug)]
#[repr(C)]
pub struct Ip4ModeData {
    // TODO Ipv4Protocol not yet in uefi-raw
    /// Set to [`Boolean::TRUE`] after an associated `Ipv4Protocol` instance
    /// has been successfully configured.
    pub is_started: Boolean,
    /// The maximum packet size, in bytes, of the packet which the
    /// upper layer driver could feed.
    pub max_packet_size: u32,
    /// Current configuration settings.
    pub config_data: Ip4ConfigData,
    // TODO Ipv4Protocol not yet in uefi-raw
    /// Set to [`Boolean::TRUE`] when an associated `Ipv4Protocol` instance
    /// has a station address and subnet mask.
    pub is_configured: Boolean,
    /// Number of joined multicast groups.
    pub group_count: u32,
    /// List of joined multicast group addresses.
    pub group_table: *const Ipv4Address,
    /// Number of entries in the routing table.
    pub route_count: u32,
    /// Routing table entries.
    pub route_table: *const Ip4RouteTable,
    /// Number of entries in the supported ICMP types list.
    pub icmp_type_count: u32,
    /// Array of ICMP types and codes that are supported.
    pub icmp_type_list: *const Ip4IcmpType,
}

/// Defined in [UEFI Specification, Section 28.3.5](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-ip4-protocol-getmodedata)
#[derive(Debug)]
#[repr(C)]
pub struct Ip4IcmpType {
    /// ICMP message type.
    pub type_: u8,
    /// ICMP message code.
    pub code: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4ConfigData {
    /// Default protocol to be used.
    ///
    /// See <https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml>.
    pub default_protocol: u8,
    /// Set to `TRUE` to receive all IPv4 packets.
    pub accept_any_protocol: Boolean,
    /// Set to `TRUE` to receive ICMP error packets.
    pub accept_icmp_errors: Boolean,
    /// Set to `TRUE` to receive broadcast IPv4 packets.
    pub accept_broadcast: Boolean,
    /// Set to `TRUE` to receive all IPv4 packets in promiscuous mode.
    pub accept_promiscuous: Boolean,
    /// Set to `TRUE` to use the default IPv4 address and routing
    /// table.
    pub use_default_address: Boolean,
    /// Station IPv4 address.
    pub station_address: Ipv4Address,
    /// Subnet mask for the station address.
    pub subnet_mask: Ipv4Address,
    /// Type of service field in transmitted IPv4 packets.
    pub type_of_service: u8,
    /// Time to live field in transmitted IPv4 packets.
    pub time_to_live: u8,
    /// Set to `TRUE` to disable fragmentation.
    pub do_not_fragment: Boolean,
    /// Set to `TRUE` to enable raw data mode.
    pub raw_data: Boolean,
    /// Receive timeout in milliseconds.
    pub receive_timeout: u32,
    /// Transmit timeout in milliseconds.
    pub transmit_timeout: u32,
}
