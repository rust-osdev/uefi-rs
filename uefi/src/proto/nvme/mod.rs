// SPDX-License-Identifier: MIT OR Apache-2.0

//! NVM Express Protocols.

use crate::mem::{AlignedBuffer, AlignmentError};
use core::alloc::LayoutError;
use core::marker::PhantomData;
use core::ptr;
use core::time::Duration;
use uefi_raw::protocol::nvme::{
    NvmExpressCommand, NvmExpressCommandCdwValidity, NvmExpressPassThruCommandPacket,
};

pub mod pass_thru;

/// Represents the completion status of an NVMe command.
///
/// This structure contains various fields related to the status and results
/// of an executed command, including fields for error codes, specific command IDs,
/// and general state of the NVMe device.
pub type NvmeCompletion = uefi_raw::protocol::nvme::NvmExpressCompletion;

/// Type of queues an NVMe command can be placed into
/// (Which queue a command should be placed into depends on the command)
pub type NvmeQueueType = uefi_raw::protocol::nvme::NvmExpressQueueType;

/// Represents a request for executing an NVMe command.
///
/// This structure encapsulates the command to be sent to the NVMe device, along with
/// optional data transfer and metadata buffers. It ensures proper alignment and safety
/// during interactions with the NVMe protocol.
///
/// # Lifetime
/// `'buffers`: Makes sure the io-buffers bound to the built request
/// stay alive until the response was interpreted.
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
        /// Set the $fieldname parameter on the constructed nvme command.
        /// This also automatically flags the parameter as valid in the command's `flags` field.
        ///
        /// # About NVMe commands
        /// NVMe commands are constructed of a bunch of numbered CDWs (command data words) and a `flags` field.
        /// The `flags´ field tells the NVMe controller which CDWs was set and whether it should respect
        /// the corresponding CDWs value.
        /// CDWs have no fixed interpretation - the interpretation depends on the command to execute.
        /// Which CDWs have to be supplied (and enabled in the `flags` field) depends on the command that
        /// should be sent to and executed by the controller.
        /// See: <https://nvmexpress.org/specifications/>
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
/// This structure provides convenient methods for configuring NVMe commands,
/// including parameters like command-specific data words (CDWs)
/// and optional buffers for transfer and metadata operations.
///
/// It ensures safe and ergonomic setup of NVMe requests.
///
/// # Lifetime
/// `'buffers`: Makes sure the io-buffers bound to the built request
/// stay alive until the response was interpreted.
#[derive(Debug)]
pub struct NvmeRequestBuilder<'buffers> {
    req: NvmeRequest<'buffers>,
}
impl<'buffers> NvmeRequestBuilder<'buffers> {
    /// Creates a new builder for configuring an NVMe request.
    ///
    /// # Parameters
    /// - `io_align`: Memory alignment requirements for buffers.
    /// - `opcode`: The opcode for the NVMe command.
    /// - `queue_type`: Specifies the type of queue the command should be placed into.
    ///
    /// # Returns
    /// An instance of [`NvmeRequestBuilder`] for further configuration.
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

    /// Configure the given timeout for this request.
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
    /// the `transfer_buffer` of the underlying [`NvmeRequest`].
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

    /// Adds a newly allocated transfer buffer to the built NVMe request.
    ///
    /// # Parameters
    /// - `len`: The size of the buffer (in bytes) to allocate for receiving data.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
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
    /// # Parameters
    /// - `bfr`: A mutable reference to an [`AlignedBuffer`] that will be used to store metadata.
    ///
    /// # Returns
    /// `Result<Self, AlignmentError>` indicating success or an alignment issue with the provided buffer.
    ///
    /// # Description
    /// This method checks the alignment of the buffer against the protocol's requirements and assigns it to
    /// the `meta_data_buffer` of the underlying [`NvmeRequest`].
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

    /// Adds a newly allocated metadata buffer to the built NVMe request.
    ///
    /// # Parameters
    /// - `len`: The size of the buffer (in bytes) to allocate for storing metadata.
    ///
    /// # Returns
    /// `Result<Self, LayoutError>` indicating success or a memory allocation error.
    pub fn with_metadata_buffer(mut self, len: usize) -> Result<Self, LayoutError> {
        let mut bfr = AlignedBuffer::from_size_align(len, self.req.io_align as usize)?;
        self.req.packet.meta_data_buffer = bfr.ptr_mut().cast();
        self.req.packet.meta_data_length = bfr.size() as u32;
        self.req.meta_data_buffer = Some(bfr);
        Ok(self)
    }

    /// Build the final [`NvmeRequest`].
    ///
    /// # Returns
    /// A fully-configured [`NvmeRequest`] ready for execution.
    #[must_use]
    pub fn build(self) -> NvmeRequest<'buffers> {
        self.req
    }
}

/// Represents the response from executing an NVMe command.
///
/// This structure encapsulates the original request, as well as the command's completion status.
///
/// # Lifetime
/// `'buffers`: Makes sure the io-buffers bound to the built request
/// stay alive until the response was interpreted.
#[derive(Debug)]
pub struct NvmeResponse<'buffers> {
    req: NvmeRequest<'buffers>,
    completion: NvmeCompletion,
}
impl<'buffers> NvmeResponse<'buffers> {
    /// Returns the buffer containing transferred data from the device (if any).
    ///
    /// # Returns
    /// `Option<&[u8]>`: A slice of the transfer buffer, or `None` if the request was started without.
    #[must_use]
    pub fn transfer_buffer(&self) -> Option<&'buffers [u8]> {
        if self.req.packet.transfer_buffer.is_null() {
            return None;
        }
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.transfer_buffer.cast(),
                self.req.packet.transfer_length as usize,
            ))
        }
    }

    /// Returns the buffer containing metadata data from the device (if any).
    ///
    /// # Returns
    /// `Option<&[u8]>`: A slice of the metadata buffer, or `None` if the request was started without.
    #[must_use]
    pub fn metadata_buffer(&self) -> Option<&'buffers [u8]> {
        if self.req.packet.meta_data_buffer.is_null() {
            return None;
        }
        unsafe {
            Some(core::slice::from_raw_parts(
                self.req.packet.meta_data_buffer.cast(),
                self.req.packet.meta_data_length as usize,
            ))
        }
    }

    /// Provides access to the completion structure of the NVMe command.
    ///
    /// # Returns
    /// A reference to the [`NvmeCompletion`] structure containing the status and results of the command.
    #[must_use]
    pub const fn completion(&self) -> &NvmeCompletion {
        &self.completion
    }
}
