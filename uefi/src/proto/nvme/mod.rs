// SPDX-License-Identifier: MIT OR Apache-2.0

//! NVM Express protocols.

use crate::mem::{AlignedBuffer, AlignmentError};
use core::alloc::LayoutError;
use core::marker::PhantomData;
use core::ptr;
use core::time::Duration;
use uefi_raw::protocol::nvme::{
    NvmExpressCommand, NvmExpressCommandCdwValidity, NvmExpressPassThruCommandPacket,
};

pub mod pass_thru;

/// Completion status of an NVMe command.
///
/// Contains error codes, command identifiers, and controller state returned by
/// an executed command.
pub type NvmeCompletion = uefi_raw::protocol::nvme::NvmExpressCompletion;

/// Queue on which to submit an NVMe command.
///
/// The selected command determines which queue type is valid.
pub type NvmeQueueType = uefi_raw::protocol::nvme::NvmExpressQueueType;

/// Request for executing an NVMe command.
///
/// Contains the command and optional transfer and metadata buffers. The
/// `'buffers` lifetime keeps attached I/O buffers alive while the response is
/// interpreted.
#[derive(Debug)]
pub struct NvmeRequest<'buffers> {
    io_align: u32,
    cmd: NvmExpressCommand,
    packet: NvmExpressPassThruCommandPacket,
    transfer_buffer: Option<AlignedBuffer>,
    meta_data_buffer: Option<AlignedBuffer>,
    _phantom: PhantomData<&'buffers u8>,
}

// NVMe commands consist of a bunch of CDWs (command data words) and a flags bitmask, where
// one bit per cdw is set when it should be read. Our request builder has one setter method
// with_cdwX() for every cdw, which also automatically sets the corresponding flag-bit.
// This macro generates one such setter method.
macro_rules! define_nvme_command_builder_with_cdw {
    ($fnname:ident: $fieldname:ident => $flagmask:expr) => {
        /// Sets the `$fieldname` field and marks it valid in the command's
        /// `flags` field.
        ///
        /// Command data words (CDWs) are interpreted by the selected command.
        /// Consult the [NVMe specifications] to determine which CDWs it
        /// requires.
        ///
        /// [NVMe specifications]: https://nvmexpress.org/specifications/
        #[must_use]
        pub const fn $fnname(mut self, $fieldname: u32) -> Self {
            self.req.cmd.$fieldname = $fieldname;
            self.req.cmd.flags |= $flagmask.bits();
            self
        }
    };
}

/// Builder for constructing an NVMe request.
///
/// Its methods configure command data words (CDWs), transfer buffers, and
/// metadata buffers. The `'buffers` lifetime keeps attached buffers alive
/// while the response is interpreted.
#[derive(Debug)]
pub struct NvmeRequestBuilder<'buffers> {
    req: NvmeRequest<'buffers>,
}
impl<'buffers> NvmeRequestBuilder<'buffers> {
    /// Creates a new builder for an NVMe command.
    ///
    /// # Arguments
    ///
    /// - `io_align`: Controller I/O buffer alignment requirement.
    /// - `opcode`: Opcode placed in command data word zero.
    /// - `queue_type`: Queue on which to submit the command.
    #[must_use]
    pub fn new(io_align: u32, opcode: u8, queue_type: NvmeQueueType) -> Self {
        Self {
            req: NvmeRequest {
                io_align,
                cmd: NvmExpressCommand {
                    cdw0: opcode as u32,
                    ..Default::default()
                },
                packet: NvmExpressPassThruCommandPacket {
                    command_timeout: 0,
                    transfer_buffer: ptr::null_mut(),
                    transfer_length: 0,
                    meta_data_buffer: ptr::null_mut(),
                    meta_data_length: 0,
                    queue_type,
                    nvme_cmd: ptr::null(),            // filled during execution
                    nvme_completion: ptr::null_mut(), // filled during execution
                },
                transfer_buffer: None,
                meta_data_buffer: None,
                _phantom: PhantomData,
            },
        }
    }

