//! Simple Network Protocol (SNP).

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};

/// The SimpleNetwork protocol provides a packet level interface to network adapters.
#[repr(C)]
#[unsafe_guid("a19832b9-ac25-11d3-9a2d-0090273fc14d")]
#[derive(Protocol)]
pub struct SimpleNetwork {
    revision: u64,
    start: extern "efiapi" fn(this: &SimpleNetwork) -> Status,
    stop: extern "efiapi" fn(this: &SimpleNetwork) -> Status,
    initialize: extern "efiapi" fn(
        this: &SimpleNetwork,
        extra_rx_buffer_size: usize,
        extra_tx_buffer_size: usize,
    ) -> Status,
    reset: extern "efiapi" fn(this: &SimpleNetwork, extended_verification: bool) -> Status,
    shutdown: extern "efiapi" fn(this: &SimpleNetwork) -> Status,
    receive_filters: extern "efiapi" fn(
        this: &SimpleNetwork,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: usize,
        mcast_filter: *const MacAddress,
    ) -> Status,
    station_address: extern "efiapi" fn(this: &SimpleNetwork, reset: bool, new: *const MacAddress) -> Status,
    statistics: extern "efiapi" fn(
        this: &SimpleNetwork,
        reset: bool,
        statistics_size: *mut usize,
        statistics_table: *mut SimpleNetworkStatistics,
    ) -> Status,
    mcast_ip_to_mac: extern "efiapi" fn(this: &SimpleNetwork, ipv6: bool, ip: *const IpAddress, mac: *mut MacAddress) -> Status,
    nv_data: extern "efiapi" fn(
        this: &SimpleNetwork,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut [u8],
    ) -> Status,
    get_status:
        extern "efiapi" fn(this: &SimpleNetwork, interrupt_status: *mut u32, tx_buf: *mut *mut u8) -> Status,
    transmit: extern "efiapi" fn(
        this: &SimpleNetwork,
        header_size: usize,
        buffer_size: usize,
        buffer: *const core::ffi::c_void,
        src_addr: *const MacAddress,
        dest_addr: *const MacAddress,
        protocol: *const u16,
    ) -> Status,
    receive: extern "efiapi" fn(
        this: &SimpleNetwork,
        header_size: *const usize,
        buffer_size: *mut usize,
        buffer: *mut core::ffi::c_void,
        src_addr: *mut MacAddress,
        dest_addr: *mut MacAddress,
        protocol: *mut u16,
    ) -> Status,
    wait_for_packet: usize,
    mode: *const SimpleNetworkMode,
}

impl SimpleNetwork {
    /// Changes the state of a network interface from “stopped” to “started”.
    ///
    /// # Errors
    /// * `uefi::Status::ALREADY_STARTED`  The network interface is already in the started state.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::DEVICE_ERROR`  This function is not supported by the network interface.
    pub fn start(&self) -> Result {
        (self.start)(self).into()
    }

    /// Changes the state of a network interface from “started” to “stopped”.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::DEVICE_ERROR`  This function is not supported by the network interface.
    pub fn stop(&self) -> Result {
        (self.stop)(self).into()
    }

    /// Resets a network adapter and allocates the transmit and receive buffers.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
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

    /// Manages the multicast receive filters of a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::INVALID_PARAMETER`  A parameter was invalid.
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  This function is not supported by the network interface.
    pub fn receive_filters(
        &self,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_cnt: usize,
        mcast_filter: *const MacAddress,
    ) -> Result {
        (self.receive_filters)(self, enable, disable, reset_mcast_filter, mcast_filter_cnt, mcast_filter).into()
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
        statistics_table: *mut SimpleNetworkStatistics,
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
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    pub fn get_status(
        &self,
        interrupt_status: *mut u32,
        tx_buf: *mut *mut u8,
    ) -> Result {
        (self.get_status)(self, interrupt_status, tx_buf).into()
    }

    /// Places a packet in the transmit queue of a network interface.
    ///
    /// # Errors
    /// * `uefi::Status::NOT_STARTED`  The network interface has not been started.
    /// * `uefi::Status::EFI_OUT_OF_RESOURCES`  There was not enough memory for the transmit and receive buffers
    /// * `uefi::Status::DEVICE_ERROR`  The command could not be sent to the network interface.
    /// * `uefi::Status::UNSUPPORTED`  The increased buffer size feature is not supported.
    pub fn transmit(
        &self,
        header_size: usize,
        buffer_size: usize,
        buffer: *const [u8],
        src_addr: *const MacAddress,
        dest_addr: *const MacAddress,
        protocol: *const u16,
    ) -> Result {
        (self.transmit)(
            self,
            header_size,
            buffer_size,
            buffer as *const _ as *const core::ffi::c_void,
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
            buffer as *mut _ as *mut core::ffi::c_void,
            src_addr,
            dest_addr,
            protocol,
        )
        .into()
    }

    /// Pointer for network mode.
    pub fn mode(&self) -> &SimpleNetworkMode {
        unsafe { &*self.mode }
    }
}

newtype_enum! {
    /// EFI_SIMPLE_NETWORK_STATE
    pub enum SimpleNetworkState: u32 => {
        NETWORK_STOPPED = 0x00,
        NETWORK_STARTED = 0x01,
        NETWORK_INITIALIZED = 0x02,
        NETWORK_MAX_STATE = 0x03,
    }
}

/// EFI_NETWORK_STATISTICS
#[repr(C)]
#[derive(Debug)]
pub struct SimpleNetworkStatistics {
    pub rx_total_frames: u64,
    pub rx_good_frames: u64,
    pub rx_undersize_frames: u64,
    pub rx_oversize_frames: u64,
    pub rx_dropped_frames: u64,
    pub rx_unicast_frames: u64,
    pub rx_broadcast_frames: u64,
    pub rx_multicast_frames: u64,
    pub rx_crc_error_frames: u64,
    pub rx_total_bytes: u64,
    pub tx_total_frames: u64,
    pub tx_good_frames: u64,
    pub tx_undersize_frames: u64,
    pub tx_oversize_frames: u64,
    pub tx_dropped_frames: u64,
    pub tx_unicast_frames: u64,
    pub tx_broadcast_frames: u64,
    pub tx_multicast_frames: u64,
    pub tx_crc_error_frames: u64,
    pub tx_total_bytes: u64,
    pub collisions: u64,
    pub unsupported_protocol: u64,
    pub rx_duplicated_frames: u64,
    pub rx_decrypt_error_frames: u64,
    pub tx_error_frames: u64,
    pub tx_retry_frames: u64,
}

impl SimpleNetworkStatistics {

