// SPDX-License-Identifier: MIT OR Apache-2.0

// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(unused_imports)]
#![cfg(feature = "alloc")]

//! TCPv4 protocol.
//!
//! See [Tcp4].

use crate::boot::{self, EventType, Tpl};
use crate::proto::unsafe_protocol;
use crate::{Error, Event, Handle, Result, ResultExt, Status, StatusExt};
use core::ffi::c_void;
use core::fmt::Debug;
use core::ptr::{self, NonNull};
use core::time::Duration;
use core::{array, hint};
use uefi_raw::Boolean;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::ip4::Ip4ModeData;
use uefi_raw::protocol::network::tcp4::{
    Tcp4AccessPoint, Tcp4CompletionToken, Tcp4ConfigData, Tcp4ConnectionState, Tcp4FragmentData,
    Tcp4IoToken, Tcp4Option, Tcp4Packet, Tcp4Protocol, Tcp4ReceiveData, Tcp4TransmitData,
};
pub use wrappers::{AccessPoint, ConfigData, ConfigOptions};

/// A TCPv4 connection.
///
/// # Examples
///
/// ```no_run
/// # fn hello_world() -> uefi::Result {
/// # extern crate alloc;
/// use alloc::string::String;
/// use core::net::Ipv4Addr;
/// use uefi::{
///     boot, print, println,
///     proto::network::tcp4::{AccessPoint, ConfigData, Tcp4,
///     Tcp4ServiceBinding,
/// },
/// };
/// use uefi_raw::{
///     Boolean, protocol::network::tcp4::Tcp4AccessPoint,
///     protocol::network::tcp4::Tcp4ClientConnectionModeParams,
/// };
///
/// let remote_address = Ipv4Addr::new(192, 0, 2, 2);
/// let remote_port = 5050;
///
/// println!("Connecting to {remote_address:?}:{remote_port}...");
/// let mut tcp = {
///     let tcp_svc_handle = boot::get_handle_for_protocol::<Tcp4ServiceBinding>()?;
///     let mut tcp_svc_proto =
///         boot::open_protocol_exclusive::<Tcp4ServiceBinding>(tcp_svc_handle)?;
///     let tcp_proto_handle = tcp_svc_proto.create_child()?;
///     let mut tcp_proto = boot::open_protocol_exclusive::<Tcp4>(tcp_proto_handle)?;
///     let config_data = ConfigData {
///         type_of_service: 0,
///         time_to_live: 255,
///         access_point: AccessPoint {
///             use_default_address: true,
///             // The following two fields are meaningless when
///             // `use_default_address == true`
///             station_address: Ipv4Addr::UNSPECIFIED,
///             subnet_mask: Ipv4Addr::UNSPECIFIED,
///             station_port: 0,
///             remote_address,
///             remote_port,
///             // true => client mode
///             active_flag: true,
///         },
///     };
///     tcp_proto
///         .configure(&config_data, None)
///         .expect("configure failed");
///     tcp_proto.connect()?;
///     tcp_proto
/// };
///
/// let tx_msg = "Hello";
/// println!("Sending {tx_msg:?} over TCP...");
/// tcp.transmit(tx_msg.as_bytes())?;
///
/// print!("Received ");
/// let mut buf = [0_u8; 64];
/// let n = tcp.receive(&mut buf)?;
/// let rx_string = String::from_utf8_lossy(&buf[..n]);
/// println!("{rx_string:?}");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(Tcp4Protocol::GUID)]
pub struct Tcp4(pub Tcp4Protocol);

impl Tcp4 {
    /// See [Tcp4Protocol::configure].
    pub fn configure(&mut self, config: &ConfigData, options: Option<&ConfigOptions>) -> Result {
        let control_option = options.map(Tcp4Option::from);
        let tcpv4_config_data = Tcp4ConfigData {
            type_of_service: config.type_of_service,
            time_to_live: config.time_to_live,
            access_point: Tcp4AccessPoint::from(&config.access_point),
            control_option: control_option
                .as_ref()
                .map(|r| ptr::from_ref(r) as *mut _)
                .unwrap_or_else(ptr::null_mut),
        };
        let mut res =
            unsafe { (self.0.configure)(self.this(), ptr::from_ref(&tcpv4_config_data) as *mut _) }
                .to_result();
        // Maximum timeout of 10 seconds.
        for _ in 0..9 {
            match res {
                Ok(()) => break,
                Err(e) if e.status() == Status::NO_MAPPING => {
                    log::debug!("DHCP still running, waiting...");
                }
                Err(e) => {
                    log::debug!("Err {e:?}; will spin and try again...");
                }
            }
            boot::stall(Duration::from_secs(1));
            res = unsafe {
                (self.0.configure)(self.this(), ptr::from_ref(&tcpv4_config_data) as *mut _)
            }
            .to_result();
        }
        res
    }

