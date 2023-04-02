#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use alloc::string::ToString;
use uefi::prelude::*;
use uefi::proto::console::serial::Serial;
use uefi::Result;
use uefi_services::{print, println};

mod boot;
mod fs;
mod proto;
mod runtime;

#[entry]
fn efi_main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&mut st).expect("Failed to initialize utilities");

    // unit tests here

    let firmware_vendor = st.firmware_vendor();
    info!("Firmware Vendor: {}", firmware_vendor);
    assert_eq!(firmware_vendor.to_string(), "EDK II");

    // Test print! and println! macros.
    let (print, println) = ("print!", "println!"); // necessary for clippy to ignore
    print!("Testing {} macro with formatting: {:#010b} ", print, 155u8);
    println!(
        "Testing {} macro with formatting: {:#010b} ",
        println, 155u8
    );

    // Reset the console before running all the other tests.
    st.stdout().reset(false).expect("Failed to reset stdout");

    // Ensure the tests are run on a version of UEFI we support.
    check_revision(st.uefi_revision());

    // Test all the boot services.
    let bt = st.boot_services();

    // Try retrieving a handle to the file system the image was booted from.
    bt.get_image_file_system(image)
        .expect("Failed to retrieve boot file system");

    boot::test(bt);

    // Test all the supported protocols.
    proto::test(image, &mut st);

    // TODO: runtime services work before boot services are exited, but we'd
    // probably want to test them after exit_boot_services. However,
    // exit_boot_services is currently called during shutdown.

    runtime::test(st.runtime_services());

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

#[derive(Clone, Copy, Debug)]
enum HostRequest {
    /// Tell the host to take a screenshot and compare against the
    /// golden image.
    Screenshot(&'static str),

    /// Tell the host that tests are complete. The host will consider
    /// the tests failed if this message is not received.
    TestsComplete,
}

fn send_request_helper(serial: &mut Serial, request: HostRequest) -> Result {
    let request = match request {
        HostRequest::Screenshot(name) => format!("SCREENSHOT: {name}\n"),
        HostRequest::TestsComplete => "TESTS_COMPLETE\n".to_string(),
    };

    // Set a 10 second timeout for the read and write operations.
    let mut io_mode = *serial.io_mode();
    io_mode.timeout = 10_000_000;
    serial.set_attributes(&io_mode)?;

    // Send a screenshot request to the host.
    serial.write(request.as_bytes()).discard_errdata()?;

    // Wait for the host's acknowledgement before moving forward.
    let mut reply = [0; 3];
    serial.read(&mut reply[..]).discard_errdata()?;

    if reply == *b"OK\n" {
        Ok(())
    } else {
        Err(Status::ABORTED.into())
    }
}

/// Send the `request` string to the host via the `serial` device, then
/// wait up to 10 seconds to receive a reply. Returns an error if the
/// reply is not `"OK\n"`.
fn send_request_to_host(bt: &BootServices, request: HostRequest) {
    let serial_handle = bt
        .get_handle_for_protocol::<Serial>()
        .expect("Failed to get serial handle");

    // Open the serial protocol in exclusive mode.
    //
    // EDK2's [console splitter driver] periodically tries to sample
    // from console devices to see if keys are being pressed, which will
    // overwrite the timeout set below and potentially swallow the reply
    // from the host. Opening in exclusive mode stops the driver from
    // using this device. However, it also prevents logs from from going
    // to the serial device, so we have to restore the connection at the
    // end with `connect_controller`.
    //
    // [console splitter driver]: https://github.com/tianocore/edk2/blob/HEAD/MdeModulePkg/Universal/Console/ConSplitterDxe/ConSplitter.c
    let mut serial = bt
        .open_protocol_exclusive::<Serial>(serial_handle)
        .expect("Could not open serial protocol");

    // Send the request, but don't check the result yet so that first
    // we can reconnect the console output for the logger.
    let res = send_request_helper(&mut serial, request);

    // Release the serial device and reconnect all controllers to the
    // serial handle. This is necessary to restore the connection
    // between the console output device used for logging and the serial
    // device, which was broken when we opened the protocol in exclusive
    // mode above.
    drop(serial);
    let _ = bt.connect_controller(serial_handle, None, None, true);

    if let Err(err) = res {
        panic!("request failed: \"{request:?}\": {:?}", err.status());
    }
}

fn shutdown(mut st: SystemTable<Boot>) -> ! {
    // Get our text output back.
    st.stdout().reset(false).unwrap();

    info!("Testing complete, shutting down...");

    // Tell the host that tests are done. We are about to exit boot
    // services, so we can't easily communicate with the host any later
    // than this.
    send_request_to_host(st.boot_services(), HostRequest::TestsComplete);

    // Exit boot services as a proof that it works :)
    let (st, _iter) = st.exit_boot_services();

    #[cfg(target_arch = "x86_64")]
    {
        // Prevent unused variable warning.
        let _ = st;

        use qemu_exit::QEMUExit;
        let custom_exit_success = 3;
        let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
        qemu_exit_handle.exit_success();
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Shut down the system
        let rt = unsafe { st.runtime_services() };
        rt.reset(
            uefi::table::runtime::ResetType::Shutdown,
            Status::SUCCESS,
            None,
        );
    }
}
