// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing Network protocols");

    http::test();
    #[cfg(feature = "pxe")]
    {
        pxe::test();
        // Currently, we are in the unfortunate situation that the SNP test
        // depends on the PXE test, as it assigns an IPv4 address to the
        // interface.
        snp::test();
    }
}

mod http;
#[cfg(feature = "pxe")]
mod pxe;
#[cfg(feature = "pxe")]
mod snp;
