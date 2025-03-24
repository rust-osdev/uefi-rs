// SPDX-License-Identifier: MIT OR Apache-2.0

//! SCSI Bus specific protocols.

use crate::mem::{AlignedBuffer, AlignmentError};
use core::alloc::LayoutError;
use core::marker::PhantomData;
use core::ptr;
use core::time::Duration;
use uefi_raw::protocol::scsi::{
    ScsiIoDataDirection, ScsiIoHostAdapterStatus, ScsiIoScsiRequestPacket, ScsiIoTargetStatus,
};

pub mod pass_thru;

/// Represents the data direction for a SCSI request.
///
/// Used to specify whether the request involves reading, writing, or bidirectional data transfer.
pub type ScsiRequestDirection = uefi_raw::protocol::scsi::ScsiIoDataDirection;

/// Represents a SCSI request packet.
///
/// This structure encapsulates the necessary data for sending a command to a SCSI device.
#[derive(Debug)]
pub struct ScsiRequest<'a> {
    packet: ScsiIoScsiRequestPacket,
    io_align: u32,
    in_data_buffer: Option<AlignedBuffer>,
    out_data_buffer: Option<AlignedBuffer>,
    sense_data_buffer: Option<AlignedBuffer>,
    cdb_buffer: Option<AlignedBuffer>,
    _phantom: PhantomData<&'a u8>,
}

/// A builder for constructing [`ScsiRequest`] instances.
///
/// Provides a safe and ergonomic interface for configuring SCSI request packets, including timeout,
/// data buffers, and command descriptor blocks.
#[derive(Debug)]
pub struct ScsiRequestBuilder<'a> {
    req: ScsiRequest<'a>,
}
impl ScsiRequestBuilder<'_> {
    /// Creates a new instance with the specified data direction and alignment.
    ///
    /// # Parameters
    /// - `direction`: Specifies the direction of data transfer (READ, WRITE, or BIDIRECTIONAL).
    /// - `io_align`: Specifies the required alignment for data buffers. (SCSI Controller specific!)
    #[must_use]
    pub fn new(direction: ScsiRequestDirection, io_align: u32) -> Self {
        Self {
            req: ScsiRequest {
                in_data_buffer: None,
                out_data_buffer: None,
                sense_data_buffer: None,
                cdb_buffer: None,
                packet: ScsiIoScsiRequestPacket {
                    timeout: 0,
                    in_data_buffer: ptr::null_mut(),
                    out_data_buffer: ptr::null_mut(),
                    sense_data: ptr::null_mut(),
                    cdb: ptr::null_mut(),
                    in_transfer_length: 0,
                    out_transfer_length: 0,
                    cdb_length: 0,
                    data_direction: direction,
                    host_adapter_status: ScsiIoHostAdapterStatus::default(),
                    target_status: ScsiIoTargetStatus::default(),
                    sense_data_length: 0,
                },
                io_align,
                _phantom: Default::default(),
            },
        }
    }

    /// Starts a new builder preconfigured for READ operations.
    ///
    /// Some examples of SCSI read commands are:
    /// - INQUIRY
    /// - READ
    /// - MODE_SENSE
    ///
    /// # Parameters
    /// - `io_align`: Specifies the required alignment for data buffers.
    #[must_use]
    pub fn read(io_align: u32) -> Self {
        Self::new(ScsiIoDataDirection::READ, io_align)
    }

    /// Starts a new builder preconfigured for WRITE operations.
    ///
    /// Some examples of SCSI write commands are:
    /// - WRITE
    /// - MODE_SELECT
    ///
    /// # Parameters
    /// - `io_align`: Specifies the required alignment for data buffers.
    #[must_use]
    pub fn write(io_align: u32) -> Self {
        Self::new(ScsiIoDataDirection::WRITE, io_align)
    }

    /// Starts a new builder preconfigured for BIDIRECTIONAL operations.
    ///
    /// Some examples of SCSI bidirectional commands are:
    /// - SEND DIAGNOSTIC
    ///
    /// # Parameters
    /// - `io_align`: Specifies the required alignment for data buffers.
    #[must_use]
    pub fn bidirectional(io_align: u32) -> Self {
        Self::new(ScsiIoDataDirection::BIDIRECTIONAL, io_align)
    }
}

