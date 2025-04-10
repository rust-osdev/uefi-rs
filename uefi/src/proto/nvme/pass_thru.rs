// SPDX-License-Identifier: MIT OR Apache-2.0

//! NVM Express Pass Thru Protocol.

use super::{NvmeRequest, NvmeResponse};
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use crate::StatusExt;
use core::alloc::LayoutError;
use core::ptr::{self, NonNull};
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::nvme::{NvmExpressCompletion, NvmExpressPassThruProtocol};
use uefi_raw::Status;

/// Nvme Pass Thru Protocol Mode structure.
///
/// This contains information regarding the specific capabilities and requirements
/// of the NVMe controller, such as buffer alignment constraints.
pub type NvmePassThruMode = uefi_raw::protocol::nvme::NvmExpressPassThruMode;

/// Identifier for an NVMe namespace.
///
/// Namespace IDs are used to target specific namespaces on an NVMe device for commands.
pub type NvmeNamespaceId = u32;

/// NVMe Pass Thru Protocol.
///
/// One protocol instance corresponds to one NVMe controller
/// (which, most of the time, corresponds to one SSD).
///
/// This API offers a safe and convenient, yet still low-level interface to NVMe devices.
/// It is designed as a foundational layer, leaving higher-level abstractions responsible for implementing
/// richer storage semantics, device-specific commands, and advanced use cases.
///
/// # UEFI Spec Description
/// The `EFI_NVM_EXPRESS_PASS_THRU_PROTOCOL` provides essential functionality for interacting
/// with NVMe controllers and namespaces. It allows sending NVMe commands to either the
/// controller itself or specific namespaces within the controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(NvmExpressPassThruProtocol::GUID)]
pub struct NvmePassThru(NvmExpressPassThruProtocol);

impl NvmePassThru {
    /// Retrieves the mode of the NVMe Pass Thru protocol.
    ///
    /// # Returns
    /// An instance of [`NvmePassThruMode`] describing the NVMe controller's capabilities.
    #[must_use]
    pub fn mode(&self) -> NvmePassThruMode {
        unsafe { (*self.0.mode).clone() }
    }

    /// Retrieves the alignment requirements for I/O buffers.
    ///
    /// # Returns
    /// An alignment value (in bytes) that all I/O buffers must adhere to for successful operation.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the necessary alignment for this NVMe Controller.
    ///
    /// You can alternatively do this yourself using the [`AlignedBuffer`] helper directly.
    /// The `nvme` api will validate that your buffers have the correct alignment and error
    /// if they don't.
    ///
    /// # Parameters
    /// - `len`: The size (in bytes) of the buffer to allocate.
    ///
    /// # Returns
    /// [`AlignedBuffer`] containing the allocated memory.
    ///
    /// # Errors
    /// This method can fail due to alignment or memory allocation issues.
    pub fn alloc_io_buffer(&self, len: usize) -> Result<AlignedBuffer, LayoutError> {
        AlignedBuffer::from_size_align(len, self.io_align() as usize)
    }

    /// Iterate over all valid namespaces on this NVMe controller.
    ///
    /// This ignores the 0-namespaces, which corresponds to the controller itself.
    /// The iterator yields [`NvmeNamespace`] instances representing individual namespaces.
    ///
    /// # Returns
    /// A [`NvmeNamespaceIterator`] for iterating through the namespaces.
    #[must_use]
    pub const fn iter_namespaces(&self) -> NvmeNamespaceIterator<'_> {
        NvmeNamespaceIterator {
            proto: &self.0,
            prev: 0xFFFFFFFF,
        }
    }

    /// Get the controller namespace (id = 0).
    /// This can be used to send ADMIN commands.
    ///
    /// # Returns
    /// A [`NvmeNamespaceIterator`] for iterating through the namespaces.
    #[must_use]
    pub const fn controller(&self) -> NvmeNamespace<'_> {
        NvmeNamespace {
            proto: &self.0,
            namespace_id: 0,
        }
    }
}

