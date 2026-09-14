// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "alloc")]

//! HTTP Protocol.
//!
//! See [`Http`].

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char, c_void};
use core::ptr::{self, NonNull};
use log::debug;

use uefi::boot::{self, ScopedProtocol};
use uefi::prelude::*;
use uefi::proto::unsafe_protocol;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::http::{
    HttpAccessPoint, HttpConfigData, HttpHeader, HttpMessage, HttpMethod, HttpProtocol,
    HttpRequestData, HttpResponseData, HttpStatusCode, HttpToken, HttpV4AccessPoint, HttpVersion,
};

/// HTTP [`Protocol`]. Send HTTP Requests.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::GUID)]
#[repr(transparent)]
pub struct Http(HttpProtocol);

impl Http {
    /// Receive HTTP Protocol configuration.
    pub fn get_mode_data(&mut self) -> uefi::Result<HttpConfigData> {
        let mut config_data = HttpConfigData::default();
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.get_mode_data)(&mut self.0, &mut config_data) };
        match status {
            Status::SUCCESS => Ok(config_data),
            _ => Err(status.into()),
        }
    }

    /// Configure HTTP Protocol.  Must be called before sending HTTP requests.
    pub fn configure(&mut self, config_data: &HttpConfigData) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.configure)(&mut self.0, config_data) };
        debug!("http raw: configure({config_data:?}) -> {status}");
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Send HTTP request.
    pub fn request(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.request)(&mut self.0, token) };
        debug!(
            "http raw: request(headers={}, body_len={}) -> {status}, token.status={}",
            // SAFETY: The memory is valid.
            unsafe { (*token.message).header_count },
            // SAFETY: The memory is valid.
            unsafe { (*token.message).body_length },
            token.status,
        );
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Cancel HTTP request.
    pub fn cancel(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.cancel)(&mut self.0, token) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Receive HTTP response.
    pub fn response(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.response)(&mut self.0, token) };
        debug!(
            "http raw: response(body_len={}) -> {status}, token.status={}",
            // SAFETY: The memory is valid.
            unsafe { (*token.message).body_length },
            token.status,
        );
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Poll network stack for updates.
    pub fn poll(&mut self) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.poll)(&mut self.0) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

/// A token that was handed to the firmware by [`Http::request`] or
/// [`Http::response`] and has not completed yet.
struct PendingToken<'a> {
    http: &'a mut Http,
    token: &'a mut HttpToken,
}

impl<'a> PendingToken<'a> {
    /// Sends the request described by `token`.
    fn request(http: &'a mut Http, token: &'a mut HttpToken) -> uefi::Result<Self> {
        http.request(token)?;
        Ok(Self { http, token })
    }

    /// Starts receiving the response described by `token`.
    fn response(http: &'a mut Http, token: &'a mut HttpToken) -> uefi::Result<Self> {
        http.response(token)?;
        Ok(Self { http, token })
    }

    /// Polls the network stack until the token completes and returns its
    /// final status.
    fn wait(self) -> uefi::Result<Status> {
        let mut polls = 0;
        while self.token.status == Status::NOT_READY {
            self.http.poll()?;
            polls += 1;
        }
        debug!(
            "http: token completed after {polls} polls with {}",
            self.token.status
        );
        Ok(self.token.status)
    }
}

impl Drop for PendingToken<'_> {
    fn drop(&mut self) {
        if self.token.status == Status::NOT_READY {
            // Nothing sensible can be done if cancelling fails.
            let _ = self.http.cancel(self.token);
        }
    }
}

/// HTTP Service Binding Protocol.
#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::SERVICE_BINDING_GUID)]
#[repr(transparent)]
pub struct HttpBinding(ServiceBindingProtocol);

impl HttpBinding {
    /// Create HTTP Protocol Handle.
    pub fn create_child(&mut self) -> uefi::Result<Handle> {
        let mut c_handle = ptr::null_mut();
        let status;
        let handle;
        // SAFETY: The memory is valid.
        unsafe {
            status = (self.0.create_child)(&mut self.0, &mut c_handle);
            handle = Handle::from_ptr(c_handle);
        };
        match status {
            Status::SUCCESS => Ok(handle.unwrap()),
            _ => Err(status.into()),
        }
    }

    /// Destroy HTTP Protocol Handle.
    pub fn destroy_child(&mut self, handle: Handle) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.destroy_child)(&mut self.0, handle.as_ptr()) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

/// Representation of the underlying UEFI HTTP response.
///
/// Helper type for [`HttpHelper`].
#[derive(Debug)]
pub struct HttpHelperResponse {
    /// HTTP Status
    pub status: HttpStatusCode,
    /// HTTP Response Headers
    pub headers: Vec<(String, String)>,
    /// Partial or entire HTTP body, depending on context.
    pub body: Vec<u8>,
}

