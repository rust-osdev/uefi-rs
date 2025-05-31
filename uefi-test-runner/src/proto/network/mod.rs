// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing Network protocols");

    http::test();
    pxe::test();
    // Currently, we are in the unfortunate situation that the SNP test
    // depends on the PXE test, as it assigns an IPv4 address to the
    // interface via DHCP.
    snp::test();
}

mod http;
mod pxe;
mod snp;
