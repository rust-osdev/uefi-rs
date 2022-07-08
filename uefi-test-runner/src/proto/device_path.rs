use uefi::prelude::*;
use uefi::proto::device_path::{text::*, DevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::BootServices;

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running device path protocol test");

    let loaded_image = bt
        .open_protocol_exclusive::<LoadedImage>(image)
        .expect("Failed to open LoadedImage protocol");

    let device_path = bt
        .open_protocol_exclusive::<DevicePath>(loaded_image.device())
        .expect("Failed to open DevicePath protocol");

    let device_path_to_text = bt
        .open_protocol_exclusive::<DevicePathToText>(
            bt.get_handle_for_protocol::<DevicePathToText>()
                .expect("Failed to get DevicePathToText handle"),
        )
        .expect("Failed to open DevicePathToText protocol");

    let device_path_from_text = bt
        .open_protocol_exclusive::<DevicePathFromText>(
            bt.get_handle_for_protocol::<DevicePathFromText>()
                .expect("Failed to get DevicePathFromText handle"),
        )
        .expect("Failed to open DevicePathFromText protocol");

    for path in device_path.node_iter() {
        info!(
            "path: type={:?}, subtype={:?}, length={}",
            path.device_type(),
            path.sub_type(),
            path.length(),
        );

        let text = device_path_to_text
            .convert_device_node_to_text(bt, path, DisplayOnly(true), AllowShortcuts(false))
            .expect("Failed to convert device path to text");
        let text = &*text;
        info!("path name: {text}");

        let convert = device_path_from_text
            .convert_text_to_device_node(text)
            .expect("Failed to convert text to device path");
        assert_eq!(path, convert);
    }
}
