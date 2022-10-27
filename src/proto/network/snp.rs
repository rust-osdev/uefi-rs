//! Simple Network Protocol
//!
//! Provides a packet level interface to a network adapter.
//! Once the adapter is initialized, the protocol provides services that allows
//! packets to be transmitted and received.
//!
//! No interface function must be called until `SimpleNetwork.start` is successfully
//! called first.

use core::ffi::c_void;
use core::ptr;
use uefi_macros::{unsafe_guid, Protocol};
use crate::{Status, Result};
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
    station_address: extern "efiapi" fn(this: &Self, reset: bool, new: Option<&MacAddress>) -> Status,
    statistics: extern "efiapi" fn(
        this: &Self,
        reset: bool,
        stats_size: Option<&mut usize>,
        stats_table: Option<&mut NetworkStats>
    ) -> Status,
    mcast_ip_to_mac: extern "efiapi" fn(
        this: &Self,
        ipv6: bool,
        ip: &IpAddress,
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
        interrupt_status: Option<&mut InterruptStatus>,
        tx_buf: Option<&mut *mut c_void>
    ) -> Status,
    transmit: extern "efiapi" fn(
        this: &Self,
        header_size: usize,
        buffer_size: usize,
        buffer: *mut c_void,
        src_addr: Option<&MacAddress>,
        dest_addr: Option<&MacAddress>,
        protocol: Option<&u16>
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

impl SimpleNetwork {
    /// Changes the state of a network from "Stopped" to "Started"
    pub fn start(&self) -> Result {
        (self.start)(self).into()
    }

    /// Changes the state of a network interface from "Started" to "Stopped"
    pub fn stop(&self) -> Result {
        (self.stop)(self).into()
    }

    /// Resets a network adapter and allocates the transmit and receive buffers
    /// required by the network interface; optionally, also requests allocation of
    /// additional transmit and receive buffers
    pub fn initialize(
        &self,
        extra_rx_buffer_size: Option<usize>,
        extra_tx_buffer_size: Option<usize>
    ) -> Result {
        (self.initialize)(self, extra_rx_buffer_size, extra_tx_buffer_size).into()
    }

    /// Resets a network adapter and reinitializes it with the parameters that were
    /// provided in the previous call to `initialize`
    pub fn reset(&self, extended_verification: bool) -> Result {
        (self.reset)(self, extended_verification).into()
    }

    /// Resets a network adapter and leaves it in a state that is safe
    /// for another driver to initialize
    pub fn shutdown(&self) -> Result {
        (self.shutdown)(self).into()
    }

    /// Manages the multicast receive filters of a network
    pub fn receive_filters(
        &self,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: Option<usize>,
        mcast_filter: Option<*const [MacAddress]>
    ) -> Result {
        (self.receive_filters)(
            self,
            enable,
            disable,
            reset_mcast_filter,
            mcast_filter_count,
            mcast_filter
        ).into()
    }

    /// Modifies or resets the current station address, if supported
    pub fn station_address(&self, reset: bool, new: Option<&MacAddress>) -> Result {
        (self.station_address)(
            self,
            reset,
            new
        ).into()
    }

    /// Resets statistics on a network interface
    pub fn reset_statistics(&self) -> Result {
        (self.statistics)(self, true, None, None).into()
    }

    /// Collects statistics on a network interface
    pub fn collect_statistics(&self) -> Result<NetworkStats> {
        let mut stats_table: NetworkStats = Default::default();
        let mut stats_size = core::mem::size_of::<NetworkStats>();
        let status = (self.statistics)(self, false, Some(&mut stats_size), Some(&mut stats_table));
        Result::from(status)?;
        Ok(stats_table)
    }

    /// Converts a multicast IP address to a multicast HW MAC Address
    pub fn mcast_ip_to_mac(&self, ipv6: bool, ip: IpAddress) -> Result<MacAddress> {
        let mut mac_address = MacAddress([0; 32]);
        let status = (self.mcast_ip_to_mac)(self, ipv6, &ip, &mut mac_address);
        Result::from(status)?;
        Ok(mac_address)
    }

    /// Performs read operations on the NVRAM device attached to
    /// a network interface
    pub fn read_nv_data(&self, offset: usize, buffer_size: usize, buffer: *mut c_void) -> Result {
        (self.nv_data)(
            self,
            true,
            offset,
            buffer_size,
            buffer
        ).into()
    }

    /// Performs write operations on the NVRAM device attached to a network interface
    pub fn write_nv_data(&self, offset: usize, buffer_size: usize, buffer: *mut c_void) -> Result {
        (self.nv_data)(
            self,
            false,
            offset,
            buffer_size,
            buffer
        ).into()
    }

    /// Reads the current interrupt status and recycled transmit buffer
    /// status from a network interface
    pub fn get_interrupt_status(&self) -> Result<InterruptStatus> {
        let mut interrupt_status = InterruptStatus::new();
        let status = (self.get_status)(self, Some(&mut interrupt_status), None);
        Result::from(status)?;
        Ok(interrupt_status)
    }

    /// Reads the current recycled transmit buffer status from a
    /// network interface
    pub fn get_recycled_transmit_buffer_status(&self) -> Result<Option<*mut u8>> {
        let mut tx_buf: *mut c_void = ptr::null_mut();
        let status = (self.get_status)(self, None, Some(&mut tx_buf));
        Result::from(status)?;
        if tx_buf == ptr::null_mut() {
            Ok(None)
        } else {
            Ok(Some(tx_buf.cast()))
        }
    }

    /// Places a packet in the transmit queue of a network interface
    pub fn transmit(
        &self,
        header_size: usize,
        buffer: &mut [u8],
        src_addr: Option<&MacAddress>,
        dest_addr: Option<&MacAddress>,
        protocol: Option<&u16>
    ) -> Result {
        (self.transmit)(
            self,
            header_size,
            buffer.len(),
            buffer.as_mut_ptr().cast(),
            src_addr,
            dest_addr,
            protocol
        ).into()
    }

    /// Receives a packet from a network interface
    ///
    /// On success, returns the size of bytes of the received packet
    pub fn receive(
        &self,
        buffer: &mut [u8],
        header_size: Option<&mut usize>,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>
    ) -> Result<usize> {
        let mut buffer_size = buffer.len();
        let status = (self.receive)(
            self,
            header_size,
            &mut buffer_size,
            buffer.as_mut_ptr().cast(),
            src_addr,
            dest_addr,
            protocol
        );
        Result::from(status)?;
        Ok(buffer_size)
    }

    /// Returns a reference to the Simple Network mode
    pub fn mode(&self) -> &NetworkMode {
        unsafe { &*self.mode }
    }
}

/// A bitmask of currently active interrupts
#[repr(transparent)]
pub struct InterruptStatus(u32);

impl InterruptStatus {
    /// Creates a new InterruptStatus instance with all bits unset
    pub fn new() -> Self {
        Self(0)
    }
    /// The receive interrupt bit
    pub fn receive_interrupt(&self) -> bool {
        self.0 & 0x01 == 1
    }
    /// The transmit interrupt bit
    pub fn transmit_interrupt(&self) -> bool {
        self.0 & 0x02 == 0x02
    }
    /// The command interrupt bit
    pub fn command_interrupt(&self) -> bool {
        self.0 & 0x04 == 0x04
    }
    /// The software interrupt bit
    pub fn software_interrupt(&self) -> bool {
        self.0 & 0x08 == 0x08
    }
}

/// Network Statistics
///
/// The description of statistics on the network with the SNP's `statistics` function
/// is returned in this structure
#[repr(C)]
#[derive(Default)]
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

/// The Simple Network Mode
#[repr(C)]
pub struct NetworkMode {
    state: NetworkState,
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

newtype_enum! {
    /// The state of a network interface
    pub enum NetworkState: u32 => {
        /// The interface has been stopped
        STOPPED = 0,
        /// The interface has been started
        STARTED = 1,
        /// The interface has been initialized
        INITIALIZED = 2,
        /// No state can have a number higher than this
        MAX_STATE = 4,
    }
}