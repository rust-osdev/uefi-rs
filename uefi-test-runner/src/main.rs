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
pub extern "win64" fn uefi_start(image: uefi::Handle, st: BootSystemTable) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    // Reset the console before running all the other tests.
    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());

    // Test all the boot services.
    let bt = st.boot_services();
    boot::test(bt);

    // Test all the supported protocols.
    proto::test(&st);

    // TODO: test the runtime services.
    // These work before boot services are exited, but we'd probably want to
    // test them after exit_boot_services...

    shutdown(image, st);
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
        let serial = bt
            .find_protocol::<Serial>()
            .expect("Could not find serial port");
        let serial = unsafe { &mut *serial.get() };

        // Set a large timeout to avoid problems with Travis
        let mut io_mode = *serial.io_mode();
        io_mode.timeout = 10_000_000;
        serial
            .set_attributes(&io_mode)
            .expect_success("Failed to configure serial port timeout");

        // Send a screenshot request to the host
        serial
            .write(b"SCREENSHOT: ")
            .expect_success("Failed to send request");
        let name_bytes = name.as_bytes();
        serial
            .write(name_bytes)
            .expect_success("Failed to send request");
        serial.write(b"\n").expect_success("Failed to send request");

        // Wait for the host's acknowledgement before moving forward
        let mut reply = [0; 3];
        serial
            .read(&mut reply[..])
            .expect_success("Failed to read host reply");

        assert_eq!(&reply[..], b"OK\n", "Unexpected screenshot request reply");
    } else {
        // Outside of QEMU, give the user some time to inspect the output
        bt.stall(3_000_000);
    }
}

fn shutdown(image: uefi::Handle, st: BootSystemTable) -> ! {
    use uefi::table::runtime::ResetType;

    // Get our text output back.
    st.stdout().reset(false).unwrap_success();

    // Inform the user, and give him time to read on real hardware
    if cfg!(not(feature = "qemu")) {
        info!("Testing complete, shutting down in 3 seconds...");
        st.boot_services().stall(3_000_000);
    } else {
        info!("Testing complete, shutting down...");
    }

    // Exit boot services as a proof that it works :)
    use crate::alloc::vec::Vec;
    let boot = st.boot_services();
    let mut memory_map_storage = Vec::with_capacity(boot.memory_map_size() + 1024);
    unsafe { memory_map_storage.set_len(memory_map_storage.capacity()); }
    let (key, _iter) = boot.memory_map(&mut memory_map_storage[..])
                           .expect_success("Failed to fetch memory map");
    let st = st.exit_boot_services(
        image,
        key,
        |map_size, memory_map| {
            // Can't read the memory map again if it's grown too big
            if map_size > memory_map_storage.capacity() {
                return None;
            }

            // Otherwise, give it another try
            memory_map(&mut memory_map_storage[..])
                .warning_as_error()
                .map(|(key, _iter)| { key })
                .ok()
        }
    ).expect_success("Failed to exit boot services");

    // Shut down the system
    let rt = unsafe { st.runtime_services() };
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}
