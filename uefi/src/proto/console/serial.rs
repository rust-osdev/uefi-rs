//! Abstraction over byte stream devices, also known as serial I/O devices.

use crate::proto::unsafe_protocol;
use crate::{Result, Status, StatusExt};
use core::fmt::Write;

pub use uefi_raw::protocol::console::serial::SerialIoMode as IoMode;
pub use uefi_raw::protocol::console::serial::{ControlBits, Parity, StopBits};

/// Provides access to a serial I/O device.
///
/// This can include standard UART devices, serial ports over a USB interface,
/// or any other character-based communication device.
///
/// Since UEFI drivers are implemented through polling, if you fail to regularly
/// check for input/output, some data might be lost.
#[repr(C)]
#[unsafe_protocol("bb25cf6f-f1d4-11d2-9a0c-0090273fc1fd")]
pub struct Serial {
    // Revision of this protocol, only 1.0 is currently defined.
    // Future versions will be backwards compatible.
    revision: u32,
    reset: extern "efiapi" fn(&mut Serial) -> Status,
    set_attributes: extern "efiapi" fn(
        &Serial,
        baud_rate: u64,
        receive_fifo_depth: u32,
        timeout: u32,
        parity: Parity,
        data_bits: u8,
        stop_bits_type: StopBits,
    ) -> Status,
    set_control_bits: extern "efiapi" fn(&mut Serial, ControlBits) -> Status,
    get_control_bits: extern "efiapi" fn(&Serial, &mut ControlBits) -> Status,
    write: unsafe extern "efiapi" fn(&mut Serial, &mut usize, *const u8) -> Status,
    read: unsafe extern "efiapi" fn(&mut Serial, &mut usize, *mut u8) -> Status,
    io_mode: *const IoMode,
}

impl Serial {
    /// Reset the device.
    pub fn reset(&mut self) -> Result {
        (self.reset)(self).to_result()
    }

    /// Returns the current I/O mode.
    #[must_use]
    pub const fn io_mode(&self) -> &IoMode {
        unsafe { &*self.io_mode }
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
        (self.set_attributes)(
            self,
            mode.baud_rate,
            mode.receive_fifo_depth,
            mode.timeout,
            mode.parity,
            mode.data_bits as u8,
            mode.stop_bits,
        )
        .to_result()
    }

    /// Retrieve the device's current control bits.
    pub fn get_control_bits(&self) -> Result<ControlBits> {
        let mut bits = ControlBits::empty();
        (self.get_control_bits)(self, &mut bits).to_result_with_val(|| bits)
    }

    /// Sets the device's new control bits.
    ///
    /// Not all bits can be modified with this function. A mask of the allowed
    /// bits is stored in the [`ControlBits::SETTABLE`] constant.
    pub fn set_control_bits(&mut self, bits: ControlBits) -> Result {
        (self.set_control_bits)(self, bits).to_result()
    }

    /// Reads data from this device.
    ///
    /// This operation will block until the buffer has been filled with data or
    /// an error occurs. In the latter case, the error will indicate how many
    /// bytes were actually read from the device.
    pub fn read(&mut self, data: &mut [u8]) -> Result<(), usize> {
        let mut buffer_size = data.len();
        unsafe { (self.read)(self, &mut buffer_size, data.as_mut_ptr()) }.to_result_with(
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
        unsafe { (self.write)(self, &mut buffer_size, data.as_ptr()) }.to_result_with(
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
