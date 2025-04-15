// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi;

use bitflags::bitflags;

use crate::{guid, Boolean, Event, Guid, IpAddress, MacAddress, Status};

#[derive(Debug)]
#[repr(C)]
pub struct SimpleNetworkProtocol {
    pub revision: u64,
    pub start: unsafe extern "efiapi" fn(this: *const Self) -> Status,
    pub stop: unsafe extern "efiapi" fn(this: *const Self) -> Status,
    pub initialize: unsafe extern "efiapi" fn(
        this: *const Self,
        extra_receive_buffer_size: usize,
        extra_transmit_buffer_size: usize,
    ) -> Status,
    pub reset:
        unsafe extern "efiapi" fn(this: *const Self, extended_verification: Boolean) -> Status,
    pub shutdown: unsafe extern "efiapi" fn(this: *const Self) -> Status,
    pub receive_filters: unsafe extern "efiapi" fn(
        this: *const Self,
        enable: ReceiveFlags,
        disable: ReceiveFlags,
        reset_multicast_filter: Boolean,
        multicast_filter_count: usize,
        multicast_filter: *const MacAddress,
    ) -> Status,
    pub station_address: unsafe extern "efiapi" fn(
        this: *const Self,
        reset: Boolean,
        new: *const MacAddress,
    ) -> Status,
    pub statistics: unsafe extern "efiapi" fn(
        this: *const Self,
        reset: Boolean,
        statistics_size: *mut usize,
        statistics_table: *mut NetworkStatistics,
    ) -> Status,
    pub multicast_ip_to_mac: unsafe extern "efiapi" fn(
        this: *const Self,
        ipv6: Boolean,
        ip: *const IpAddress,
        mac: *mut MacAddress,
    ) -> Status,
    pub non_volatile_data: unsafe extern "efiapi" fn(
        this: *const Self,
        read: Boolean,
        offset: usize,
        buffer_size: usize,
        buffer: *mut ffi::c_void,
    ) -> Status,
    pub get_status: unsafe extern "efiapi" fn(
        this: *const Self,
        interrupt_status: *mut InterruptStatus,
        transmit_buffer: *mut *mut ffi::c_void,
    ) -> Status,
    pub transmit: unsafe extern "efiapi" fn(
        this: *const Self,
        header_size: usize,
        buffer_size: usize,
        buffer: *const ffi::c_void,
        source_address: *const MacAddress,
        dest_address: *const MacAddress,
        protocol: *const u16,
    ) -> Status,
    pub receive: unsafe extern "efiapi" fn(
        this: *const Self,
        header_size: *mut usize,
        buffer_size: *mut usize,
        buffer: *mut ffi::c_void,
        source_address: *mut MacAddress,
        dest_address: *mut MacAddress,
        protocol: *mut u16,
    ) -> Status,
    pub wait_for_packet: Event,
    pub mode: *mut NetworkMode,
}

impl SimpleNetworkProtocol {
    pub const GUID: Guid = guid!("a19832b9-ac25-11d3-9a2d-0090273fc14d");
}

