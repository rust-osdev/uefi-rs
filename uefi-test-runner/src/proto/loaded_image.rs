// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;

pub fn test() {
    info!("Running loaded image protocol test");

    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())
        .expect("Failed to open LoadedImage protocol");

    let load_options = loaded_image.load_options_as_bytes();
    info!("LoadedImage options: {:?}", load_options);

    let (image_base, image_size) = loaded_image.info();
    info!(
        "LoadedImage image address: {:?}, image size: {} bytes",
        image_base, image_size
    );
}
