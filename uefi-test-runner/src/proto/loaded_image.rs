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
        .expect_success("Failed to open LoadedImage protocol");
    let loaded_image = unsafe { &*loaded_image.interface.get() };

    let mut buffer = vec![0; 128];
    let load_options = loaded_image
        .load_options(&mut buffer)
        .expect("Failed to get load options");
    info!("LoadedImage options: \"{}\"", load_options);

    let (image_base, image_size) = loaded_image.info();
    info!(
        "LoadedImage image address: {:?}, image size: {} bytes",
        image_base, image_size
    );
}