impl<'a> ScsiRequestBuilder<'a> {
    /// Sets a timeout for the SCSI request.
    ///
    /// # Parameters
    /// - `timeout`: A [`Duration`] representing the maximum time allowed for the request.
    ///   The value is converted to 100-nanosecond units.
    ///
    /// # Description
    /// By default (without calling this method, or by calling with [`Duration::ZERO`]),
    /// SCSI requests have no timeout.
    /// Setting a timeout here will cause SCSI commands to potentially fail with [`crate::Status::TIMEOUT`].
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.req.packet.timeout = (timeout.as_nanos() / 100) as u64;
        self
    }

    // # IN BUFFER
    // ########################################################################################

    /// Uses a user-supplied buffer for reading data from the device.
    ///
    /// # Parameters
    /// - `bfr`: A mutable reference to an [`AlignedBuffer`] that will be used to store data read from the device.
    ///
    /// # Returns
    /// `Result<Self, AlignmentError>` indicating success or an alignment issue with the provided buffer.
    ///
    /// # Description
    /// This method checks the alignment of the buffer against the protocol's requirements and assigns it to
    /// the `in_data_buffer` of the underlying `ScsiRequest`.
    pub fn use_read_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.in_data_buffer = None;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Adds a newly allocated read buffer to the built SCSI request.
    ///
    /// # Parameters
    /// - `len`: The size of the buffer (in bytes) to allocate for receiving data.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
    pub fn with_read_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        self.req.in_data_buffer = Some(bfr);
        Ok(self)
    }

    // # SENSE BUFFER
    // ########################################################################################

    /// Adds a newly allocated sense buffer to the built SCSI request.
    ///
    /// # Parameters
    /// - `len`: The size of the buffer (in bytes) to allocate for receiving sense data.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
    pub fn with_sense_buffer(mut self, len: u8) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len as usize, self.req.io_align as usize)?;
        self.req.packet.sense_data = bfr.ptr_mut().cast();
        self.req.packet.sense_data_length = len;
        self.req.sense_data_buffer = Some(bfr);
        Ok(self)
    }

    // # WRITE BUFFER
    // ########################################################################################

    /// Uses a user-supplied buffer for writing data to the device.
    ///
    /// # Parameters
    /// - `bfr`: A mutable reference to an [`AlignedBuffer`] containing the data to be written to the device.
    ///
    /// # Returns
    /// `Result<Self, AlignmentError>` indicating success or an alignment issue with the provided buffer.
    ///
    /// # Description
    /// This method checks the alignment of the buffer against the protocol's requirements and assigns it to
    /// the `out_data_buffer` of the underlying `ScsiRequest`.
    pub fn use_write_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.out_data_buffer = None;
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Adds a newly allocated write buffer to the built SCSI request that is filled from the
    /// given data buffer. (Done for memory alignment and lifetime purposes)
    ///
    /// # Parameters
    /// - `data`: A slice of bytes representing the data to be written.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
    pub fn with_write_data(mut self, data: &[u8]) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(data.len(), self.req.io_align as usize)?;
        bfr.copy_from_slice(data);
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        self.req.out_data_buffer = Some(bfr);
        Ok(self)
    }

    // # COMMAND BUFFER
    // ########################################################################################

    /// Uses a user-supplied Command Data Block (CDB) buffer.
    ///
    /// # Parameters
    /// - `data`: A mutable reference to an [`AlignedBuffer`] containing the CDB to be sent to the device.
    ///
    /// # Returns
    /// `Result<Self, AlignmentError>` indicating success or an alignment issue with the provided buffer.
    ///
    /// # Notes
    /// The maximum length of a CDB is 255 bytes.
    pub fn use_command_buffer(
        mut self,
        data: &'a mut AlignedBuffer,
    ) -> Result<Self, AlignmentError> {
        assert!(data.size() <= 255);
        // check alignment of externally supplied buffer
        data.check_alignment(self.req.io_align as usize)?;
        self.req.cdb_buffer = None;
        self.req.packet.cdb = data.ptr_mut().cast();
        self.req.packet.cdb_length = data.size() as u8;
        Ok(self)
    }

    /// Adds a newly allocated Command Data Block (CDB) buffer to the built SCSI request that is filled from the
    /// given data buffer. (Done for memory alignment and lifetime purposes)
    ///
    /// # Parameters
    /// - `data`: A slice of bytes representing the command to be sent.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
    ///
    /// # Notes
    /// The maximum length of a CDB is 255 bytes.
    pub fn with_command_data(mut self, data: &[u8]) -> Result<Self, LayoutError> {
        assert!(data.len() <= 255);
        let mut bfr = AlignedBuffer::from_size_align(data.len(), self.req.io_align as usize)?;
        bfr.copy_from_slice(data);
        self.req.packet.cdb = bfr.ptr_mut().cast();
        self.req.packet.cdb_length = bfr.size() as u8;
        self.req.cdb_buffer = Some(bfr);
        Ok(self)
    }

    /// Build the final `ScsiRequest`.
    ///
    /// # Returns
    /// A fully-configured [`ScsiRequest`] ready for execution.
    #[must_use]
    pub fn build(self) -> ScsiRequest<'a> {
        self.req
    }
}

