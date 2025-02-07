// SPDX-License-Identifier: MIT OR Apache-2.0

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
use uefi::boot::{self, LoadImageSource};
use uefi::prelude::*;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::{DevicePath, DeviceSubType, DeviceType, LoadedImageDevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::BootPolicy;

/// Get the device path of the shell app. This is the same as the
/// currently-loaded image's device path, but with the file path part changed.
fn get_shell_app_device_path(storage: &mut Vec<u8>) -> &DevicePath {
    let loaded_image_device_path =
        boot::open_protocol_exclusive::<LoadedImageDevicePath>(boot::image_handle())
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
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    let mut storage = Vec::new();
    let shell_image_path = get_shell_app_device_path(&mut storage);

    // Load the shell app.
    let shell_image_handle = boot::load_image(
        boot::image_handle(),
        LoadImageSource::FromDevicePath {
            device_path: shell_image_path,
            boot_policy: BootPolicy::ExactMatch,
        },
    )
    .expect("failed to load shell app");

    // Set the command line passed to the shell app so that it will run the
    // test-runner app. This automatically turns off the five-second delay.
    let mut shell_loaded_image = boot::open_protocol_exclusive::<LoadedImage>(shell_image_handle)
        .expect("failed to open LoadedImage protocol");
    let load_options = cstr16!(r"shell.efi test_runner.efi arg1 arg2");
    unsafe {
        shell_loaded_image.set_load_options(
            load_options.as_ptr().cast(),
            load_options.num_bytes() as u32,
        );
    }

    info!("launching the shell app");
    boot::start_image(shell_image_handle).expect("failed to launch the shell app");

    Status::SUCCESS
}
