use uefi::prelude::*;
use uefi::proto::rng::Rng;
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running rng protocol test");

    let handle = *bt
        .find_handles::<Rng>()
        .expect_success("Failed to get Rng handles")
        .first()
        .expect("No Rng handles");

    let rng = bt
        .open_protocol::<Rng>(
            OpenProtocolParams {
                handle,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
        .expect_success("Failed to open Rng protocol");
    let _rng = unsafe { &*rng.interface.get() };
}
