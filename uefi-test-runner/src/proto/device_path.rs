use uefi::prelude::*;
use uefi::proto::device_path::DevicePath;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::BootServices;

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running device path protocol test");

    let loaded_image = bt
        .handle_protocol::<LoadedImage>(image)
        .expect_success("Failed to open LoadedImage protocol");
    let loaded_image = unsafe { &*loaded_image.get() };

    let device_path = bt
        .handle_protocol::<DevicePath>(loaded_image.device())
        .expect_success("Failed to open DevicePath protocol");
    let device_path = unsafe { &*device_path.get() };

    for path in device_path.iter() {
        info!(
            "path: type={:?}, subtype={:?}, length={}",
            path.device_type(),
            path.sub_type(),
            path.length(),
        );
    }
}
