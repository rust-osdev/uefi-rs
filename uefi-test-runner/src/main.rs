#![no_std]
#![no_main]
#![feature(alloc)]
#![feature(asm)]
#![feature(const_slice_len)]
#![feature(slice_patterns)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use uefi::prelude::*;
use uefi::proto::console::serial::Serial;
use uefi_exts::BootServicesExt;

mod boot;
mod proto;

#[no_mangle]
pub extern "win64" fn uefi_start(_handle: uefi::Handle, st: &'static SystemTable) -> Status {
    // Initialize logging.
    uefi_services::init(st);

    // Reset the console before running all the other tests.
    st.stdout()
        .reset(false)
        .expect("Failed to reset stdout")
        .expect("Warnings encountered while resetting stdout");

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

/// Ask the test runner to check the current screen output against a reference
///
/// This functionality is very specific to our QEMU-based test runner. Outside
/// of it, we just pause the tests for a couple of seconds to allow visual
/// inspection of the output.
///
fn check_screenshot(bt: &BootServices, name: &str) {
    if cfg!(feature = "qemu") {
        // Access the serial port (in a QEMU environment, it should always be there)
        let mut serial = bt
            .find_protocol::<Serial>()
            .expect("Could not find serial port");
        let serial = unsafe { serial.as_mut() };

        // Set a large timeout to avoid problems
        let mut io_mode = *serial.io_mode();
        io_mode.timeout = 3_000_000;
        serial
            .set_attributes(&io_mode)
            .expect("Failed to configure serial port timeout")
            .expect("Warnings encountered while configuring serial port timeout");

        // Send a screenshot request to the host
        serial
            .write(b"SCREENSHOT: ")
            .expect("Failed to send request")
            .expect("Request triggered a warning");
        let name_bytes = name.as_bytes();
        serial
            .write(name_bytes)
            .expect("Failed to send request")
            .expect("Request triggered a warning");
        serial
            .write(b"\n")
            .expect("Failed to send request")
            .expect("Request triggered a warning");

        // Wait for the host's acknowledgement before moving forward
        let mut reply = [0; 3];
        serial
            .read(&mut reply[..])
            .expect("Failed to read host reply")
            .expect("Request triggered a warning");

        assert_eq!(&reply[..], b"OK\n", "Unexpected screenshot request reply");
    } else {
        // Outside of QEMU, give the user some time to inspect the output
        bt.stall(3_000_000);
    }
}

fn shutdown(st: &SystemTable) -> ! {
    use uefi::table::runtime::ResetType;

    // Get our text output back.
    st.stdout().reset(false).unwrap().unwrap();

    // Inform the user, and give him time to read on real hardware
    if cfg!(not(feature = "qemu")) {
        info!("Testing complete, shutting down in 3 seconds...");
        st.boot.stall(3_000_000);
    } else {
        info!("Testing complete, shutting down...");
    }

    let rt = st.runtime;
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}
