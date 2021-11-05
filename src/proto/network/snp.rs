//! Network I/O protocols.

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};

/// The Snp protocol.
#[repr(C)]
#[unsafe_guid("a19832b9-ac25-11d3-9a2d-0090273fc14d")]
#[derive(Protocol)]
pub struct Snp {
    pub revision: u64,
    start: extern "efiapi" fn(this: &Snp) -> Status,
    stop: extern "efiapi" fn(this: &Snp) -> Status,
    initialize: extern "efiapi" fn(
        this: &Snp,
        extra_rx_buffer_size: usize,
        extra_tx_buffer_size: usize,
    ) -> Status,
    reset: extern "efiapi" fn(this: &Snp, extended_verification: bool) -> Status,
    shutdown: extern "efiapi" fn(this: &Snp) -> Status,
    receive_filters: extern "efiapi" fn(
        this: &Snp,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: usize,
        mcast_filter: *const MacAddress,
    ) -> Status,
    station_address: extern "efiapi" fn(this: &Snp, reset: bool, new: *const MacAddress) -> Status,
    statistics: extern "efiapi" fn(
        this: &Snp,
        reset: bool,
        statistics_size: *mut usize,
        statistics_table: *mut NetworkStatistics,
    ) -> Status,
    mcast_ip_to_mac: extern "efiapi" fn(this: &Snp, ipv6: bool, ip: *const IpAddress, mac: *mut MacAddress) -> Status,
    nv_data: extern "efiapi" fn(
        this: &Snp,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut [u8],
    ) -> Status,
    get_status:
        extern "efiapi" fn(this: &Snp, interrupt_status: *mut u32, tx_buf: *mut *mut [u8]) -> Status,
    transmit: extern "efiapi" fn(
        this: &Snp,
        header_size: usize,
        buffer_size: usize,
        buffer: *const [u8],
        src_addr: *const MacAddress,
        dest_addr: *const MacAddress,
        protocol: u16,
    ) -> Status,
    receive: extern "efiapi" fn(
        this: &Snp,
        header_size: *const usize,
        buffer_size: *mut usize,
        buffer: *mut [u8],
        src_addr: *mut MacAddress,
        dest_addr: *mut MacAddress,
        protocol: *mut u16,
    ) -> Status,
    wait_for_packet: usize,
    mode: *const NetworkMode,
}

impl Snp {
    /// Changes the state of a network interface from “stopped” to “started”.
    ///
    /// # Errors
    /// * `uefi::Status::ALREADY_STARTED`  The network interface is already in the started state.
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::DEVICE_ERROR`  This function is not supported by the network interface.
    pub fn start(&self) -> Result {
        (self.start)(self).into()
    }

    /// Changes the state of a network interface from “started” to “stopped”.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::DEVICE_ERROR`  This function is not supported by the network interface.
    pub fn stop(&mut self) -> Result {
        (self.stop)(self).into()
    }

    /// Resets a network adapter and allocates the transmit and receive buffers.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  The increased buffer size feature is not supported.
    pub fn initialize(&self, extra_rx_buffer_size: usize, extra_tx_buffer_size: usize) -> Result {
        (self.initialize)(self, extra_rx_buffer_size, extra_tx_buffer_size).into()
    }

    /// Resets a network adapter and reinitializes it with the parameters that were provided in the previous call to initialize().
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn reset(&self, extended_verification: bool) -> Result {
        (self.reset)(self, extended_verification).into()
    }

