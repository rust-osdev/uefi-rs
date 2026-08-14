// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI Simple Network protocol.
//!
//! Provides a packet level interface to a network adapter.
//! Once the adapter is initialized, the protocol provides services that allows
//! packets to be transmitted and received.
//!
//! No interface function must be called until `SimpleNetwork.start` is successfully
//! called first.

use crate::data_types::Event;
use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use core::ffi::c_void;
use core::net::IpAddr;
use core::ptr;
use core::ptr::NonNull;
use uefi::Error;
use uefi_raw::protocol::network::snp::SimpleNetworkProtocol;
use uefi_raw::{Boolean, IpAddress as EfiIpAddr, MacAddress as EfiMacAddr, Status};

pub use uefi_raw::protocol::network::snp::{
    InterruptStatus, NetworkMode, NetworkState, NetworkStatistics, ReceiveFlags,
};

/// Simple Network [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(SimpleNetworkProtocol::GUID)]
pub struct SimpleNetwork(SimpleNetworkProtocol);

impl SimpleNetwork {
    /// Changes the network state from stopped to started.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not stopped or cannot be started.
    pub fn start(&self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.start)(&self.0) }.to_result()
    }

    /// Changes the network state from started to stopped.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not started or cannot be stopped.
    pub fn stop(&self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.stop)(&self.0) }.to_result()
    }

    /// Initializes the network adapter and its transmit and receive buffers.
    ///
    /// # Arguments
    ///
    /// - `extra_rx_buffer_size`: Additional receive-buffer bytes to allocate.
    /// - `extra_tx_buffer_size`: Additional transmit-buffer bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not started or initialization fails.
    pub fn initialize(&self, extra_rx_buffer_size: usize, extra_tx_buffer_size: usize) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.initialize)(&self.0, extra_rx_buffer_size, extra_tx_buffer_size) }
            .to_result()
    }

    /// Reinitializes the adapter with its previous parameters.
    ///
    /// # Arguments
    ///
    /// - `extended_verification`: Whether to perform additional diagnostics.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not initialized or reset fails.
    pub fn reset(&self, extended_verification: bool) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.reset)(&self.0, Boolean::from(extended_verification)) }.to_result()
    }

    /// Shuts down the adapter for use by another driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not initialized or shutdown fails.
    pub fn shutdown(&self) -> Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.shutdown)(&self.0) }.to_result()
    }

    /// Manage the multicast receive filters of a network.
    ///
    /// # Arguments
    ///
    /// - `enable`: Receive filters to enable.
    /// - `disable`: Receive filters to disable.
    /// - `reset_mcast_filter`: Whether to clear the multicast filter list first.
    /// - `mcast_filter`: Multicast addresses to add to the filter list.
    ///
    /// # Errors
    ///
    /// Returns an error if the flags or addresses are invalid, unsupported, or
    /// cannot be applied.
    pub fn receive_filters(
        &self,
        enable: ReceiveFlags,
        disable: ReceiveFlags,
        reset_mcast_filter: bool,
        mcast_filter: Option<&[EfiMacAddr]>,
    ) -> Result {
        let filter_count = mcast_filter.map(|filters| filters.len()).unwrap_or(0);
        let filters = mcast_filter
            .map(|filters| filters.as_ptr())
            .unwrap_or(core::ptr::null_mut());

        // SAFETY: The memory is valid.
        unsafe {
            (self.0.receive_filters)(
                &self.0,
                enable,
                disable,
                Boolean::from(reset_mcast_filter),
                filter_count,
                filters,
            )
        }
        .to_result()
    }

    /// Modify or reset the current station address, if supported.
    ///
    /// # Arguments
    ///
    /// - `reset`: Whether to restore the permanent station address.
    /// - `new`: New station address when `reset` is `false`.
    ///
    /// # Errors
    ///
    /// Returns an error if changing the address is unsupported or fails.
    pub fn station_address(&self, reset: bool, new: Option<&EfiMacAddr>) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.station_address)(
                &self.0,
                Boolean::from(reset),
                new.map(ptr::from_ref).unwrap_or(ptr::null()),
            )
        }
        .to_result()
    }

    /// Reset statistics on a network interface.
    ///
    /// # Errors
    ///
    /// Returns an error if statistics are unsupported or cannot be reset.
    pub fn reset_statistics(&self) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.statistics)(
                &self.0,
                Boolean::from(true),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
        .to_result()
    }

    /// Collect statistics on a network interface.
    ///
    /// # Errors
    ///
    /// Returns an error if statistics are unsupported or cannot be read.
    pub fn collect_statistics(&self) -> Result<NetworkStatistics> {
        let mut stats_table: NetworkStatistics = Default::default();
        let mut stats_size = size_of::<NetworkStatistics>();
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.statistics)(
                &self.0,
                Boolean::from(false),
                &mut stats_size,
                &mut stats_table,
            )
        };
        status.to_result_with_val(|| stats_table)
    }

    /// Converts a multicast IP address to a hardware MAC address.
    ///
    /// # Arguments
    ///
    /// - `ipv6`: Whether `ip` is interpreted as an IPv6 address.
    /// - `ip`: Multicast address to convert.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid or conversion is unsupported.
    pub fn mcast_ip_to_mac(&self, ipv6: bool, ip: IpAddr) -> Result<EfiMacAddr> {
        let mut mac_address = EfiMacAddr([0; 32]);
        let ip = EfiIpAddr::from(ip);
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.multicast_ip_to_mac)(
                &self.0,
                Boolean::from(ipv6),
                &raw const ip,
                &mut mac_address,
            )
        };
        status.to_result_with_val(|| mac_address)
    }

    /// Reads network-interface NVRAM into `dst_buffer`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is invalid, NVRAM is unsupported, or the
    /// read fails.
    pub fn read_nv_data(&self, offset: usize, dst_buffer: &mut [u8]) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.non_volatile_data)(
                &self.0,
                Boolean::from(true),
                offset,
                dst_buffer.len(),
                dst_buffer.as_mut_ptr().cast(),
            )
        }
        .to_result()
    }

    /// Writes `src_buffer` to network-interface NVRAM.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is invalid, NVRAM is unsupported or
    /// read-only, or the write fails.
    pub fn write_nv_data(&self, offset: usize, src_buffer: &[u8]) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.non_volatile_data)(
                &self.0,
                Boolean::from(false),
                offset,
                src_buffer.len(),
                // SAFETY: The buffer is only used for reading.
                src_buffer.as_ptr().cast::<c_void>().cast_mut(),
            )
        }
        .to_result()
    }

    /// Returns the current network interrupt status.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not initialized or status cannot be
    /// read.
    pub fn get_interrupt_status(&self) -> Result<InterruptStatus> {
        let mut interrupt_status = InterruptStatus::empty();
        let status =
            // SAFETY: The memory is valid.
            unsafe { (self.0.get_status)(&self.0, &mut interrupt_status, ptr::null_mut()) };
        status.to_result_with_val(|| interrupt_status)
    }

    /// Returns the next recycled transmit buffer, if available.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface is not initialized or status cannot be
    /// read.
    pub fn get_recycled_transmit_buffer_status(&self) -> Result<Option<NonNull<u8>>> {
        let mut tx_buf: *mut c_void = ptr::null_mut();
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.get_status)(&self.0, ptr::null_mut(), &mut tx_buf) };
        status.to_result_with_val(|| NonNull::new(tx_buf.cast()))
    }

    /// Place a packet in the transmit queue of the network interface.
    ///
    /// The packet structure varies based on the type of network interface. In
    /// typical scenarios, the protocol is implemented for Ethernet devices,
    /// meaning this function transmits Ethernet frames.
    ///
    /// The header of the packet can be filled by the function with the given
    /// parameters, but the buffer must already reserve the space for the
    /// header.
    ///
    /// # Arguments
    /// - `header_size`: The size in bytes of the media header to be filled by
    ///   the `transmit()` function. If this is `0`, the (ethernet frame) header
    ///   will not be filled by the function and taken as-is from the buffer.
    ///   If it is nonzero, then it must be equal to `media_header_size` of
    ///   the corresponding [`NetworkMode`] and the `dst_addr` and `protocol`
    ///   parameters must not be `None`.
    /// - `buffer`: The buffer containing the whole network packet with all
    ///   its payload including the header for the medium.
    /// - `src_addr`: The optional source address.
    /// - `dst_addr`: The optional destination address.
    /// - `protocol`: Ether Type as of RFC 3232. See
    ///   [IANA IEEE 802 Numbers][ethertype] for examples. Typically, this is
    ///   `0x0800` (IPv4) or `0x0806` (ARP).
    ///
    /// [ethertype]: https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml#ieee-802-numbers-1
    ///
    /// # Errors
    ///
    /// Returns an error if the packet parameters are invalid, the transmit queue
    /// is full, or transmission fails.
    pub fn transmit(
        &self,
        header_size: usize,
        buffer: &[u8],
        src_addr: Option<EfiMacAddr>,
        dst_addr: Option<EfiMacAddr>,
        protocol: Option<u16>,
    ) -> Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.transmit)(
                &self.0,
                header_size,
                buffer.len(),
                buffer.as_ptr().cast(),
                src_addr.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
                dst_addr.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
                protocol.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
            )
        }
        .to_result()
    }

    /// Receive a packet from a network interface.
    ///
    /// On success, returns the number of bytes received.
    ///
    /// # Arguments
    ///
    /// - `buffer`: Destination for the packet, including its media header.
    /// - `header_size`: Optional output for the media-header size.
    /// - `src_addr`: Optional output for the source hardware address.
    /// - `dest_addr`: Optional output for the destination hardware address.
    /// - `protocol`: Optional output for the network protocol.
    ///
    /// # Errors
    ///
    /// Returns an error if no packet is ready, `buffer` is too small, or receive
    /// fails.
    pub fn receive(
        &self,
        buffer: &mut [u8],
        header_size: Option<&mut usize>,
        src_addr: Option<&mut EfiMacAddr>,
        dest_addr: Option<&mut EfiMacAddr>,
        protocol: Option<&mut u16>,
    ) -> Result<usize> {
        let mut buffer_size = buffer.len();
        // SAFETY: The memory is valid.
        let status = unsafe {
            (self.0.receive)(
                &self.0,
                header_size.map(ptr::from_mut).unwrap_or(ptr::null_mut()),
                &mut buffer_size,
                buffer.as_mut_ptr().cast(),
                src_addr.map(ptr::from_mut).unwrap_or(ptr::null_mut()),
                dest_addr.map(ptr::from_mut).unwrap_or(ptr::null_mut()),
                protocol.map(ptr::from_mut).unwrap_or(ptr::null_mut()),
            )
        };
        status.to_result_with_val(|| buffer_size)
    }

    /// Event that fires once a packet is available to be received.
    ///
    /// On QEMU, this event seems to never fire. Verify that the target firmware
    /// implements it correctly before relying on it.
    ///
    /// # Errors
    ///
    /// Returns [`Status::UNSUPPORTED`] if firmware provides no event.
    pub fn wait_for_packet_event(&self) -> Result<Event> {
        // SAFETY: The memory is valid.
        unsafe { Event::from_ptr(self.0.wait_for_packet) }.ok_or(Error::from(Status::UNSUPPORTED))
    }

    /// Returns a reference to the Simple Network mode.
    #[must_use]
    pub fn mode(&self) -> &NetworkMode {
        // SAFETY: The memory is valid.
        unsafe { &*self.0.mode }
    }
}
