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
use uefi::proto::console::serial::Serial;
use uefi_exts::BootServicesExt;

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

/// Ask the test runner to check the current screen output against a reference
/// TODO: This is obviously highly QEMU-specific
/// TODO: Turn it into something which waits a bit for user inspection elsewhere
fn check_screenshot(bt: &BootServices, name: &str) {
    // Access the serial port (in a QEMU environment, it should always be there)
    let mut serial = bt
        .find_protocol::<Serial>()
        .expect("Could not find serial port");
    let serial = unsafe { serial.as_mut() };

    // Set a large timeout to avoid problems
    let mut io_mode = serial.io_mode().clone();
    io_mode.timeout = 1_000_000;
    serial
        .set_attributes(&io_mode)
        .expect("Failed to configure serial port");

    // Send a screenshot request to QEMU
    // TODO: Do not hardcode the screenshot name
    // TODO: Implement write!() for serial ports
    let screenshot_request = b"SCREENSHOT: gop_test\n";
    let write_size = serial
        .write(screenshot_request)
        .expect("Failed to write screenshot command");
    assert_eq!(
        write_size,
        screenshot_request.len(),
        "Screenshot request timed out"
    );

    // Wait for QEMU's acknowledgement before moving forward
    let mut reply = [0; 3];
    let read_size = serial
        .read(&mut reply[..])
        .expect("Failed to read host reply");
    assert_eq!(read_size, 3, "Screenshot request timed out");
    assert_eq!(&reply[..], b"OK\n", "Unexpected screenshot request reply");
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
