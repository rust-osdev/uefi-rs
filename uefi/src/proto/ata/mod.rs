// SPDX-License-Identifier: MIT OR Apache-2.0

//! ATA Protocols.

use crate::mem::{AlignedBuffer, AlignmentError};
use crate::util::usize_from_u32;
use core::alloc::LayoutError;
use core::marker::PhantomData;
use core::ptr;
use core::time::Duration;
use uefi_raw::protocol::ata::{
    AtaCommandBlock, AtaPassThruCommandPacket, AtaPassThruLength, AtaStatusBlock,
};

pub mod pass_thru;

/// Represents the protocol for ATA Pass Thru command handling.
///
/// This type defines the protocols supported for passing commands through the ATA controller.
pub use uefi_raw::protocol::ata::AtaPassThruCommandProtocol;

/// Represents an ATA request built for execution on an ATA controller.
#[derive(Debug)]
pub struct AtaRequest<'a> {
    io_align: u32,
    acb: AtaCommandBlock,
    packet: AtaPassThruCommandPacket,
    in_data_buffer: Option<AlignedBuffer>,
    out_data_buffer: Option<AlignedBuffer>,
    asb: AlignedBuffer,
    _phantom: PhantomData<&'a u8>,
}

/// Builder for creating and configuring an [`AtaRequest`].
///
/// This builder simplifies the creation of an [`AtaRequest`] by providing chainable methods for
/// configuring fields like timeout, buffers, and ATA command details.
#[derive(Debug)]
pub struct AtaRequestBuilder<'a> {
    req: AtaRequest<'a>,
}

impl<'a> AtaRequestBuilder<'a> {
    /// Creates a new [`AtaRequestBuilder`] with the specified alignment, command, and protocol.
    ///
    /// # Parameters
    /// - `io_align`: The I/O buffer alignment required for the ATA controller.
    /// - `command`: The ATA command byte specifying the operation to execute.
    /// - `protocol`: The protocol type for the command (e.g., DMA, UDMA, etc.).
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or memory allocation failure.
    ///
    /// # Errors
    /// This method can fail due to alignment or memory allocation issues.
    fn new(
        io_align: u32,
        command: u8,
        protocol: AtaPassThruCommandProtocol,
    ) -> Result<Self, LayoutError> {
        // status block has alignment requirements!
        let mut asb =
            AlignedBuffer::from_size_align(size_of::<AtaStatusBlock>(), usize_from_u32(io_align))?;
        Ok(Self {
            req: AtaRequest {
                io_align,
                acb: AtaCommandBlock {
                    command,
                    ..Default::default()
                },
                packet: AtaPassThruCommandPacket {
                    asb: asb.ptr_mut().cast(),
                    acb: ptr::null(), // filled during execution
                    timeout: 0,
                    in_data_buffer: ptr::null_mut(),
                    out_data_buffer: ptr::null(),
                    in_transfer_length: 0,
                    out_transfer_length: 0,
                    protocol,
                    length: AtaPassThruLength::BYTES,
                },
                in_data_buffer: None,
                out_data_buffer: None,
                asb,
                _phantom: PhantomData,
            },
        })
    }

    /// Creates a builder for a UDMA read operation.
    ///
    /// # Parameters
    /// - `io_align`: The I/O buffer alignment required for the ATA controller.
    /// - `command`: The ATA command byte specifying the read operation.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or memory allocation failure.
    ///
    /// # Errors
    /// This method can fail due to alignment or memory allocation issues.
    pub fn read_udma(io_align: u32, command: u8) -> Result<Self, LayoutError> {
        Self::new(io_align, command, AtaPassThruCommandProtocol::UDMA_DATA_IN)
    }

    /// Creates a builder for a UDMA write operation.
    ///
    /// # Parameters
    /// - `io_align`: The I/O buffer alignment required for the ATA controller.
    /// - `command`: The ATA command byte specifying the write operation.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or memory allocation failure.
    ///
    /// # Errors
    /// This method can fail due to alignment or memory allocation issues.
    pub fn write_udma(io_align: u32, command: u8) -> Result<Self, LayoutError> {
        Self::new(io_align, command, AtaPassThruCommandProtocol::UDMA_DATA_OUT)
    }

