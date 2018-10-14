use uefi::prelude::*;
use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::table::boot::BootServices;
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    info!("Running serial protocol test");
    if let Some(serial) = bt.find_protocol::<Serial>() {
        let serial = unsafe { &mut *serial.get() };

        let old_ctrl_bits = serial
            .get_control_bits()
            .expect_success("Failed to get device control bits");
        let mut ctrl_bits = ControlBits::empty();

        // For the purposes of testing, we're _not_ going to implement
        // software flow control.
        ctrl_bits |= ControlBits::HARDWARE_FLOW_CONTROL_ENABLE;

        // Use a loop back device for testing.
        ctrl_bits |= ControlBits::SOFTWARE_LOOPBACK_ENABLE;

        serial
            .set_control_bits(ctrl_bits)
            .expect_success("Failed to set device control bits");

        // Keep this message short, we need it to fit in the FIFO.
        const OUTPUT: &[u8] = b"Hello world!";
        const MSG_LEN: usize = OUTPUT.len();

        let len = serial
            .write(OUTPUT)
            .expect_success("Failed to write to serial port");
        assert_eq!(len, MSG_LEN, "Bad serial port write length");

        let mut input = [0u8; MSG_LEN];
        let len = serial
            .read(&mut input)
            .expect_success("Failed to read from serial port");
        assert_eq!(len, MSG_LEN, "Bad serial port read length");

        assert_eq!(&OUTPUT[..], &input[..MSG_LEN]);

        // Clean up after ourselves
        serial
            .reset()
            .expect_success("Could not reset the serial device");
        serial
            .set_control_bits(old_ctrl_bits & ControlBits::SETTABLE)
            .expect_success("Could not restore the serial device state");
    } else {
        warn!("No serial device found");
    }
}
