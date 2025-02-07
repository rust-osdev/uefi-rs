// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::reconnect_serial_to_console;
use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::{boot, Result, ResultExt, Status};

// For the duration of this function, the serial device is opened in
// exclusive mode. That means logs will not work, which means we should
// avoid panicking here because the panic log would be hidden. Instead,
// return a result that gets asserted in `test` *after* the logger has
// been restored.
fn serial_test_helper(serial: &mut Serial) -> Result {
    let old_ctrl_bits = serial.get_control_bits()?;
    let mut ctrl_bits = ControlBits::empty();

    // For the purposes of testing, we're _not_ going to implement
    // software flow control.
    ctrl_bits |= ControlBits::HARDWARE_FLOW_CONTROL_ENABLE;

    // Use a loop back device for testing.
    ctrl_bits |= ControlBits::SOFTWARE_LOOPBACK_ENABLE;

    serial.set_control_bits(ctrl_bits)?;

    // Keep this message short, we need it to fit in the FIFO.
    const OUTPUT: &[u8] = b"Hello world!";
    const MSG_LEN: usize = OUTPUT.len();

    serial.write(OUTPUT).discard_errdata()?;

    let mut input = [0u8; MSG_LEN];
    serial.read(&mut input).discard_errdata()?;

    // Clean up after ourselves
    serial.reset()?;
    serial.set_control_bits(old_ctrl_bits & ControlBits::SETTABLE)?;

    if OUTPUT == input {
        Ok(())
    } else {
        Err(Status::ABORTED.into())
    }
}

pub unsafe fn test() {
    // The serial device under aarch64 doesn't support the software
    // loopback feature needed for this test.
    if cfg!(target_arch = "aarch64") {
        return;
    }

    info!("Running serial protocol test");
    let handle = boot::get_handle_for_protocol::<Serial>().expect("missing Serial protocol");

    let mut serial =
        boot::open_protocol_exclusive::<Serial>(handle).expect("failed to open serial protocol");

    // Send the request, but don't check the result yet so that first
    // we can reconnect the console output for the logger.
    let res = serial_test_helper(&mut serial);

    // Release the serial device and reconnect all controllers to the
    // serial handle. This is necessary to restore the connection
    // between the console output device used for logging and the serial
    // device, which was broken when we opened the protocol in exclusive
    // mode above.
    drop(serial);
    reconnect_serial_to_console(handle);

    if let Err(err) = res {
        panic!("serial test failed: {:?}", err.status());
    }
}
