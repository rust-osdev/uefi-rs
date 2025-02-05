// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::string::ToString;
use uefi::boot::{LoadImageSource, SearchType};
use uefi::fs::FileSystem;
use uefi::proto::console::text::Output;
use uefi::proto::device_path::media::FilePath;
use uefi::proto::device_path::{DevicePath, LoadedImageDevicePath};
use uefi::proto::BootPolicy;
use uefi::{boot, CString16, Identify};

mod memory;
mod misc;

pub fn test() {
    info!("Testing boot services");
    memory::test();
    misc::test();
    test_locate_handles();
    test_load_image();
}

fn test_locate_handles() {
    info!("Testing the `locate_handle_buffer`/`find_handles` functions");

    {
        // search all handles
        let handles = boot::locate_handle_buffer(SearchType::AllHandles)
            .expect("Failed to locate handle buffer");
        assert!(!handles.is_empty(), "Could not find any handles");
    }

    {
        // search by protocol
        let handles = boot::locate_handle_buffer(SearchType::ByProtocol(&Output::GUID))
            .expect("Failed to locate handle buffer");
        assert!(
            !handles.is_empty(),
            "Could not find any OUTPUT protocol handles"
        );

        // Compare with `boot::find_handles`. This implicitly tests
        // `boot::locate_handle` as well.
        let handles_vec = boot::find_handles::<Output>().unwrap();
        assert_eq!(*handles, handles_vec);
    }
}

/// This test loads the "self image" again into memory using the `load_image`
/// boot service function. The image is not started but just loaded into memory.
///
/// It transitively tests the protocol [`LoadedImageDevicePath`] which is
/// required as helper.
fn test_load_image() {
    /// The path of the loaded image executing this integration test.
    const LOADED_IMAGE_PATH: &str = r"\EFI\BOOT\TEST_RUNNER.EFI";

    info!("Testing the `load_image` function");

    let image_device_path_protocol =
        boot::open_protocol_exclusive::<LoadedImageDevicePath>(boot::image_handle())
            .expect("should open LoadedImage protocol");

    // Note: This is the full device path. The LoadedImage protocol would only
    // provide us with the file-path portion of the device path.
    let image_device_path: &DevicePath = &image_device_path_protocol;

    // Get the file-path portion of the device path which is typically behind
    // device path node (0x4, 0x4). The string is in upper case.

    let image_device_path_file_path = image_device_path
        .node_iter()
        .find_map(|node| {
            let node: &FilePath = node.try_into().ok()?;
            let path = node.path_name().to_cstring16().ok()?;
            Some(path.to_string().to_uppercase())
        })
        .expect("should have file-path portion in device path");

    assert_eq!(image_device_path_file_path.as_str(), LOADED_IMAGE_PATH);

    // Variant A: FromBuffer
    {
        let fs =
            boot::get_image_file_system(boot::image_handle()).expect("should open file system");
        let path = CString16::try_from(image_device_path_file_path.as_str()).unwrap();
        let image_data = FileSystem::new(fs)
            .read(&*path)
            .expect("should read file content");
        let load_source = LoadImageSource::FromBuffer {
            buffer: image_data.as_slice(),
            file_path: None,
        };
        let loaded_image =
            boot::load_image(boot::image_handle(), load_source).expect("should load image");

        log::debug!("load_image with FromBuffer strategy works");

        // Check that the `LoadedImageDevicePath` protocol can be opened and
        // that the interface data is `None`.
        let loaded_image_device_path =
            boot::open_protocol_exclusive::<LoadedImageDevicePath>(loaded_image)
                .expect("should open LoadedImageDevicePath protocol");
        assert!(loaded_image_device_path.get().is_none());
    }
    // Variant B: FromDevicePath
    {
        let load_source = LoadImageSource::FromDevicePath {
            device_path: image_device_path,
            boot_policy: BootPolicy::ExactMatch,
        };
        let _ = boot::load_image(boot::image_handle(), load_source).expect("should load image");

        log::debug!("load_image with FromFilePath strategy works");
    }
}
