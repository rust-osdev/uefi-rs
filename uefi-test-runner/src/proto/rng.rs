use uefi::prelude::*;
use uefi::proto::rng::{Rng, RngAlgorithmType};
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running rng protocol test");

    let handle = *bt
        .find_handles::<Rng>()
        .expect("Failed to get Rng handles")
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
        .expect("Failed to open Rng protocol");
    let rng = unsafe { &mut *rng.interface.get() };

    let mut list = [RngAlgorithmType::EMPTY_ALGORITHM; 4];

    let list = rng.get_info(&mut list).unwrap();
    info!("Supported rng algorithms : {:?}", list);

    let mut buf = [0u8; 4];

    rng.get_rng(Some(list[0]), &mut buf).unwrap();

    assert_ne!([0u8; 4], buf);
    info!("Random buffer : {:?}", buf);
}
