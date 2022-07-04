// ANCHOR: all
#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use log::info;
use uefi::prelude::*;
use uefi::proto::device_path::text::{
    AllowShortcuts, DevicePathToText, DisplayOnly,
};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::SearchType;
use uefi::{Identify, Result};

// ANCHOR: main
#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();

    print_image_path(boot_services).unwrap();

    boot_services.stall(10_000_000);
    Status::SUCCESS
}
// ANCHOR_END: main

// ANCHOR: print_image_path
fn print_image_path(boot_services: &BootServices) -> Result {
    // ANCHOR_END: print_image_path
    // ANCHOR: loaded_image
    let loaded_image = boot_services
        .open_protocol_exclusive::<LoadedImage>(boot_services.image_handle())?;
    // ANCHOR_END: loaded_image

    // ANCHOR: device_path
    let device_path_to_text_handle = *boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&DevicePathToText::GUID))?
        .handles()
        .first()
        .expect("DevicePathToText is missing");

    let device_path_to_text = boot_services
        .open_protocol_exclusive::<DevicePathToText>(
            device_path_to_text_handle,
        )?;
    // ANCHOR_END: device_path

    // ANCHOR: text
    let image_device_path =
        loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(
            boot_services,
            image_device_path,
            DisplayOnly(true),
            AllowShortcuts(false),
        )
        .expect("convert_device_path_to_text failed");

    info!("Image path: {}", &*image_device_path_text);
    Ok(())
}
// ANCHOR_END: text
// ANCHOR_END: all
