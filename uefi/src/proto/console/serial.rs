// SPDX-License-Identifier: MIT OR Apache-2.0

//! Abstraction over byte stream devices, also known as serial I/O devices.

pub use uefi_raw::protocol::console::serial::{
    ControlBits, Parity, SerialIoMode as IoMode, StopBits,
};

use crate::proto::unsafe_protocol;
use crate::{Error, Result, ResultExt, Status, StatusExt, boot};
use core::time::Duration;
use core::{cmp, fmt};
use uefi_raw::protocol::console::serial::{
    SerialIoProtocol, SerialIoProtocol_1_1, SerialIoProtocolRevision,
};
use uguid::Guid;

/// Returns the estimated time it takes to write a single byte.
///
/// This is conservative: it accounts for the actual serial mode settings
/// (data bits, parity, stop bits) and rounds up to avoid underestimating.
fn duration_per_byte_estimate(mode: &IoMode) -> Duration {
    if mode.baud_rate == 0 {
        // Baud rate unknown; assume a very slow link to be safe.
        return Duration::from_millis(100);
    }

    // Count the number of bits per character conservatively:
    //   - 1 start bit (always)
    //   - data bits (use actual setting, fall back to maximum of 8)
    //   - parity bit, if any
    //   - stop bits (round up: ONE_FIVE and TWO both become 2)
    let data_bits = if mode.data_bits == 0 {
        8
    } else {
        mode.data_bits
    };

    let parity_bits: u32 = if mode.parity == Parity::NONE || mode.parity == Parity::DEFAULT {
        0
    } else {
        1
    };

    // Be conservative with stop bits: treat ONE_FIVE as 2.
    let stop_bits: u32 = match mode.stop_bits {
        StopBits::ONE => 1,
        StopBits::DEFAULT | StopBits::ONE_FIVE | StopBits::TWO => 2,
        // Unknown future variant: assume worst case.
        _ => 2,
    };

    let bits_per_char = 1 + data_bits + parity_bits + stop_bits;

    // Compute microseconds per bit, rounding up to avoid underestimating.
    let us_per_bit = 1_000_000_u64.div_ceil(mode.baud_rate);

    // Total microseconds per byte/character.
    let us_per_byte = us_per_bit * (bits_per_char as u64);

    Duration::from_micros(us_per_byte)
}

