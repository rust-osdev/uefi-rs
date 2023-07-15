use bitflags::bitflags;

bitflags! {
    /// The control bits of a device. These are defined in the [RS-232] standard.
    ///
    /// [RS-232]: https://en.wikipedia.org/wiki/RS-232
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ControlBits: u32 {
        /// Clear to send
        const CLEAR_TO_SEND = 0x10;
        /// Data set ready
        const DATA_SET_READY = 0x20;
        /// Indicates that a phone line is ringing
        const RING_INDICATE = 0x40;
        /// Indicates the connection is still connected
        const CARRIER_DETECT = 0x80;
        /// The input buffer is empty
        const INPUT_BUFFER_EMPTY = 0x100;
        /// The output buffer is empty
        const OUTPUT_BUFFER_EMPTY = 0x200;

        /// Terminal is ready for communications
        const DATA_TERMINAL_READY = 0x1;
        /// Request the device to send data
        const REQUEST_TO_SEND = 0x2;
        /// Enable hardware loop-back
        const HARDWARE_LOOPBACK_ENABLE = 0x1000;
        /// Enable software loop-back
        const SOFTWARE_LOOPBACK_ENABLE = 0x2000;
        /// Allow the hardware to handle flow control
        const HARDWARE_FLOW_CONTROL_ENABLE = 0x4000;

        /// Bitmask of the control bits that can be set.
        ///
        /// Up to date as of UEFI 2.7 / Serial protocol v1
        const SETTABLE =
            ControlBits::DATA_TERMINAL_READY.bits()
            | ControlBits::REQUEST_TO_SEND.bits()
            | ControlBits::HARDWARE_LOOPBACK_ENABLE.bits()
            | ControlBits::SOFTWARE_LOOPBACK_ENABLE.bits()
            | ControlBits::HARDWARE_FLOW_CONTROL_ENABLE.bits();
    }
}