/// HTTP Helper, makes using the [HTTP] [`Protocol`] more convenient.
///
/// [HTTP]: Http
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
pub struct HttpHelper {
    child_handle: Handle,
    binding: ScopedProtocol<HttpBinding>,
    protocol: Option<ScopedProtocol<Http>>,
}

/// Frees the headers of a response message.
///
/// The driver allocates the header array as well as each field name and
/// value from the pool, and the caller has to free all of them.
///
/// # Safety
///
/// `msg` must be a response message filled by the driver.
unsafe fn free_response_headers(msg: &HttpMessage) {
    let Some(headers) = NonNull::new(msg.header) else {
        return;
    };
    for i in 0..msg.header_count {
        // SAFETY: The driver wrote `header_count` entries.
        let header = unsafe { &*headers.as_ptr().add(i) };
        for field in [header.field_name, header.field_value] {
            if let Some(field) = NonNull::new(field.cast_mut()) {
                // SAFETY: The string was allocated by the matching UEFI
                // allocator.
                let _ = unsafe { boot::free_pool(field.cast()) };
            }
        }
    }
    // SAFETY: The array was allocated by the matching UEFI allocator.
    let _ = unsafe { boot::free_pool(headers.cast()) };
}

impl HttpHelper {
    /// Create new HTTP helper instance for the given NIC handle.
    pub fn new(nic_handle: Handle) -> uefi::Result<Self> {
        // SAFETY: The memory is valid.
        let mut binding = unsafe {
            boot::open_protocol::<HttpBinding>(
                boot::OpenProtocolParams {
                    handle: nic_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                boot::OpenProtocolAttributes::GetProtocol,
            )?
        };
        debug!("http: binding proto ok");

        let child_handle = binding.create_child()?;
        debug!("http: child handle ok");

        // SAFETY: The memory is valid.
        let protocol_res = unsafe {
            boot::open_protocol::<Http>(
                boot::OpenProtocolParams {
                    handle: child_handle,
                    agent: boot::image_handle(),
                    controller: None,
                },
                boot::OpenProtocolAttributes::GetProtocol,
            )
        };
        if let Err(e) = protocol_res {
            let _ = binding.destroy_child(child_handle);
            return Err(e);
        }
        debug!("http: protocol ok");

        Ok(Self {
            child_handle,
            binding,
            protocol: Some(protocol_res.unwrap()),
        })
    }

    /// Configure the HTTP Protocol with some sane defaults.
    pub fn configure(&mut self) -> uefi::Result<()> {
        let ip4 = HttpV4AccessPoint {
            use_default_addr: true.into(),
            ..Default::default()
        };

        let config = HttpConfigData {
            http_version: HttpVersion::HTTP_VERSION_10,
            time_out_millisec: 10_000,
            local_addr_is_ipv6: false.into(),
            access_point: HttpAccessPoint { ipv4_node: &ip4 },
        };

        self.protocol.as_mut().unwrap().configure(&config)?;
        debug!("http: configure ok");

        Ok(())
    }

    /// Send HTTP request
    pub fn request(
        &mut self,
        method: HttpMethod,
        url: &str,
        body: Option<&mut [u8]>,
    ) -> uefi::Result<()> {
        let url16 = uefi::CString16::try_from(url).unwrap();

        let scheme = url.split(':').next().unwrap_or("<missing>");
        let Some(hostname) = url.split('/').nth(2) else {
            return Err(Status::INVALID_PARAMETER.into());
        };
        let mut c_hostname = String::from(hostname);
        c_hostname.push('\0');
        debug!(
            "http: request setup: method={method:?}, scheme={scheme}, host={hostname}, body_len={}",
            body.as_ref().map_or(0, |body| body.len())
        );

        let mut tx_req = HttpRequestData {
            method,
            url: url16.as_ptr().cast::<u16>(),
        };

        let mut tx_hdr = Vec::new();
        tx_hdr.push(HttpHeader {
            field_name: c"Host".as_ptr().cast::<u8>(),
            field_value: c_hostname.as_ptr(),
        });

        let mut tx_msg = HttpMessage::default();
        tx_msg.data.request = &mut tx_req;
        tx_msg.header_count = tx_hdr.len();
        tx_msg.header = tx_hdr.as_mut_ptr();
        if let Some(body) = body {
            tx_msg.body_length = body.len();
            tx_msg.body = body.as_mut_ptr().cast::<c_void>();
        }

        let mut tx_token = HttpToken {
            status: Status::NOT_READY,
            message: &mut tx_msg,
            ..Default::default()
        };

        let p = self.protocol.as_mut().unwrap();
        let pending = PendingToken::request(p, &mut tx_token)?;
        debug!("http: request sent ok");

        let status = pending.wait()?;
        if status != Status::SUCCESS {
            return Err(status.into());
        };

        debug!("http: request status ok");

        Ok(())
    }

    /// Send HTTP GET request
    pub fn request_get(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::GET, url, None)?;
        Ok(())
    }