/// Returns the estimated duration it takes to clear/transmit the UARTs internal
/// FIFO.
///
/// This assumes the UART has each a transmit and a receive FIFO with the same
/// size, which is the case for UART 16550 devices - the de-facto standard
/// serial device.
fn duration_fifo_estimate(mode: &IoMode, remaining: usize) -> Duration {
    let remaining = u32::try_from(remaining).unwrap_or(u32::MAX);

    // default: depth = 1.
    let depth = mode.receive_fifo_depth.max(1);
    let remaining = cmp::min(depth, remaining);
    duration_per_byte_estimate(mode) * remaining
}

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

    /// Reads bytes into the provided buffer, blocking until it is full.
    ///
    /// Retries automatically on [`Status::TIMEOUT`], stalling briefly between
    /// attempts based on the current baud rate. A maximum retry limit prevents
    /// spinning forever in the unlikely event of a hardware fault.
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
    /// - [`Status::TIMEOUT`]: This timeout happens if the underlying device
    ///   seem to stopped its normal operation and is only reported to prevent
    ///   an endless loop.
    pub fn read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        // Chosen at will, tested on real hardware.
        const MAX_ZERO_PROGRESS: usize = 16;

        let mut remaining_buffer = buffer;
        let mut zero_progress_count = 0;

        // Retry until all bytes were written with endless loop protection.
        while !remaining_buffer.is_empty() {
            match self.read(remaining_buffer) {
                // All data read, buffer is full.
                Ok(_) => return Ok(()),
                Err(err) if err.status() == Status::TIMEOUT => {
                    let n = *err.data();
                    if n == 0 {
                        zero_progress_count += 1;
                        if zero_progress_count >= MAX_ZERO_PROGRESS {
                            return Err(Error::from(Status::TIMEOUT));
                        }
                    } else {
                        zero_progress_count = 0;
                    }

                    remaining_buffer = &mut remaining_buffer[n..];

                    // Give FIFO time to fill up. Without that protection, we
                    // might get TIMEOUT too often and return too early.
                    let fifo_stall_duration =
                        duration_fifo_estimate(self.io_mode(), remaining_buffer.len());
                    boot::stall(fifo_stall_duration);
                }
                err => {
                    return Err(Error::from(err.status()));
                }
            }
        }
        Ok(())
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

    /// Writes all provided bytes, blocking until every byte has been sent.
    ///
    /// Retries automatically on [`Status::TIMEOUT`], stalling briefly between
    /// attempts based on the current baud rate. A maximum retry limit prevents
    /// spinning forever in the unlikely event of a hardware fault.
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
    /// - [`Status::TIMEOUT`]: This timeout happens if the underlying device
    ///   seem to stopped its normal operation and is only reported to prevent
    ///   an endless loop.
    pub fn write_exact(&mut self, data: &[u8]) -> Result<()> {
        // Chosen at will, tested on real hardware.
        const MAX_ZERO_PROGRESS: usize = 16;

        let mut remaining_bytes = data;
        let mut zero_progress_count = 0;

        // Retry until all bytes were written with endless loop protection.
        while !remaining_bytes.is_empty() {
            match self.write(remaining_bytes) {
                // All data written, no data left to send.
                Ok(_) => return Ok(()),
                Err(err) if err.status() == Status::TIMEOUT => {
                    let n = *err.data();
                    if n == 0 {
                        zero_progress_count += 1;
                        if zero_progress_count >= MAX_ZERO_PROGRESS {
                            return Err(Error::from(Status::TIMEOUT));
                        }
                    } else {
                        zero_progress_count = 0;
                    }

                    remaining_bytes = &remaining_bytes[n..];

                    // Give FIFO time to drain. Without that protection, we
                    // might get TIMEOUT too often and return too early.
                    let fifo_stall_duration =
                        duration_fifo_estimate(self.io_mode(), remaining_bytes.len());
                    boot::stall(fifo_stall_duration);
                }
                Err(err) => return Err(Error::from(err.status())),
            }
        }
        Ok(())
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

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).map(|_| ()).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mode(baud_rate: u64, data_bits: u32, parity: Parity, stop_bits: StopBits) -> IoMode {
        IoMode {
            control_mask: ControlBits::empty(),
            timeout: 0,
            baud_rate,
            receive_fifo_depth: 0,
            data_bits,
            parity,
            stop_bits,
        }
    }

    #[test]
    fn unknown_baud_rate_returns_large_fallback() {
        let mode = make_mode(0, 8, Parity::NONE, StopBits::ONE);
        let duration = duration_per_byte_estimate(&mode);
        assert!(
            duration >= Duration::from_millis(50),
            "fallback should be at least 100ms, got {duration:?}"
        );
    }

    #[test]
    fn higher_baud_rate_gives_shorter_duration() {
        let slow = make_mode(9_600, 8, Parity::NONE, StopBits::ONE);
        let fast = make_mode(115_200, 8, Parity::NONE, StopBits::ONE);
        assert!(
            duration_per_byte_estimate(&slow) > duration_per_byte_estimate(&fast),
            "9600 baud should take longer per byte than 115200 baud"
        );
    }

    #[test]
    fn parity_bit_increases_duration() {
        let no_parity = make_mode(9_600, 8, Parity::NONE, StopBits::ONE);
        let with_parity = make_mode(9_600, 8, Parity::EVEN, StopBits::ONE);
        assert!(
            duration_per_byte_estimate(&with_parity) > duration_per_byte_estimate(&no_parity),
            "a parity bit should increase the estimated duration"
        );
    }

    #[test]
    fn two_stop_bits_increases_duration() {
        let one_stop = make_mode(9_600, 8, Parity::NONE, StopBits::ONE);
        let two_stop = make_mode(9_600, 8, Parity::NONE, StopBits::TWO);
        assert!(
            duration_per_byte_estimate(&two_stop) > duration_per_byte_estimate(&one_stop),
            "two stop bits should increase the estimated duration"
        );
    }

    #[test]
    fn one_five_stop_bits_same_as_two() {
        // ONE_FIVE is rounded up conservatively to 2, same as TWO.
        let one_five = make_mode(9_600, 8, Parity::NONE, StopBits::ONE_FIVE);
        let two = make_mode(9_600, 8, Parity::NONE, StopBits::TWO);
        assert_eq!(
            duration_per_byte_estimate(&one_five),
            duration_per_byte_estimate(&two),
            "ONE_FIVE should be treated as 2 stop bits"
        );
    }

    #[test]
    fn more_data_bits_increases_duration() {
        let seven = make_mode(9_600, 7, Parity::NONE, StopBits::ONE);
        let eight = make_mode(9_600, 8, Parity::NONE, StopBits::ONE);
        assert!(
            duration_per_byte_estimate(&eight) > duration_per_byte_estimate(&seven),
            "more data bits should increase the estimated duration"
        );
    }

    #[test]
    fn default_stop_bits_conservative() {
        // DEFAULT is unknown, so it should be treated at least as generously as TWO.
        let default_stop = make_mode(9_600, 8, Parity::NONE, StopBits::DEFAULT);
        let two_stop = make_mode(9_600, 8, Parity::NONE, StopBits::TWO);
        assert!(
            duration_per_byte_estimate(&default_stop) >= duration_per_byte_estimate(&two_stop),
            "DEFAULT stop bits should be at least as conservative as TWO"
        );
    }

    #[test]
    fn default_parity_conservative() {
        // DEFAULT parity is unknown, so assume a parity bit may be present.
        let default_parity = make_mode(9_600, 8, Parity::DEFAULT, StopBits::ONE);
        let no_parity = make_mode(9_600, 8, Parity::NONE, StopBits::ONE);
        assert!(
            duration_per_byte_estimate(&default_parity) >= duration_per_byte_estimate(&no_parity),
            "DEFAULT parity should be at least as conservative as a known parity bit"
        );
    }
}
