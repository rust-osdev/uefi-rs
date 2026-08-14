// SPDX-License-Identifier: MIT OR Apache-2.0

//! NVM Express Pass Thru protocol.

use super::{NvmeRequest, NvmeResponse};
use crate::StatusExt;
use crate::mem::{AlignedBuffer, PoolAllocation};
use crate::proto::device_path::PoolDevicePathNode;
use core::alloc::LayoutError;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use uefi_macros::unsafe_protocol;
use uefi_raw::Status;
use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::nvme::{NvmExpressCompletion, NvmExpressPassThruProtocol};

/// NVMe Pass Thru mode.
///
/// Describes controller capabilities and requirements such as buffer alignment.
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
/// This is a safe, convenient, but still low-level interface. Higher-level
/// abstractions remain responsible for storage semantics and device-specific
/// commands.
///
/// # UEFI Specification
/// The `EFI_NVM_EXPRESS_PASS_THRU_PROTOCOL` provides essential functionality for interacting
/// with NVMe controllers and namespaces. It allows sending NVMe commands to either the
/// controller itself or specific namespaces within the controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(NvmExpressPassThruProtocol::GUID)]
pub struct NvmePassThru(UnsafeCell<NvmExpressPassThruProtocol>);

impl NvmePassThru {
    /// Returns the NVMe Pass Thru mode.
    #[must_use]
    pub fn mode(&self) -> NvmePassThruMode {
        // SAFETY: The memory is valid.
        let mut mode = unsafe { (*(*self.0.get()).mode).clone() };
        mode.io_align = mode.io_align.max(1); // 0 and 1 is the same, says UEFI spec
        mode
    }

    /// Returns the required I/O buffer alignment in bytes.
    #[must_use]
    pub fn io_align(&self) -> u32 {
        self.mode().io_align
    }

    /// Allocates an I/O buffer with the alignment required by this controller.
    ///
    /// Callers can instead allocate an [`AlignedBuffer`] directly. NVMe request
    /// builders validate user-provided buffer alignment.
    ///
    /// # Arguments
    ///
    /// - `len`: Number of bytes to allocate.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer cannot be allocated with the required
    /// alignment.
    pub fn alloc_io_buffer(&self, len: usize) -> Result<AlignedBuffer, LayoutError> {
        AlignedBuffer::from_size_align(len, self.io_align() as usize)
    }

    /// Returns an iterator over all valid namespaces on this NVMe controller.
    ///
    /// This omits namespace zero, which represents the controller itself.
    #[must_use]
    pub const fn iter_namespaces(&self) -> NvmeNamespaceIterator<'_> {
        NvmeNamespaceIterator {
            proto: &self.0,
            prev: 0xFFFFFFFF,
        }
    }

    /// Returns the controller namespace (ID 0), which can receive admin
    /// commands.
    #[must_use]
    pub const fn controller(&self) -> NvmeNamespace<'_> {
        NvmeNamespace {
            proto: &self.0,
            namespace_id: 0,
        }
    }

    /// Returns the broadcast namespace (ID `0xffff_ffff`).
    #[must_use]
    pub const fn broadcast(&self) -> NvmeNamespace<'_> {
        NvmeNamespace {
            proto: &self.0,
            namespace_id: 0xffffffff,
        }
    }
}

/// Represents one namespace on an NVMe controller.
///
/// A namespace is a partition of the controller's storage. Consumer devices
/// typically expose one namespace with ID 1.
#[derive(Debug)]
pub struct NvmeNamespace<'a> {
    proto: &'a UnsafeCell<NvmExpressPassThruProtocol>,
    namespace_id: NvmeNamespaceId,
}

impl NvmeNamespace<'_> {
    /// Returns this namespace's identifier (NSID).
    #[must_use]
    pub const fn namespace_id(&self) -> NvmeNamespaceId {
        self.namespace_id
    }

    /// Returns the final device path node for this namespace.
    ///
    /// Append this node to the controller's device path to form a complete
    /// [`crate::proto::device_path::DevicePath`].
    ///
    /// # Errors
    ///
    /// Returns an error if firmware cannot build the node or allocation fails.
    pub fn path_node(&self) -> crate::Result<PoolDevicePathNode> {
        // SAFETY: The memory is valid.
        unsafe {
            let mut path_ptr: *const DevicePathProtocol = ptr::null();
            ((*self.proto.get()).build_device_path)(
                self.proto.get(),
                self.namespace_id,
                &mut path_ptr,
            )
            .to_result()?;
            NonNull::new(path_ptr.cast_mut())
                .map(|p| PoolDevicePathNode(PoolAllocation::new(p.cast())))
                .ok_or_else(|| Status::OUT_OF_RESOURCES.into())
        }
    }

    /// Sends an NVM Express command to this namespace.
    ///
    /// # Errors
    /// - [`Status::BAD_BUFFER_SIZE`]: a buffer exceeds the allowed transfer
    ///   size.
    /// - [`Status::NOT_READY`]: the controller is not ready; retry later.
    /// - [`Status::DEVICE_ERROR`]: the controller reported an error.
    /// - [`Status::INVALID_PARAMETER`]: the namespace ID or request is invalid.
    /// - [`Status::UNSUPPORTED`]: the controller does not support the command.
    /// - [`Status::TIMEOUT`]: the command timed out.
    pub fn execute_command<'req>(
        &mut self,
        mut req: NvmeRequest<'req>,
    ) -> crate::Result<NvmeResponse<'req>> {
        let mut completion = NvmExpressCompletion::default();
        // prepare cmd packet
        req.cmd.nsid = self.namespace_id;
        req.packet.nvme_cmd = &req.cmd;
        req.packet.nvme_completion = &mut completion;
        // SAFETY: The memory is valid.
        unsafe {
            ((*self.proto.get()).pass_thru)(
                self.proto.get(),
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
/// Each item represents one namespace on the controller.
#[derive(Debug)]
pub struct NvmeNamespaceIterator<'a> {
    proto: &'a UnsafeCell<NvmExpressPassThruProtocol>,
    prev: NvmeNamespaceId,
}

impl<'a> Iterator for NvmeNamespaceIterator<'a> {
    type Item = NvmeNamespace<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result =
            // SAFETY: The memory is valid.
            unsafe { ((*self.proto.get()).get_next_namespace)(self.proto.get(), &mut self.prev) };
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
