// SPDX-License-Identifier: MIT OR Apache-2.0

//! TCPv4 Protocol
//!
//! This module provides the TCPv4 Protocol interface definitions. The
//! TCPv4 Protocol provides services to send and receive data streams
//! over IPv4 networks.
//!
//! The protocol is defined in the [UEFI Specification, Section 28.1](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcpv4-protocol).

use crate::protocol::network::ip4::Ip4ModeData;
use crate::protocol::network::snp::NetworkMode;
use crate::{Boolean, Event, Guid, Handle, Ipv4Address, Status, guid, newtype_enum};
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4Protocol {
    /// Get the current operational status.
    ///
    /// `get_mode_data` copies the current operational settings of
    /// this instance into user-supplied structs. This function can
    /// also be used to retrieve the operational setting of underlying
    /// drivers such as IPv4, MNP, or SNP.
    #[allow(clippy::type_complexity)]
    pub get_mode_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        connection_state: *mut Tcp4ConnectionState,
        config_data: *mut Tcp4ConfigData,
        ip4_mode_data: *mut Ip4ModeData,
        managed_network_config_data: *mut c_void,
        simple_network_mode: *mut NetworkMode,
    ) -> Status,

    /// Initialize or brutally reset the operational parameters for
    /// this instance.
    ///
    /// No other [`Tcp4Protocol`] operation can be executed by this
    /// instance until it is configured properly. For an active
    /// [`Tcp4Protocol`] instance, after a proper configuration it may
    /// call [`Tcp4Protocol::connect`] to initiates the three-way
    /// handshake. For a passive [`Tcp4Protocol`] instance, its state
    /// will transition to [`Tcp4ConnectionState::LISTEN`] after
    /// configuration, and [`accept`][Self::accept] may be called to
    /// listen for incoming TCP connection requests. If `config_data`
    /// is set to `NULL`, the instance is reset. Resetting process
    /// will be done brutally; the state machine will be set to
    /// [`Tcp4ConnectionState::CLOSED`] directly, the receive queue
    /// and transmit queue will be flushed, and no traffic will be
    /// allowed through this instance.
    pub configure:
        unsafe extern "efiapi" fn(this: *mut Self, config_data: *const Tcp4ConfigData) -> Status,

    /// Add or delete routing entries.
    ///
    /// The most specific route is selected by comparing the
    /// `subnet_address` with the destination IP address
    /// arithmetically ANDed with the `subnet_mask`.
    ///
    /// The default route is added with both `subnet_address` and
    /// `subnet_mask` set to `0.0.0.0`. The default route matches all
    /// destination IP addresses if there is no more specific route.
    ///
    /// A direct route is added with `gateway_address` set to
    /// `0.0.0.0`. Packets are sent to the destination host if its
    /// address can be found in the Address Resolution Protocol (ARP)
    /// cache or it is on the local subnet. If the instance is
    /// configured to use default address, a direct route to the local
    /// network will be added automatically.
    ///
    /// Each TCP instance has its own independent routing table. An
    /// instance that uses the default IP address will have a copy of
    /// the
    /// [`Ipv4Config2Protocol`][super::ip4_config2::Ip4Config2Protocol]'s
    /// routing table. The copy will be updated automatically whenever
    /// the IP driver reconfigures its instance. As a result, the
    /// previous modification to the instance's local copy will be
    /// lost.
    ///
    /// The priority of checking the route table is specific to the IP
    /// implementation, and every IP implementation must comply with
    /// RFC 1122.
    ///
    /// Note: There is no way to set up routes to other network
    /// interface cards (NICs) because each NIC has its own
    /// independent network stack that shares information only through
    /// EFI TCPv4 variable.
    pub routes: unsafe extern "efiapi" fn(
        this: *mut Self,
        delete_route: Boolean,
        subnet_address: *const Ipv4Address,
        subnet_mask: *const Ipv4Address,
        gateway_address: *const Ipv4Address,
    ) -> Status,

    /// Initiate a nonblocking TCP connection request for an active
    /// TCP instance.
    ///
    /// `connect` initiates an active open to the remote peer
    /// configured in the current TCP instance if it is configured as
    /// active. If the connection succeeds or fails due to any error,
    /// the [`token.completion_token.event`][Tcp4CompletionToken::event]
    /// will be signaled and `token.completion_token.status` will be
    /// updated accordingly. This function can only be called for the
    /// TCP instance in the [`Tcp4ConnectionState::CLOSED`] state. The
    /// instance will transition to [`Tcp4ConnectionState::SYN_SENT`]
    /// if the function returns [`Status::SUCCESS`]. If the TCP
    /// three-way handshake succeeds, its state will become
    /// [`Tcp4ConnectionState::ESTABLISHED`], otherwise, the state
    /// will return to [`Tcp4ConnectionState::CLOSED`].
    pub connect:
        unsafe extern "efiapi" fn(this: *mut Self, token: *mut Tcp4ConnectionToken) -> Status,

    /// Listen on the passive instance to accept an incoming
    /// connection request. This is a nonblocking operation.
    ///
    /// The `accept` function initiates an asynchronous accept request
    /// to wait for an incoming connection on the passive TCP
    /// instance. If a remote peer successfully establishes a
    /// connection with this instance, a new TCP instance will be
    /// created and its handle will be returned in
    /// [`new_child_handle`][Tcp4ListenToken::new_child_handle]. The
    /// newly created instance is configured by inheriting the passive
    /// instance's configuration and is ready for use upon return. The
    /// instance is in the [`Tcp4ConnectionState::ESTABLISHED`]
    /// state.
    ///
    /// The [`new_child_handle`][Tcp4ListenToken::new_child_handle]
    /// will be signaled when a new connection is accepted, the user
    /// aborts the listen, or the connection is reset.
    ///
    /// This function can only be called when the current TCP instance
    /// is in [`Tcp4ConnectionState::LISTEN`] state.
    pub accept:
        unsafe extern "efiapi" fn(this: *mut Self, listen_token: *mut Tcp4ListenToken) -> Status,

    /// Queues outgoing data into the transmit queue.
    ///
    /// The `transmit` function queues a sending request to this
    /// instance along with the user data. The status of the token is
    /// updated and the event in the token will be signaled once the
    /// data is sent out or some error occurs.
    pub transmit: unsafe extern "efiapi" fn(this: *mut Self, token: *mut Tcp4IoToken) -> Status,

    /// Places an asynchronous receive request into the receiving
    /// queue.
    ///
    /// `receive` places a completion token into the receive packet
    /// queue. This function is always asynchronous. The caller must
    /// allocate the
    /// [`token.completion_token.event`][Tcp4CompletionToken::event]
    /// and the `fragment_buffer` used to receive data. They also must
    /// fill [`data_length`][Tcp4ReceiveData::data_length] which
    /// represents the whole length of all `fragment_buffer`. When the
    /// receive operation completes, the driver updates the
    /// [`token.completion_token.status`][Tcp4CompletionToken::status]
    /// and [`token.packet.rx_data`][Tcp4Packet::rx_data] fields and
    /// and the `token.completion_token.event` is signaled. If data
    /// was received, the data and its length will be copied into the
    /// `fragment_table`. At the same time, the full length of
    /// received data will be recorded in the `data_length`
    /// fields. Providing a proper notification function and context
    /// for the event will enable the user to receive the notification
    /// and receiving status. That notification function is guaranteed
    /// to not be re-entered.
    pub receive: unsafe extern "efiapi" fn(this: *mut Self, token: *mut Tcp4IoToken) -> Status,

    /// Disconnect a TCP connection gracefully or reset a TCP
    /// connection. This function is a nonblocking operation.
    ///
    /// Initiate an asynchronous close token to TCP driver. After
    /// `close` is called, any buffered transmission data will be sent
    /// by TCP driver and the current instance will have a graceful
    /// close working flow described as RFC 793 if `abort_on_close` is
    /// set to `false`, otherwise, a reset packet will be sent by the
    /// TCP driver to quickly disconnect this connection. When the
    /// close operation completes successfully the TCP instance is in
    /// [`Tcp4ConnectionState::CLOSED`] state, all pending
    /// asynchronous operations are signaled, and any buffers used for
    /// TCP network traffic are flushed.
    pub close:
        unsafe extern "efiapi" fn(this: *mut Self, close_token: *mut Tcp4CloseToken) -> Status,

    /// Abort an asynchronous connection, listen, transmission or
    /// receive request.
    ///
    /// The `cancel` function aborts a pending connection, listen,
    /// transmit or receive request. If `completion_token` is not
    /// `NULL` and the token is in the connection, listen,
    /// transmission or receive queue when it is being cancelled, its
    /// `status` will be set to [`Status::ABORTED`] and then `event`
    /// will be signaled. If the token is not in one of the queues,
    /// which usually means that the asynchronous operation has
    /// completed, [`Status::NOT_FOUND`] is returned. If
    /// `completion_token` is `NULL`, all asynchronous tokens issued
    /// by [`Tcp4Protocol::connect`], [`Tcp4Protocol::accept`],
    /// [`Tcp4Protocol::transmit`] and [`Tcp4Protocol::receive`]
    /// will be aborted.
    pub cancel: unsafe extern "efiapi" fn(
        this: *mut Self,
        completion_token: *mut Tcp4CompletionToken,
    ) -> Status,

    /// Poll to receive incoming data and transmit outgoing segments.
    ///
    /// The `poll` function increases the rate that data is moved
    /// between the network and application and can be called when the
    /// TCP instance is created successfully. Its use is optional.
    ///
    /// In some implementations, the periodical timer in the MNP
    /// driver may not poll the underlying communications device fast
    /// enough to avoid dropping packets. Drivers and applications
    /// that are experiencing packet loss should try calling the
    /// `poll` function at a high frequency.
    pub poll: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

