// SPDX-License-Identifier: MIT OR Apache-2.0

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
use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use uefi_raw::protocol::network::snp::SimpleNetworkProtocol;
use uefi_raw::Boolean;

pub use uefi_raw::protocol::network::snp::{
    InterruptStatus, NetworkMode, NetworkState, NetworkStatistics, ReceiveFlags,
};

/// The Simple Network Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(SimpleNetworkProtocol::GUID)]
pub struct SimpleNetwork(SimpleNetworkProtocol);

impl SimpleNetwork {
    /// Change the state of a network from "Stopped" to "Started".
    pub fn start(&self) -> Result {
        unsafe { (self.0.start)(&self.0) }.to_result()
    }

    /// Change the state of a network interface from "Started" to "Stopped".
    pub fn stop(&self) -> Result {
        unsafe { (self.0.stop)(&self.0) }.to_result()
    }

    /// Reset a network adapter and allocate the transmit and receive buffers
    /// required by the network interface; optionally, also request allocation of
    /// additional transmit and receive buffers.
    pub fn initialize(&self, extra_rx_buffer_size: usize, extra_tx_buffer_size: usize) -> Result {
        unsafe { (self.0.initialize)(&self.0, extra_rx_buffer_size, extra_tx_buffer_size) }
            .to_result()
    }

    /// Reset a network adapter and reinitialize it with the parameters that were
    /// provided in the previous call to `initialize`.
    pub fn reset(&self, extended_verification: bool) -> Result {
        unsafe { (self.0.reset)(&self.0, Boolean::from(extended_verification)) }.to_result()
    }

    /// Reset a network adapter, leaving it in a state that is safe
    /// for another driver to initialize
    pub fn shutdown(&self) -> Result {
        unsafe { (self.0.shutdown)(&self.0) }.to_result()
    }

    /// Manage the multicast receive filters of a network.
    pub fn receive_filters(
        &self,
        enable: ReceiveFlags,
        disable: ReceiveFlags,
        reset_mcast_filter: bool,
        mcast_filter: Option<&[MacAddress]>,
    ) -> Result {
        let filter_count = mcast_filter.map(|filters| filters.len()).unwrap_or(0);
        let filters = mcast_filter
            .map(|filters| filters.as_ptr())
            .unwrap_or(core::ptr::null_mut());

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
    pub fn station_address(&self, reset: bool, new: Option<&MacAddress>) -> Result {
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
    pub fn reset_statistics(&self) -> Result {
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
    pub fn collect_statistics(&self) -> Result<NetworkStatistics> {
        let mut stats_table: NetworkStatistics = Default::default();
        let mut stats_size = size_of::<NetworkStatistics>();
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

    /// Convert a multicast IP address to a multicast HW MAC Address.
    pub fn mcast_ip_to_mac(&self, ipv6: bool, ip: IpAddress) -> Result<MacAddress> {
        let mut mac_address = MacAddress([0; 32]);
        let status = unsafe {
            (self.0.multicast_ip_to_mac)(
                &self.0,
                Boolean::from(ipv6),
                ip.as_raw_ptr(),
                &mut mac_address,
            )
        };
        status.to_result_with_val(|| mac_address)
    }

    /// Perform read operations on the NVRAM device attached to
    /// a network interface.
    pub fn read_nv_data(&self, offset: usize, buffer: &[u8]) -> Result {
        unsafe {
            (self.0.non_volatile_data)(
                &self.0,
                Boolean::from(true),
                offset,
                buffer.len(),
                buffer.as_ptr() as *mut c_void,
            )
        }
        .to_result()
    }

    /// Perform write operations on the NVRAM device attached to a network interface.
    pub fn write_nv_data(&self, offset: usize, buffer: &mut [u8]) -> Result {
        unsafe {
            (self.0.non_volatile_data)(
                &self.0,
                Boolean::from(false),
                offset,
                buffer.len(),
                buffer.as_mut_ptr().cast(),
            )
        }
        .to_result()
    }

    /// Read the current interrupt status and recycled transmit buffer
    /// status from a network interface.
    pub fn get_interrupt_status(&self) -> Result<InterruptStatus> {
        let mut interrupt_status = InterruptStatus::empty();
        let status =
            unsafe { (self.0.get_status)(&self.0, &mut interrupt_status, ptr::null_mut()) };
        status.to_result_with_val(|| interrupt_status)
    }

    /// Read the current recycled transmit buffer status from a
    /// network interface.
    pub fn get_recycled_transmit_buffer_status(&self) -> Result<Option<NonNull<u8>>> {
        let mut tx_buf: *mut c_void = ptr::null_mut();
        let status = unsafe { (self.0.get_status)(&self.0, ptr::null_mut(), &mut tx_buf) };
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
        unsafe {
            (self.0.transmit)(
                &self.0,
                header_size,
                buffer.len(),
                buffer.as_ptr().cast(),
                src_addr.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
                dest_addr.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
                protocol.as_ref().map(ptr::from_ref).unwrap_or(ptr::null()),
            )
        }
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
    /// On QEMU, this event seems to never fire; it is suggested to verify that your implementation
    /// of UEFI properly implements this event before using it.
    #[must_use]
    pub const fn wait_for_packet(&self) -> &Event {
        unsafe { &*(ptr::from_ref(&self.0.wait_for_packet).cast::<Event>()) }
    }

    /// Returns a reference to the Simple Network mode.
    #[must_use]
    pub fn mode(&self) -> &NetworkMode {
        unsafe { &*self.0.mode }
    }
}
