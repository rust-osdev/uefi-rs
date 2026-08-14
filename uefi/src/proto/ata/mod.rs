// SPDX-License-Identifier: MIT OR Apache-2.0

//! ATA protocols.

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

/// ATA command protocol used by the pass-through interface.
///
/// UEFI abstracts the transports for ATA commands behind this protocol, so
/// the same API supports PATA and modern AHCI-only SATA controllers. See the
/// [Platform Initialization specification].
///
/// [Platform Initialization specification]: https://uefi.org/specs/PI/1.8/V5_IDE_Controller.html
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
/// Its chainable methods configure the timeout, buffers, and ATA command
/// fields.
#[derive(Debug)]
pub struct AtaRequestBuilder<'a> {
    req: AtaRequest<'a>,
}

impl<'a> AtaRequestBuilder<'a> {
    /// Creates a new [`AtaRequestBuilder`].
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    /// - `command`: ATA command byte to execute.
    /// - `protocol`: Transport protocol for the command.
    ///
    /// # Errors
    /// Returns an error if the status buffer cannot satisfy `io_align`.
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

    // # PIO
    // ########################################################################

    /// Creates a builder for a PIO read operation.
    ///
    /// Since the ATA specification mandates the support for PIO mode for all
    /// compliant drives and controllers, this is the protocol variant with the
    /// highest compatibility. Prefer it when probing ports with ATA IDENTIFY
    /// commands to detect connected devices.
    /// If this returns [`uefi_raw::Status::UNSUPPORTED`], try UDMA next.
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    /// - `command`: ATA command byte to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if the status buffer cannot satisfy `io_align`.
    pub fn read_pio(io_align: u32, command: u8) -> Result<Self, LayoutError> {
        Self::new(io_align, command, AtaPassThruCommandProtocol::PIO_DATA_IN)
    }

    // # UDMA
    // ########################################################################

    /// Creates a builder for a UDMA read operation.
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    /// - `command`: ATA command byte to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if the status buffer cannot satisfy `io_align`.
    pub fn read_udma(io_align: u32, command: u8) -> Result<Self, LayoutError> {
        Self::new(io_align, command, AtaPassThruCommandProtocol::UDMA_DATA_IN)
    }

    /// Creates a builder for a UDMA write operation.
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    /// - `command`: ATA command byte to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if the status buffer cannot satisfy `io_align`.
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
    /// FEATURES (7:0)
    #[must_use]
    pub const fn with_features(mut self, features: u8) -> Self {
        self.req.acb.features = features;
        self
    }

    /// Configure the `sector_number` field.
    /// LBA (7:0)
    #[must_use]
    pub const fn with_sector_number(mut self, sector_number: u8) -> Self {
        self.req.acb.sector_number = sector_number;
        self
    }

    /// Configure the `cylinder` fields (low and high combined).
    /// low:  LBA (15:8)
    /// high: LBA (23:16)
    #[must_use]
    pub const fn with_cylinder(mut self, low: u8, high: u8) -> Self {
        self.req.acb.cylinder_low = low;
        self.req.acb.cylinder_high = high;
        self
    }

    /// Configure the `device_head` field.
    ///
    /// This field contains the ATA DEVICE bit.
    #[must_use]
    pub const fn with_device_head(mut self, device_head: u8) -> Self {
        self.req.acb.device_head = device_head;
        self
    }

    /// Configure the `sector_number_exp` field.
    /// LBA (31:24)
    #[must_use]
    pub const fn with_sector_number_exp(mut self, sector_number_exp: u8) -> Self {
        self.req.acb.sector_number_exp = sector_number_exp;
        self
    }

    /// Configure the `cylinder_exp` fields (low and high combined).
    /// low_exp:  LBA (39:32)
    /// high_exp: LBA (47:40)
    #[must_use]
    pub const fn with_cylinder_exp(mut self, low_exp: u8, high_exp: u8) -> Self {
        self.req.acb.cylinder_low_exp = low_exp;
        self.req.acb.cylinder_high_exp = high_exp;
        self
    }

    /// Configure the `features_exp` field.
    /// FEATURES (15:8)
    #[must_use]
    pub const fn with_features_exp(mut self, features_exp: u8) -> Self {
        self.req.acb.features_exp = features_exp;
        self
    }

    /// Configure the `sector_count` field.
    /// COUNT (7:0)
    #[must_use]
    pub const fn with_sector_count(mut self, sector_count: u8) -> Self {
        self.req.acb.sector_count = sector_count;
        self
    }

    /// Configure the `sector_count_exp` field.
    /// COUNT (15:8)
    #[must_use]
    pub const fn with_sector_count_exp(mut self, sector_count_exp: u8) -> Self {
        self.req.acb.sector_count_exp = sector_count_exp;
        self
    }

    // # READ BUFFER
    // ########################################################################################

    /// Uses `bfr` to receive data from the device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer in which to receive the data.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not meet the protocol's alignment
    /// requirement.
    pub fn use_read_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.in_data_buffer = None;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates a `len`-byte buffer to receive data from the device.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested buffer layout is invalid.
    pub fn with_read_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.in_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.in_transfer_length = bfr.size() as u32;
        self.req.in_data_buffer = Some(bfr);
        Ok(self)
    }

    // # WRITE BUFFER
    // ########################################################################################

    /// Uses `bfr` to send data to the device.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer containing the data to send.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not meet the protocol's alignment
    /// requirement.
    pub fn use_write_buffer(mut self, bfr: &'a mut AlignedBuffer) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.out_data_buffer = None;
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates a write buffer, copies `data` into it, and uses it for the
    /// request.
    ///
    /// # Arguments
    ///
    /// - `data`: Data to copy into the request buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested buffer layout is invalid.
    pub fn with_write_data(mut self, data: &[u8]) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(data.len(), self.req.io_align as usize)?;
        bfr.copy_from_slice(data);
        self.req.packet.out_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.out_transfer_length = bfr.size() as u32;
        self.req.out_data_buffer = Some(bfr);
        Ok(self)
    }

    /// Builds the configured [`AtaRequest`].
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
    /// Returns the status block from the response.
    #[must_use]
    pub const fn status(&self) -> &'a AtaStatusBlock {
        // SAFETY: The memory is valid.
        unsafe {
            self.req
                .asb
                .ptr()
                .cast::<AtaStatusBlock>()
                .as_ref()
                .unwrap()
        }
    }

    /// Returns the data read from the device, if a read buffer was used.
    #[must_use]
    pub const fn read_buffer(&self) -> Option<&'a [u8]> {
        if self.req.packet.in_data_buffer.is_null() {
            return None;
        }
        // SAFETY: The memory is valid.
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.in_data_buffer.cast(),
                self.req.packet.in_transfer_length as usize,
            ))
        }
    }
}
