// SPDX-License-Identifier: MIT OR Apache-2.0

//! Abstraction over byte stream devices, also known as serial I/O devices.

use crate::proto::unsafe_protocol;
use crate::{Error, Result, StatusExt};
use core::fmt;
use core::fmt::Write;
use uefi_raw::Status;
use uefi_raw::protocol::console::serial::{SerialIoProtocol, SerialIoProtocolRevision};
use uguid::Guid;

pub use uefi_raw::protocol::console::serial::{
    ControlBits, Parity, SerialIoMode as IoMode, StopBits,
};

/// Serial IO [`Protocol`]. Provides access to a serial I/O device.
///
/// This can include standard UART devices, serial ports over a USB interface,
/// or any other character-based communication device. The protocol is
/// typically used to connect to a Terminal.
///
/// # Connection Properties and I/O Hints
///
/// ## General
///
/// Special care must be taken if a significant amount of data is going to be
/// read from a serial device. Since UEFI drivers are polled mode drivers,
/// characters received on a serial device might be missed. It is the
/// responsibility of the software that uses the protocol to check for new data
/// often enough to guarantee that no characters will be missed. The required
/// polling frequency depends on the baud rate of the connection and the depth
/// of the receive FIFO.
///
/// ## UART
///
/// The default attributes for all UART-style serial device interfaces are:
/// 115,200 baud, a 1 byte receive FIFO, a 1,000,000 microsecond (1s) timeout
/// per character, no parity, 8 data bits, and 1 stop bit.
///
/// Flow control is the responsibility of the software that uses the protocol.
/// Hardware flow control can be implemented through the use of the
/// [`Serial::get_control_bits`] and [`Serial::set_control_bits`] functions
/// to monitor and assert the flow control signals.
///
/// The XON/XOFF flow control algorithm can be implemented in software by
/// inserting XON and XOFF characters into the serial data stream as required.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(SerialIoProtocol::GUID)]
pub struct Serial(SerialIoProtocol);

impl Serial {
    /// Returns the revision of the protocol.
    #[must_use]
    pub const fn revision(&self) -> SerialIoProtocolRevision {
        self.0.revision
    }

    /// Reset the device.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: serial device could not be reset.
    pub fn reset(&mut self) -> Result {
        unsafe { (self.0.reset)(&mut self.0) }.to_result()
    }

    /// Returns the current I/O mode.
    #[must_use]
    pub const fn io_mode(&self) -> &IoMode {
        unsafe { &*self.0.mode }
    }

    /// Sets the device's new attributes.
    ///
    /// The given [`IoMode`] will become the device's new [`IoMode`],
    /// with some exceptions:
    ///
    /// - `control_mask` is ignored, since it's a read-only field;
    ///
    /// - values set to `0` / `Default` will be filled with the device's
    ///   default parameters
    ///
    /// - if either `baud_rate` or `receive_fifo_depth` is less than
    ///   the device's minimum, an error will be returned;
    ///   this value will be rounded down to the nearest value supported by the device
    ///
    /// # Errors
    ///
    /// - [`Status::INVALID_PARAMETER`]: one or more of the attributes has an
    ///   unsupported value
    /// - [`Status::DEVICE_ERROR`]: serial device is not functioning correctly
    pub fn set_attributes(&mut self, mode: &IoMode) -> Result {
        unsafe {
            (self.0.set_attributes)(
                &mut self.0,
                mode.baud_rate,
                mode.receive_fifo_depth,
                mode.timeout,
                mode.parity,
                mode.data_bits as u8,
                mode.stop_bits,
            )
        }
        .to_result()
    }

    /// Retrieve the device's current control bits.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: serial device is not functioning correctly
    pub fn get_control_bits(&self) -> Result<ControlBits> {
        let mut bits = ControlBits::empty();
        unsafe { (self.0.get_control_bits)(&self.0, &mut bits) }.to_result_with_val(|| bits)
    }

