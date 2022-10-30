use uefi::prelude::*;

pub fn test(bt: &BootServices) {
    info!("Testing Network protocols");

    pxe::test(bt);
    snp::test(bt);
}

mod pxe;
mod snp;
