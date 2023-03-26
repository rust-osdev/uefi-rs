//! Simple Network Protocol
//!
//! Provides a packet level interface to a network adapter.
//! Once the adapter is initialized, the protocol provides services that allows
//! packets to be transmitted and received.
//!
//! No interface function must be called until `SimpleNetwork.start` is successfully
//! called first.

use super::{IpAddress, MacAddress};
use crate::data_types::Event;
use crate::{Result, Status, StatusExt};
use bitflags::bitflags;
use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use uefi_macros::unsafe_protocol;

/// The Simple Network Protocol
#[repr(C)]
#[unsafe_protocol("a19832b9-ac25-11d3-9a2d-0090273fc14d")]
pub struct SimpleNetwork {
    revision: u64,
    start: extern "efiapi" fn(this: &Self) -> Status,
    stop: extern "efiapi" fn(this: &Self) -> Status,
    initialize: extern "efiapi" fn(
        this: &Self,
        extra_recv_buffer_size: usize,
        extra_transmit_buffer_size: usize,
    ) -> Status,
    reset: extern "efiapi" fn(this: &Self, extended_verification: bool) -> Status,
    shutdown: extern "efiapi" fn(this: &Self) -> Status,
    receive_filters: extern "efiapi" fn(
        this: &Self,
        enable: u32,
        disable: u32,
        reset_mcast_filter: bool,
        mcast_filter_count: usize,
        mcast_filter: Option<NonNull<MacAddress>>,
    ) -> Status,
    station_address:
        extern "efiapi" fn(this: &Self, reset: bool, new: Option<&MacAddress>) -> Status,
    statistics: extern "efiapi" fn(
        this: &Self,
        reset: bool,
        stats_size: Option<&mut usize>,
        stats_table: Option<&mut NetworkStats>,
    ) -> Status,
    mcast_ip_to_mac:
        extern "efiapi" fn(this: &Self, ipv6: bool, ip: &IpAddress, mac: &mut MacAddress) -> Status,
    nv_data: extern "efiapi" fn(
        this: &Self,
        read_write: bool,
        offset: usize,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    get_status: extern "efiapi" fn(
        this: &Self,
        interrupt_status: Option<&mut InterruptStatus>,
        tx_buf: Option<&mut *mut c_void>,
    ) -> Status,
    transmit: extern "efiapi" fn(
        this: &Self,
        header_size: usize,
        buffer_size: usize,
        buffer: *const c_void,
        src_addr: Option<&MacAddress>,
        dest_addr: Option<&MacAddress>,
        protocol: Option<&u16>,
    ) -> Status,
    receive: extern "efiapi" fn(
        this: &Self,
        header_size: Option<&mut usize>,
        buffer_size: &mut usize,
        buffer: *mut c_void,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>,
    ) -> Status,
    // On QEMU, this event seems to never fire.
    wait_for_packet: Event,
    mode: *const NetworkMode,
}

impl SimpleNetwork {
    /// Change the state of a network from "Stopped" to "Started".
    pub fn start(&self) -> Result {
        (self.start)(self).to_result()
    }

    /// Change the state of a network interface from "Started" to "Stopped".
    pub fn stop(&self) -> Result {
        (self.stop)(self).to_result()
    }

    /// Reset a network adapter and allocate the transmit and receive buffers
    /// required by the network interface; optionally, also request allocation of
    /// additional transmit and receive buffers.
    pub fn initialize(&self, extra_rx_buffer_size: usize, extra_tx_buffer_size: usize) -> Result {
        (self.initialize)(self, extra_rx_buffer_size, extra_tx_buffer_size).to_result()
    }

    /// Reset a network adapter and reinitialize it with the parameters that were
    /// provided in the previous call to `initialize`.
    pub fn reset(&self, extended_verification: bool) -> Result {
        (self.reset)(self, extended_verification).to_result()
    }

    /// Reset a network adapter, leaving it in a state that is safe
    /// for another driver to initialize
    pub fn shutdown(&self) -> Result {
        (self.shutdown)(self).to_result()
    }

