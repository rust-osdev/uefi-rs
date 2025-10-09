// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "alloc")]

//! TCPv4 protocol.
//!
//! See [Tcpv4].

use crate::{
    Error, Event, Handle, Result, ResultExt, Status, StatusExt,
    boot::{self, EventType, Tpl},
    proto::unsafe_protocol,
};
use core::{fmt::Debug, ptr, time::Duration};
use uefi_raw::protocol::{
    driver::ServiceBindingProtocol,
    network::tcpv4::{
        Ipv4ModeData, Tcpv4CompletionToken, Tcpv4ConfigData, Tcpv4ConnectionMode,
        Tcpv4ConnectionState, Tcpv4FragmentData, Tcpv4IoToken, Tcpv4Packet, Tcpv4Protocol,
        Tcpv4ReceiveData, Tcpv4TransmitData,
    },
};

fn make_completion_token(event: &Event) -> Tcpv4CompletionToken {
    // Safety: The lifetime of this token is bound by the lifetime
    //         of the ManagedEvent.
    let event_clone = unsafe { event.unsafe_clone() };
    Tcpv4CompletionToken {
        event: event_clone.as_ptr(),
        status: Status::SUCCESS,
    }
}

fn make_io_token<'a>(
    event: &Event,
    tx: Option<&'a Tcpv4TransmitData>,
    rx: Option<&'a Tcpv4ReceiveData>,
) -> Tcpv4IoToken<'a> {
    let packet = {
        if tx.is_some() {
            Tcpv4Packet { tx_data: tx }
        } else {
            let rx_ref = rx.as_ref();
            rx_ref.expect("Either RX or TX data handles must be provided");
            Tcpv4Packet { rx_data: rx }
        }
    };
    Tcpv4IoToken {
        completion_token: make_completion_token(event),
        packet,
    }
}

/// A TCPv4 connection.
///
/// # Examples
///
/// ```no_run
/// # fn hello_world() -> uefi::Result {
/// extern crate alloc;
///
/// use alloc::string::String;
/// use uefi::{
///     boot, print, println,
///     proto::network::tcpv4::{Tcpv4, Tcpv4ServiceBinding},
/// };
/// use uefi_raw::{
///     Ipv4Address,
///     protocol::network::tcpv4::{Tcpv4ClientConnectionModeParams, Tcpv4ConnectionMode},
/// };
///
/// let addr = Ipv4Address([192, 0, 2, 2]);
/// let port = 5050;
///
/// println!("Connecting to {addr:?}:{port}...");
/// let mut tcp = {
///     let tcp_svc_handle = boot::get_handle_for_protocol::<Tcpv4ServiceBinding>()?;
///     let mut tcp_svc_proto =
///         boot::open_protocol_exclusive::<Tcpv4ServiceBinding>(tcp_svc_handle)?;
///     let tcp_proto_handle = tcp_svc_proto.create_child()?;
///     let mut tcp_proto = boot::open_protocol_exclusive::<Tcpv4>(tcp_proto_handle)?;
///     tcp_proto
///         .configure(Tcpv4ConnectionMode::Client(
///             Tcpv4ClientConnectionModeParams::new(addr, port),
///         ))
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
#[unsafe_protocol(Tcpv4Protocol::GUID)]
pub struct Tcpv4(pub Tcpv4Protocol);

