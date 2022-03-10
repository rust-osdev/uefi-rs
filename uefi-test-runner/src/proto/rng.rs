use uefi::prelude::*;
use uefi::proto::rng::{Rng, RngAlgorithm};
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
    let rng = unsafe { &mut *rng.interface.get() };

    let mut list = [RngAlgorithm::default(); 4];

    rng.get_info(&mut list).unwrap_success();

    let mut buf = [0u8; 4];

    rng.get_rng(Some(list[0]), &mut buf).unwrap_success();

    assert_ne!([0u8; 4], buf);
}