    // ########################################################################

    /// Configure the given timeout for this request.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.req.packet.timeout = (timeout.as_nanos() / 100) as u64;
        self
    }

    /// Configure the `features` field.
    #[must_use]
    pub const fn with_features(mut self, features: u8) -> Self {
        self.req.acb.features = features;
        self
    }

    /// Configure the `sector_number` field.
    #[must_use]
    pub const fn with_sector_number(mut self, sector_number: u8) -> Self {
        self.req.acb.sector_number = sector_number;
        self
    }

    /// Configure the `cylinder` fields (low and high combined).
    #[must_use]
    pub const fn with_cylinder(mut self, low: u8, high: u8) -> Self {
        self.req.acb.cylinder_low = low;
        self.req.acb.cylinder_high = high;
        self
    }

    /// Configure the `device_head` field.
    #[must_use]
    pub const fn with_device_head(mut self, device_head: u8) -> Self {
        self.req.acb.device_head = device_head;
        self
    }

    /// Configure the `sector_number_exp` field.
    #[must_use]
    pub const fn with_sector_number_exp(mut self, sector_number_exp: u8) -> Self {
        self.req.acb.sector_number_exp = sector_number_exp;
        self
    }

    /// Configure the `cylinder_exp` fields (low and high combined).
    #[must_use]
    pub const fn with_cylinder_exp(mut self, low_exp: u8, high_exp: u8) -> Self {
        self.req.acb.cylinder_low_exp = low_exp;
        self.req.acb.cylinder_high_exp = high_exp;
        self
    }

    /// Configure the `features_exp` field.
    #[must_use]
    pub const fn with_features_exp(mut self, features_exp: u8) -> Self {
        self.req.acb.features_exp = features_exp;
        self
    }

    /// Configure the `sector_count` field.
    #[must_use]
    pub const fn with_sector_count(mut self, sector_count: u8) -> Self {
        self.req.acb.sector_count = sector_count;
        self
    }

    /// Configure the `sector_count_exp` field.
    #[must_use]
    pub const fn with_sector_count_exp(mut self, sector_count_exp: u8) -> Self {
        self.req.acb.sector_count_exp = sector_count_exp;
        self
    }

    // # READ BUFFER
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
    /// the `in_data_buffer` of the underlying [`AtaRequest`].
    pub fn use_read_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.in_data_buffer = None;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Adds a newly allocated read buffer to the built ATA request.
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
    /// the `out_data_buffer` of the underlying [`AtaRequest`].
    pub fn use_write_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.out_data_buffer = None;
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Adds a newly allocated write buffer to the built ATA request that is filled from the
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

    /// Build the final [`AtaRequest`].
    ///
    /// # Returns
    /// A fully-configured [`AtaRequest`] ready for execution.
    #[must_use]
    pub fn build(self) -> AtaRequest<'a> {
        self.req
    }
}

/// Represents a response from an ATA request.
///
/// This structure provides access to the status block, read buffer, and other
/// details returned by the ATA controller after executing a request.
#[derive(Debug)]
pub struct AtaResponse<'a> {
    req: AtaRequest<'a>,
}

impl<'a> AtaResponse<'a> {
    /// Retrieves the status block from the response.
    ///
    /// # Returns
    /// A reference to the [`AtaStatusBlock`] containing details about the status of the executed operation.
    #[must_use]
    pub fn status(&self) -> &'a AtaStatusBlock {
        unsafe {
            self.req
                .asb
                .ptr()
                .cast::<AtaStatusBlock>()
                .as_ref()
                .unwrap()
        }
    }

    /// Retrieves the buffer containing data read from the device (if available).
    ///
    /// # Returns
    /// `Option<&[u8]>`: A slice of the data read from the device, or `None` if no read buffer was used.
    #[must_use]
    pub fn read_buffer(&self) -> Option<&'a [u8]> {
        if self.req.packet.in_data_buffer.is_null() {
            return None;
        }
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.in_data_buffer.cast(),
                self.req.packet.in_transfer_length as usize,
            ))
        }
    }
}
