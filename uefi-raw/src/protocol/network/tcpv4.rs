// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Event, Guid, Ipv4Address, Status, guid};
use core::{
    ffi::c_void,
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ptr::NonNull,
};

pub type UnmodelledPtr = NonNull<c_void>;

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4AccessPoint {
    pub use_default_address: bool,
    pub station_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub station_port: u16,
    pub remote_address: Ipv4Address,
    pub remote_port: u16,
    pub active_flag: bool,
}

impl Tcpv4AccessPoint {
    pub fn new(connection_mode: Tcpv4ConnectionMode) -> Tcpv4AccessPoint {
        let (remote_ip, remote_port, is_client) = match connection_mode {
            Tcpv4ConnectionMode::Client(params) => (params.remote_ip, params.remote_port, true),
            Tcpv4ConnectionMode::Server => (Ipv4Address([0, 0, 0, 0]), 0, false),
        };
        Self {
            use_default_address: true,
            // These two fields are meaningless because we set use_default_address above
            station_address: Ipv4Address([0, 0, 0, 0]),
            subnet_mask: Ipv4Address([0, 0, 0, 0]),
            // Chosen on-demand
            station_port: 0,
            remote_address: remote_ip,
            remote_port,
            active_flag: is_client,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4Option {
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_syn_back_log: u32,
    pub connection_timeout: u32,
    pub data_retries: u32,
    pub fin_timeout: u32,
    pub time_wait_timeout: u32,
    pub keep_alive_probes: u32,
    pub keep_alive_time: u32,
    pub keep_alive_interval: u32,
    pub enable_nagle: bool,
    pub enable_time_stamp: bool,
    pub enable_window_scaling: bool,
    pub enable_selective_ack: bool,
    pub enable_path_mtu_discovery: bool,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4ConfigData<'a> {
    pub type_of_service: u8,
    pub time_to_live: u8,
    pub access_point: Tcpv4AccessPoint,
    pub option: Option<&'a Tcpv4Option>,
}

#[derive(Debug)]
pub struct Tcpv4ClientConnectionModeParams {
    pub remote_ip: Ipv4Address,
    pub remote_port: u16,
}

impl Tcpv4ClientConnectionModeParams {
    pub fn new(remote_ip: Ipv4Address, remote_port: u16) -> Tcpv4ClientConnectionModeParams {
        Self {
            remote_ip,
            remote_port,
        }
    }
}

#[derive(Debug)]
pub enum Tcpv4ConnectionMode {
    Client(Tcpv4ClientConnectionModeParams),
    // TODO(PT): There may be parameters we need to model when operating as a server
    Server,
}

impl<'a> Tcpv4ConfigData<'a> {
    pub fn new(
        connection_mode: Tcpv4ConnectionMode,
        options: Option<&'a Tcpv4Option>,
    ) -> Tcpv4ConfigData<'a> {
        Tcpv4ConfigData {
            type_of_service: 0,
            time_to_live: 255,
            access_point: Tcpv4AccessPoint::new(connection_mode),
            option: options,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4IoToken<'a> {
    pub completion_token: Tcpv4CompletionToken,
    pub packet: Tcpv4Packet<'a>,
}

#[repr(C)]
pub union Tcpv4Packet<'a> {
    pub rx_data: Option<&'a Tcpv4ReceiveData<'a>>,
    pub tx_data: Option<&'a Tcpv4TransmitData<'a>>,
}

impl Debug for Tcpv4Packet<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tcpv4Packet").finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4CompletionToken {
    pub event: Event,
    pub status: Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4FragmentData<'a> {
    pub fragment_length: u32,
    pub fragment_buf: *const c_void,
    pub _pd: PhantomData<&'a mut [u8]>,
}

impl<'a> Tcpv4FragmentData<'a> {
    pub fn with_buf(buf: &'a [u8]) -> Tcpv4FragmentData<'a> {
        Self {
            fragment_length: buf.len() as u32,
            fragment_buf: buf.as_ptr() as *const c_void,
            _pd: PhantomData,
        }
    }

    pub fn with_mut_buf(buf: &'a mut [u8]) -> Tcpv4FragmentData<'a> {
        Self {
            fragment_length: buf.len() as u32,
            fragment_buf: buf.as_ptr() as *const c_void,
            _pd: PhantomData,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum Tcpv4ConnectionState {
    Closed = 0,
    Listen = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    Closing = 7,
    TimeWait = 8,
    CloseWait = 9,
    LastAck = 10,
}

/// Current IPv4 configuration data used by the TCPv4 instance.
#[derive(Debug)]
#[repr(C)]
pub struct Ipv4ModeData<'a> {
    pub is_started: bool,
    pub max_packet_size: u32,
    pub config_data: Ipv4ConfigData,
    pub is_configured: bool,
    pub group_count: bool,
    pub group_table: &'a [Ipv4Address; 0],
    pub route_count: u32,
    pub ip4_route_table: &'a [Ipv4RouteTable; 0],
    pub icmp_type_count: u32,
    pub icmp_type_list: &'a [Ipv4IcmpType; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Ipv4ConfigData {
    pub default_protocol: u8,
    pub accept_any_protocol: bool,
    pub accept_icmp_errors: bool,
    pub accept_broadcast: bool,
    pub accept_promiscuous: bool,
    pub use_default_address: bool,
    pub station_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub type_of_service: u8,
    pub time_to_live: u8,
    pub do_not_fragment: bool,
    pub raw_data: bool,
    pub receive_timeout: u32,
    pub transmit_timeout: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ipv4RouteTable {
    pub subnet_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway_address: Ipv4Address,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ipv4IcmpType {
    pub type_: u8,
    pub code: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4Protocol {
    // TODO: should this &mut? It's plain *This in the spec but
    //       HttpProtocol uses `*const Self` for `get_mode_data`.
    #[allow(clippy::type_complexity)]
    pub get_mode_data: extern "efiapi" fn(
        this: &Self,
        out_connection_state: Option<&mut Tcpv4ConnectionState>,
        out_config_data: Option<UnmodelledPtr>,
        out_ip4_mode_data: Option<&mut Ipv4ModeData>,
        out_managed_network_config_data: Option<UnmodelledPtr>,
        out_simple_network_mode: Option<UnmodelledPtr>,
    ) -> Status,
    pub configure: extern "efiapi" fn(&mut Self, config_data: Option<&Tcpv4ConfigData>) -> Status,
    pub routes: extern "efiapi" fn(
        &mut Self,
        delete_route: bool,
        subnet_address: &Ipv4Address,
        subnet_mask: &Ipv4Address,
        gateway_address: &Ipv4Address,
    ) -> Status,
    pub connect: extern "efiapi" fn(&mut Self, connection_token: &Tcpv4CompletionToken) -> Status,
    pub accept: extern "efiapi" fn(&mut Self, listen_token: UnmodelledPtr) -> Status,
    pub transmit: extern "efiapi" fn(&mut Self, token: &Tcpv4IoToken) -> Status,
    pub receive: extern "efiapi" fn(&mut Self, token: &Tcpv4IoToken) -> Status,
    pub close: extern "efiapi" fn(&mut Self, close_token: UnmodelledPtr) -> Status,
    pub cancel: extern "efiapi" fn(&mut Self, completion_token: UnmodelledPtr) -> Status,
    pub poll: extern "efiapi" fn(&mut Self) -> Status,
}

impl Tcpv4Protocol {
    pub const GUID: Guid = guid!("65530BC7-A359-410F-B010-5AADC7EC2B62");
    pub const SERVICE_BINDING_GUID: Guid = guid!("00720665-67EB-4a99-BAF7-D3C33A1C7CC9");
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4ReceiveData<'a> {
    pub urgent: bool,
    pub data_length: u32,
    pub fragment_count: u32,
    pub fragment_table: [Tcpv4FragmentData<'a>; 1],
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcpv4TransmitData<'a> {
    pub push: bool,
    pub urgent: bool,
    pub data_length: u32,
    pub fragment_count: u32,
    pub fragment_table: [Tcpv4FragmentData<'a>; 1],
}
