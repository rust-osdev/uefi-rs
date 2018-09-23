use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::table::boot::BootServices;
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    if let Some(mut serial) = bt.find_protocol::<Serial>() {
        let serial = unsafe { serial.as_mut() };

        let old_ctrl_bits = serial.get_control_bits()
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
        let output = b"Hello world!";
        let msg_len = output.len();

        let len = serial
            .write(output)
            .expect("Failed to write to serial port");
        assert_eq!(len, msg_len, "Serial port write timed-out!");

        let mut input = [0u8; 128];
        let len = serial
            .read(&mut input)
            .expect("Failed to read from serial port");
        assert_eq!(len, msg_len, "Serial port read timed-out!");

        assert_eq!(&output[..], &input[..msg_len]);

        // Clean up after ourselves
        serial.reset().expect("Could not reset the serial device");
        serial.set_control_bits(old_ctrl_bits & Serial::settable_control_bits())
              .expect("Could not restore the serial device state");
    } else {
        warn!("No serial device found");
    }
}