    // Create a blank `SimpleNetworkStatistics`.
    pub fn new() -> SimpleNetworkStatistics {
        SimpleNetworkStatistics {
            rx_total_frames: 0,
            rx_good_frames: 0,
            rx_undersize_frames: 0,
            rx_oversize_frames: 0,
            rx_dropped_frames: 0,
            rx_unicast_frames: 0,
            rx_broadcast_frames: 0,
            rx_multicast_frames: 0,
            rx_crc_error_frames: 0,
            rx_total_bytes: 0,
            tx_total_frames: 0,
            tx_good_frames: 0,
            tx_undersize_frames: 0,
            tx_oversize_frames: 0,
            tx_dropped_frames: 0,
            tx_unicast_frames: 0,
            tx_broadcast_frames: 0,
            tx_multicast_frames: 0,
            tx_crc_error_frames: 0,
            tx_total_bytes: 0,
            collisions: 0,
            unsupported_protocol: 0,
            rx_duplicated_frames: 0,
            rx_decrypt_error_frames: 0,
            tx_error_frames: 0,
            tx_retry_frames: 0,
        }
    }
}

/// EFI_MAC_ADDRESS
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MacAddress {
    pub addr: [u8; 32],
}

impl MacAddress {

    // Create a `MacAddress` from the given 6 bytes.
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
    pub addr: [u8; 4],
}

/// EFI_SIMPLE_NETWORK_MODE
#[repr(C)]
#[derive(Debug)]
pub struct SimpleNetworkMode {

    /// The current state of the network interface.
    pub state: SimpleNetworkState,

    /// The size, in bytes, of the network interface’s HW address.
    pub hw_address_size: u32,

    /// The size, in bytes, of the network interface’s media header.
    pub media_header_size: u32,

    /// The maximum size, in bytes, of the packets supported by the network interface.
    pub max_packet_size: u32,

    /// The size, in bytes, of the NVRAM device attached to the network interface.
    pub nv_ram_size: u32,

    /// The size that must be used for all NVRAM reads and writes.
    pub nv_ram_access_size: u32,

    /// The multicast receive filter settings supported by the network interface.
    pub receive_filter_mask: u32,

    /// The current multicast receive filter settings.
    pub receive_filter_setting: u32,

    /// The maximum number of multicast address receive filters supported by the driver.
    pub max_mcast_filter_count: u32,

    /// The current number of multicast address receive filters.
    pub mcast_filter_count: u32,

    /// Array containing the addresses of the current multicast address receive filters.
    pub mcast_filter: [MacAddress; 16],

    /// The current HW MAC address for the network interface.
    pub current_address: MacAddress,

    /// The current HW MAC address for broadcast packets.
    pub broadcast_address: MacAddress,

    /// The permanent HW MAC address for the network interface.
    pub permanent_address: MacAddress,

    /// The interface type of the network interface. See RFC 3232, section "Number Hardware Type."
    pub if_type: u8,

    /// Whether the HW MAC address can be changed.
    pub mac_address_changeable: bool,

    /// Whether the network interface can transmit more than one packet at a time.
    pub multiple_tx_supported: bool,

    /// Whether the presence of media can be determined.
    pub media_present_supported: bool,

    /// Whether media are connected to the network interface.
    pub media_present: bool,
}
