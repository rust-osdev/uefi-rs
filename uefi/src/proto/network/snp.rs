//! Simple Network Protocol

use core::ffi::c_void;
use crate::Status;
use crate::data_types::Event;
use super::{IpAddress, MacAddress};

/// The Simple Network Protocol
#[repr(C)]
#[unsafe_guid("a19832b9-ac25-11d3-9a2d-0090273fc14d")]
#[derive(Protocol)]
pub struct SimpleNetwork {
    revision: u64,
    start: extern "efiapi" fn(this: &Self) -> Status,
    stop: extern "efiapi" fn(this: &Self) -> Status,
    initialize: extern "efiapi" fn(
        this: &Self,
        extra_recv_buffer_size: Option<usize>,
        extra_transmit_buffer_size: Option<usize>
    ) -> Status,
    reset: extern "efiapi" fn(this: &Self, extended_verification: bool) -> Status,
    shutdown: extern "efiapi" fn(this: &Self) -> Status,
    receive_filters: extern "efiapi" fn(
        this: &Self,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: Option<usize>,
        mcast_filter: Option<*const [MacAddress]>
    ) -> Status,
    station_address: extern "efiapi" fn(this: &Self, reset: bool, new: Option<MacAddress>) -> Status,
    statistics: extern "efiapi" fn(
        this: &Self,
        reset: bool,
        stats_size: Option<&mut usize>,
        stats_table: Option<&mut NetworkStats>
    ) -> Status,
    mcast_ip_to_mac: extern "efiapi" fn(
        this: &Self,
        ipv6: bool,
        ip: &IpAdddress,
        mac: &mut MacAddress
    ) -> Status,
    nv_data: extern "efiapi" fn(
        this: &Self,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut c_void
    ) -> Status,
    get_status: extern "efiapi" fn(
        this: &Self,
        interrupt_status: Option<&mut u32>,
        tx_buf: Option<&mut *mut c_void>
    ) -> Status,
    transmit: extern "efiapi" fn(
        this: &Self,
        header_size: usize,
        buffer_size: usize,
        buffer: *mut c_void,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>
    ) -> Status,
    receive: extern "efiapi" fn(
        this: &Self,
        header_size: Option<&mut usize>,
        buffer_size: &mut usize,
        buffer: *mut c_void,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>
    ) -> Status,
    wait_for_packet: Event,
    mode: *const NetworkMode,
}

/// Network Statistics
///
/// The description of statistics on the network with the SNP's `statistics` function
/// is returned in this structure
#[repr(C)]
pub struct NetworkStats {
    total_frames_rx: u64,
    good_frames_rx: u64,
    undersize_frames_rx: u64,
    oversize_frames_rx: u64,
    dropped_frames_rx: u64,
    unicast_frames_rx: u64,
    broadcast_frames_rx: u64,
    multicast_frames_rx: u64,
    crc_error_frames_rx: u64,
    total_bytes_rx: u64,
    total_frames_tx: u64,
    good_frames_tx: u64,
    undersize_frames_tx: u64,
    oversize_frames_tx: u64,
    dropped_frames_tx: u64,
    unicast_frames_tx: u64,
    broadcast_frames_tx: u64,
    multicast_frames_tx: u64,
    crc_error_frames_tx: u64,
    total_bytes_tx: u64,
    collisions: u64,
    unsupported_protocol: u64,
    duplicated_frames_rx: u64,
    decrypt_error_frames_rx: u64,
    error_frames_tx: u64,
    retry_frames_tx: u64
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkMode {
    state: u32,
    hw_address_size: u32,
    media_header_size: u32,
    max_packet_size: u32,
    nv_ram_size: u32,
    nv_ram_access_size: u32,
    receive_filter_mask: u32,
    receive_filter_setting: u32,
    max_mcast_filter_count: u32,
    mcast_filter_count: u32,
    mcast_filter: [MacAddress; 16],
    current_address: MacAddress,
    broadcast_address: MacAddress,
    permanent_address: MacAddress,
    if_type: u8,
    mac_address_changeable: bool,
    multiple_tx_supported: bool,
    media_present_supported: bool,
    media_present: bool
}