    /// See [`Tcp4Protocol::connect`].
    pub fn connect(&mut self) -> Result {
        unsafe {
            let event = boot::create_event(
                EventType::NOTIFY_WAIT,
                Tpl::CALLBACK,
                Some(helpers::noop),
                None,
            )?;
            let mut completion_token = helpers::make_connection_token(&event);
            (self.0.connect)(self.this(), &mut completion_token).to_result()?;
            boot::wait_for_event(&mut [event.unsafe_clone()]).expect("can't fail waiting for event")
        };
        Ok(())
    }

    /// See [Tcp4Protocol::transmit].
    pub fn transmit_vectored(&mut self, data: &[&[u8]]) -> Result {
        // SAFETY: safe because there is no callback nor callback-data.
        let event = unsafe {
            boot::create_event(
                EventType::NOTIFY_WAIT,
                Tpl::CALLBACK,
                Some(helpers::noop),
                None,
            )
        }?;
        let tx_data = helpers::TransmitData::new(data);
        let mut token = helpers::make_tx_token(&event, &tx_data);
        unsafe { (self.0.transmit)(self.this(), &mut token) }.to_result()?;
        // See docs on `poll` for why this is crucial for performance.
        self.poll()?;
        boot::wait_for_event(&mut [event]).discard_errdata()?;
        Ok(())
    }

    /// See [`Tcp4Protocol::transmit`].
    pub fn transmit(&mut self, data: &[u8]) -> Result {
        self.transmit_vectored(&[data])
    }

    /// Receives data from the remote connection. On success, returns
    /// the number of bytes read.
    ///
    /// See [Tcp4Protocol::receive].
    pub fn receive_vectored(&mut self, bufs: &[&mut [u8]]) -> Result<usize> {
        // SAFETY: safe because there is no callback nor callback-data.
        let event = unsafe {
            boot::create_event(
                EventType::NOTIFY_WAIT,
                Tpl::CALLBACK,
                Some(helpers::noop),
                None,
            )
        }?;
        let rx_data = helpers::ReceiveData::new(bufs);
        let mut token = helpers::make_rx_token(&event, &rx_data);
        unsafe { (self.0.receive)(self.this(), &mut token) }.to_result()?;
        // See docs on `poll` for why this is crucial for performance.
        self.poll()?;
        boot::wait_for_event(&mut [event]).discard_errdata()?;
        let rx_data_len = rx_data.len();
        Ok(rx_data_len)
    }

    /// Receives data from the remote connection. On success, returns
    /// the number of bytes read.
    ///
    /// See [Tcp4Protocol::receive].
    pub fn receive(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.receive_vectored(&[buf])
    }

    /// Receives the exact number of bytes required to fill `buf`.
    ///
    /// This function receives as many bytes as necessary to
    /// completely fill the specified buffer `buf`.
    pub fn receive_exact(&mut self, mut buf: &mut [u8]) -> Result {
        while !buf.is_empty() {
            let n = self.receive(buf)?;
            buf = &mut buf[n..];
        }
        Ok(())
    }
}

// Private API
impl Tcp4 {
    /// Convenience method to return a non-null pointer to our inner
    /// Tcp4Protocol.
    const fn this(&mut self) -> *mut Tcp4Protocol {
        ptr::from_mut(&mut self.0)
    }

    /// **28.1.13. EFI_TCP4_PROTOCOL.Poll()**:
    ///
    /// > The Poll() function polls for incoming data packets and
    /// > processes outgoing data packets. Network drivers and
    /// > applications can call the EFI_IP4_PROTOCOL .Poll()
    /// > function to increase the rate that data packets are
    /// > moved between the communications device and the transmit
    /// > and receive queues.
    /// >
    /// > In some systems the periodic timer event may not poll the
    /// > underlying communications device fast enough to transmit
    /// > and/or receive all data packets without missing incoming
    /// > packets or dropping outgoing packets. Drivers and
    /// > applications that are experiencing packet loss should
    /// > try calling the EFI_IP4_PROTOCOL .Poll() function more
    /// > often.
    fn poll(&mut self) -> Result {
        unsafe { (self.0.poll)(self.this()) }.to_result()?;
        Ok(())
    }
}

