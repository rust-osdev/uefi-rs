// SPDX-License-Identifier: MIT OR Apache-2.0

//! Abstraction over byte stream devices, also known as serial I/O devices.

pub use uefi_raw::protocol::console::serial::{
    ControlBits, Parity, SerialIoMode as IoMode, StopBits,
};

use crate::proto::unsafe_protocol;
use crate::{Error, Result, StatusExt};
use core::fmt::Write;
use uefi_raw::Status;
use uefi_raw::protocol::console::serial::SerialIoProtocol;
#[cfg(feature = "alloc")]
use {crate::ResultExt, alloc::vec::Vec};

/// Serial IO [`Protocol`]. Provides access to a serial I/O device.
///
/// This can include standard UART devices, serial ports over a USB interface,
/// or any other character-based communication device.
///
/// Since UEFI drivers are implemented through polling, if you fail to regularly
/// check for input/output, some data might be lost.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
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
    /// This function will try to fill the whole buffer with data read from the
    /// device. If this is not possible in the configured timeout (see
    /// [`IoMode`]), the function will return the data that it was able to read
    /// so far.
    ///
    /// To prevent missing data (overrun), it is recommended to call this
    /// function multiple times.
    pub fn read(
        &mut self,
        data: &mut [u8],
    ) -> Result<usize /* read bytes*/, usize /* read bytes */> {
        let mut buffer_size = data.len();
        let status = unsafe { (self.0.read)(&mut self.0, &mut buffer_size, data.as_mut_ptr()) };
        match status {
            Status::SUCCESS => Ok(buffer_size),
            // UEFI was not able to fill the whole buffer in the specified
            // timeout, but we still read data (good case).
            Status::TIMEOUT => Err(Error::new(status, buffer_size)),
            // any other error
            _ => Err(Error::new(status, buffer_size)),
        }
    }

    /// Reads all data that is currently available from the device.
    ///
    /// It is strongly recommended to configure a **very short timeout**
    /// (for example, `1 µs`) to avoid unintended blocking behavior
    /// (see [`IoMode`]).
    ///
    /// Note that the timeout applies to completion of the entire buffer:
    /// if a timeout of `5 s` is specified, a buffer of `1024` bytes is provided,
    /// and only `1023` bytes become available, this function will block for the
    /// full `5 s` before returning.
    #[cfg(feature = "alloc")]
    pub fn read_to_end(&mut self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        let mut buf = [0; 2048];
        loop {
            // We read from a temporary buffer to grow the vector dynamically
            // in the next step.
            let res = self.read(&mut buf);
            let status = res.status();
            let n = match (res, status) {
                (Ok(n), _) => n,
                // Okay, not an error in this case.
                (Err(err), Status::TIMEOUT) => *err.data(),
                (Err(err), _) => {
                    return Err(err.status().into());
                }
            };
            if n == 0 {
                break;
            } else {
                // Grew vector dynamically.
                vec.extend_from_slice(&buf[..n]);
            }
        }

        Ok(vec)
    }

    /// Writes data to this device.
    ///
    /// This operation will block until the data has been fully written or an
    /// error occurs. In the latter case, the error will indicate how many bytes
    /// were actually written to the device.
    pub fn write(&mut self, data: &[u8]) -> Result<(), usize> {
        let mut buffer_size = data.len();
        unsafe { (self.0.write)(&mut self.0, &mut buffer_size, data.as_ptr()) }
            .to_result_with(|| (), |_| buffer_size)
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}
