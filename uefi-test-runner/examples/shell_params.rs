// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

use log::error;
// ANCHOR: use
//use log::info;
use uefi::CStr16;
use uefi::{
    prelude::*,
    proto::shell_params::ShellParameters,
    table::boot::{OpenProtocolAttributes, OpenProtocolParams, SearchType},
    Identify,
};
use uefi_services::println;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi_services::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    // ANCHOR_END: services

    // ANCHOR: params
    let shell_params_h = boot_services
        .locate_handle_buffer(SearchType::ByProtocol(&ShellParameters::GUID));
    let shell_params_h = match shell_params_h {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get ShellParameters protocol");
            return e.status();
        }
    };
    println!("Found {} ShellParams handles", (*shell_params_h).len());
    for handle in &*shell_params_h {
        let params_handle = unsafe {
            boot_services
                .open_protocol::<ShellParameters>(
                    OpenProtocolParams {
                        handle: *handle,
                        agent: boot_services.image_handle(),
                        controller: None,
                    },
                    OpenProtocolAttributes::GetProtocol,
                )
                .expect("Failed to open ShellParams handle")
        };

        // TODO: Ehm why are there two and one has no args?
        // Maybe one is the shell itself?
        if params_handle.argc == 0 {
            continue;
        }

        // Get as Vec of String, only with alloc feature
        let args: Vec<String> = params_handle.get_args().collect();
        println!("Args: {:?}", args);

        // Or without allocating, get a slice of the pointers
        let args = params_handle.get_args_slice();
        println!("Num args: {}", args.len());
        if args.len() > 1 {
            unsafe {
                println!("First real arg: '{}'", CStr16::from_ptr(args[1]));
            }
        }
    }
    // ANCHOR_END: params

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return
// ANCHOR_END: all
