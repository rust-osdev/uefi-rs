// SPDX-License-Identifier: MIT OR Apache-2.0

//! EDK2 IoMmu protocol.

use core::ffi::c_void;
use uefi::{
    Handle, Result, StatusExt, data_types::PhysicalAddress, mem::memory_map::MemoryType,
    proto::unsafe_protocol,
};

pub use crate::{
    proto::dma::{DmaBuffer, Mapping},
    uefi_raw::protocol::iommu::{
        EdkiiIommuAccess, EdkiiIommuAttribute, EdkiiIommuOperation, EdkiiIommuProtocol,
    },
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

    /// Set access attributes for a mapping
    pub fn set_attribute(
        &self,
        device_handle: Handle,
        mapping: &Mapping,
        iommu_access: EdkiiIommuAccess,
    ) -> Result {
        let mapping_raw = mapping.as_ptr();
        let status = unsafe {
            (self.0.set_attribute)(
                &self.0,
                device_handle.as_ptr(),
                mapping_raw,
                iommu_access.bits(),
            )
        };

        status.to_result()
    }

    /// Map a buffer for DMA operations
    pub fn map(
        &self,
        operation: EdkiiIommuOperation,
        host_buffer: &DmaBuffer,
        number_of_bytes: usize,
    ) -> Result<(PhysicalAddress, Mapping, usize)> {
        let mut number_of_bytes = number_of_bytes;

        let mut mapping_raw: *mut c_void = core::ptr::null_mut();
        let mut device_address: u64 = 0;

        let host_address: *mut c_void = host_buffer.as_ptr();

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
            let mapping = unsafe { Mapping::from_raw(mapping_raw, self) };
            (device_address, mapping, number_of_bytes)
        })
    }

    /// Unmap a previously mapped buffer
    pub(crate) fn unmap_raw(&self, mapping: *mut c_void) -> Result {
        let status = unsafe { (self.0.unmap)(&self.0, mapping) };
        status.to_result()
    }

    /// Allocate a buffer suitable for DMA operations
    pub fn allocate_buffer(
        &self,
        memory_type: MemoryType,
        pages: usize,
        attributes: EdkiiIommuAttribute,
    ) -> Result<DmaBuffer> {
        let mut host_address: *mut c_void = core::ptr::null_mut();

        // Must be ignored
        let allocate_type = 0u32;

        let status = unsafe {
            (self.0.allocate_buffer)(
                &self.0,
                allocate_type,
                memory_type,
                pages,
                &mut host_address,
                attributes.bits(),
            )
        };

        let dma_buffer = unsafe { DmaBuffer::from_raw(host_address, pages, self) };

        status.to_result_with_val(|| dma_buffer)
    }

    /// Free a buffer allocated with allocate_buffer
    pub(crate) fn free_buffer_raw(&self, ptr: *mut c_void, pages: usize) -> Result {
        let status = unsafe { (self.0.free_buffer)(&self.0, pages, ptr) };
        status.to_result()
    }
}