    /// Sets this request's timeout in 100-nanosecond units.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.req.packet.command_timeout = (timeout.as_nanos() / 100) as u64;
        self
    }

    // define the with_cdwX() builder methods
    define_nvme_command_builder_with_cdw!(with_cdw2: cdw2 => NvmExpressCommandCdwValidity::CDW_2);
    define_nvme_command_builder_with_cdw!(with_cdw3: cdw3 => NvmExpressCommandCdwValidity::CDW_3);
    define_nvme_command_builder_with_cdw!(with_cdw10: cdw10 => NvmExpressCommandCdwValidity::CDW_10);
    define_nvme_command_builder_with_cdw!(with_cdw11: cdw11 => NvmExpressCommandCdwValidity::CDW_11);
    define_nvme_command_builder_with_cdw!(with_cdw12: cdw12 => NvmExpressCommandCdwValidity::CDW_12);
    define_nvme_command_builder_with_cdw!(with_cdw13: cdw13 => NvmExpressCommandCdwValidity::CDW_13);
    define_nvme_command_builder_with_cdw!(with_cdw14: cdw14 => NvmExpressCommandCdwValidity::CDW_14);
    define_nvme_command_builder_with_cdw!(with_cdw15: cdw15 => NvmExpressCommandCdwValidity::CDW_15);

    // # TRANSFER BUFFER
    // ########################################################################################

    /// Uses a caller-provided transfer buffer.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Buffer used for the command's data transfer.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not satisfy the required alignment.
    pub fn use_transfer_buffer(
        mut self,
        bfr: &'buffers mut AlignedBuffer,
    ) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.transfer_buffer = None;
        self.req.packet.transfer_buffer = bfr.ptr_mut().cast();
        self.req.packet.transfer_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates a transfer buffer for the NVMe request.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn with_transfer_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.transfer_buffer = bfr.ptr_mut().cast();
        self.req.packet.transfer_length = bfr.size() as u32;
        self.req.transfer_buffer = Some(bfr);
        Ok(self)
    }

    // # METADATA BUFFER
    // ########################################################################################

    /// Uses a user-supplied metadata buffer.
    ///
    /// # Arguments
    ///
    /// - `bfr`: Metadata buffer to attach to the request.
    ///
    /// # Errors
    ///
    /// Returns an error if `bfr` does not satisfy the required alignment.
    pub fn use_metadata_buffer(
        mut self,
        bfr: &'buffers mut AlignedBuffer,
    ) -> Result<Self, AlignmentError> {
        // check alignment of externally supplied buffer
        bfr.check_alignment(self.req.io_align as usize)?;
        self.req.meta_data_buffer = None;
        self.req.packet.meta_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.meta_data_length = bfr.size() as u32;
        Ok(self)
    }

    /// Allocates a metadata buffer for the NVMe request.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn with_metadata_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.meta_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.meta_data_length = bfr.size() as u32;
        self.req.meta_data_buffer = Some(bfr);
        Ok(self)
    }

    /// Builds the configured [`NvmeRequest`].
    #[must_use]
    pub fn build(self) -> NvmeRequest<'buffers> {
        self.req
    }
}

/// Response returned after executing an NVMe command.
///
/// Contains the original request and command completion status. The `'buffers`
/// lifetime keeps attached I/O buffers alive while the response is interpreted.
#[derive(Debug)]
pub struct NvmeResponse<'buffers> {
    req: NvmeRequest<'buffers>,
    completion: NvmeCompletion,
}
impl<'buffers> NvmeResponse<'buffers> {
    /// Returns the transfer buffer, if one was assigned to the request.
    #[must_use]
    pub const fn transfer_buffer(&self) -> Option<&'buffers [u8]> {
        if self.req.packet.transfer_buffer.is_null() {
            return None;
        }
        // SAFETY: The memory is valid.
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.transfer_buffer.cast(),
                self.req.packet.transfer_length as usize,
            ))
        }
    }

    /// Returns the metadata buffer, if one was assigned to the request.
    #[must_use]
    pub const fn metadata_buffer(&self) -> Option<&'buffers [u8]> {
        if self.req.packet.meta_data_buffer.is_null() {
            return None;
        }
        // SAFETY: The memory is valid.
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.meta_data_buffer.cast(),
                self.req.packet.meta_data_length as usize,
            ))
        }
    }

    /// Returns the completion status of the NVMe command.
    #[must_use]
    pub const fn completion(&self) -> &NvmeCompletion {
        &self.completion
    }
}
