use crate::{guid, Guid, Status};
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

/// Structure representing the device's current parameters.
///
/// The default values for all UART-like devices is:
/// - 115,200 baud
/// - 1 byte receive FIFO
/// - 1'000'000 microsecond timeout
/// - no parity
/// - 8 data bits
/// - 1 stop bit
///
/// The software is responsible for flow control.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(C)]
pub struct SerialIoMode {
    /// Bitmask of the control bits that this device supports.
    pub control_mask: ControlBits,
    /// If applicable, the number of microseconds to wait before assuming an
    /// operation timed out.
    pub timeout: u32,
    /// Device's baud rate, or 0 if unknown.
    pub baud_rate: u64,
    /// Size in character's of the device's buffer.
    pub receive_fifo_depth: u32,
    /// Number of data bits in each character.
    pub data_bits: u32,
    /// If applicable, the parity that is computed or checked for each character.
    pub parity: Parity,
    /// If applicable, the number of stop bits per character.
    pub stop_bits: StopBits,
}

#[repr(C)]
pub struct SerialIoProtocol {
    pub revision: u32,
    pub reset: unsafe extern "efiapi" fn(*mut Self) -> Status,
    pub set_attributes: unsafe extern "efiapi" fn(
        *const Self,
        baud_rate: u64,
        receive_fifo_depth: u32,
        timeout: u32,
        parity: Parity,
        data_bits: u8,
        stop_bits_type: StopBits,
    ) -> Status,
    pub set_control_bits: unsafe extern "efiapi" fn(*mut Self, ControlBits) -> Status,
    pub get_control_bits: unsafe extern "efiapi" fn(*const Self, *mut ControlBits) -> Status,
    pub write: unsafe extern "efiapi" fn(*mut Self, *mut usize, *const u8) -> Status,
    pub read: unsafe extern "efiapi" fn(*mut Self, *mut usize, *mut u8) -> Status,
    pub mode: *const SerialIoMode,
}

impl SerialIoProtocol {
    pub const GUID: Guid = guid!("bb25cf6f-f1d4-11d2-9a0c-0090273fc1fd");
    pub const REVISION: u32 = 0x00010000;
    pub const REVISION1P1: u32 = 0x00010001;
}

newtype_enum! {
    /// The parity of the device.
    pub enum Parity: u32 => {
        /// Device default
        DEFAULT = 0,
        /// No parity
        NONE = 1,
        /// Even parity
        EVEN = 2,
        /// Odd parity
        ODD = 3,
        /// Mark parity
        MARK = 4,
        /// Space parity
        SPACE = 5,
    }
}

newtype_enum! {
    /// Number of stop bits per character.
    pub enum StopBits: u32 => {
        /// Device default
        DEFAULT = 0,
        /// 1 stop bit
        ONE = 1,
        /// 1.5 stop bits
        ONE_FIVE = 2,
        /// 2 stop bits
        TWO = 3,
    }
}
