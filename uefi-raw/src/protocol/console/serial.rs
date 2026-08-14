// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Guid, Status, guid, newtype_enum};
use bitflags::bitflags;

bitflags! {
    /// The control bits of a device. These are defined in the [RS-232] standard.
    ///
    /// [RS-232]: https://en.wikipedia.org/wiki/RS-232
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ControlBits: u32 {
        /// Indicates that the remote device is clear to send.
        const CLEAR_TO_SEND = 0x10;
        /// Indicates that the remote device is ready.
        const DATA_SET_READY = 0x20;
        /// Indicates that a phone line is ringing.
        const RING_INDICATE = 0x40;
        /// Indicates that the connection is active.
        const CARRIER_DETECT = 0x80;
        /// Indicates that the input buffer is empty.
        const INPUT_BUFFER_EMPTY = 0x100;
        /// Indicates that the output buffer is empty.
        const OUTPUT_BUFFER_EMPTY = 0x200;

        /// Marks the terminal as ready for communication.
        const DATA_TERMINAL_READY = 0x1;
        /// Requests that the device send data.
        const REQUEST_TO_SEND = 0x2;
        /// Enables hardware loopback.
        const HARDWARE_LOOPBACK_ENABLE = 0x1000;
        /// Enables software loopback.
        const SOFTWARE_LOOPBACK_ENABLE = 0x2000;
        /// Allows the hardware to handle flow control.
        const HARDWARE_FLOW_CONTROL_ENABLE = 0x4000;

        /// Bitmask of the control bits that can be set.
        ///
        /// This list is current as of UEFI 2.7 and Serial I/O protocol 1.0.
        const SETTABLE =
            ControlBits::DATA_TERMINAL_READY.bits()
            | ControlBits::REQUEST_TO_SEND.bits()
            | ControlBits::HARDWARE_LOOPBACK_ENABLE.bits()
            | ControlBits::SOFTWARE_LOOPBACK_ENABLE.bits()
            | ControlBits::HARDWARE_FLOW_CONTROL_ENABLE.bits();
    }
}

/// Current serial device parameters.
///
/// The default values for all UART-like devices are:
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
    /// Receive FIFO depth in characters.
    pub receive_fifo_depth: u32,
    /// Number of data bits in each character.
    pub data_bits: u32,
    /// If applicable, the parity that is computed or checked for each character.
    pub parity: Parity,
    /// If applicable, the number of stop bits per character.
    pub stop_bits: StopBits,
}

newtype_enum! {
    /// The revision of the [`SerialIoProtocol`].
    #[derive(Default)]
    pub enum SerialIoProtocolRevision: u32  => {
        /// Initial version 1.0.
        REVISION_1_0 = 0x00010000,
        /// Version 1.1.
        REVISION_1_1 = 0x00010001,
    }
}

/// Serial I/O protocol (revision 1.0).
#[derive(Debug)]
#[repr(C)]
pub struct SerialIoProtocol {
    pub revision: SerialIoProtocolRevision,
    pub reset: unsafe extern "efiapi" fn(*mut Self) -> Status,
    pub set_attributes: unsafe extern "efiapi" fn(
        *mut Self,
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
}

/// Serial I/O protocol (revision 1.1).
#[derive(Debug)]
#[repr(C)]
pub struct SerialIoProtocol_1_1 {
    pub base_protocol: SerialIoProtocol,
    pub device_type_guid: *const Guid,
}

impl SerialIoProtocol_1_1 {
    pub const GUID: Guid = SerialIoProtocol::GUID;
}

newtype_enum! {
    /// The parity of the device.
    pub enum Parity: u32 => {
        /// Uses the device default.
        DEFAULT = 0,
        /// Disables parity.
        NONE = 1,
        /// Uses even parity.
        EVEN = 2,
        /// Uses odd parity.
        ODD = 3,
        /// Uses mark parity.
        MARK = 4,
        /// Uses space parity.
        SPACE = 5,
    }
}

newtype_enum! {
    /// Number of stop bits per character.
    pub enum StopBits: u32 => {
        /// Uses the device default.
        DEFAULT = 0,
        /// Uses one stop bit.
        ONE = 1,
        /// Uses one and a half stop bits.
        ONE_FIVE = 2,
        /// Uses two stop bits.
        TWO = 3,
    }
}