    /// Sets the device's new control bits.
    ///
    /// Not all bits can be modified with this function. A mask of the allowed
    /// bits is stored in the [`ControlBits::SETTABLE`] constant.
    ///
    /// # Errors
    ///
    /// - [`Status::UNSUPPORTED`]: serial device does not support this operation
    /// - [`Status::DEVICE_ERROR`]: serial device is not functioning correctly
    pub fn set_control_bits(&mut self, bits: ControlBits) -> Result {
        unsafe { (self.0.set_control_bits)(&mut self.0, bits) }.to_result()
    }

    /// Reads data from the device. This function has the raw semantics of the
    /// underlying UEFI protocol.
    ///
    /// The function will read bytes until either the buffer is full or a
    /// timeout or overrun error occurs.
    ///
    /// # Arguments
    ///
    /// - `buffer`: buffer to fill
    ///
    /// # Tips
    ///
    /// Consider setting non-default properties via [`Self::set_attributes`]
    /// and [`Self::set_control_bits`] matching your use-case. For more info,
    /// please read the general [documentation](Self) of the protocol.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: serial device reported an error
    /// - [`Status::TIMEOUT`]: operation was stopped due to a timeout or overrun
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), usize /* read bytes on timeout*/> {
        let mut buffer_size = buffer.len();
        unsafe { (self.0.read)(&mut self.0, &mut buffer_size, buffer.as_mut_ptr()) }.to_result_with(
            || {
                // By spec: Either reads all requested bytes (and blocks) or
                // returns early with an error.
                assert_eq!(buffer_size, buffer.len())
            },
            |_| buffer_size,
        )
    }

    /// Writes data to this device. This function has the raw semantics of the
    /// underlying UEFI protocol.
    ///
    /// The function will try to write all provided bytes in the configured
    /// timeout.
    ///
    /// # Arguments
    ///
    /// - `data`: bytes to write
    ///
    /// # Tips
    ///
    /// Consider setting non-default properties via [`Self::set_attributes`]
    /// and [`Self::set_control_bits`] matching your use-case. For more info,
    /// please read the general [documentation](Self) of the protocol.
    ///
    /// # Errors
    ///
    /// - [`Status::DEVICE_ERROR`]: serial device reported an error
    /// - [`Status::TIMEOUT`]: data write was stopped due to a timeout
    pub fn write(&mut self, data: &[u8]) -> Result<(), usize /* bytes written on timeout */> {
        let mut buffer_size = data.len();
        unsafe { (self.0.write)(&mut self.0, &mut buffer_size, data.as_ptr()) }.to_result_with(
            || {
                // By spec: Either reads all requested bytes (and blocks) or
                // returns early with an error.
                assert_eq!(buffer_size, data.len())
            },
            |_| buffer_size,
        )
    }

    /// Pointer to a GUID identifying the device connected to the serial port.
    ///
    /// This is either `Ok` if [`Self::revision`] is at least
    /// [`SerialIoProtocolRevision::REVISION_1P1`] or `Err` with
    /// [`Status::UNSUPPORTED`].
    ///
    /// This field is `None` when the protocol is installed by the serial port
    /// driver and may be populated by a platform driver for a serial port with
    /// a known device attached. The field will remain `None` if there is no
    /// platform serial device identification information available.
    ///
    /// # Errors
    ///
    /// - [`Status::UNSUPPORTED`]: If the revision is older than
    ///   [`SerialIoProtocolRevision::REVISION_1P1`].
    pub fn device_type_guid(&self) -> Result<Option<&'_ Guid>> {
        if self.revision() < SerialIoProtocolRevision::REVISION_1P1 {
            return Err(Error::from(Status::UNSUPPORTED));
        }
        // SAFETY: We trust the pointer is either null or points to a valid
        // object.
        let maybe_guid = unsafe { self.0.device_type_guid.as_ref() };
        Ok(maybe_guid)
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).map(|_| ()).map_err(|_| fmt::Error)
    }
}
