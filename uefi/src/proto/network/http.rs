//! `http` protocol.

use crate::alloc::borrow::ToOwned;
use crate::proto::unsafe_protocol;
use crate::table::boot::{BootServices, EventType, TimerTrigger, Tpl};
use crate::Event;
use crate::{CString16, Error, Result, Status, StatusExt};
use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_void, CStr};
use core::iter::IntoIterator;
use core::ptr;
use uefi_raw::protocol::http::*;

pub use uefi_raw::protocol::http::{Code, Method};

/// HTTP Protocol
#[repr(transparent)]
#[unsafe_protocol(HttpProtocol::GUID, HttpProtocol::SERVICE_GUID, HttpProtocol)]
pub struct Http<'a> {
    proto: &'a mut HttpProtocol,
}

impl From<*mut HttpProtocol> for Http<'_> {
    fn from(proto: *mut HttpProtocol) -> Self {
        Self {
            proto: unsafe { &mut *proto },
        }
    }
}

impl Http<'_> {
    /// Reset the client, closing all active connections with remote hosts,
    /// canceling all asynchronous tokens, and flush request and response
    /// buffers without informing the appropriate hosts.
    pub fn reset(&mut self) -> Result {
        unsafe { (self.proto.configure)(&mut self.proto, ptr::null_mut()).to_result() }
    }

    /// Setup client using bound IPv4 address.
    pub fn setup_ipv4(&mut self) -> Result {
        let ipv4_node = V4AccessPoint {
            use_default_addr: true,
            local_address: [0; 4],
            local_subnet: [0; 4],
            local_port: 0,
        };
        let config = ConfigData {
            http_version: Version::HTTP_VERSION_11,
            timeout_ms: 15000, // Ignored by EDK2 for some reason
            local_addr_is_ipv6: false,
            access_point: AccessPoint {
                ipv4_node: &ipv4_node,
            },
        };
        unsafe { (self.proto.configure)(&mut self.proto, &config).to_result() }
    }

    /// Synchronously perform an HTTP request. To receive a response, [read]
    /// must be used before another request is sent.
    pub fn request<'a>(
        &mut self,
        bt: &BootServices,
        method: Method,
        url: impl Into<String>,
        headers: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        body: Option<&[u8]>,
    ) -> Result<(), String> {
        // Convert Method+URL
        let c_url: CString16 = Into::<String>::into(url).as_str().try_into().map_err(|_| {
            Error::new(
                Status::INVALID_PARAMETER,
                "URL has invalid characters".to_owned(),
            )
        })?;
        let url = c_url.to_u16_slice_with_nul().as_ptr();
        let request = RequestData { method, url };
        let data = RequestOrResponse {
            request: ptr::addr_of!(request),
        };

        // Convert Headers
        let headers = Headers::try_from_iter(headers)?;
        let header_count = headers.0.len();
        let header = headers.0.first().ok_or(Error::new(
            Status::INVALID_PARAMETER,
            "EDK2 does not support sending zero headers".to_owned(),
        ))? as *const Header as *mut Header;

        // Convert Body
        let (body, body_length) = match body {
            Some(bytes) => (bytes.as_ptr() as *mut c_void, bytes.len()),
            None => (ptr::null_mut(), 0),
        };

        // Create HTTP token
        let request_event = ScopedEvent {
            bt,
            event: unsafe {
                bt.create_event(EventType::empty(), Tpl::CALLBACK, None, None)
                    .map_err(|e| Error::new(e.status(), "creating request event".to_owned()))?
            },
        };
        let message = Message {
            data,
            header_count,
            header,
            body_length,
            body,
        };
        let token = Token {
            event: request_event.as_ptr(),
            status: Status::SUCCESS,
            message: &message as *const Message as *mut Message,
        };

        // Send request
        unsafe {
            (self.proto.request)(&mut self.proto, &token as *const Token as *mut Token)
                .to_result_with_err(|_| "sending request token".to_owned())?;
        };

        // Poll every 100ms
        let poll_interval = ScopedEvent {
            bt,
            event: unsafe {
                bt.create_event(EventType::TIMER, Tpl::APPLICATION, None, None)
                    .map_err(|e| {
                        Error::new(
                            e.status(),
                            "creating request poll interval event".to_owned(),
                        )
                    })?
            },
        };
        bt.set_timer(&poll_interval, TimerTrigger::Periodic(100_000_000 /* ns */))
            .map_err(|e| {
                Error::new(
                    e.status(),
                    "setting request poll interval timer on event".to_owned(),
                )
            })?;
        let mut request_or_poll =
            unsafe { [request_event.unsafe_clone(), poll_interval.unsafe_clone()] };
        loop {
            match bt.wait_for_event(&mut request_or_poll) {
                Ok(0) => {
                    return Ok(());
                }
                Ok(_) => {
                    let _ = self.poll();
                    continue;
                }
                Err(e) => {
                    return Err(Error::new(
                        e.status(),
                        "waiting for HTTP request event signal".to_owned(),
                    ));
                }
            }
        }
    }

    /// Read will copy response body bytes into the provided buffer. The number
    /// of bytes copied, status code, and any headers read will be returned.
    ///
    /// This method will not error when there is no more data to read, so it is
    /// the caller's responsibility to look at the Content-Length header on the
    /// first read.
    pub fn read_response(
        &mut self,
        bt: &BootServices,
        buf: &mut [u8],
    ) -> Result<(usize, Code, Vec<(String, String)>), String> {
        let response_event = ScopedEvent {
            bt,
            event: unsafe {
                bt.create_event(EventType::empty(), Tpl::CALLBACK, None, None)
                    .map_err(|e| {
                        Error::new(e.status(), "failed to create response event".to_owned())
                    })?
            },
        };

        let mut message = Message {
            data: RequestOrResponse {
                response: &mut ResponseData {
                    status_code: Code::UNSUPPORTED,
                },
            },
            header_count: 0,
            header: ptr::null_mut(),
            body_length: buf.len(),
            body: buf.as_ptr() as *mut c_void,
        };

        // Wait for response
        unsafe {
            (self.proto.response)(
                &mut self.proto,
                &mut Token {
                    event: response_event.as_ptr(),
                    status: Status::SUCCESS,
                    message: &mut message,
                },
            )
            .to_result_with_err(|_| "sending response token".to_owned())?;
        }
        // Poll every 100ms
        let poll_interval = ScopedEvent {
            bt,
            event: unsafe {
                bt.create_event(EventType::TIMER, Tpl::APPLICATION, None, None)
                    .map_err(|e| {
                        Error::new(
                            e.status(),
                            "creating response poll interval event".to_owned(),
                        )
                    })?
            },
        };
        bt.set_timer(&poll_interval, TimerTrigger::Periodic(100_000_000 /* ns */))
            .map_err(|e| {
                Error::new(
                    e.status(),
                    "setting response poll interval timer on event".to_owned(),
                )
            })?;
        let mut response_or_poll =
            unsafe { [response_event.unsafe_clone(), poll_interval.unsafe_clone()] };
        loop {
            match bt.wait_for_event(&mut response_or_poll) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    let _ = self.poll();
                    continue;
                }
                Err(e) => {
                    return Err(Error::new(
                        e.status(),
                        "waiting for HTTP response event signal".to_owned(),
                    ));
                }
            }
        }

        // Parse headers
        unsafe {
            Ok((
                message.body_length,
                (*message.data.response).status_code,
                core::slice::from_raw_parts(message.header, message.header_count)
                    .iter()
                    .map(|hdr| {
                        (
                            CStr::from_ptr(hdr.field_name as *const i8)
                                .to_owned()
                                .into_string()
                                .expect("unable to decode header name"),
                            CStr::from_ptr(hdr.field_value as *const i8)
                                .to_owned()
                                .into_string()
                                .expect("unable to decode header value"),
                        )
                    })
                    .collect(),
            ))
        }
    }

    /// Polls for incoming data packets and processes outgoing data packets.
    ///
    /// The Poll() function can be used by network drivers and applications to increase the rate
    /// that data packets are moved between the communication devices and the transmit and receive
    /// queues. In some systems, the periodic timer event in the managed network driver may not
    /// poll the underlying communications device fast enough to transmit and/or receive all data
    /// packets without missing incoming packets or dropping outgoing packets. Drivers and
    /// applications that are experiencing packet loss should try calling the Poll() function more
    /// often.
    fn poll(&mut self) -> Result {
        unsafe { (self.proto.poll)(&mut self.proto).to_result() }
    }
}

