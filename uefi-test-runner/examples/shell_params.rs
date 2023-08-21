// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

use log::error;
// ANCHOR: use
//use log::info;
use uefi::CStr16;
use uefi::{prelude::*, proto::shell_params::ShellParameters};
use uefi_services::println;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    // ANCHOR_END: services

    // ANCHOR: params
    let shell_params =
        boot_services.open_protocol_exclusive::<ShellParameters>(image_handle);
    let shell_params = match shell_params {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get ShellParameters protocol");
            return e.status();
        }
    };

    // Get as Vec of String, only with alloc feature
    let args: Vec<String> = shell_params.get_args().collect();
    println!("Args: {:?}", args);

    // Or without allocating, get a slice of the pointers
    let args = shell_params.get_args_slice();
    println!("Num args: {}", args.len());
    if args.len() > 1 {
        unsafe {
            println!("First real arg: '{}'", CStr16::from_ptr(args[1]));
        }
    }
    // ANCHOR_END: params

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return
// ANCHOR_END: all