    /// Resets a network adapter and leaves it in a state that is safe for another driver to initialize.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    pub fn shutdown(&self) -> Result {
        (self.shutdown)(self).into()
    }

    /// Modifies or resets the current station address, if supported.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn station_address(
        &self,
        reset: bool,
        new: *const MacAddress,
    ) -> Result {
        (self.station_address)(self, reset, new).into()
    }

    /// Resets or collects the statistics on a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn statistics(
        &self,
        reset: bool,
        statistics_size: *mut usize,
        statistics_table: *mut NetworkStatistics,
    ) -> Result {
        (self.statistics)(self, reset, statistics_size, statistics_table).into()
    }

    /// Converts a multicast IP address to a multicast HW MAC address.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn mcast_ip_to_mac(
        &self,
        ipv6: bool,
        ip: *const IpAddress,
        mac: *mut MacAddress,
    ) -> Result {
        (self.mcast_ip_to_mac)(self, ipv6, ip, mac).into()
    }

    /// Performs read and write operations on the NVRAM device attached to a network interface. 
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn nv_data(
        &self,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut [u8],
    ) -> Result {
        (self.nv_data)(self, read_write, offset, buffer_size, buffer).into()
    }

    /// Reads the current interrupt status and recycled transmit buffer status from a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    pub fn get_status(
        &self,
        interrupt_status: *mut u32,
        tx_buf: *mut *mut [u8],
    ) -> Result {
        (self.get_status)(self, interrupt_status, tx_buf).into()
    }

    /// Places a packet in the transmit queue of a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  The increased buffer size feature is not supported.
    pub fn transmit(
        &self,
        header_size: usize,
        buffer_size: usize,
        buffer: *const [u8],
        src_addr: *const MacAddress,
        dest_addr: *const MacAddress,
        protocol: u16,
    ) -> Result {
        (self.transmit)(
            self,
            header_size,
            buffer_size,
            buffer,
            src_addr,
            dest_addr,
            protocol,
        )
        .into()
    }

    /// Receives a packet from a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::NOT_READY`  No packets have been received on the network interface.
    /// * `uefi::Status::BUFFER_TOO_SMALL`  BufferSize is too small for the received packets. BufferSize has been updated to the required size.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    pub fn receive(
        &self,
        header_size: *mut usize,
        buffer_size: *mut usize,
        buffer: *mut [u8],
        src_addr: *mut MacAddress,
        dest_addr: *mut MacAddress,
        protocol: *mut u16,
    ) -> Result {
        (self.receive)(
            self,
            header_size,
            buffer_size,
            buffer,
            src_addr,
            dest_addr,
            protocol,
        )
        .into()
    }

    /// Pointer for network mode.
    pub fn mode(&self) -> &NetworkMode {
        unsafe { &*self.mode }
    }
}

newtype_enum! {
    /// EFI_SIMPLE_NETWORK_STATE
    pub enum NetworkState: u32 => {
        NETWORK_STOPPED = 0x00,
        NETWORK_STARTED = 0x01,
        NETWORK_INITIALIZED = 0x02,
        NETWORK_MAX_STATE = 0x03,
    }
}

/// EFI_NETWORK_STATISTICS
#[repr(C)]
#[derive(Debug)]
pub struct NetworkStatistics {
    rx_total_frames: u64,
    rx_good_frames: u64,
    rx_undersize_frames: u64,
    rx_oversize_frames: u64,
    rx_dropped_frames: u64,
    rx_unicast_frames: u64,
    rx_broadcast_frames: u64,
    rx_multicast_frames: u64,
    rx_crc_error_frames: u64,
    rx_total_bytes: u64,
    tx_total_frames: u64,
    tx_good_frames: u64,
    tx_undersize_frames: u64,
    tx_oversize_frames: u64,
    tx_dropped_frames: u64,
    tx_unicast_frames: u64,
    tx_broadcast_frames: u64,
    tx_multicast_frames: u64,
    tx_crc_error_frames: u64,
    tx_total_bytes: u64,
    collisions: u64,
    unsupported_protocol: u64,
    rx_duplicated_frames: u64,
    rx_decrypt_error_frames: u64,
    tx_error_frames: u64,
    tx_retry_frames: u64,
}

/// EFI_MAC_ADDRESS
#[repr(C)]
#[derive(Debug)]
pub struct MacAddress {
    addr: [u8; 32],
}

impl MacAddress {
    pub fn new(mac: [u8; 6]) -> MacAddress {
        let mut data = [0u8; 32];
        for (a, b) in data.iter_mut().zip(mac.iter()) {
            *a = *b;
        }
        MacAddress{addr: data}
    }
}

/// EFI_IP_ADDRESS
#[repr(C)]
#[derive(Debug)]
pub struct IpAddress {
    addr: [u8; 4],
}

/// EFI_SIMPLE_NETWORK_MODE
#[repr(C)]
#[derive(Debug)]
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
    media_present: bool,
}

impl NetworkMode {

    /// Reports the current state of the network interface.
    pub fn state(&self) -> NetworkState {
        self.state
    }

    /// The size, in bytes, of the network interface’s HW address.
    pub fn hw_address_size(&self) -> u32 {
        self.hw_address_size
    }

    /// The size, in bytes, of the network interface’s media header.
    pub fn media_header_size(&self) -> u32 {
        self.media_header_size
    }

    /// The current HW MAC address for the network interface.
    pub fn current_address(&self) -> &MacAddress {
        &self.current_address
    }

    /// The interface type of the network interface. See RFC 3232, section "Number Hardware Type."
    pub fn if_type(&self) -> u8 {
        self.if_type
    }
}
