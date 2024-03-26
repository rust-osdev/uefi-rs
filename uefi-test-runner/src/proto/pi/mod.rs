use uefi::prelude::*;

pub fn test(bt: &BootServices) {
    info!("Testing Platform Initialization protocols");

    mp::test(bt);
}

mod mp;