impl Tcp4Protocol {
    /// The GUID for the TCPv4 protocol.
    ///
    /// Defined in the [UEFI Specification, Section 28.1.4](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol).
    pub const GUID: Guid = guid!("65530BC7-A359-410F-B010-5AADC7EC2B62");

    /// The GUID for the TCPv4 service binding protocol.
    ///
    /// Defined in the [UEFI Specification, Section 28.1.2](https://uefi.org/specs/UEFI/2.11/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-service-binding-protocol).
    pub const SERVICE_BINDING_GUID: Guid = guid!("00720665-67EB-4a99-BAF7-D3C33A1C7CC9");
}

newtype_enum! {
    pub enum Tcp4ConnectionState: i32 => {
        CLOSED = 0,
        LISTEN = 1,
        SYN_SENT = 2,
        SYN_RECEIVED = 3,
        ESTABLISHED = 4,
        FIN_WAIT1 = 5,
        FIN_WAIT2 = 6,
        CLOSING = 7,
        TIME_WAIT = 8,
        CLOSE_WAIT = 9,
        LAST_ACK = 10,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4ConfigData {
    /// Type of service field in transmitted IPv4 packets.
    pub type_of_service: u8,
    /// Time to live field in transmitted IPv4 packets.
    pub time_to_live: u8,
    /// Access point configuration.
    pub access_point: Tcp4AccessPoint,
    /// Optional TCP configuration parameters.
    pub control_option: *mut Tcp4Option,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Tcp4AccessPoint {
    /// Set to `TRUE` to use the default IP address.
    pub use_default_address: Boolean,
    /// The local IP address assigned to this TCP instance.
    pub station_address: Ipv4Address,
    /// The subnet mask associated with the station address.
    pub subnet_mask: Ipv4Address,
    /// The local port number.
    pub station_port: u16,
    /// The remote IP address.
    pub remote_address: Ipv4Address,
    /// The remote port number.
    pub remote_port: u16,
    /// Set to `TRUE` for active open, `FALSE` for passive open.
    pub active_flag: Boolean,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4Option {
    /// Size of the TCP receive buffer.
    pub receive_buffer_size: u32,
    /// Size of the TCP send buffer.
    pub send_buffer_size: u32,
    /// Maximum number of pending connections for passive instances.
    pub max_syn_back_log: u32,
    /// Connection timeout in seconds.
    pub connection_timeout: u32,
    /// Number of data retransmission attempts.
    pub data_retries: u32,
    /// FIN timeout in seconds.
    pub fin_timeout: u32,
    /// TIME_WAIT timeout in seconds.
    pub time_wait_timeout: u32,
    /// Number of keep-alive probes.
    pub keep_alive_probes: u32,
    /// Time before sending keep-alive probes in seconds.
    pub keep_alive_time: u32,
    /// Interval between keep-alive probes in seconds.
    pub keep_alive_interval: u32,
    /// Set to `TRUE` to enable Nagle algorithm.
    pub enable_nagle: Boolean,
    /// Set to `TRUE` to enable TCP timestamps.
    pub enable_time_stamp: Boolean,
    /// Set to `TRUE` to enable window scaling.
    pub enable_window_scaling: Boolean,
    /// Set to `TRUE` to enable selective acknowledgment.
    pub enable_selective_ack: Boolean,
    /// Set to `TRUE` to enable path MTU discovery.
    pub enable_path_mtu_discovery: Boolean,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4CompletionToken {
    /// Event to signal when the operation completes.
    pub event: Event,
    /// Status of the completed operation.
    pub status: Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4ConnectionToken {
    pub completion_token: Tcp4CompletionToken,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4ListenToken {
    /// Completion token for the listen operation.
    pub completion_token: Tcp4CompletionToken,
    /// The new TCP instance handle created for the established
    /// connection.
    pub new_child_handle: Handle,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4IoToken {
    /// Completion token for the I/O operation.
    pub completion_token: Tcp4CompletionToken,
    /// Packet data for the I/O operation.
    pub packet: Tcp4Packet,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4CloseToken {
    /// Completion token for the close operation.
    pub completion_token: Tcp4CompletionToken,
    /// Abort the TCP connection on close instead of the standard TCP
    /// close process when it is set to TRUE. This option can be used
    /// to satisfy a fast disconnect.
    pub abort_on_close: Boolean,
}

#[repr(C)]
pub union Tcp4Packet {
    /// Pointer to receive data structure.
    pub rx_data: *mut Tcp4ReceiveData,
    /// Pointer to transmit data structure.
    pub tx_data: *mut Tcp4TransmitData,
}

impl Debug for Tcp4Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tcp4Packet").finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4FragmentData {
    /// Length of the fragment in bytes.
    pub fragment_length: u32,
    /// Pointer to an array of contiguous bytes, at least
    /// `fragment_length` in length.
    pub fragment_buf: *mut u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4ReceiveData {
    /// When `TRUE`, the instance is in urgent mode. The
    /// implementations of this specification should follow RFC793 to
    /// process urgent data, and should NOT mix the data across the
    /// urgent point in one token.
    pub urgent: Boolean,
    /// When calling [`receive`][Tcp4Protocol::receive], the caller
    /// is responsible for setting it to the byte counts of all
    /// fragment buffers. When the token is signaled by the driver
    /// it is the length of received data in the fragments.
    pub data_length: u32,
    /// Number of fragments in the following fragment table.
    pub fragment_count: u32,
    /// Variable-length array of fragment descriptors.
    ///
    /// NOTE: this is a flexible array member.
    pub fragment_table: [Tcp4FragmentData; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4TransmitData {
    /// If `TRUE`, data must be transmitted promptly, and the PUSH bit
    /// in the last TCP segment created will be set. If `FALSE`, data
    /// transmission may be delayed to combine with data from
    /// subsequent [`transmit`][Tcp4Protocol::transmit] for
    /// efficiency.
    pub push: Boolean,
    /// The data in the fragment table are urgent and urgent point is
    /// in effect if `TRUE`. Otherwise those data are NOT considered
    /// urgent.
    pub urgent: Boolean,
    /// Total length of data to transmit.
    pub data_length: u32,
    /// Number of fragments in the following fragment table.
    pub fragment_count: u32,
    /// Variable-length array of fragment descriptors.
    ///
    /// NOTE: this is a flexible array member.
    pub fragment_table: [Tcp4FragmentData; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Tcp4ClientConnectionModeParams {
    /// Remote IP address for the connection.
    pub remote_ip: Ipv4Address,
    /// Remote port for the connection.
    pub remote_port: u16,
}
