//! Network I/O protocols.

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};

/// The SNP protocol.
#[repr(C)]
#[unsafe_guid("a19832b9-ac25-11d3-9a2d-0090273fc14d")]
#[derive(Protocol)]
pub struct SNP {
    pub revision: u64,
    start           : extern "efiapi" fn(this: &SNP) -> Status,
    stop            : extern "efiapi" fn(this: &SNP) -> Status,
    initialize      : extern "efiapi" fn(this: &SNP, extra_rx_buffer_size: usize, extra_tx_buffer_size: usize) -> Status,
    reset           : extern "efiapi" fn(this: &SNP, extended_verification: bool) -> Status,
    shutdown        : extern "efiapi" fn(this: &SNP) -> Status,
    receive_filters : extern "efiapi" fn(this: &SNP, enable: u32, disable: u32, reset_mcast_filter: bool, mcast_filter_count: usize, mcast_filter: *const MacAddress) -> Status,
    station_address : extern "efiapi" fn(this: &SNP, reset: bool, new: *const MacAddress) -> Status,
    statistics      : extern "efiapi" fn(this: &SNP, reset: bool, statistics_size: *mut usize, statistics_table: *mut NetworkStatistics) -> Status,
    mcast_ip_to_mac : extern "efiapi" fn(this: &SNP, ipv6: bool, ip: *const IpAddress) -> Status,
    nv_data         : extern "efiapi" fn(this: &SNP, read_write: bool, offset: usize, buffer_size: usize, buffer: *mut u8) -> Status,
    get_status      : extern "efiapi" fn(this: &SNP, interrupt_status: *mut u32, tx_buf: *mut *mut u8) -> Status,
    transmit        : extern "efiapi" fn(this: &SNP, header_size: usize, buffer_size: usize, buffer: *const u8, src_addr: *const MacAddress, dest_addr: *const MacAddress, protocol: u16) -> Status,
    receive         : extern "efiapi" fn(this: &SNP, header_size: *const usize, buffer_size: *mut usize, buffer: *mut u8, src_addr: *mut MacAddress, dest_addr: *mut MacAddress, protocol: *mut u16) -> Status,
    wait_for_packet : usize,
    mode: *const NetworkMode,
}

impl SNP {

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

    /// Resets or collects the statistics on a network interface.
    ///
    /// # Errors
    pub fn statistics(&self, reset: bool, statistics_size: *mut usize, statistics_table: *mut NetworkStatistics) -> Result {
        (self.statistics)(self, reset, statistics_size, statistics_table).into()
    }

    /// Places a packet in the transmit queue of a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
    /// * `uefi::Status::INVALID_PARAMETER`  This parameter was NULL or did not point to a valid EFI_SIMPLE_NETWORK_PROTOCOL structure
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  The increased buffer size feature is not supported.
    pub fn transmit(&self, header_size: usize, buffer: &[u8], src_addr: *const MacAddress, dest_addr: *const MacAddress, protocol: u16) -> Result {
        (self.transmit)(self, header_size, buffer.len(), buffer.as_ptr(), src_addr, dest_addr, protocol).into()
    }

    /// Pointer for network mode.
    pub fn mode(&self) -> &NetworkMode {
        unsafe { &*self.mode }
    }
}

/// Network statistics structure
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
	pub addr: [u8; 32],
}

/// EFI_IP_ADDRESS
#[repr(C)]
#[derive(Debug)]
pub struct IpAddress {
	addr: [u8; 4],
}

/// Network mode structure
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
	media_present: bool,
}

impl NetworkMode {

	///
    pub fn current_address(&self) -> &MacAddress {
        &self.current_address
    }

	///
    pub fn if_type(&self) -> u8 {
        self.if_type
    }
}
