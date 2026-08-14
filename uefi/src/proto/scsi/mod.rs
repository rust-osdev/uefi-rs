// SPDX-License-Identifier: MIT OR Apache-2.0

//! SCSI bus protocols.

use crate::mem::{AlignedBuffer, AlignmentError};
use core::alloc::LayoutError;
use core::marker::PhantomData;
use core::ptr;
use core::time::Duration;
use uefi_raw::protocol::scsi::{
    ScsiIoDataDirection, ScsiIoHostAdapterStatus, ScsiIoScsiRequestPacket, ScsiIoTargetStatus,
};

pub mod pass_thru;

/// Data direction for a SCSI request.
///
/// Specifies whether a request reads, writes, or transfers data bidirectionally.
pub type ScsiRequestDirection = uefi_raw::protocol::scsi::ScsiIoDataDirection;

/// Represents a SCSI request packet.
///
/// Contains the command and buffers needed to communicate with a SCSI device.
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

/// Builder for constructing [`ScsiRequest`] values.
///
/// Its methods configure the timeout, data buffers, sense buffer, and command
/// descriptor block (CDB).
#[derive(Debug)]
pub struct ScsiRequestBuilder<'a> {
    req: ScsiRequest<'a>,
}
impl ScsiRequestBuilder<'_> {
    /// Creates a new request builder.
    ///
    /// # Arguments
    ///
    /// - `direction`: Direction of data transfer for the request.
    /// - `io_align`: Controller I/O buffer alignment requirement.
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
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    ///
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
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    ///
    #[must_use]
    pub fn write(io_align: u32) -> Self {
        Self::new(ScsiIoDataDirection::WRITE, io_align)
    }

    /// Starts a new builder preconfigured for BIDIRECTIONAL operations.
    ///
    /// Some examples of SCSI bidirectional commands are:
    /// - SEND DIAGNOSTIC
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    ///
    #[must_use]
    pub fn bidirectional(io_align: u32) -> Self {
        Self::new(ScsiIoDataDirection::BIDIRECTIONAL, io_align)
    }
}

impl<'a> ScsiRequestBuilder<'a> {
    /// Sets a timeout for the SCSI request.
    ///
    /// Sets this request's timeout in 100-nanosecond units. A zero duration
    /// disables the timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.req.packet.timeout = (timeout.as_nanos() / 100) as u64;
        self
    }

    // # IN BUFFER
    // ########################################################################################

    /// Uses a caller-provided buffer to receive data from the device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer in which to receive data.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not satisfy the required alignment.
    pub fn use_read_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.in_data_buffer = None;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates a read buffer for the SCSI request.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn with_read_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        self.req.in_data_buffer = Some(bfr);
        Ok(self)
    }

    // # SENSE BUFFER
    // ########################################################################################

    /// Allocates a sense-data buffer for the SCSI request.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn with_sense_buffer(mut self, len: u8) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len as usize, self.req.io_align as usize)?;
        self.req.packet.sense_data = bfr.ptr_mut().cast();
        self.req.packet.sense_data_length = len;
        self.req.sense_data_buffer = Some(bfr);
        Ok(self)
    }

    // # WRITE BUFFER
    // ########################################################################################

    /// Uses a caller-provided buffer to send data to the device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer containing the data to send.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not satisfy the required alignment.
    pub fn use_write_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.out_data_buffer = None;
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates an aligned write buffer and copies `data` into it.
    ///
    /// # Arguments
    ///
    /// - `data`: Data to copy into the request buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
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

    /// Uses a caller-provided command descriptor block (CDB).
    ///
    /// # Arguments
    ///
    /// - `data`: CDB to attach to the request.
    ///
    /// # Errors
    ///
    /// Returns an error if `data` does not satisfy the required alignment.
    ///
    /// # Panics
    ///
    /// Panics if `data` is longer than 255 bytes.
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

    /// Allocates an aligned command descriptor block and copies `data` into it.
    ///
    /// # Arguments
    ///
    /// - `data`: CDB to copy into the request.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    ///
    /// # Panics
    ///
    /// Panics if `data` is longer than 255 bytes.
    pub fn with_command_data(mut self, data: &[u8]) -> Result<Self, LayoutError> {
        assert!(data.len() <= 255);
        let mut bfr = AlignedBuffer::from_size_align(data.len(), self.req.io_align as usize)?;
        bfr.copy_from_slice(data);
        self.req.packet.cdb = bfr.ptr_mut().cast();
        self.req.packet.cdb_length = bfr.size() as u8;
        self.req.cdb_buffer = Some(bfr);
        Ok(self)
    }

    /// Builds the configured [`ScsiRequest`].
    #[must_use]
    pub fn build(self) -> ScsiRequest<'a> {
        self.req
    }
}

/// Response returned after executing a SCSI request.
///
/// Contains read and sense-data buffers together with status codes from the
/// host adapter and target device.
#[derive(Debug)]
#[repr(transparent)]
pub struct ScsiResponse<'a>(ScsiRequest<'a>);
impl<'a> ScsiResponse<'a> {
    /// Returns data read from the device, if a read buffer was assigned.
    #[must_use]
    pub const fn read_buffer(&self) -> Option<&'a [u8]> {
        if self.0.packet.in_data_buffer.is_null() {
            return None;
        }
        // SAFETY: The memory is valid.
        unsafe {
            Some(core::slice::from_raw_parts(
                self.0.packet.in_data_buffer.cast(),
                self.0.packet.in_transfer_length as usize,
            ))
        }
    }

    /// Returns sense data from the device, if a sense buffer was assigned.
    #[must_use]
    pub const fn sense_data(&self) -> Option<&'a [u8]> {
        if self.0.packet.sense_data.is_null() {
            return None;
        }
        // SAFETY: The memory is valid.
        unsafe {
            Some(core::slice::from_raw_parts(
                self.0.packet.sense_data.cast(),
                self.0.packet.sense_data_length as usize,
            ))
        }
    }

    /// Returns the host adapter's status after executing the request.
    #[must_use]
    pub const fn host_adapter_status(&self) -> ScsiIoHostAdapterStatus {
        self.0.packet.host_adapter_status
    }

    /// Returns the target device's status after executing the request.
    #[must_use]
    pub const fn target_status(&self) -> ScsiIoTargetStatus {
        self.0.packet.target_status
    }
}