    /// Send HTTP HEAD request
    pub fn request_head(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::HEAD, url, None)?;
        Ok(())
    }

    /// Receive the start of the http response, the headers and (parts of) the
    /// body.
    ///
    /// Depending on the HTTP response, its length, its encoding, and its
    /// transmission method (chunked or not), users may have to call
    /// [`Self::response_more`] afterward.
    pub fn response_first(&mut self, expect_body: bool) -> uefi::Result<HttpHelperResponse> {
        let mut rx_rsp = HttpResponseData {
            status_code: HttpStatusCode::STATUS_UNSUPPORTED,
        };

        let mut body = vec![0; if expect_body { 16 * 1024 } else { 0 }];
        let mut rx_msg = HttpMessage::default();
        // The firmware writes the status code through this pointer, so it
        // must carry write permission although the field is `*const` in the
        // spec.
        rx_msg.data.response = ptr::from_mut(&mut rx_rsp).cast_const();
        rx_msg.body_length = body.len();
        rx_msg.body = if !body.is_empty() {
            body.as_mut_ptr()
        } else {
            ptr::null()
        } as *mut c_void;

        let mut rx_token = HttpToken {
            status: Status::NOT_READY,
            message: &mut rx_msg,
            ..Default::default()
        };

        let p = self.protocol.as_mut().unwrap();
        let status = PendingToken::response(p, &mut rx_token)?.wait()?;

        debug!("http: response: {status} / {:?}", rx_rsp.status_code);

        if status != Status::SUCCESS && status != Status::HTTP_ERROR {
            // SAFETY: `rx_msg` was filled by the driver.
            unsafe { free_response_headers(&rx_msg) };
            return Err(status.into());
        };

        debug!("http: headers: {}", rx_msg.header_count);
        let mut headers: Vec<(String, String)> = Vec::new();
        for i in 0..rx_msg.header_count {
            let n;
            let v;
            // SAFETY: The memory is valid.
            unsafe {
                n = CStr::from_ptr((*rx_msg.header.add(i)).field_name.cast::<c_char>());
                v = CStr::from_ptr((*rx_msg.header.add(i)).field_value.cast::<c_char>());
            }
            // The strings come from the server and are not guaranteed to be
            // UTF-8.
            headers.push((
                String::from_utf8_lossy(n.to_bytes()).to_lowercase(),
                String::from_utf8_lossy(v.to_bytes()).into_owned(),
            ));
        }
        // SAFETY: `rx_msg` was filled by the driver.
        unsafe { free_response_headers(&rx_msg) };

        debug!("http: body: {}/{}", rx_msg.body_length, body.len());

        let rsp = HttpHelperResponse {
            status: rx_rsp.status_code,
            headers,
            body: body[0..rx_msg.body_length].to_vec(),
        };
        Ok(rsp)
    }

    /// Try to receive more of the HTTP response and append any new data to the
    /// provided  `body` vector.
    pub fn response_more<'a>(&mut self, body: &'a mut Vec<u8>) -> uefi::Result<&'a [u8]> {
        let mut body_recv_buffer = vec![0; 16 * 1024];
        let mut rx_msg = HttpMessage {
            body_length: body_recv_buffer.len(),
            body: body_recv_buffer.as_mut_ptr().cast::<c_void>(),
            ..Default::default()
        };

        let mut rx_token = HttpToken {
            status: Status::NOT_READY,
            message: &mut rx_msg,
            ..Default::default()
        };

        let p = self.protocol.as_mut().unwrap();
        let status = PendingToken::response(p, &mut rx_token)?.wait()?;

        debug!("http: response: {status}");

        if status != Status::SUCCESS {
            return Err(status.into());
        };

        debug!(
            "http: body: {}/{}",
            rx_msg.body_length,
            body_recv_buffer.len()
        );

        let new_data = &body_recv_buffer[0..rx_msg.body_length];
        body.extend(new_data);
        let new_data_slice = &body[body.len() - new_data.len()..];
        Ok(new_data_slice)
    }
}

impl Drop for HttpHelper {
    fn drop(&mut self) {
        // protocol must go out of scope before calling destroy_child
        self.protocol = None;
        let _ = self.binding.destroy_child(self.child_handle);
    }
}
