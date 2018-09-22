use uefi::proto::console::serial::{ControlBits, Serial};
use uefi::table::boot::BootServices;
use uefi_exts::BootServicesExt;

pub fn test(bt: &BootServices) {
    if let Some(mut serial) = bt.find_protocol::<Serial>() {
        let serial = unsafe { serial.as_mut() };

        // For the purposes of testing, we're _not_ going to implement
        // software flow control.
        serial.set_control_bits(ControlBits::HARDWARE_FLOW_CONTROL_ENABLE)
            .expect("Device does not support HW control flow");

        let mut buffer = vec![0u8; 16];

        let len = serial.read(&mut buffer).expect("Failed to read from serial port");

        assert_eq!(len, buffer.len(), "Serial port read timed-out!");

        buffer.resize(len, 0);
    } else {
        warn!("No serial device found");
    }
}
