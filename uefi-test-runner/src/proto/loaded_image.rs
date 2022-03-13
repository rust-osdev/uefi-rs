use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running loaded image protocol test");

    let loaded_image = bt
        .open_protocol::<LoadedImage>(
            OpenProtocolParams {
                handle: image,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
        .expect("Failed to open LoadedImage protocol");
    let loaded_image = unsafe { &*loaded_image.interface.get() };

    let load_options = loaded_image.load_options_as_bytes();
    info!("LoadedImage options: {:?}", load_options);

    let (image_base, image_size) = loaded_image.info();
    info!(
        "LoadedImage image address: {:?}, image size: {} bytes",
        image_base, image_size
    );
}
