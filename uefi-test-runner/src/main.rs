// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]

#[macro_use]
extern crate log;
#[macro_use]
extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::console::serial::Serial;
use uefi::proto::device_path::build::{self, DevicePathBuilder};
use uefi::proto::device_path::messaging::Vendor;
use uefi::{print, println, system, Result};

mod boot;
mod fs;
mod proto;
mod runtime;

#[entry]
fn efi_main() -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi::helpers::init().expect("Failed to initialize utilities");

    // Test print! and println! macros.
    let (print, println) = ("print!", "println!"); // necessary for clippy to ignore
    print!("Testing {} macro with formatting: {:#010b} ", print, 155u8);
    println!(
        "Testing {} macro with formatting: {:#010b} ",
        println, 155u8
    );

    // Reset the console before running all the other tests.
    system::with_stdout(|stdout| stdout.reset(false).expect("Failed to reset stdout"));

    // Check the `uefi::system` module.
    check_system();

    // Try retrieving a handle to the file system the image was booted from.
    uefi::boot::get_image_file_system(uefi::boot::image_handle())
        .expect("Failed to retrieve boot file system");

    boot::test();

    // Test all the supported protocols.
    proto::test();

    // TODO: runtime services work before boot services are exited, but we'd
    // probably want to test them after exit_boot_services. However,
    // exit_boot_services is currently called during shutdown.

    runtime::test();

    shutdown();
}

fn check_revision(rev: uefi::table::Revision) {
    assert_eq!(system::uefi_revision(), rev);

    let (major, minor) = (rev.major(), rev.minor());

    info!("UEFI {}.{}", major, minor / 10);

    assert!(major >= 2, "Running on an old, unsupported version of UEFI");
    assert!(
        minor >= 30,
        "Old version of UEFI 2, some features might not be available."
    );
}

fn check_system() {
    info!("Firmware Vendor: {}", system::firmware_vendor());
    info!("Firmware Revision: {}", system::firmware_revision());

    assert_eq!(system::firmware_vendor(), cstr16!("EDK II"));
    check_revision(system::uefi_revision());

    system::with_stdout(|stdout| {
        stdout
            .output_string(cstr16!("test system::with_stdout\n"))
            .unwrap()
    });
    system::with_stderr(|stdout| {
        stdout
            .output_string(cstr16!("test system::with_stderr\n"))
            .unwrap()
    });
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

/// Reconnect the serial device to the output console.
///
/// This must be called after opening the serial protocol in exclusive mode, as
/// that breaks the connection to the console, which in turn prevents logs from
/// getting to the host.
fn reconnect_serial_to_console(serial_handle: Handle) {
    let mut storage = Vec::new();
    // Create a device path that specifies the terminal type.
    let terminal_guid = if cfg!(target_arch = "aarch64") {
        Vendor::VT_100
    } else {
        Vendor::VT_UTF8
    };
    let terminal_device_path = DevicePathBuilder::with_vec(&mut storage)
        .push(&build::messaging::Vendor {
            vendor_guid: terminal_guid,
            vendor_defined_data: &[],
        })
        .unwrap()
        .finalize()
        .unwrap();

    uefi::boot::connect_controller(serial_handle, None, Some(terminal_device_path), true)
        .expect("failed to reconnect serial to console");
}

/// Send the `request` string to the host via the `serial` device, then
/// wait up to 10 seconds to receive a reply. Returns an error if the
/// reply is not `"OK\n"`.
fn send_request_to_host(request: HostRequest) {
    let serial_handle =
        uefi::boot::get_handle_for_protocol::<Serial>().expect("Failed to get serial handle");

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
    let mut serial = uefi::boot::open_protocol_exclusive::<Serial>(serial_handle)
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
    reconnect_serial_to_console(serial_handle);

    if let Err(err) = res {
        panic!("request failed: \"{request:?}\": {:?}", err.status());
    }
}

fn shutdown() -> ! {
    // Get our text output back.
    system::with_stdout(|stdout| stdout.reset(false).unwrap());

    // Tell the host that tests are done. We are about to exit boot
    // services, so we can't easily communicate with the host any later
    // than this.
    send_request_to_host(HostRequest::TestsComplete);

    // Send a special log to the host so that we can verify that logging works
    // up until exiting boot services. See `reconnect_serial_to_console` for the
    // type of regression this prevents.
    info!("LOGGING_STILL_WORKING_RIGHT_BEFORE_EBS");

    info!("Testing complete, exiting boot services...");

    // Exit boot services as a proof that it works :)
    let mmap = unsafe { uefi::boot::exit_boot_services(None) };

    info!("Memory Map:");
    for desc in mmap.entries() {
        info!(
            "start=0x{:016x} size=0x{:016x} type={:?}, attr={:?}",
            desc.phys_start,
            desc.page_count * 4096,
            desc.ty,
            desc.att
        );
    }

    info!("Shutting down...");

    #[cfg(target_arch = "x86_64")]
    {
        use qemu_exit::QEMUExit;
        let custom_exit_success = 3;
        let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
        qemu_exit_handle.exit_success();
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Shut down the system
        uefi::runtime::reset(uefi::runtime::ResetType::SHUTDOWN, Status::SUCCESS, None);
    }
}
