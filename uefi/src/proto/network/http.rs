// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg(feature = "alloc")]

//! UEFI HTTP protocol.
//!
//! See [`Http`].

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char, c_void};
use core::ptr;
use log::debug;

use uefi::boot::ScopedProtocol;
use uefi::prelude::*;
use uefi::proto::unsafe_protocol;
use uefi_raw::protocol::driver::ServiceBindingProtocol;
use uefi_raw::protocol::network::http::{
    HttpAccessPoint, HttpConfigData, HttpHeader, HttpMessage, HttpMethod, HttpProtocol,
    HttpRequestData, HttpResponseData, HttpStatusCode, HttpToken, HttpV4AccessPoint, HttpVersion,
};

/// Sends and receives HTTP messages through the UEFI HTTP protocol.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::GUID)]
pub struct Http(HttpProtocol);

impl Http {
    /// Returns the current HTTP configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot report the configuration.
    pub fn get_mode_data(&mut self) -> uefi::Result<HttpConfigData> {
        let mut config_data = HttpConfigData::default();
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.get_mode_data)(&mut self.0, &mut config_data) };
        match status {
            Status::SUCCESS => Ok(config_data),
            _ => Err(status.into()),
        }
    }

    /// Configures the HTTP protocol.
    ///
    /// This must be called before sending requests.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware rejects the configuration or cannot
    /// initialize the network stack.
    pub fn configure(&mut self, config_data: &HttpConfigData) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.configure)(&mut self.0, config_data) };
        debug!("http raw: configure({config_data:?}) -> {status}");
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Queues an HTTP request.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or firmware cannot queue the
    /// request.
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

    /// Cancels an HTTP transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or firmware cannot cancel the
    /// transaction.
    pub fn cancel(&mut self, token: &mut HttpToken) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.cancel)(&mut self.0, token) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }

    /// Queues an HTTP response operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is invalid or firmware cannot queue the
    /// response operation.
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

    /// Polls the network stack for progress.
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is not configured or polling fails.
    pub fn poll(&mut self) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.poll)(&mut self.0) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

/// HTTP service-binding protocol.
#[derive(Debug)]
#[unsafe_protocol(HttpProtocol::SERVICE_BINDING_GUID)]
pub struct HttpBinding(ServiceBindingProtocol);

impl HttpBinding {
    /// Creates a child handle with an HTTP protocol instance.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot create the child handle.
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

    /// Destroys an HTTP child handle.
    ///
    /// # Errors
    ///
    /// Returns an error if `handle` is not a child of this binding or firmware
    /// cannot destroy it.
    pub fn destroy_child(&mut self, handle: Handle) -> uefi::Result<()> {
        // SAFETY: The memory is valid.
        let status = unsafe { (self.0.destroy_child)(&mut self.0, handle.as_ptr()) };
        match status {
            Status::SUCCESS => Ok(()),
            _ => Err(status.into()),
        }
    }
}

/// Response returned by [`HttpHelper`].
///
/// Helper type for [`HttpHelper`].
#[derive(Debug)]
pub struct HttpHelperResponse {
    /// HTTP status code.
    pub status: HttpStatusCode,
    /// HTTP response headers.
    pub headers: Vec<(String, String)>,
    /// Partial or entire HTTP body, depending on context.
    pub body: Vec<u8>,
}

/// Convenience wrapper around the UEFI [HTTP] [`Protocol`].
///
/// [HTTP]: Http
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
pub struct HttpHelper {
    child_handle: Handle,
    binding: ScopedProtocol<HttpBinding>,
    protocol: Option<ScopedProtocol<Http>>,
}

impl HttpHelper {
    /// Creates an HTTP helper for a network-interface handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the service binding cannot be opened, a child cannot
    /// be created, or its HTTP protocol cannot be opened.
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

    /// Configures the HTTP protocol with IPv4 defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware rejects the configuration or cannot
    /// initialize the network stack.
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

    /// Sends an HTTP request and waits for completion.
    ///
    /// # Arguments
    ///
    /// - `method`: HTTP method to send.
    /// - `url`: Absolute URL, including a host component.
    /// - `body`: Optional request body.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL has no host, firmware rejects the request,
    /// or polling fails.
    ///
    /// # Panics
    ///
    /// Panics if `url` cannot be represented as a null-terminated UCS-2 string.
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
        p.request(&mut tx_token)?;
        debug!("http: request sent ok");

        let mut polls = 0;
        loop {
            if tx_token.status != Status::NOT_READY {
                break;
            }
            polls += 1;
            p.poll()?;
        }
        debug!(
            "http: request token completed after {polls} polls with {}",
            tx_token.status
        );

        if tx_token.status != Status::SUCCESS {
            return Err(tx_token.status.into());
        };

        debug!("http: request status ok");

        Ok(())
    }

    /// Sends an HTTP GET request and waits for completion.
    ///
    /// # Errors
    ///
    /// Returns the errors documented by [`Self::request`].
    pub fn request_get(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::GET, url, None)?;
        Ok(())
    }

    /// Sends an HTTP HEAD request and waits for completion.
    ///
    /// # Errors
    ///
    /// Returns the errors documented by [`Self::request`].
    pub fn request_head(&mut self, url: &str) -> uefi::Result<()> {
        self.request(HttpMethod::HEAD, url, None)?;
        Ok(())
    }

    /// Receives the response status, headers, and initial body data.
    ///
    /// Depending on the HTTP response, its length, its encoding, and its
    /// transmission method (chunked or not), users may have to call
    /// [`Self::response_more`] afterward.
    ///
    /// # Arguments
    ///
    /// - `expect_body`: Whether to allocate a buffer for initial body data.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot receive the response or polling
    /// fails.
    pub fn response_first(&mut self, expect_body: bool) -> uefi::Result<HttpHelperResponse> {
        let mut rx_rsp = HttpResponseData {
            status_code: HttpStatusCode::STATUS_UNSUPPORTED,
        };

        let mut body = vec![0; if expect_body { 16 * 1024 } else { 0 }];
        let mut rx_msg = HttpMessage::default();
        rx_msg.data.response = &mut rx_rsp;
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
        p.response(&mut rx_token)?;

        loop {
            if rx_token.status != Status::NOT_READY {
                break;
            }
            p.poll()?;
        }

        debug!(
            "http: response: {} / {:?}",
            rx_token.status, rx_rsp.status_code
        );

        if rx_token.status != Status::SUCCESS && rx_token.status != Status::HTTP_ERROR {
            return Err(rx_token.status.into());
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
            headers.push((
                n.to_str().unwrap().to_lowercase(),
                String::from(v.to_str().unwrap()),
            ));
        }

        debug!("http: body: {}/{}", rx_msg.body_length, body.len());

        let rsp = HttpHelperResponse {
            status: rx_rsp.status_code,
            headers,
            body: body[0..rx_msg.body_length].to_vec(),
        };
        Ok(rsp)
    }

    /// Appends the next response-body chunk to `body`.
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot receive more data or polling fails.
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
        p.response(&mut rx_token)?;

        loop {
            if rx_token.status != Status::NOT_READY {
                break;
            }
            p.poll()?;
        }

        debug!("http: response: {}", rx_token.status);

        if rx_token.status != Status::SUCCESS {
            return Err(rx_token.status.into());
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
