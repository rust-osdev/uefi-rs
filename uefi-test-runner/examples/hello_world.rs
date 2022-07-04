// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
#![feature(abi_efiapi)]
// ANCHOR_END: features

// ANCHOR: use
use log::info;
use uefi::prelude::*;
// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi_services::init(&mut system_table).unwrap();
    // ANCHOR_END: services
    // ANCHOR: log
    info!("Hello world!");
    system_table.boot_services().stall(10_000_000);
    // ANCHOR_END: log
    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return
// ANCHOR_END: all
