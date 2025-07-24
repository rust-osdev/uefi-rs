// SPDX-License-Identifier: MIT OR Apache-2.0

//! EDK2 IoMmu  protocol.

use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

use uefi_raw::table::boot::PAGE_SIZE;

use crate::proto::dma::iommu::Iommu;

pub mod iommu;

/// A smart pointer for DMA buffers
#[must_use]
#[derive(Debug)]
pub struct DmaBuffer<'a> {
    ptr: *mut c_void,
    pages: usize,
    iommu: &'a Iommu,
}

impl<'a> DmaBuffer<'a> {
    /// Create a new DmaBuffer from a raw pointer and page count
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `ptr` is a valid pointer to memory allocated by the IOMMU protocol
    /// - `pages` correctly represents the number of pages allocated
    pub const unsafe fn from_raw(ptr: *mut c_void, pages: usize, iommu: &'a Iommu) -> Self {
        Self { ptr, pages, iommu }
    }

    /// Get the raw pointer to the buffer
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }

    /// Get the number of pages in the buffer
    #[must_use]
    pub const fn pages(&self) -> usize {
        self.pages
    }

    /// Get the size of the buffer in bytes
    #[must_use]
    pub const fn size(&self) -> usize {
        self.pages * PAGE_SIZE
    }
}

impl<'a> Deref for DmaBuffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, self.pages * PAGE_SIZE) }
    }
}

impl<'a> DerefMut for DmaBuffer<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.cast::<u8>(), self.pages * PAGE_SIZE) }
    }
}

impl<'a> Drop for DmaBuffer<'a> {
    fn drop(&mut self) {
        let ptr = self.ptr;
        let pages = self.pages;
        let _ = self.iommu.free_buffer_raw(ptr, pages);
    }
}

/// A smart pointer for IOMMU mappings
#[must_use]
#[derive(Debug)]
pub struct Mapping<'a> {
    ptr: *mut c_void,
    iommu: &'a Iommu,
}

impl<'a> Mapping<'a> {
    /// Create a new Mapping from a raw pointer
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `ptr` is a valid mapping pointer returned by the IOMMU protocol
    /// - The mapping is currently active and valid
    pub const unsafe fn from_raw(ptr: *mut c_void, iommu: &'a Iommu) -> Self {
        Self { ptr, iommu }
    }

    /// Get the raw mapping pointer
    #[must_use] 
    pub const fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl<'a> Drop for Mapping<'a> {
    fn drop(&mut self) {
        let ptr = self.ptr;
        let _ = self.iommu.unmap_raw(ptr);
    }
}
