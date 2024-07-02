use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};
use uefi::Handle;

pub unsafe fn test(image: Handle, bt: &BootServices) {
    info!("Running serial protocol test");
    if let Ok(handle) = bt.get_handle_for_protocol::<Serial>() {
        let mut serial = bt
            .open_protocol::<Serial>(
                OpenProtocolParams {
                    handle,
                    agent: image,
                    controller: None,
                },
                // For this test, don't open in exclusive mode. That
                // would break the connection between stdout and the
                // serial device.
                OpenProtocolAttributes::GetProtocol,
            )
            .expect("failed to open serial protocol");
        // BUG: there are multiple failures in the serial tests on AArch64
        if cfg!(target_arch = "aarch64") {
            return;
        }

        let old_ctrl_bits = serial
            .get_control_bits()
            .expect("Failed to get device control bits");
        let mut ctrl_bits = ControlBits::empty();

        // For the purposes of testing, we're _not_ going to implement
        // software flow control.
        ctrl_bits |= ControlBits::HARDWARE_FLOW_CONTROL_ENABLE;

        // Use a loop back device for testing.
        ctrl_bits |= ControlBits::SOFTWARE_LOOPBACK_ENABLE;

        serial
            .set_control_bits(ctrl_bits)
            .expect("Failed to set device control bits");

        // Keep this message short, we need it to fit in the FIFO.
        const OUTPUT: &[u8] = b"Hello world!";
        const MSG_LEN: usize = OUTPUT.len();

        serial
            .write(OUTPUT)
            .expect("Failed to write to serial port");

        let mut input = [0u8; MSG_LEN];
        serial
            .read(&mut input)
            .expect("Failed to read from serial port");

        assert_eq!(OUTPUT, &input[..]);

        // Clean up after ourselves
        serial.reset().expect("Could not reset the serial device");
        serial
            .set_control_bits(old_ctrl_bits & ControlBits::SETTABLE)
            .expect("Could not restore the serial device state");
    } else {
        warn!("No serial device found");
    }
}
