use crate::Ipv4Address;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct Ip4RouteTable {
    pub subnet_addr: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway_addr: Ipv4Address,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4ModeData<'a> {
    is_started: bool,
    max_packet_size: u32,
    config_data: Ip4ConfigData,
    is_configured: bool,
    group_count: bool,
    group_table: &'a [Ipv4Address; 0],
    route_count: u32,
    ip4_route_table: &'a [Ip4RouteTable; 0],
    icmp_type_count: u32,
    icmp_type_list: &'a [Ip4IcmpType; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4ConfigData {
    default_protocol: u8,
    accept_any_protocol: bool,
    accept_icmp_errors: bool,
    accept_broadcast: bool,
    accept_promiscuous: bool,
    use_default_address: bool,
    station_address: Ipv4Address,
    subnet_mask: Ipv4Address,
    type_of_service: u8,
    time_to_live: u8,
    do_not_fragment: bool,
    raw_data: bool,
    receive_timeout: u32,
    transmit_timeout: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4IcmpType {
    _type: u8,
    code: u8,
}