    /// Manage the multicast receive filters of a network.
    pub fn receive_filters(
        &self,
        enable: ReceiveFlags,
        disable: ReceiveFlags,
        reset_mcast_filter: bool,
        mcast_filter: Option<&[MacAddress]>,
    ) -> Result {
        if let Some(mcast_filter) = mcast_filter {
            (self.receive_filters)(
                self,
                enable.bits(),
                disable.bits(),
                reset_mcast_filter,
                mcast_filter.len(),
                NonNull::new(mcast_filter.as_ptr() as *mut _),
            )
            .to_result()
        } else {
            (self.receive_filters)(
                self,
                enable.bits(),
                disable.bits(),
                reset_mcast_filter,
                0,
                None,
            )
            .to_result()
        }
    }

    /// Modify or reset the current station address, if supported.
    pub fn station_address(&self, reset: bool, new: Option<&MacAddress>) -> Result {
        (self.station_address)(self, reset, new).to_result()
    }

    /// Reset statistics on a network interface.
    pub fn reset_statistics(&self) -> Result {
        (self.statistics)(self, true, None, None).to_result()
    }

    /// Collect statistics on a network interface.
    pub fn collect_statistics(&self) -> Result<NetworkStats> {
        let mut stats_table: NetworkStats = Default::default();
        let mut stats_size = core::mem::size_of::<NetworkStats>();
        let status = (self.statistics)(self, false, Some(&mut stats_size), Some(&mut stats_table));
        status.to_result_with_val(|| stats_table)
    }

    /// Convert a multicast IP address to a multicast HW MAC Address.
    pub fn mcast_ip_to_mac(&self, ipv6: bool, ip: IpAddress) -> Result<MacAddress> {
        let mut mac_address = MacAddress([0; 32]);
        let status = (self.mcast_ip_to_mac)(self, ipv6, &ip, &mut mac_address);
        status.to_result_with_val(|| mac_address)
    }

    /// Perform read operations on the NVRAM device attached to
    /// a network interface.
    pub fn read_nv_data(&self, offset: usize, buffer: &[u8]) -> Result {
        (self.nv_data)(
            self,
            true,
            offset,
            buffer.len(),
            buffer.as_ptr() as *mut c_void,
        )
        .to_result()
    }

    /// Perform write operations on the NVRAM device attached to a network interface.
    pub fn write_nv_data(&self, offset: usize, buffer: &mut [u8]) -> Result {
        (self.nv_data)(
            self,
            false,
            offset,
            buffer.len(),
            buffer.as_mut_ptr().cast(),
        )
        .to_result()
    }

    /// Read the current interrupt status and recycled transmit buffer
    /// status from a network interface.
    pub fn get_interrupt_status(&self) -> Result<InterruptStatus> {
        let mut interrupt_status = InterruptStatus::empty();
        let status = (self.get_status)(self, Some(&mut interrupt_status), None);
        status.to_result_with_val(|| interrupt_status)
    }

    /// Read the current recycled transmit buffer status from a
    /// network interface.
    pub fn get_recycled_transmit_buffer_status(&self) -> Result<Option<NonNull<u8>>> {
        let mut tx_buf: *mut c_void = ptr::null_mut();
        let status = (self.get_status)(self, None, Some(&mut tx_buf));
        status.to_result_with_val(|| NonNull::new(tx_buf.cast()))
    }

    /// Place a packet in the transmit queue of a network interface.
    pub fn transmit(
        &self,
        header_size: usize,
        buffer: &[u8],
        src_addr: Option<MacAddress>,
        dest_addr: Option<MacAddress>,
        protocol: Option<u16>,
    ) -> Result {
        (self.transmit)(
            self,
            header_size,
            buffer.len() + header_size,
            buffer.as_ptr().cast(),
            src_addr.as_ref(),
            dest_addr.as_ref(),
            protocol.as_ref(),
        )
        .to_result()
    }

    /// Receive a packet from a network interface.
    ///
    /// On success, returns the size of bytes of the received packet.
    pub fn receive(
        &self,
        buffer: &mut [u8],
        header_size: Option<&mut usize>,
        src_addr: Option<&mut MacAddress>,
        dest_addr: Option<&mut MacAddress>,
        protocol: Option<&mut u16>,
    ) -> Result<usize> {
        let mut buffer_size = buffer.len();
        let status = (self.receive)(
            self,
            header_size,
            &mut buffer_size,
            buffer.as_mut_ptr().cast(),
            src_addr,
            dest_addr,
            protocol,
        );
        status.to_result_with_val(|| buffer_size)
    }