struct Headers(Vec<Header>);

impl Headers {
    fn try_from_iter<T, K, V>(value: T) -> Result<Self, String>
    where
        T: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        value
            .into_iter()
            .map(|(name, value)| {
                let field_name = CString::new(name.into().as_bytes())
                    .map_err(|_| {
                        Error::new(
                            Status::INVALID_PARAMETER,
                            "Header is not valid ASCII".to_owned(),
                        )
                    })?
                    .into_raw()
                    .cast();
                let field_value = CString::new(value.into().as_bytes())
                    .map_err(|_| {
                        Error::new(
                            Status::INVALID_PARAMETER,
                            "Header is not valid ASCII".to_owned(),
                        )
                    })?
                    .into_raw()
                    .cast();
                Ok(Header {
                    field_name,
                    field_value,
                })
            })
            .collect::<Result<Vec<Header>, _>>()
            .map(|headers| Headers(headers))
    }
}

impl Drop for Headers {
    fn drop(&mut self) {
        for header in &mut self.0 {
            unsafe {
                drop(CString::from_raw(header.field_name as *mut i8));
                drop(CString::from_raw(header.field_value as *mut i8));
            }
        }
    }
}

struct ScopedEvent<'a> {
    bt: &'a BootServices,
    event: Event,
}

impl ScopedEvent<'_> {
    #[allow(unused)]
    unsafe fn unsafe_inner(&self) -> Event {
        self.unsafe_clone()
    }
}

impl core::ops::Deref for ScopedEvent<'_> {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl core::ops::DerefMut for ScopedEvent<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl Drop for ScopedEvent<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = self.bt.close_event(self.unsafe_clone());
        }
    }
}
