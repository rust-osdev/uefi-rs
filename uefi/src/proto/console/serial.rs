// SPDX-License-Identifier: MIT OR Apache-2.0

//! Abstraction over byte stream devices, also known as serial I/O devices.

use crate::proto::unsafe_protocol;
use crate::{Error, Result, Status, StatusExt};
use core::fmt::Write;
use uefi_raw::protocol::console::serial::{
    SerialIoProtocol, SerialIoProtocol_1_1, SerialIoProtocolRevision,
};
use uguid::Guid;

pub use uefi_raw::protocol::console::serial::{
    ControlBits, Parity, SerialIoMode as IoMode, StopBits,
};

/// Serial IO [`Protocol`]. Provides access to a serial I/O device.
///
/// This can include standard UART devices, serial ports over a USB interface,
/// or any other character-based communication device. The protocol is
/// typically used to connect to a terminal.
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

    /// Reads data from this device.
    ///
    /// This operation will block until the buffer has been filled with data or
    /// an error occurs. In the latter case, the error will indicate how many
    /// bytes were actually read from the device.
    pub fn read(&mut self, data: &mut [u8]) -> Result<(), usize> {
        let mut buffer_size = data.len();
        unsafe { (self.0.read)(&mut self.0, &mut buffer_size, data.as_mut_ptr()) }.to_result_with(
            || debug_assert_eq!(buffer_size, data.len()),
            |_| buffer_size,
        )
    }

    /// Writes data to this device.
    ///
    /// This operation will block until the data has been fully written or an
    /// error occurs. In the latter case, the error will indicate how many bytes
    /// were actually written to the device.
    pub fn write(&mut self, data: &[u8]) -> Result<(), usize> {
        let mut buffer_size = data.len();
        unsafe { (self.0.write)(&mut self.0, &mut buffer_size, data.as_ptr()) }.to_result_with(
            || debug_assert_eq!(buffer_size, data.len()),
            |_| buffer_size,
        )
    }

    /// Pointer to a GUID identifying the device connected to the serial port.
    ///
    /// This is either `Ok` if [`Self::revision`] is at least
    /// [`SerialIoProtocolRevision::REVISION_1_1`] or `Err` with
    /// [`Status::UNSUPPORTED`].
    ///
    /// This GUID is `None` when the protocol is installed by the serial port
    /// driver and may be populated by a platform driver for a serial port with
    /// a known device attached. The GUID will remain `None` if there is no
    /// platform serial device identification information available.
    ///
    /// # Errors
    ///
    /// - [`Status::UNSUPPORTED`]: If the revision is older than
    ///   [`SerialIoProtocolRevision::REVISION_1_1`].
    pub fn device_type_guid(&self) -> Result<Option<&'_ Guid>> {
        let proto = self.as_revision_1_1()?;
        // SAFETY: spec guarantees the layout of the underlying type
        let device_type_guid = unsafe { proto.device_type_guid.as_ref() };
        Ok(device_type_guid)
    }

    /// Casts the underlying [`SerialIoProtocol`] to an
    /// [`SerialIoProtocol_1_1`].
    fn as_revision_1_1(&self) -> Result<&'_ SerialIoProtocol_1_1> {
        if self.revision() < SerialIoProtocolRevision::REVISION_1_1 {
            return Err(Error::from(Status::UNSUPPORTED));
        }

        let ptr = &raw const self.0;
        // SAFETY: ptr is guaranteed to be not null and by checking the revision
        // we know the underlying allocation has the correct size.
        let protocol = unsafe {
            ptr.cast::<SerialIoProtocol_1_1>()
                .as_ref()
                .unwrap_unchecked()
        };
        Ok(protocol)
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}
