// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing Network protocols");

    pxe::test();
    snp::test();
}

mod pxe;
mod snp;
