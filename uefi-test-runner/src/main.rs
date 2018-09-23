#![no_std]
#![no_main]
#![feature(slice_patterns)]
#![feature(alloc)]
#![feature(asm)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use uefi::prelude::*;

mod boot;
mod proto;

#[no_mangle]
pub extern "C" fn uefi_start(_handle: uefi::Handle, st: &'static SystemTable) -> Status {
    // Initialize logging.
    uefi_services::init(st);

    // Reset the console before running all the other tests.
    st.stdout().reset(false).expect("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());

    // Test all the boot services.
    let bt = st.boot;
    boot::test(bt);

    // TODO: test the runtime services.
    // We would have to call `exit_boot_services` first to ensure things work properly.

    // Test all the supported protocols.
    proto::test(st);

    shutdown(st);
}

fn check_revision(rev: uefi::table::Revision) {
    let (major, minor) = (rev.major(), rev.minor());

    info!("UEFI {}.{}", major, minor / 10);

    assert!(major >= 2, "Running on an old, unsupported version of UEFI");
    assert!(
        minor >= 30,
        "Old version of UEFI 2, some features might not be available."
    );
}

fn shutdown(st: &SystemTable) -> ! {
    use uefi::table::runtime::ResetType;

    // Get our text output back.
    st.stdout().reset(false).unwrap();

    // Inform the user.
    info!("Testing complete, shutting down in 3 seconds...");
    st.boot.stall(3_000_000);

    let rt = st.runtime;
    rt.reset(ResetType::Shutdown, Status::Success, None);
}
