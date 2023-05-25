//! This application launches the UEFI shell app and runs the main
//! uefi-test-running app inside that shell. This allows testing of protocols
//! that require the shell.
//!
//! Launching the shell this way (rather than directly making it the boot
//! executable) makes it possible to avoid the shell's built-in five second
//! startup delay.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::{DevicePath, DeviceSubType, DeviceType, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::LoadImageSource;
use uefi::Status;

/// Get the device path of the shell app. This is the same as the
/// currently-loaded image's device path, but with the file path part changed.
fn get_shell_app_device_path<'a>(
    boot_services: &BootServices,
    storage: &'a mut Vec<u8>,
) -> &'a DevicePath {
    let loaded_image_device_path = boot_services
        .open_protocol_exclusive::<LoadedImageDevicePath>(boot_services.image_handle())
        .expect("failed to open LoadedImageDevicePath protocol");

    let mut builder = DevicePathBuilder::with_vec(storage);
    for node in loaded_image_device_path.node_iter() {
        if node.full_type() == (DeviceType::MEDIA, DeviceSubType::MEDIA_FILE_PATH) {
            break;
        }
        builder = builder.push(&node).unwrap();
    }
    builder = builder
        .push(&build::media::FilePath {
            path_name: cstr16!(r"efi\boot\shell.efi"),
        })
        .unwrap();
    builder.finalize().unwrap()
}

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    let boot_services = st.boot_services();

    let mut storage = Vec::new();
    let shell_image_path = get_shell_app_device_path(boot_services, &mut storage);

    // Load the shell app.
    let shell_image_handle = boot_services
        .load_image(
            image,
            LoadImageSource::FromDevicePath {
                device_path: shell_image_path,
                from_boot_manager: false,
            },
        )
        .expect("failed to load shell app");

    // Set the command line passed to the shell app so that it will run the
    // test-runner app. This automatically turns off the five-second delay.
    let mut shell_loaded_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(shell_image_handle)
        .expect("failed to open LoadedImage protocol");
    let load_options = cstr16!(r"shell.efi test_runner.efi");
    unsafe {
        shell_loaded_image.set_load_options(
            load_options.as_ptr().cast(),
            load_options.num_bytes() as u32,
        );
    }

    info!("launching the shell app");
    boot_services
        .start_image(shell_image_handle)
        .expect("failed to launch the shell app");

    Status::SUCCESS
}