/// Represents the response of a SCSI request.
///
/// This struct encapsulates the results of a SCSI operation, including data buffers
/// for read and sense data, as well as status codes returned by the host adapter and target device.
#[derive(Debug)]
#[repr(transparent)]
pub struct ScsiResponse<'a>(ScsiRequest<'a>);
impl<'a> ScsiResponse<'a> {
    /// Retrieves the buffer containing data read from the device (if any).
    ///
    /// # Returns
    /// `Option<&[u8]>`: A slice of the data read from the device, or `None` if no read buffer was assigned.
    ///
    /// # Safety
    /// - If the buffer pointer is `NULL`, the method returns `None` and avoids dereferencing it.
    #[must_use]
    pub fn read_buffer(&self) -> Option<&'a [u8]> {
        if self.0.packet.in_data_buffer.is_null() {
            return None;
        }
        unsafe {
            Some(core::slice::from_raw_parts(
                self.0.packet.in_data_buffer.cast(),
                self.0.packet.in_transfer_length as usize,
            ))
        }
    }

    /// Retrieves the buffer containing sense data returned by the device (if any).
    ///
    /// # Returns
    /// `Option<&[u8]>`: A slice of the sense data, or `None` if no sense data buffer was assigned.
    ///
    /// # Safety
    /// - If the buffer pointer is `NULL`, the method returns `None` and avoids dereferencing it.
    #[must_use]
    pub fn sense_data(&self) -> Option<&'a [u8]> {
        if self.0.packet.sense_data.is_null() {
            return None;
        }
        unsafe {
            Some(core::slice::from_raw_parts(
                self.0.packet.sense_data.cast(),
                self.0.packet.sense_data_length as usize,
            ))
        }
    }

    /// Retrieves the status of the host adapter after executing the SCSI request.
    ///
    /// # Returns
    /// [`ScsiIoHostAdapterStatus`]: The status code indicating the result of the operation from the host adapter.
    #[must_use]
    pub const fn host_adapter_status(&self) -> ScsiIoHostAdapterStatus {
        self.0.packet.host_adapter_status
    }

    /// Retrieves the status of the target device after executing the SCSI request.
    ///
    /// # Returns
    /// [`ScsiIoTargetStatus`]: The status code returned by the target device.
    #[must_use]
    pub const fn target_status(&self) -> ScsiIoTargetStatus {
        self.0.packet.target_status
    }
}
