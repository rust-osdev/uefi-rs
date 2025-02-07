// SPDX-License-Identifier: MIT OR Apache-2.0

// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

use log::error;
// ANCHOR: use
use uefi::prelude::*;
use uefi::proto::shell_params::ShellParameters;
use uefi::{boot, println};

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main() -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi::helpers::init().unwrap();
    // ANCHOR_END: services

    // ANCHOR: params
    let shell_params =
        boot::open_protocol_exclusive::<ShellParameters>(boot::image_handle());
    let shell_params = match shell_params {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get ShellParameters protocol");
            return e.status();
        }
    };

    // Get as Vec of String, only with alloc feature
    let args: Vec<String> =
        shell_params.args().map(|x| x.to_string()).collect();
    println!("Args: {:?}", args);

    // Or without allocating, get a slice of the pointers
    println!("Num args: {}", args.len());
    if shell_params.args_len() > 1 {
        println!("First real arg: '{}'", args[1]);
    }
    // ANCHOR_END: params

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return
// ANCHOR_END: all