    /// Event that fires once a packet is available to be received.
    ///
    /// On QEMU, this event seems to never fire; it is suggested to verify that your implementation
    /// of UEFI properly implements this event before using it.
    #[must_use]
    pub fn wait_for_packet(&self) -> &Event {
        &self.wait_for_packet
    }

    /// Returns a reference to the Simple Network mode.
    #[must_use]
    pub fn mode(&self) -> &NetworkMode {
        unsafe { &*self.mode }
    }
}

bitflags! {
    /// Flags to pass to receive_filters to enable/disable reception of some kinds of packets.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ReceiveFlags : u32 {
        /// Receive unicast packets.
        const UNICAST = 0x01;
        /// Receive multicast packets.
        const MULTICAST = 0x02;
        /// Receive broadcast packets.
        const BROADCAST = 0x04;
        /// Receive packets in promiscuous mode.
        const PROMISCUOUS = 0x08;
        /// Receive packets in promiscuous multicast mode.
        const PROMISCUOUS_MULTICAST = 0x10;
    }
}

bitflags! {
    /// Flags returned by get_interrupt_status to indicate which interrupts have fired on the
    /// interface since the last call.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct InterruptStatus : u32 {
        /// Packet received.
        const RECEIVE = 0x01;
        /// Packet transmitted.
        const TRANSMIT = 0x02;
        /// Command interrupt fired.
        const COMMAND = 0x04;
        /// Software interrupt fired.
        const SOFTWARE = 0x08;
    }
}

/// Network Statistics
///
/// The description of statistics on the network with the SNP's `statistics` function
/// is returned in this structure
///
/// Any of these statistics may or may not be available on the device. So, all the
/// retriever functions of the statistics return `None` when a statistic is not supported
#[repr(C)]
#[derive(Default, Debug)]
pub struct NetworkStats {
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

impl NetworkStats {
    /// Any statistic value of -1 is not available
    fn available(&self, stat: u64) -> bool {
        stat as i64 != -1
    }

    /// Takes a statistic and converts it to an option
    ///
    /// When the statistic is not available, `None` is returned
    fn to_option(&self, stat: u64) -> Option<u64> {
        match self.available(stat) {
            true => Some(stat),
            false => None,
        }
    }

    /// The total number of frames received, including error frames
    /// and dropped frames
    #[must_use]
    pub fn rx_total_frames(&self) -> Option<u64> {
        self.to_option(self.rx_total_frames)
    }

    /// The total number of good frames received and copied
    /// into receive buffers
    #[must_use]
    pub fn rx_good_frames(&self) -> Option<u64> {
        self.to_option(self.rx_good_frames)
    }

