use alloc::string::ToString;
use alloc::vec::Vec;
use uefi::prelude::*;
use uefi::proto::device_path::text::*;
use uefi::proto::device_path::{DevicePath, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::BootServices;

pub fn test(image: Handle, bt: &BootServices) {
    info!("Running device path protocol test");

    // test 1/2: test low-level API by directly opening all protocols
    {
        let loaded_image = bt
            .open_protocol_exclusive::<LoadedImage>(image)
            .expect("Failed to open LoadedImage protocol");

        let device_path = bt
            .open_protocol_exclusive::<DevicePath>(loaded_image.device().unwrap())
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

        // Get the `LoadedImageDevicePath`. Verify it start with the same nodes as
        // `device_path`.
        let loaded_image_device_path = bt
            .open_protocol_exclusive::<LoadedImageDevicePath>(image)
            .expect("Failed to open LoadedImageDevicePath protocol");

        for (n1, n2) in device_path
            .node_iter()
            .zip(loaded_image_device_path.node_iter())
        {
            assert_eq!(n1, n2);
        }
    }

    // test 2/2: test high-level to-string api
    {
        let loaded_image_device_path = bt
            .open_protocol_exclusive::<LoadedImageDevicePath>(image)
            .expect("Failed to open LoadedImageDevicePath protocol");
        let device_path: &DevicePath = &loaded_image_device_path;

        let path_components = device_path
            .node_iter()
            .map(|node| node.to_string(bt, DisplayOnly(false), AllowShortcuts(false)))
            .map(|str| str.unwrap().unwrap().to_string())
            .collect::<Vec<_>>();

        let expected_device_path_str_components = &[
            "PciRoot(0x0)",
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            "Pci(0x1F,0x2)",
            #[cfg(target_arch = "aarch64")]
            "Pci(0x4,0x0)",
            // Sata device only used on x86.
            // See xtask utility.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            "Sata(0x0,0xFFFF,0x0)",
            "HD(1,MBR,0xBE1AFDFA,0x3F,0xFBFC1)",
            "\\efi\\boot\\test_runner.efi",
        ];
        let expected_device_path_str = expected_device_path_str_components.join("/");

        assert_eq!(
            path_components.as_slice(),
            expected_device_path_str_components
        );

        // Test that to_string works for device_paths
        let path = device_path
            .to_string(bt, DisplayOnly(false), AllowShortcuts(false))
            .unwrap()
            .unwrap()
            .to_string();

        assert_eq!(path, expected_device_path_str);
    }
}
