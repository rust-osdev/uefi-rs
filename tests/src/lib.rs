#![no_std]

#![feature(lang_items)]
#![feature(compiler_builtins_lib)]

extern crate uefi;
extern crate uefi_logger;

#[macro_use]
extern crate log;

mod no_std;
mod boot;

use uefi::{Handle, Status};
use uefi::table;

#[no_mangle]
pub extern "C" fn uefi_start(handle: Handle, st: &'static table::SystemTable) -> Status {
    let stdout = st.stdout();
    stdout.reset(false).unwrap();

    let logger = uefi_logger::UefiLogger::new(stdout);

    unsafe {
        log::set_logger_raw(|log_level| {
            // Log everything.
            log_level.set(log::LogLevelFilter::Info);

            &logger
        }).unwrap(); // Can only fail if already initialized.
    }

    info!("# uefi-rs test runner");
    info!("Image handle: {:?}", handle);

    {
        let revision = st.uefi_revision();
        let (major, minor) = (revision.major(), revision.minor());

        info!("UEFI {}.{}.{}", major, minor / 10, minor % 10);
    }

    let bt = st.boot;

    match boot::boot_services_test(bt) {
        Ok(_) => info!("Boot services test passed."),
        Err(status) => error!("Boot services test failed with status {:?}", status),
    }

    bt.stall(4_000_000);

    let rt = st.runtime;
    rt.reset(table::runtime::ResetType::Shutdown, Status::Success, None);
}
