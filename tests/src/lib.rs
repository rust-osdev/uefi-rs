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

static mut LOGGER: Option<uefi_logger::Logger> = None;

#[no_mangle]
pub extern "C" fn uefi_start(handle: Handle, st: &'static table::SystemTable) -> Status {
    let stdout = st.stdout();
    stdout.reset(false).unwrap();

    // Switch to the maximum supported graphics mode.
    let best_mode = stdout.modes().last().unwrap();
    stdout.set_mode(best_mode).unwrap();

    // Construct the logger.
    let logger = unsafe {
        LOGGER = Some(uefi_logger::Logger::new(stdout));

        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Log everything.
    log::set_max_level(log::LevelFilter::Info);

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
