// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;

use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::device_path::DevicePath;
use uefi::proto::network::http::{HttpBinding, HttpHelper};
use uefi::proto::network::ip4config2::Ip4Config2;
use uefi::{boot, Handle};

use uefi_raw::protocol::network::http::HttpStatusCode;

pub fn print_handle_devpath(prefix: &str, handle: &Handle) {
    let Ok(dp) = boot::open_protocol_exclusive::<DevicePath>(*handle) else {
        info!("{}no device path for handle", prefix);
        return;
    };
    if let Ok(string) = dp.to_string(DisplayOnly(true), AllowShortcuts(true)) {
        info!("{}{}", prefix, string);
    }
}

fn fetch_http(handle: Handle, url: &str) -> Option<Vec<u8>> {
    info!("http: fetching {} ...", url);

    let http_res = HttpHelper::new(handle);
    if let Err(e) = http_res {
        error!("http new: {}", e);
        return None;
    }
    let mut http = http_res.unwrap();

    let res = http.configure();
    if let Err(e) = res {
        error!("http configure: {}", e);
        return None;
    }

    let res = http.request_get(url);
    if let Err(e) = res {
        error!("http request: {}", e);
        return None;
    }

    let res = http.response_first(true);
    if let Err(e) = res {
        error!("http response: {}", e);
        return None;
    }

    let rsp = res.unwrap();
    if rsp.status != HttpStatusCode::STATUS_200_OK {
        error!("http server error: {:?}", rsp.status);
        return None;
    }
    let Some(cl_hdr) = rsp.headers.iter().find(|h| h.0 == "content-length") else {
        // The only way to figure when your transfer is complete is to
        // get the content length header and count the bytes you got.
        // So missing header -> fatal error.
        error!("no content length");
        return None;
    };
    let Ok(cl) = cl_hdr.1.parse::<usize>() else {
        error!("parse content length ({})", cl_hdr.1);
        return None;
    };
    info!("http: size is {} bytes", cl);

    let mut data = rsp.body;
    loop {
        if data.len() >= cl {
            break;
        }

        let res = http.response_more();
        if let Err(e) = res {
            error!("read response: {}", e);
            return None;
        }

        let mut buf = res.unwrap();
        data.append(&mut buf);
    }

    Some(data)
}

pub fn test() {
    info!("Testing ip4 config2 + http protocols");

    let handles = boot::locate_handle_buffer(boot::SearchType::from_proto::<HttpBinding>())
        .expect("get nic handles");

    for h in handles.as_ref() {
        print_handle_devpath("nic: ", h);

        info!("Bring up interface (ip4 config2 protocol)");
        let mut ip4 = Ip4Config2::new(*h).expect("open ip4 config2 protocol");
        ip4.ifup(true).expect("acquire ipv4 address");

        // hard to find web sites which still allow plain http these days ...
        info!("Testing HTTP");
        fetch_http(*h, "http://example.com/").expect("http request failed");

        // FYI: not all firmware builds support modern tls versions.
        // request() -> ABORTED typically is a tls handshake error.
        // check the firmware log for details.
        info!("Testing HTTPS");
        fetch_http(
            *h,
            "https://raw.githubusercontent.com/rust-osdev/uefi-rs/refs/heads/main/Cargo.toml",
        )
        .expect("https request failed");

        info!("PASSED");
    }
}
