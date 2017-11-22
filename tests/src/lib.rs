#![no_std]

#![feature(lang_items)]
#![feature(compiler_builtins_lib)]

extern crate uefi;
extern crate uefi_logger;

#[macro_use]
extern crate log;

mod no_std;

use uefi::{Handle, Status};
use uefi::table;

#[no_mangle]
pub extern fn uefi_start(handle: Handle, st: &'static table::SystemTable) -> Status {
    let stdout = st.stdout();
    stdout.reset(false);

    let logger = uefi_logger::UefiLogger::new(stdout);

    unsafe {
        log::set_logger_raw(|log_level| {
            // Log everything.
            log_level.set(log::LogLevelFilter::Info);

            &logger
        }).unwrap(); // Can only fail if already initialized.
    }

    info!("Hello world!");
    info!("Image handle: {:?}", handle);
    {
        let revision = st.uefi_revision();
        let (major, minor) = (revision.major(), revision.minor());

        info!("UEFI {}.{}.{}", major, minor / 10, minor % 10);
    }

    loop {

    }

    Status::Success
}
