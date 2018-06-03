#![no_std]
#![no_main]

#![feature(slice_patterns)]
#![feature(alloc)]
#![feature(asm)]

extern crate uefi;
extern crate uefi_services;
extern crate uefi_utils;

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

mod boot;
mod proto;
mod ucs2;

use uefi::{Handle, Status};
use uefi::table;

#[no_mangle]
pub extern "C" fn uefi_start(_handle: Handle, st: &'static table::SystemTable) -> Status {
    uefi_services::init(st);

    let stdout = st.stdout();
    stdout.reset(false).expect("Failed to reset stdout");

    // Switch to the maximum supported graphics mode.
    let best_mode = stdout.modes().last().unwrap();
    stdout.set_mode(best_mode).expect("Failed to change graphics mode");

    info!("# uefi-rs test runner");

    {
        let revision = st.uefi_revision();
        let (major, minor) = (revision.major(), revision.minor());

        info!("UEFI {}.{}.{}", major, minor / 10, minor % 10);
    }

    info!("");

    // Print all modes.
    for (index, mode) in stdout.modes().enumerate() {
        info!("Graphics mode #{}: {} rows by {} columns", index, mode.rows(), mode.columns());
    }

    info!("");

    {
        info!("Memory Allocation Test");

        let mut values = vec![-5, 16, 23, 4, 0];

        values.sort();

        info!("Sorted vector: {:?}", values);
    }

    info!("");

    let bt = st.boot;

    match boot::boot_services_test(bt) {
        Ok(_) => info!("Boot services test passed."),
        Err(status) => error!("Boot services test failed with status {:?}", status),
    }

    match proto::protocol_test(bt) {
        Ok(_) => info!("Protocol test passed."),
        Err(status) => error!("Protocol test failed with status {:?}", status),
    }

    match ucs2::ucs2_encoding_test() {
        Ok(_) => info!("UCS-2 encoding test passed."),
        Err(status) => error!("UCS-2 encoding test failed with status {:?}", status),
    }

    info!("");

    {
        let mut pointer = uefi_utils::proto::find_protocol::<uefi::proto::console::pointer::Pointer>()
            .expect("No pointer device was found");

        let pointer = unsafe { pointer.as_mut() };

        pointer.reset(false).expect("Failed to reset pointer device");

        if let Ok(state) = pointer.state() {
            info!("Pointer State: {:#?}", state);
        } else {
            error!("Failed to retrieve pointer state");
        }
    }

    bt.stall(10_000_000);

    let rt = st.runtime;
    rt.reset(table::runtime::ResetType::Shutdown, Status::Success, None);
}
