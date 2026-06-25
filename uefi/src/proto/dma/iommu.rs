// SPDX-License-Identifier: MIT OR Apache-2.0

//! EDKII IOMMU protocol.

use crate::data_types::PhysicalAddress;
use crate::mem::memory_map::MemoryType;
use crate::proto::unsafe_protocol;
use crate::{Handle, Result, Status, StatusExt};
use core::ffi::c_void;
use uefi_raw::table::boot::AllocateType;

pub use crate::proto::dma::{DmaBuffer, Mapping};
pub use uefi_raw::protocol::iommu::{
    EdkiiIommuAccess, EdkiiIommuAttribute, EdkiiIommuOperation, EdkiiIommuProtocol,
};

/// EDK2 IoMmu [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(EdkiiIommuProtocol::GUID)]
pub struct Iommu(EdkiiIommuProtocol);

impl Iommu {
    /// Get the IOMMU protocol revision
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.0.revision
    }

    /// Set access attributes for a mapping.
    ///
    /// # Errors
    ///
    /// * [`crate::Status::INVALID_PARAMETER`]: invalid device handle, mapping, or access flags
    /// * [`crate::Status::UNSUPPORTED`]: operation not supported by this IOMMU
    /// * [`crate::Status::OUT_OF_RESOURCES`]: insufficient resources to modify IOMMU access
    /// * [`crate::Status::DEVICE_ERROR`]: IOMMU device reported an error
    pub fn set_attribute(
        &self,
        device_handle: Handle,
        mapping: &mut Mapping<'_, '_>,
        iommu_access: EdkiiIommuAccess,
    ) -> Result {
        let mapping_raw = mapping.as_mut_ptr();
        // SAFETY: `mapping_raw` comes from a live `Mapping`, and `device_handle`
        // is an opaque firmware handle passed through unchanged.
        let status = unsafe {
            (self.0.set_attribute)(&self.0, device_handle.as_ptr(), mapping_raw, iommu_access)
        };

        status.to_result()
    }

    /// Map a buffer for DMA operations.
    ///
    /// Returns the device address, mapping handle, and actual number of bytes mapped.
    /// The mapping is tied to `host_buffer` and will be automatically unmapped when
    /// dropped.
    ///
    /// # Errors
    ///
    /// * [`crate::Status::INVALID_PARAMETER`]: invalid operation or buffer
    /// * [`crate::Status::BAD_BUFFER_SIZE`]: `number_of_bytes` is larger than `host_buffer`
    /// * [`crate::Status::UNSUPPORTED`]: host address cannot be mapped as a common buffer
    /// * [`crate::Status::OUT_OF_RESOURCES`]: insufficient resources
    /// * [`crate::Status::DEVICE_ERROR`]: system hardware could not map the requested address
    pub fn map<'iommu, 'buf>(
        &'iommu self,
        operation: EdkiiIommuOperation,
        host_buffer: &'buf mut DmaBuffer<'iommu>,
        number_of_bytes: usize,
    ) -> Result<(PhysicalAddress, Mapping<'iommu, 'buf>, usize)> {
        if number_of_bytes > host_buffer.size() {
            return Err(Status::BAD_BUFFER_SIZE.into());
        }

        let mut number_of_bytes = number_of_bytes;

        let mut mapping_raw: *mut c_void = core::ptr::null_mut();
        let mut device_address: u64 = 0;

        let host_address: *mut c_void = host_buffer.as_mut_ptr();

        // SAFETY: `host_address` points into `host_buffer`, which is valid for
        // `number_of_bytes` because oversized lengths were rejected above.
        let status = unsafe {
            (self.0.map)(
                &self.0,
                operation,
                host_address,
                &mut number_of_bytes,
                &mut device_address,
                &mut mapping_raw,
            )
        };

        status.to_result_with_val(|| {
            // SAFETY: `mapping_raw` was returned by this IOMMU protocol, and
            // `host_buffer` remains mutably borrowed for the mapping lifetime.
            let mapping = unsafe { Mapping::from_raw(mapping_raw, self, host_buffer) };
            (device_address, mapping, number_of_bytes)
        })
    }

    /// Unmap a previously mapped buffer
    pub(crate) fn unmap_raw(&self, mapping: *mut c_void) -> Result {
        // SAFETY: The safe `Mapping` API only stores active mapping
        // pointers returned by this protocol, and `Drop` calls this once.
        let status = unsafe { (self.0.unmap)(&self.0, mapping) };
        status.to_result()
    }

    /// Allocate a buffer suitable for DMA operations.
    ///
    /// The buffer will be automatically freed when dropped.
    ///
    /// # Errors
    ///
    /// * [`crate::Status::INVALID_PARAMETER`]: invalid memory type or attributes
    /// * [`crate::Status::UNSUPPORTED`]: unsupported attributes
    /// * [`crate::Status::OUT_OF_RESOURCES`]: memory pages could not be allocated
    pub fn allocate_buffer(
        &self,
        memory_type: MemoryType,
        pages: usize,
        attributes: EdkiiIommuAttribute,
    ) -> Result<DmaBuffer<'_>> {
        let mut host_address: *mut c_void = core::ptr::null_mut();

        // Per spec, AllocateType is ignored by the IOMMU allocate_buffer implementation.
        let allocate_type = AllocateType::ANY_PAGES;

        // SAFETY: `host_address` is a valid out-pointer, and all other
        // arguments are plain values forwarded to firmware.
        let status = unsafe {
            (self.0.allocate_buffer)(
                &self.0,
                allocate_type,
                memory_type,
                pages,
                &mut host_address,
                attributes,
            )
        };

        status.to_result_with_val(|| {
            // SAFETY: On success, firmware initialized `host_address` with a
            // buffer allocated by this IOMMU protocol for `pages` pages.
            unsafe { DmaBuffer::from_raw(host_address, pages, self) }
        })
    }

    /// Free a buffer allocated with allocate_buffer
    pub(crate) fn free_buffer_raw(&self, ptr: *mut c_void, pages: usize) -> Result {
        // SAFETY: `DmaBuffer` calls this only for buffers allocated by this
        // protocol, preserving the original page count.
        let status = unsafe { (self.0.free_buffer)(&self.0, pages, ptr) };
        status.to_result()
    }
}
