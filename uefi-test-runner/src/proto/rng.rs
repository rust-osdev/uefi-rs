// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::proto::rng::{Rng, RngAlgorithmType};

pub fn test() {
    info!("Running rng protocol test");

    let handle = boot::get_handle_for_protocol::<Rng>().expect("No Rng handles");

    let mut rng =
        boot::open_protocol_exclusive::<Rng>(handle).expect("Failed to open Rng protocol");

    let mut list = [RngAlgorithmType::EMPTY_ALGORITHM; 4];

    let list = rng.get_info(&mut list).unwrap();
    info!("Supported rng algorithms : {:?}", list);

    let mut buf = [0u8; 4];

    rng.get_rng(Some(list[0]), &mut buf).unwrap();

    assert_ne!([0u8; 4], buf);
    info!("Random buffer : {:?}", buf);
}