    /// The number of frames below the minimum length for the
    /// communications device
    #[must_use]
    pub fn rx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_undersize_frames)
    }

    /// The number of frames longer than the maximum length for
    /// the communications length device
    #[must_use]
    pub fn rx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_oversize_frames)
    }

    /// The number of valid frames that were dropped because
    /// the receive buffers were full
    #[must_use]
    pub fn rx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.rx_dropped_frames)
    }

    /// The number of valid unicast frames received and not dropped
    #[must_use]
    pub fn rx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_unicast_frames)
    }

    /// The number of valid broadcast frames received and not dropped
    #[must_use]
    pub fn rx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_broadcast_frames)
    }

    /// The number of valid multicast frames received and not dropped
    #[must_use]
    pub fn rx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_multicast_frames)
    }

    /// Number of frames with CRC or alignment errors
    #[must_use]
    pub fn rx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_crc_error_frames)
    }

    /// The total number of bytes received including frames with errors
    /// and dropped frames
    #[must_use]
    pub fn rx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.rx_total_bytes)
    }

    /// The total number of frames transmitted including frames
    /// with errors and dropped frames
    #[must_use]
    pub fn tx_total_frames(&self) -> Option<u64> {
        self.to_option(self.tx_total_frames)
    }

    /// The total number of valid frames transmitted and copied
    /// into receive buffers
    #[must_use]
    pub fn tx_good_frames(&self) -> Option<u64> {
        self.to_option(self.tx_good_frames)
    }

    /// The number of frames below the minimum length for
    /// the media. This would be less than 64 for Ethernet
    #[must_use]
    pub fn tx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_undersize_frames)
    }

    /// The number of frames longer than the maximum length for
    /// the media. This would be 1500 for Ethernet
    #[must_use]
    pub fn tx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_oversize_frames)
    }

    /// The number of valid frames that were dropped because
    /// received buffers were full
    #[must_use]
    pub fn tx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.tx_dropped_frames)
    }

    /// The number of valid unicast frames transmitted and not
    /// dropped
    #[must_use]
    pub fn tx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_unicast_frames)
    }

    /// The number of valid broadcast frames transmitted and
    /// not dropped
    #[must_use]
    pub fn tx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_broadcast_frames)
    }

    /// The number of valid multicast frames transmitted
    /// and not dropped
    #[must_use]
    pub fn tx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_multicast_frames)
    }

    /// The number of transmitted frames with CRC or
    /// alignment errors
    #[must_use]
    pub fn tx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_crc_error_frames)
    }

    /// The total number of bytes transmitted including
    /// error frames and dropped frames
    #[must_use]
    pub fn tx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.tx_total_bytes)
    }

    /// The number of collisions detected on this subnet
    #[must_use]
    pub fn collisions(&self) -> Option<u64> {
        self.to_option(self.collisions)
    }

    /// The number of frames destined for unsupported protocol
    #[must_use]
    pub fn unsupported_protocol(&self) -> Option<u64> {
        self.to_option(self.unsupported_protocol)
    }

    /// The number of valid frames received that were duplicated
    #[must_use]
    pub fn rx_duplicated_frames(&self) -> Option<u64> {
        self.to_option(self.rx_duplicated_frames)
    }

    /// The number of encrypted frames received that failed
    /// to decrypt
    #[must_use]
    pub fn rx_decrypt_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_decrypt_error_frames)
    }

    /// The number of frames that failed to transmit after
    /// exceeding the retry limit
    #[must_use]
    pub fn tx_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_error_frames)
    }

    /// The number of frames that transmitted successfully
    /// after more than one attempt
    #[must_use]
    pub fn tx_retry_frames(&self) -> Option<u64> {
        self.to_option(self.tx_retry_frames)
    }
}

/// The Simple Network Mode
#[repr(C)]
#[derive(Debug)]
pub struct NetworkMode {
    /// Reports the current state of the network interface
    pub state: NetworkState,
    /// The size of the network interface's hardware address in bytes
    pub hw_address_size: u32,
    /// The size of the network interface's media header in bytes
    pub media_header_size: u32,
    /// The maximum size of the packets supported by the network interface in bytes
    pub max_packet_size: u32,
    /// The size of the NVRAM device attached to the network interface in bytes
    pub nv_ram_size: u32,
    /// The size that must be used for all NVRAM reads and writes
    pub nv_ram_access_size: u32,
    /// The multicast receive filter settings supported by the network interface
    pub receive_filter_mask: u32,
    /// The current multicast receive filter settings
    pub receive_filter_setting: u32,
    /// The maximum number of multicast address receive filters supported by the driver
    pub max_mcast_filter_count: u32,
    /// The current number of multicast address receive filters
    pub mcast_filter_count: u32,
    /// The array containing the addresses of the current multicast address receive filters
    pub mcast_filter: [MacAddress; 16],
    /// The current hardware MAC address for the network interface
    pub current_address: MacAddress,
    /// The current hardware MAC address for broadcast packets
    pub broadcast_address: MacAddress,
    /// The permanent hardware MAC address for the network interface
    pub permanent_address: MacAddress,
    /// The interface type of the network interface
    pub if_type: u8,
    /// Tells if the MAC address can be changed
    pub mac_address_changeable: bool,
    /// Tells if the network interface can transmit more than one packet at a time
    pub multiple_tx_supported: bool,
    /// Tells if the presence of the media can be determined
    pub media_present_supported: bool,
    /// Tells if media are connected to the network interface
    pub media_present: bool,
}

newtype_enum! {
    /// The state of a network interface.
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
