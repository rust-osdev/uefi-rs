use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::table::boot::BootServices;
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    info!("Running serial protocol test");
    if let Some(mut serial) = bt.find_protocol::<Serial>() {
        let serial = unsafe { serial.as_mut() };

        let old_ctrl_bits = serial
            .get_control_bits()
            .expect("Failed to get device control bits")
            .expect("Warnings encountered while getting device control bits");
        let mut ctrl_bits = ControlBits::empty();

        // For the purposes of testing, we're _not_ going to implement
        // software flow control.
        ctrl_bits |= ControlBits::HARDWARE_FLOW_CONTROL_ENABLE;

        // Use a loop back device for testing.
        ctrl_bits |= ControlBits::SOFTWARE_LOOPBACK_ENABLE;

        serial
            .set_control_bits(ctrl_bits)
            .expect("Failed to set device control bits")
            .expect("Warnings encountered while setting device control bits");

        // Keep this message short, we need it to fit in the FIFO.
        const OUTPUT: &[u8] = b"Hello world!";
        const MSG_LEN: usize = OUTPUT.len();

        let len = serial
            .write(OUTPUT)
            .expect("Failed to write to serial port")
            .expect("Warnings encountered while writing to serial port");
        assert_eq!(len, MSG_LEN, "Bad serial port write length");

        let mut input = [0u8; MSG_LEN];
        let len = serial
            .read(&mut input)
            .expect("Failed to read from serial port")
            .expect("Warnings encountered while reading from serial port");
        assert_eq!(len, MSG_LEN, "Bad serial port read length");

        assert_eq!(&OUTPUT[..], &input[..MSG_LEN]);

        // Clean up after ourselves
        serial
            .reset()
            .expect("Could not reset the serial device")
            .expect("Warnings encountered while resetting serial device");
        serial
            .set_control_bits(old_ctrl_bits & ControlBits::SETTABLE)
            .expect("Could not restore the serial device state")
            .expect("Warnings encountered while restoring serial device state");
    } else {
        warn!("No serial device found");
    }
}