bitflags! {
    /// Flags to pass to receive_filters to enable/disable reception of some kinds of packets.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ReceiveFlags: u32 {
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
    pub struct InterruptStatus: u32 {
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
pub struct NetworkStatistics {
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

impl NetworkStatistics {
    /// Any statistic value of -1 is not available
    const fn available(&self, stat: u64) -> bool {
        stat as i64 != -1
    }

    /// Takes a statistic and converts it to an option
    ///
    /// When the statistic is not available, `None` is returned
    const fn to_option(&self, stat: u64) -> Option<u64> {
        match self.available(stat) {
            true => Some(stat),
            false => None,
        }
    }

    /// The total number of frames received, including error frames
    /// and dropped frames
    #[must_use]
    pub const fn rx_total_frames(&self) -> Option<u64> {
        self.to_option(self.rx_total_frames)
    }

    /// The total number of good frames received and copied
    /// into receive buffers
    #[must_use]
    pub const fn rx_good_frames(&self) -> Option<u64> {
        self.to_option(self.rx_good_frames)
    }

    /// The number of frames below the minimum length for the
    /// communications device
    #[must_use]
    pub const fn rx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_undersize_frames)
    }

    /// The number of frames longer than the maximum length for
    /// the communications length device
    #[must_use]
    pub const fn rx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_oversize_frames)
    }

    /// The number of valid frames that were dropped because
    /// the receive buffers were full
    #[must_use]
    pub const fn rx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.rx_dropped_frames)
    }

    /// The number of valid unicast frames received and not dropped
    #[must_use]
    pub const fn rx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_unicast_frames)
    }

    /// The number of valid broadcast frames received and not dropped
    #[must_use]
    pub const fn rx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_broadcast_frames)
    }

    /// The number of valid multicast frames received and not dropped
    #[must_use]
    pub const fn rx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_multicast_frames)
    }

    /// Number of frames with CRC or alignment errors
    #[must_use]
    pub const fn rx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_crc_error_frames)
    }

    /// The total number of bytes received including frames with errors
    /// and dropped frames
    #[must_use]
    pub const fn rx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.rx_total_bytes)
    }

    /// The total number of frames transmitted including frames
    /// with errors and dropped frames
    #[must_use]
    pub const fn tx_total_frames(&self) -> Option<u64> {
        self.to_option(self.tx_total_frames)
    }

    /// The total number of valid frames transmitted and copied
    /// into receive buffers
    #[must_use]
    pub const fn tx_good_frames(&self) -> Option<u64> {
        self.to_option(self.tx_good_frames)
    }

    /// The number of frames below the minimum length for
    /// the media. This would be less than 64 for Ethernet
    #[must_use]
    pub const fn tx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_undersize_frames)
    }

    /// The number of frames longer than the maximum length for
    /// the media. This would be 1500 for Ethernet
    #[must_use]
    pub const fn tx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_oversize_frames)
    }

    /// The number of valid frames that were dropped because
    /// received buffers were full
    #[must_use]
    pub const fn tx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.tx_dropped_frames)
    }

    /// The number of valid unicast frames transmitted and not
    /// dropped
    #[must_use]
    pub const fn tx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_unicast_frames)
    }

    /// The number of valid broadcast frames transmitted and
    /// not dropped
    #[must_use]
    pub const fn tx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_broadcast_frames)
    }

    /// The number of valid multicast frames transmitted
    /// and not dropped
    #[must_use]
    pub const fn tx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_multicast_frames)
    }

    /// The number of transmitted frames with CRC or
    /// alignment errors
    #[must_use]
    pub const fn tx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_crc_error_frames)
    }

    /// The total number of bytes transmitted including
    /// error frames and dropped frames
    #[must_use]
    pub const fn tx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.tx_total_bytes)
    }

    /// The number of collisions detected on this subnet
    #[must_use]
    pub const fn collisions(&self) -> Option<u64> {
        self.to_option(self.collisions)
    }

    /// The number of frames destined for unsupported protocol
    #[must_use]
    pub const fn unsupported_protocol(&self) -> Option<u64> {
        self.to_option(self.unsupported_protocol)
    }

    /// The number of valid frames received that were duplicated
    #[must_use]
    pub const fn rx_duplicated_frames(&self) -> Option<u64> {
        self.to_option(self.rx_duplicated_frames)
    }

    /// The number of encrypted frames received that failed
    /// to decrypt
    #[must_use]
    pub const fn rx_decrypt_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_decrypt_error_frames)
    }

    /// The number of frames that failed to transmit after
    /// exceeding the retry limit
    #[must_use]
    pub const fn tx_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_error_frames)
    }

    /// The number of frames that transmitted successfully
    /// after more than one attempt
    #[must_use]
    pub const fn tx_retry_frames(&self) -> Option<u64> {
        self.to_option(self.tx_retry_frames)
    }
}

/// Information about the current configuration of an interface obtained by the
/// [`SimpleNetworkProtocol`].
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
    pub mac_address_changeable: Boolean,
    /// Tells if the network interface can transmit more than one packet at a time
    pub multiple_tx_supported: Boolean,
    /// Tells if the presence of the media can be determined
    pub media_present_supported: Boolean,
    /// Tells if media are connected to the network interface
    pub media_present: Boolean,
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