impl Tcpv4 {
    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-configure>
    pub fn configure(
        &mut self,
        connection_mode: Tcpv4ConnectionMode,
    ) -> uefi::Result<(), &'static str> {
        let configuration = Tcpv4ConfigData::new(connection_mode, None);
        // Maximum timeout of 10 seconds
        for _ in 0..10 {
            let result = (self.0.configure)(&mut self.0, Some(&configuration));
            if result == Status::SUCCESS {
                log::debug!("Configured connection! {result:?}");
                return Ok(());
            } else if result == Status::NO_MAPPING {
                log::debug!("DHCP still running, waiting...");
                boot::stall(Duration::from_secs(1));
            } else {
                log::warn!("Error {result:?}, will spin and try again");
                boot::stall(Duration::from_secs(1));
            }
        }
        Err(Error::new(
            Status::PROTOCOL_ERROR,
            "timeout before configuring the connection succeeded",
        ))
    }

    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-get-mode-data>
    pub fn get_tcp_connection_state(&self) -> Result<Tcpv4ConnectionState> {
        let mut connection_state = core::mem::MaybeUninit::<Tcpv4ConnectionState>::uninit();
        let connection_state_ptr = connection_state.as_mut_ptr();
        unsafe {
            (self.0.get_mode_data)(
                &self.0,
                Some(&mut *connection_state_ptr),
                None,
                None,
                None,
                None,
            )
            .to_result()?;
            Ok(connection_state.assume_init())
        }
    }

    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-get-mode-data>
    pub fn get_ipv4_mode_data(&self) -> Result<Ipv4ModeData<'_>> {
        let mut mode_data = core::mem::MaybeUninit::<Ipv4ModeData>::uninit();
        let mode_data_ptr = mode_data.as_mut_ptr();
        unsafe {
            (self.0.get_mode_data)(&self.0, None, None, Some(&mut *mode_data_ptr), None, None)
                .to_result()?;
            Ok(mode_data.assume_init())
        }
    }

    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-connect>
    pub fn connect(&mut self) -> Result {
        // SAFETY: safe because there is no callback nor callback-data.
        let event =
            unsafe { boot::create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(noop), None) }?;
        let completion_token = make_completion_token(&event);
        (self.0.connect)(&mut self.0, &completion_token).to_result()?;
        unsafe {
            boot::wait_for_event(&mut [event.unsafe_clone()]).expect("can't fail waiting for event")
        };
        Ok(())
    }

    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-transmit>
    pub fn transmit(&mut self, data: &[u8]) -> Result {
        // SAFETY: safe because there is no callback nor callback-data.
        let event =
            unsafe { boot::create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(noop), None) }?;
        let tx_data = Tcpv4TransmitData {
            push: false,
            urgent: false,
            data_length: data.len() as u32,
            fragment_count: 1,
            fragment_table: [Tcpv4FragmentData::with_buf(data)],
        };
        let io_token = make_io_token(&event, Some(&tx_data), None);
        (self.0.transmit)(&mut self.0, &io_token).to_result()?;
        // See docs on `poll` for why this is crucial for performance.
        self.poll()?;
        unsafe { boot::wait_for_event(&mut [event.unsafe_clone()]).discard_errdata()? };
        Ok(())
    }

    /// Receives data from the remote connection. On success, returns
    /// the number of bytes read.
    ///
    /// See <https://uefi.org/specs/UEFI/2.10/28_Network_Protocols_TCP_IP_and_Configuration.html#efi-tcp4-protocol-receive>
    pub fn receive(&mut self, buf: &mut [u8]) -> Result<usize> {
        let rx_data_len = {
            // SAFETY: safe because there is no callback nor callback-data.
            let event = unsafe {
                boot::create_event(EventType::NOTIFY_WAIT, Tpl::CALLBACK, Some(noop), None)
            }?;
            let rx_data = Tcpv4ReceiveData {
                urgent: false,
                data_length: buf.len() as u32,
                fragment_count: 1,
                fragment_table: [Tcpv4FragmentData::with_mut_buf(buf); 1],
            };
            let io_token = make_io_token(&event, None, Some(&rx_data));
            (self.0.receive)(&mut self.0, &io_token).to_result()?;
            // See docs on `poll` for why this is crucial for performance.
            self.poll()?;
            unsafe { boot::wait_for_event(&mut [event.unsafe_clone()]).discard_errdata()? };
            // SAFETY: calling `len` after a callback doesn't reflect
            //         the value updated by uefi.
            unsafe { core::ptr::read_volatile(&rx_data.data_length as *const u32) as usize }
        };
        Ok(rx_data_len)
    }

    /// Receives the exact number of bytes required to fill buf.
    ///
    /// This function receives as many bytes as necessary to
    /// completely fill the specified buffer buf.
    pub fn receive_exact(&mut self, mut buf: &mut [u8]) -> Result {
        while !buf.is_empty() {
            let n = self.receive(buf)?;
            buf = &mut buf[n..];
        }
        Ok(())
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
        (self.0.poll)(&mut self.0).to_result()?;
        Ok(())
    }
}

/// TCPv4 Service Binding Protocol.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(Tcpv4Protocol::SERVICE_BINDING_GUID)]
pub struct Tcpv4ServiceBinding(ServiceBindingProtocol);

impl Tcpv4ServiceBinding {
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

extern "efiapi" fn noop(_event: Event, _context: Option<ptr::NonNull<core::ffi::c_void>>) {}

mod example {
    #[allow(dead_code)]
    fn hello_world() -> uefi::Result {
        use alloc::string::String;
        use uefi::{
            boot, print, println,
            proto::network::tcpv4::{Tcpv4, Tcpv4ServiceBinding},
        };
        use uefi_raw::{
            Ipv4Address,
            protocol::network::tcpv4::{Tcpv4ClientConnectionModeParams, Tcpv4ConnectionMode},
        };

        let addr = Ipv4Address([192, 0, 2, 2]);
        let port = 5050;

        println!("Connecting to {addr:?}:{port}...");
        let mut tcp = {
            let tcp_svc_handle = boot::get_handle_for_protocol::<Tcpv4ServiceBinding>()?;
            let mut tcp_svc_proto =
                boot::open_protocol_exclusive::<Tcpv4ServiceBinding>(tcp_svc_handle)?;
            let tcp_proto_handle = tcp_svc_proto.create_child()?;
            let mut tcp_proto = boot::open_protocol_exclusive::<Tcpv4>(tcp_proto_handle)?;
            tcp_proto
                .configure(Tcpv4ConnectionMode::Client(
                    Tcpv4ClientConnectionModeParams::new(addr, port),
                ))
                .expect("configure failed");
            tcp_proto.connect()?;
            tcp_proto
        };

        let tx_msg = "Hello";
        println!("Sending {tx_msg:?} over TCP...");
        tcp.transmit(tx_msg.as_bytes())?;

        print!("Received ");
        let mut buf = [0_u8; 64];
        let n = tcp.receive(&mut buf)?;
        let rx_string = String::from_utf8_lossy(&buf[..n]);
        println!("{rx_string:?}");

        Ok(())
    }
}
