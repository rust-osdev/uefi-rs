use uefi::prelude::*;
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running device path protocol test");

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

    let device_path = bt
        .open_protocol::<DevicePath>(
            OpenProtocolParams {
                handle: loaded_image.device(),
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
        .expect("Failed to open DevicePath protocol");
    let device_path = unsafe { &*device_path.interface.get() };

    for path in device_path.iter() {
        info!(
            "path: type={:?}, subtype={:?}, length={}",
            path.device_type(),
            path.sub_type(),
            path.length(),
        );
    }
}
