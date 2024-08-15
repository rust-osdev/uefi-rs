// Note: In Rust 1.82.0-nightly and before, the `uefi_std` feature is
// required for accessing `std::os::uefi::env::*`. The other default
// functionality doesn't need a nightly toolchain (with Rust 1.80 and later),
// but with that limited functionality you - currently - also can't integrate
// the `uefi` crate.
#![feature(uefi_std)]

use std::os::uefi as uefi_std;
use uefi::runtime::ResetType;
use uefi::{Handle, Status};

/// Performs the necessary setup code for the `uefi` crate.
fn setup_uefi_crate() {
    let st = uefi_std::env::system_table();
    let ih = uefi_std::env::image_handle();

    // Mandatory setup code for `uefi` crate.
    unsafe {
        uefi::table::set_system_table(st.as_ptr().cast());

        let ih = Handle::from_ptr(ih.as_ptr().cast()).unwrap();
        uefi::boot::set_image_handle(ih);
    }
}

fn main() {
    println!("Hello World from uefi_std");
    setup_uefi_crate();
    println!("UEFI-Version is {}", uefi::system::uefi_revision());
    uefi::runtime::reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}
