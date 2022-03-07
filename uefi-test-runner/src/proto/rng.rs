use uefi::prelude::*;
use uefi::proto::rng::Rng;
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running loaded image protocol test");

    let rng = bt
        .open_protocol::<Rng>(
            OpenProtocolParams {
                handle: image,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
        .expect_success("Failed to open LoadedImage protocol");
    let _rng = unsafe { &*rng.interface.get() };

    info!("Rng loaded !");
}