/// TCPv4 Service Binding Protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(Tcp4Protocol::SERVICE_BINDING_GUID)]
pub struct Tcp4ServiceBinding(ServiceBindingProtocol);

impl Tcp4ServiceBinding {
    /// Create TCPv4 Protocol Handle.
    pub fn create_child(&mut self) -> uefi::Result<Handle> {
        let mut c_handle = ptr::null_mut();
        let status;
        let handle;
        unsafe {
            status = (self.0.create_child)(&mut self.0, &mut c_handle);
            handle = Handle::from_ptr(c_handle);
        };
        match status {
            Status::SUCCESS => Ok(handle.unwrap()),
            _ => Err(status.into()),
        }
    }

    /// Destroy TCPv4  Protocol Handle.
    pub fn destroy_child(&mut self, handle: Handle) -> uefi::Result<()> {
        let status = unsafe { (self.0.destroy_child)(&mut self.0, handle.as_ptr()) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

mod helpers {
    use alloc::alloc::{alloc, dealloc};
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use core::alloc::Layout;
    use core::ffi::c_void;
    use core::marker::{PhantomData, PhantomPinned};
    use core::ptr::{self, NonNull, read_volatile};
    use core::{array, mem};
    use uefi::Event;
    use uefi_raw::protocol::network::tcp4::{
        Tcp4CompletionToken, Tcp4ConnectionToken, Tcp4FragmentData, Tcp4IoToken, Tcp4Packet,
        Tcp4ReceiveData, Tcp4TransmitData,
    };
    use uefi_raw::{Boolean, Status};

    #[derive(Debug)]
    #[repr(C)]
    pub struct ReceiveData<'a> {
        ptr: NonNull<Tcp4ReceiveData>,
        layout: Layout,
        _pd: PhantomData<FragmentData<'a>>,
    }

    impl<'a> ReceiveData<'a> {
        pub fn new(data: &'a [&mut [u8]]) -> Self {
            let urgent = Boolean::FALSE;
            let data_length = data.iter().map(|d| d.len() as u32).sum();
            let fragment_count = data.len() as u32;
            let header_layout = Layout::new::<Tcp4ReceiveData>();
            let payload_layout =
                Layout::array::<FragmentData>(data.len()).expect("overflow not expected");
            let fragment_table: Vec<FragmentData> =
                data.iter().map(|d| FragmentData::new(d)).collect();
            let (layout, _) = header_layout
                .extend(payload_layout)
                .expect("overflow not expected");
            let ptr = unsafe {
                let ptr = alloc(layout).cast::<Tcp4ReceiveData>();
                let mut ptr = NonNull::new(ptr).expect("Allocation failed");
                ptr.as_mut().urgent = urgent;
                ptr.as_mut().data_length = data_length;
                ptr.as_mut().fragment_count = fragment_count;
                ptr::copy_nonoverlapping(
                    fragment_table.as_ptr() as *mut Tcp4FragmentData,
                    ptr.as_mut().fragment_table.as_mut_ptr(),
                    fragment_table.len(),
                );
                ptr
            };
            Self {
                ptr,
                layout,
                _pd: PhantomData,
            }
        }

        pub const fn as_mut_ptr(&self) -> *mut Tcp4ReceiveData {
            self.ptr.as_ptr()
        }

        pub const fn as_ref(&self) -> &Tcp4ReceiveData {
            unsafe { self.ptr.as_ref() }
        }

        pub fn len(&self) -> usize {
            let len = unsafe { ptr::read_volatile(&self.as_ref().data_length) };
            len as usize
        }
    }

    impl<'a> Drop for ReceiveData<'a> {
        fn drop(&mut self) {
            unsafe { dealloc(self.ptr.cast::<u8>().as_ptr(), self.layout) }
        }
    }

    /// This is the same as [`Tcp4TransmitData`], but with generically
    /// sized fragment table to allow for vectored writes.
    #[derive(Debug)]
    #[repr(C)]
    pub struct TransmitData<'a> {
        ptr: NonNull<Tcp4TransmitData>,
        layout: Layout,
        _pd: PhantomData<FragmentData<'a>>,
    }

    impl<'a> TransmitData<'a> {
        pub fn new(data: &'a [&[u8]]) -> Self {
            let data_length = data.iter().map(|d| d.len() as u32).sum();
            let fragment_table: Vec<FragmentData> =
                data.iter().map(|d| FragmentData::new(d)).collect();
            let layout = {
                let header_layout = Layout::new::<Tcp4TransmitData>();
                let payload_layout =
                    Layout::array::<FragmentData>(data.len()).expect("overflow not expected");
                let (layout, _) = header_layout
                    .extend(payload_layout)
                    .expect("overflow not expected");
                layout
            };
            let ptr = unsafe {
                let ptr = alloc(layout).cast::<Tcp4TransmitData>();
                let mut ptr = NonNull::new(ptr).expect("Allocation failed");
                ptr.as_mut().push = Boolean::FALSE;
                ptr.as_mut().urgent = Boolean::FALSE;
                ptr.as_mut().data_length = data_length;
                ptr.as_mut().fragment_count = data.len() as u32;
                ptr::copy_nonoverlapping(
                    fragment_table.as_ptr() as *mut Tcp4FragmentData,
                    ptr.as_mut().fragment_table.as_mut_ptr(),
                    fragment_table.len(),
                );
                ptr
            };
            Self {
                ptr,
                layout,
                _pd: PhantomData,
            }
        }

        pub const fn as_mut_ptr(&self) -> *mut Tcp4TransmitData {
            self.ptr.as_ptr()
        }
    }

    impl<'a> Drop for TransmitData<'a> {
        fn drop(&mut self) {
            unsafe { dealloc(self.ptr.cast::<u8>().as_ptr(), self.layout) }
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct FragmentData<'a> {
        fragment_length: u32,
        fragment_buf: NonNull<c_void>,
        _pd: PhantomData<&'a [u8]>,
    }

    impl<'a> FragmentData<'a> {
        pub const fn new(buf: &'a [u8]) -> Self {
            let fragment_length = buf.len() as u32;
            let fragment_buf = NonNull::new(ptr::from_ref(buf) as *mut c_void)
                .expect("buf is always a valid reference");
            let _pd = PhantomData;
            FragmentData {
                fragment_length,
                fragment_buf,
                _pd,
            }
        }
    }

    const fn make_completion_token(event: &Event) -> Tcp4CompletionToken {
        Tcp4CompletionToken {
            event: event.as_ptr(),
            status: Status::SUCCESS,
        }
    }

    pub const fn make_connection_token(event: &Event) -> Tcp4ConnectionToken {
        Tcp4ConnectionToken {
            completion_token: make_completion_token(event),
        }
    }

    pub const fn make_rx_token(event: &Event, rx_data: &ReceiveData) -> Tcp4IoToken {
        let rx_data = rx_data.as_mut_ptr();
        let packet = Tcp4Packet { rx_data };
        let completion_token = make_completion_token(event);
        Tcp4IoToken {
            completion_token,
            packet,
        }
    }

    pub const fn make_tx_token(event: &Event, tx_data: &TransmitData) -> Tcp4IoToken {
        let tx_data = tx_data.as_mut_ptr();
        let packet = Tcp4Packet { tx_data };
        let completion_token = make_completion_token(event);
        Tcp4IoToken {
            completion_token,
            packet,
        }
    }

    /// Dummy callback used for TCP events.
    #[allow(clippy::missing_const_for_fn)]
    pub extern "efiapi" fn noop(_event: Event, _context: Option<NonNull<c_void>>) {}
}

mod wrappers {
    use core::net::Ipv4Addr;
    use uefi_raw::protocol::network::tcp4::{Tcp4AccessPoint, Tcp4ConfigData, Tcp4Option};

    /// See [Tcp4AccessPoint]
    #[derive(Debug, Clone)]
    pub struct AccessPoint {
        /// Set to `true` to use the default IP address and default
        /// routing table.
        pub use_default_address: bool,
        /// The local IP address assigned to this TCP instance.
        ///
        /// Not used when `use_default_address` is `true`.
        pub station_address: Ipv4Addr,
        /// The subnet mask associated with the station address.
        ///
        /// Not used when `use_default_address` is `true`.
        pub subnet_mask: Ipv4Addr,
        /// The local port number.
        ///
        /// Set to 0 to get an ephemeral port.
        pub station_port: u16,
        /// The remote IP address to which this TCP instance is
        /// connected.
        ///
        /// If `active_flag` is `true` ('client mode'), the instance
        /// will connect to `remote_address`.
        ///
        /// If `active_flag` is `false` ('serve mode'), the instance
        /// only accepts connections from this address. If
        /// `active_flag` is `false` and `remote_address` is
        /// `0.0.0.0`, the instance will accept connections from any
        /// address.
        pub remote_address: Ipv4Addr,
        /// The remote port number.
        ///
        /// If `active_flag` is `true` ('client mode'), the instance
        /// will connect to the remote on this port. When
        /// `active_flag` is `true`, `remote_port` cannot be set to 0.
        ///
        /// If `active_flag` is `false` ('server mode'), `remote_port`
        /// port can be set to 0 to allow connections from any client
        /// port. Otherwise, the instance will only accept connections
        /// from clients with this port.
        pub remote_port: u16,
        /// Set to `true` to operate as a client and connect to remote
        /// host. Set to `false` to accept incoming connections from
        /// remote clients.
        pub active_flag: bool,
    }

    impl From<AccessPoint> for Tcp4AccessPoint {
        fn from(other: AccessPoint) -> Self {
            let AccessPoint {
                use_default_address,
                station_address,
                subnet_mask,
                station_port,
                remote_address,
                remote_port,
                active_flag,
            } = other;
            Self {
                use_default_address: use_default_address.into(),
                station_address: station_address.into(),
                subnet_mask: subnet_mask.into(),
                station_port,
                remote_address: remote_address.into(),
                remote_port,
                active_flag: active_flag.into(),
            }
        }
    }

    impl From<&AccessPoint> for Tcp4AccessPoint {
        fn from(other: &AccessPoint) -> Self {
            Self::from(other.clone())
        }
    }

    /// See [Tcp4ConfigData].
    #[derive(Debug, Clone)]
    pub struct ConfigData {
        /// Type of service field in transmitted IPv4 packets.
        pub type_of_service: u8,
        /// Time to live field in transmitted IPv4 packets.
        pub time_to_live: u8,
        /// Access point configuration.
        pub access_point: AccessPoint,
    }

    /// See [Tcp4Option].
    #[derive(Debug, Clone)]
    pub struct ConfigOptions {
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
        /// `FIN` timeout in seconds.
        pub fin_timeout: u32,
        /// `TIME_WAIT` timeout in seconds.
        pub time_wait_timeout: u32,
        /// Number of keep-alive probes.
        pub keep_alive_probes: u32,
        /// Time before sending keep-alive probes in seconds.
        pub keep_alive_time: u32,
        /// Interval between keep-alive probes in seconds.
        pub keep_alive_interval: u32,
        /// Set to `true` to enable Nagle algorithm.
        pub enable_nagle: bool,
        /// Set to `true` to enable TCP timestamps.
        pub enable_time_stamp: bool,
        /// Set to `true` to enable window scaling.
        pub enable_window_scaling: bool,
        /// Set to `true` to enable selective acknowledgment.
        pub enable_selective_ack: bool,
        /// Set to `true` to enable path MTU discovery.
        pub enable_path_mtu_discovery: bool,
    }

    impl From<ConfigOptions> for Tcp4Option {
        fn from(other: ConfigOptions) -> Self {
            let ConfigOptions {
                receive_buffer_size,
                send_buffer_size,
                max_syn_back_log,
                connection_timeout,
                data_retries,
                fin_timeout,
                time_wait_timeout,
                keep_alive_probes,
                keep_alive_time,
                keep_alive_interval,
                enable_nagle,
                enable_time_stamp,
                enable_window_scaling,
                enable_selective_ack,
                enable_path_mtu_discovery,
            } = other;
            Self {
                receive_buffer_size,
                send_buffer_size,
                max_syn_back_log,
                connection_timeout,
                data_retries,
                fin_timeout,
                time_wait_timeout,
                keep_alive_probes,
                keep_alive_time,
                keep_alive_interval,
                enable_nagle: enable_nagle.into(),
                enable_time_stamp: enable_time_stamp.into(),
                enable_window_scaling: enable_window_scaling.into(),
                enable_selective_ack: enable_selective_ack.into(),
                enable_path_mtu_discovery: enable_path_mtu_discovery.into(),
            }
        }
    }

    impl From<&ConfigOptions> for Tcp4Option {
        fn from(other: &ConfigOptions) -> Self {
            Self::from(other.clone())
        }
    }
}
