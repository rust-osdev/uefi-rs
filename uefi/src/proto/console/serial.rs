//! Abstraction over byte stream devices, also known as serial I/O devices.

use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use core::fmt::Write;
use uefi_raw::protocol::console::serial::SerialIoProtocol;

pub use uefi_raw::protocol::console::serial::SerialIoMode as IoMode;
pub use uefi_raw::protocol::console::serial::{ControlBits, Parity, StopBits};

/// Provides access to a serial I/O device.
///
/// This can include standard UART devices, serial ports over a USB interface,
/// or any other character-based communication device.
///
/// Since UEFI drivers are implemented through polling, if you fail to regularly
/// check for input/output, some data might be lost.
#[repr(transparent)]
#[unsafe_protocol(SerialIoProtocol::GUID)]
pub struct Serial(SerialIoProtocol);

impl Serial {
    /// Reset the device.
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
    /// The given `IoMode` will become the device's new `IoMode`,
    /// with some exceptions:
    ///
    /// - `control_mask` is ignored, since it's a read-only field;
    ///
    /// - values set to `0` / `Default` will be filled with the device's
    ///   default parameters
    ///
    /// - if either `baud_rate` or `receive_fifo_depth` is less than
    ///   the device's minimum, an error will be returned;
    ///   this value will be rounded down to the nearest value supported by the device;
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
    pub fn get_control_bits(&self) -> Result<ControlBits> {
        let mut bits = ControlBits::empty();
        unsafe { (self.0.get_control_bits)(&self.0, &mut bits) }.to_result_with_val(|| bits)
    }

    /// Sets the device's new control bits.
    ///
    /// Not all bits can be modified with this function. A mask of the allowed
    /// bits is stored in the [`ControlBits::SETTABLE`] constant.
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
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}
