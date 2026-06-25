// SPDX-License-Identifier: MIT OR Apache-2.0

//! EDK2 IOMMU protocol.

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use uefi_raw::table::boot::PAGE_SIZE;

use crate::proto::dma::iommu::Iommu;

pub mod iommu;

/// A smart pointer for DMA buffers allocated through the IOMMU protocol.
///
/// The buffer can be accessed as a byte slice and is returned to firmware when
/// dropped.
#[must_use]
#[derive(Debug)]
pub struct DmaBuffer<'a> {
    ptr: *mut c_void,
    pages: usize,
    iommu: &'a Iommu,
}

impl<'a> DmaBuffer<'a> {
    /// Create a new DmaBuffer from a raw pointer and page count.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `ptr` is valid for `pages * PAGE_SIZE` bytes allocated by the IOMMU protocol.
    /// - `pages` correctly represents the number of pages allocated.
    /// - `iommu` is the protocol instance that allocated `ptr`.
    /// - This `DmaBuffer` is the unique owner responsible for freeing `ptr`.
    pub const unsafe fn from_raw(ptr: *mut c_void, pages: usize, iommu: &'a Iommu) -> Self {
        Self { ptr, pages, iommu }
    }

    /// Get the raw pointer to the buffer.
    #[must_use]
    pub const fn as_ptr(&self) -> *const c_void {
        self.ptr.cast_const()
    }

    /// Get the raw mutable pointer to the buffer.
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut c_void {
        self.ptr
    }

    /// Get the number of pages in the buffer.
    #[must_use]
    pub const fn pages(&self) -> usize {
        self.pages
    }

    /// Get the size of the buffer in bytes.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.pages * PAGE_SIZE
    }
}

impl<'a> Deref for DmaBuffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        // SAFETY: `DmaBuffer::from_raw` requires `ptr` to be valid for
        // `pages * PAGE_SIZE` bytes for this buffer's lifetime.
        unsafe { core::slice::from_raw_parts(self.ptr.cast(), self.pages * PAGE_SIZE) }
    }
}

impl<'a> DerefMut for DmaBuffer<'a> {
    fn deref_mut(&mut self) -> &mut [u8] {
        // SAFETY: `&mut self` guarantees unique access to the owned DMA buffer,
        // whose raw memory is valid for `pages * PAGE_SIZE` bytes.
        unsafe { core::slice::from_raw_parts_mut(self.ptr.cast::<u8>(), self.pages * PAGE_SIZE) }
    }
}

impl<'a> Drop for DmaBuffer<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.iommu.free_buffer_raw(self.ptr, self.pages) {
            log::error!("IOMMU free_buffer failed: {e:?}");
        }
    }
}

/// A smart pointer for active DMA buffer mappings.
///
/// The mapping keeps the firmware mapping alive and unmaps it when
/// dropped.
#[must_use]
#[derive(Debug)]
pub struct Mapping<'a, 'buf> {
    ptr: *mut c_void,
    iommu: &'a Iommu,
    _buffer: PhantomData<&'buf mut DmaBuffer<'a>>,
}

impl<'a, 'buf> Mapping<'a, 'buf> {
    /// Create a new Mapping from a raw pointer.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `ptr` is a valid mapping pointer returned by the IOMMU protocol.
    /// - The mapping is currently active and valid.
    /// - `iommu` is the protocol instance that created `ptr`.
    /// - `_buffer` is the `DmaBuffer` used to create this mapping and remains
    ///   exclusively borrowed for the returned mapping's lifetime.
    pub const unsafe fn from_raw(
        ptr: *mut c_void,
        iommu: &'a Iommu,
        _buffer: &'buf mut DmaBuffer<'a>,
    ) -> Self {
        Self {
            ptr,
            iommu,
            _buffer: PhantomData,
        }
    }

    /// Get the raw mapping pointer.
    #[must_use]
    pub const fn as_ptr(&self) -> *const c_void {
        self.ptr.cast_const()
    }

    /// Get the raw mutable mapping pointer.
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut c_void {
        self.ptr
    }
}

impl<'a, 'buf> Drop for Mapping<'a, 'buf> {
    fn drop(&mut self) {
        if let Err(e) = self.iommu.unmap_raw(self.as_mut_ptr()) {
            log::error!("IOMMU unmap failed: {e:?}");
        }
    }
}
