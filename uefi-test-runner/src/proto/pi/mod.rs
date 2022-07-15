use uefi::prelude::*;

pub fn test(image: Handle, bt: &BootServices) {
    info!("Testing Platform Initialization protocols");

    mp::test(image, bt);
}

mod mp;
