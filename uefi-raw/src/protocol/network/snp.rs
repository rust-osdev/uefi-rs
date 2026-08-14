// SPDX-License-Identifier: MIT OR Apache-2.0

//! Raw Simple Network protocol and data types.

use core::ffi;

use bitflags::bitflags;

use crate::{Boolean, Event, Guid, IpAddress, MacAddress, Status, guid, newtype_enum};

/// Simple Network protocol.
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
    /// Packet classes accepted by the receive filter.
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
    /// Interrupts reported since the previous status query.
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

/// Statistics reported by the Simple Network protocol.
///
/// Individual statistics may be unavailable. Each accessor returns `None` for
/// a statistic that the device does not support.
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
    /// Returns whether a statistic is available.
    const fn available(&self, stat: u64) -> bool {
        stat as i64 != -1
    }

    /// Converts a raw statistic to an optional value.
    ///
    /// An unavailable statistic produces `None`.
    const fn to_option(&self, stat: u64) -> Option<u64> {
        match self.available(stat) {
            true => Some(stat),
            false => None,
        }
    }

    /// Returns all received frames, including errors and drops.
    #[must_use]
    pub const fn rx_total_frames(&self) -> Option<u64> {
        self.to_option(self.rx_total_frames)
    }

    /// Returns valid frames copied into receive buffers.
    #[must_use]
    pub const fn rx_good_frames(&self) -> Option<u64> {
        self.to_option(self.rx_good_frames)
    }

    /// Returns frames below the device's minimum length.
    #[must_use]
    pub const fn rx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_undersize_frames)
    }

    /// Returns frames above the device's maximum length.
    #[must_use]
    pub const fn rx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.rx_oversize_frames)
    }

    /// Returns valid frames dropped because receive buffers were full.
    #[must_use]
    pub const fn rx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.rx_dropped_frames)
    }

    /// Returns valid unicast frames received without being dropped.
    #[must_use]
    pub const fn rx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_unicast_frames)
    }

    /// Returns valid broadcast frames received without being dropped.
    #[must_use]
    pub const fn rx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_broadcast_frames)
    }

    /// Returns valid multicast frames received without being dropped.
    #[must_use]
    pub const fn rx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.rx_multicast_frames)
    }

    /// Returns received frames with CRC or alignment errors.
    #[must_use]
    pub const fn rx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_crc_error_frames)
    }

    /// Returns all received bytes, including errors and drops.
    #[must_use]
    pub const fn rx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.rx_total_bytes)
    }

    /// Returns all transmitted frames, including errors and drops.
    #[must_use]
    pub const fn tx_total_frames(&self) -> Option<u64> {
        self.to_option(self.tx_total_frames)
    }

    /// Returns valid frames accepted for transmission.
    #[must_use]
    pub const fn tx_good_frames(&self) -> Option<u64> {
        self.to_option(self.tx_good_frames)
    }

    /// Returns frames below the medium's minimum length.
    #[must_use]
    pub const fn tx_undersize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_undersize_frames)
    }

    /// Returns frames above the medium's maximum length.
    #[must_use]
    pub const fn tx_oversize_frames(&self) -> Option<u64> {
        self.to_option(self.tx_oversize_frames)
    }

    /// Returns valid transmit frames dropped because buffers were full.
    #[must_use]
    pub const fn tx_dropped_frames(&self) -> Option<u64> {
        self.to_option(self.tx_dropped_frames)
    }

    /// Returns valid unicast frames transmitted without being dropped.
    #[must_use]
    pub const fn tx_unicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_unicast_frames)
    }

    /// Returns valid broadcast frames transmitted without being dropped.
    #[must_use]
    pub const fn tx_broadcast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_broadcast_frames)
    }

    /// Returns valid multicast frames transmitted without being dropped.
    #[must_use]
    pub const fn tx_multicast_frames(&self) -> Option<u64> {
        self.to_option(self.tx_multicast_frames)
    }

    /// Returns transmitted frames with CRC or alignment errors.
    #[must_use]
    pub const fn tx_crc_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_crc_error_frames)
    }

    /// Returns all transmitted bytes, including errors and drops.
    #[must_use]
    pub const fn tx_total_bytes(&self) -> Option<u64> {
        self.to_option(self.tx_total_bytes)
    }

    /// Returns collisions detected on the subnet.
    #[must_use]
    pub const fn collisions(&self) -> Option<u64> {
        self.to_option(self.collisions)
    }

    /// Returns frames for unsupported network protocols.
    #[must_use]
    pub const fn unsupported_protocol(&self) -> Option<u64> {
        self.to_option(self.unsupported_protocol)
    }

    /// Returns valid received frames that were duplicates.
    #[must_use]
    pub const fn rx_duplicated_frames(&self) -> Option<u64> {
        self.to_option(self.rx_duplicated_frames)
    }

    /// Returns encrypted frames that failed decryption.
    #[must_use]
    pub const fn rx_decrypt_error_frames(&self) -> Option<u64> {
        self.to_option(self.rx_decrypt_error_frames)
    }

    /// Returns frames that exceeded the transmit retry limit.
    #[must_use]
    pub const fn tx_error_frames(&self) -> Option<u64> {
        self.to_option(self.tx_error_frames)
    }

    /// Returns frames transmitted successfully after a retry.
    #[must_use]
    pub const fn tx_retry_frames(&self) -> Option<u64> {
        self.to_option(self.tx_retry_frames)
    }
}

/// Current configuration of a Simple Network interface.
#[repr(C)]
#[derive(Debug)]
pub struct NetworkMode {
    /// Current state of the network interface.
    pub state: NetworkState,
    /// Hardware-address size in bytes.
    pub hw_address_size: u32,
    /// Media-header size in bytes.
    pub media_header_size: u32,
    /// Maximum supported packet size in bytes.
    pub max_packet_size: u32,
    /// Attached NVRAM size in bytes.
    pub nv_ram_size: u32,
    /// Required granularity of NVRAM reads and writes.
    pub nv_ram_access_size: u32,
    /// Supported receive-filter settings.
    pub receive_filter_mask: u32,
    /// Active receive-filter settings.
    pub receive_filter_setting: u32,
    /// Maximum number of multicast-address filters.
    pub max_mcast_filter_count: u32,
    /// Number of active multicast-address filters.
    pub mcast_filter_count: u32,
    /// Active multicast-address filters.
    pub mcast_filter: [MacAddress; 16],
    /// Current hardware MAC address.
    pub current_address: MacAddress,
    /// Hardware MAC address used for broadcasts.
    pub broadcast_address: MacAddress,
    /// Permanent hardware MAC address.
    pub permanent_address: MacAddress,
    /// Network interface type.
    pub if_type: u8,
    /// Whether the MAC address can be changed.
    pub mac_address_changeable: Boolean,
    /// Whether multiple packets can be transmitted concurrently.
    pub multiple_tx_supported: Boolean,
    /// Whether media presence can be detected.
    pub media_present_supported: Boolean,
    /// Whether media are connected to the interface.
    pub media_present: Boolean,
}

newtype_enum! {
    /// The state of a network interface.
    pub enum NetworkState: u32 => {
        /// The interface is stopped.
        STOPPED = 0,
        /// The interface is started.
        STARTED = 1,
        /// The interface is initialized.
        INITIALIZED = 2,
        /// Sentinel above every valid state.
        MAX_STATE = 3,
    }
}