/// Represents one namespace on an NVMe controller.
///
/// A namespace is a shard of storage that the controller can be partitioned into.
/// Typically, consumer devices only have a single namespace where all the data resides (id 1).
#[derive(Debug)]
pub struct NvmeNamespace<'a> {
    proto: &'a NvmExpressPassThruProtocol,
    namespace_id: NvmeNamespaceId,
}

impl NvmeNamespace<'_> {
    fn proto_mut(&mut self) -> *mut NvmExpressPassThruProtocol {
        ptr::from_ref(self.proto).cast_mut()
    }

    /// Retrieves the namespace identifier (NSID) associated with this NVMe namespace.
    #[must_use]
    pub const fn namespace_id(&self) -> NvmeNamespaceId {
        self.namespace_id
    }

    /// Get the final device path node for this namespace.
    ///
    /// For a full [`crate::proto::device_path::DevicePath`] pointing to this namespace on the
    /// corresponding NVMe controller.
    pub fn path_node(&self) -> crate::Result<PoolDevicePathNode> {
        unsafe {
            let mut path_ptr: *const DevicePathProtocol = ptr::null();
            (self.proto.build_device_path)(self.proto, self.namespace_id, &mut path_ptr)
                .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or(Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Sends an NVM Express command to this namespace (Namespace ID ≥ 1).
    ///
    /// # Parameters
    /// - `req`: The [`NvmeRequest`] containing the command and associated data to send to the namespace.
    ///
    /// # Returns
    /// - [`NvmeResponse`] containing the results of the operation, such as data and status.
    ///
    /// # Errors
    /// - [`Status::BAD_BUFFER_SIZE`] The NVM Express Command Packet was not executed. The number
    ///   of bytes that could be transferred is returned in `TransferLength`.
    /// - [`Status::NOT_READY`] The NVM Express Command Packet could not be sent because the controller
    ///   is not ready. The caller may retry later.
    /// - [`Status::DEVICE_ERROR`] A device error occurred while attempting to send the NVM Express
    ///   Command Packet. Additional status information is available in `NvmeCompletion`.
    /// - [`Status::INVALID_PARAMETER`] The Namespace ID or the contents of the Command Packet are invalid.
    ///   The NVM Express Command Packet was not sent, and no additional status information is available.
    /// - [`Status::UNSUPPORTED`] The command described by the NVM Express Command Packet is not supported
    ///   by the NVM Express controller. The Command Packet was not sent, and no additional status
    ///   information is available.
    /// - [`Status::TIMEOUT`] A timeout occurred while executing the NVM Express Command Packet.
    ///   Additional status information is available in `NvmeCompletion`.
    pub fn execute_command<'req>(
        &mut self,
        mut req: NvmeRequest<'req>,
    ) -> crate::Result<NvmeResponse<'req>> {
        let mut completion = NvmExpressCompletion::default();
        // prepare cmd packet
        req.cmd.nsid = self.namespace_id;
        req.packet.nvme_cmd = &req.cmd;
        req.packet.nvme_completion = &mut completion;
        unsafe {
            (self.proto.pass_thru)(
                self.proto_mut(),
                self.namespace_id,
                &mut req.packet,
                ptr::null_mut(),
            )
            .to_result_with_val(|| NvmeResponse { req, completion })
        }
    }
}

/// An iterator over the namespaces of an NVMe controller.
///
/// The iterator yields [`NvmeNamespace`] instances, each representing one namespace
/// on the NVMe controller.
#[derive(Debug)]
pub struct NvmeNamespaceIterator<'a> {
    proto: &'a NvmExpressPassThruProtocol,
    prev: NvmeNamespaceId,
}

impl<'a> Iterator for NvmeNamespaceIterator<'a> {
    type Item = NvmeNamespace<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = unsafe { (self.proto.get_next_namespace)(self.proto, &mut self.prev) };
        match result {
            Status::SUCCESS => Some(NvmeNamespace {
                proto: self.proto,
                namespace_id: self.prev,
            }),
            Status::NOT_FOUND => None,
            _ => panic!("Must not happen according to spec!"),
        }
    }
}
