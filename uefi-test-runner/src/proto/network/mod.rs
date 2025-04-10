// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing Network protocols");

    http::test();
    pxe::test();
    snp::test();
}

mod http;
mod pxe;
mod snp;
