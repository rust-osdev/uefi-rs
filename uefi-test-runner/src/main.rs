#![no_std]
#![no_main]

#![feature(slice_patterns)]
#![feature(alloc)]
#![feature(asm)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

extern crate x86_64;

use uefi::prelude::*;

mod boot;
mod proto;

#[no_mangle]
pub extern "C" fn uefi_start(_handle: uefi::Handle, st: &'static SystemTable) -> Status {
    // Set up a hook to shut down QEMU when a panic occurs
    unsafe { uefi_services::set_panic_shutdown_hook(&qemu_panic_shutdown) }

    // Initialize logging.
    uefi_services::init(st);

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

    info!("UEFI {}.{}", major, minor);

    assert!(major >= 2, "Running on an old, unsupported version of UEFI");
    assert!(minor >= 30, "Old version of UEFI 2, some features might not be available.");
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

fn qemu_panic_shutdown() -> ! {
    use core::ptr;
    use x86_64::instructions::port::Port;

    // Sleep a bit to let the user see the error message. Using a busy loop like
    // this is admittedly crude, but it has two advantages over st.boot.stall():
    //
    // 1. It works even if boot-time services have been exited
    // 2. There is no need to track the state associated with the SystemTable
    //
    let mut dummy = 0u64;
    for i in 0..300_000_000 {
        unsafe { ptr::write_volatile(&mut dummy, i); }
    }

    // QEMU has been configured such that writing to the f4 port will abort...
    let mut port = Port::<u32>::new(0xf4);
    unsafe { port.write(42); }

    // ...so this code will never be reached
    loop {}
}