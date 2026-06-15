// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;

use uefi::proto::network::http::{HttpBinding, HttpHelper};
use uefi::proto::network::ip4config2::Ip4Config2;
use uefi::{Handle, boot};

use uefi_raw::protocol::network::http::HttpStatusCode;

fn fetch_http(handle: Handle, url: &str) -> Option<Vec<u8>> {
    info!("http: fetching {url} ...");

    let http_res = HttpHelper::new(handle);
    if let Err(e) = http_res {
        error!("http new: {e}");
        return None;
    }
    let mut http = http_res.unwrap();

    let res = http.configure();
    if let Err(e) = res {
        error!("http configure: {e}");
        return None;
    }

    let res = http.request_get(url);
    if let Err(e) = res {
        error!("http request: {e}");
        return None;
    }

    let res = http.response_first(true);
    if let Err(e) = res {
        error!("http response: {e}");
        return None;
    }

    let rsp = res.unwrap();
    if rsp.status != HttpStatusCode::STATUS_200_OK {
        error!("http server error: {:?}", rsp.status);
        return None;
    }
    let cl_hdr = rsp.headers.iter().find(|h| h.0 == "content-length");
    if cl_hdr.is_none() {
        // The only way to figure when your transfer is complete is to
        // get the content length header and count the bytes you got.
        // So missing header -> give up and pretend things are okay.
        warn!("no content length header, we might not have the whole body");
        return Some(rsp.body);
    };
    let cl_hdr = cl_hdr.unwrap();
    let Ok(cl) = cl_hdr.1.parse::<usize>() else {
        error!("parse content length ({})", cl_hdr.1);
        return None;
    };
    info!("http: size is {cl} bytes");

    let mut data = rsp.body;
    loop {
        if data.len() >= cl {
            break;
        }

        let res = http.response_more(&mut data);
        if let Err(e) = res {
            error!("read response: {e}");
            return None;
        }
    }

    Some(data)
}

pub fn test() {
    info!("Testing ip4 config2 + http protocols");

    let handles = boot::locate_handle_buffer(boot::SearchType::from_proto::<HttpBinding>())
        .expect("get nic handles");

    for h in handles.as_ref() {
        info!("nic: {}", h.device_path().expect("should have device path"));

        info!("Bring up interface (ip4 config2 protocol)");
        let mut ip4 = Ip4Config2::new(*h).expect("open ip4 config2 protocol");
        ip4.ifup().expect("acquire ipv4 address");

        // hard to find web sites which still allow plain http these days ...
        info!("Testing HTTP");
        fetch_http(*h, "http://example.com/").expect("http request to http://example.com failed");

        // Since edk2-stable202511, the default OpenSSL security level has been
        // raised from 0 to 3, which rejects older RSA-based keys. Unfortunately,
        // the EDK2 aarch64 build forcefully disables all EC-based keys
        // (-DEDK2_OPENSSL_NOEC=1), effectively preventing connections to most
        // HTTPS/TLS hosts. Temporarily disable this test on aarch64 until
        // EC-based keys are supported there.
        //
        // See https://github.com/rust-osdev/uefi-rs/issues/1975
        #[cfg(not(target_arch = "aarch64"))]
        {
            // EDK2 uses platform-specific OpenSSL configurations, which can affect
            // certificate compatibility with HTTPS hosts. Because both our test
            // hosts and their certificates may change, try multiple candidates and
            // require at least one request to succeed.
            //
            // Not all firmware builds support modern tls versions.
            // request() -> ABORTED typically is a tls handshake error.
            // check the firmware log for details.
            let https_url_candidates = [
                "https://example.com/",
                "https://raw.githubusercontent.com/rust-osdev/uefi-rs/refs/heads/main/Cargo.toml",
                "https://www.cloudflare.com/",
                "https://www.google.com/",
            ];

            info!("Testing HTTPS");
            let https_results = https_url_candidates
                .iter()
                .map(|url| (url, fetch_http(*h, url)))
                .collect::<Vec<_>>();
            for (url, res) in &https_results {
                debug!(
                    "HTTPS request to: {url}: {}",
                    if res.is_some() { "OK" } else { "FAILED" }
                );
            }
            assert!(
                https_results.iter().any(|(_, res)| res.is_some()),
                "No HTTPS request succeeded"
            );
        }

        #[cfg(target_arch = "aarch64")]
        {
            // See https://github.com/rust-osdev/uefi-rs/issues/1975
            warn!("Skipping HTTPS test on aarch64 (see #1975)");
        }

        info!("PASSED");
    }
}
