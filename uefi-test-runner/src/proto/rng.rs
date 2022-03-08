use core::mem;
use core::mem::size_of_val;
use uefi::prelude::*;
use uefi::proto::rng::{Rng, RngAlgorithm};
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};
use uefi::Guid;

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

    match rng.get_info(&mut list) {
        Ok(nb) => {
            for i in 0..nb.unwrap() {
                info!("OK {} : {}", nb.unwrap(), list[i].0)
            }
        }
        Err(e) => {
            error!("ERROR : {:#?}", e.status())
        }
    }
}
