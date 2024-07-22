//! Types and helpers to work with the UEFI memory map.
//!
//!
//! ## Traits
//!
//! - [`MemoryMap`]
//! - [`MemoryMapMut`]
//!
//! ## Types
//!
//! - [`MemoryMapOwned`]
//! - [`MemoryMapRef`]
//! - [`MemoryMapRefMut`]

pub use uefi_raw::table::boot::{MemoryAttribute, MemoryDescriptor, MemoryType};

use crate::data_types::Align;
use crate::table::system_table_boot;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};
use core::ptr::NonNull;
use core::{mem, ptr};
use uefi_raw::PhysicalAddress;

impl Align for MemoryDescriptor {
    fn alignment() -> usize {
        mem::align_of::<Self>()
    }
}

/// A unique identifier of a memory map.
///
/// If the memory map changes, this value is no longer valid.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(pub(crate) usize);

/// The backing memory for the UEFI memory app on the UEFI heap, allocated using
/// the UEFI boot services allocator. This occupied memory will also be
/// reflected in the memory map itself.
///
/// Although untyped, it is similar to the `Box` type in terms of heap
/// allocation and deallocation, as well as ownership of the corresponding
/// memory. Apart from that, this type only has the semantics of a buffer.
///
/// The memory is untyped, which is necessary due to the nature of the UEFI
/// spec. It still ensures a correct alignment to hold [`MemoryDescriptor`]. The
/// size of the buffer is sufficient to hold the memory map at the point in time
/// where this is created. Note that due to (not obvious or asynchronous)
/// allocations/deallocations in your environment, this might be outdated at the
/// time you store the memory map in it.
///
/// Note that due to the nature of the UEFI memory app, this buffer might
/// hold (a few) bytes more than necessary. The `map_size` reported by
/// `get_memory_map` tells the actual size.
///
/// When this type is dropped and boot services are not exited yet, the memory
/// is freed.
///
/// # Usage
/// The type is intended to be used like this:
/// 1. create it using [`MemoryMapBackingMemory::new`]
/// 2. pass it to [`BootServices::get_memory_map`]
/// 3. construct a [`MemoryMapOwned`] from it
#[derive(Debug)]
#[allow(clippy::len_without_is_empty)] // this type is never empty
pub(crate) struct MemoryMapBackingMemory(NonNull<[u8]>);

impl MemoryMapBackingMemory {
    /// Constructs a new [`MemoryMapBackingMemory`].
    ///
    /// # Parameters
    /// - `memory_type`: The memory type for the memory map allocation.
    ///   Typically, [`MemoryType::LOADER_DATA`] for regular UEFI applications.
    pub(crate) fn new(memory_type: MemoryType) -> crate::Result<Self> {
        let st = system_table_boot().expect("Should have boot services activated");
        let bs = st.boot_services();

        let memory_map_meta = bs.memory_map_size();
        let len = Self::safe_allocation_size_hint(memory_map_meta);
        let ptr = bs.allocate_pool(memory_type, len)?.as_ptr();

        // Should be fine as UEFI always has  allocations with a guaranteed
        // alignment of 8 bytes.
        assert_eq!(ptr.align_offset(mem::align_of::<MemoryDescriptor>()), 0);

        // If this panics, the UEFI implementation is broken.
        assert_eq!(memory_map_meta.map_size % memory_map_meta.desc_size, 0);

        unsafe { Ok(Self::from_raw(ptr, len)) }
    }

    unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        assert_eq!(ptr.align_offset(mem::align_of::<MemoryDescriptor>()), 0);

        let ptr = NonNull::new(ptr).expect("UEFI should never return a null ptr. An error should have been reflected via an Err earlier.");
        let slice = NonNull::slice_from_raw_parts(ptr, len);

        Self(slice)
    }

    /// Creates an instance from the provided memory, which is not necessarily
    /// on the UEFI heap.
    #[cfg(test)]
    fn from_slice(buffer: &mut [u8]) -> Self {
        let len = buffer.len();
        unsafe { Self::from_raw(buffer.as_mut_ptr(), len) }
    }

    /// Returns a "safe" best-effort size hint for the memory map size with
    /// some additional bytes in buffer compared to the [`MemoryMapMeta`].
    /// This helps
    #[must_use]
    fn safe_allocation_size_hint(mmm: MemoryMapMeta) -> usize {
        // Allocate space for extra entries beyond the current size of the
        // memory map. The value of 8 matches the value in the Linux kernel:
        // https://github.com/torvalds/linux/blob/e544a07438/drivers/firmware/efi/libstub/efistub.h#L173
        const EXTRA_ENTRIES: usize = 8;

        let extra_size = mmm.desc_size * EXTRA_ENTRIES;
        mmm.map_size + extra_size
    }

    /// Returns a slice to the underlying memory.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { self.0.as_ref() }
    }

    /// Returns a mutable slice to the underlying memory.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { self.0.as_mut() }
    }
}

impl Drop for MemoryMapBackingMemory {
    fn drop(&mut self) {
        if let Some(bs) = system_table_boot() {
            let res = unsafe { bs.boot_services().free_pool(self.0.as_ptr().cast()) };
            if let Err(e) = res {
                log::error!("Failed to deallocate memory map: {e:?}");
            }
        } else {
            log::debug!("Boot services are exited. Memory map won't be freed using the UEFI boot services allocator.");
        }
    }
}

/// A structure containing the meta attributes associated with a call to
/// `GetMemoryMap` of UEFI. Note that all values refer to the time this was
/// called. All following invocations (hidden, subtle, and asynchronous ones)
/// will likely invalidate this.
#[derive(Copy, Clone, Debug)]
pub struct MemoryMapMeta {
    /// The actual size of the map.
    pub map_size: usize,
    /// The reported memory descriptor size. Note that this is the reference
    /// and never `size_of::<MemoryDescriptor>()`!
    pub desc_size: usize,
    /// A unique memory key bound to a specific memory map version/state.
    pub map_key: MemoryMapKey,
    /// The version of the descriptor struct.
    pub desc_version: u32,
}

impl MemoryMapMeta {
    /// Returns the amount of entries in the map.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        assert_eq!(self.map_size % self.desc_size, 0);
        self.map_size / self.desc_size
    }

    /// Runs some sanity assertions.
    pub fn assert_sanity_checks(&self) {
        assert!(self.desc_size > 0);
        // Although very unlikely, this might fail if the memory descriptor is
        // extended by a future UEFI revision by a significant amount, we
        // update the struct, but an old UEFI implementation reports a small
        // size.
        assert!(self.desc_size >= mem::size_of::<MemoryDescriptor>());
        assert!(self.map_size > 0);

        // Ensure the mmap size is (somehow) sane.
        const ONE_GB: usize = 1024 * 1024 * 1024;
        assert!(self.map_size <= ONE_GB);
    }
}

/// An accessory to the UEFI memory map and associated metadata that can be
/// either iterated or indexed like an array.
///
/// A [`MemoryMap`] is always associated with the unique [`MemoryMapKey`]
/// bundled with the map.
///
/// To iterate over the entries, call [`MemoryMap::entries`].
///
/// ## UEFI pitfalls
/// Note that a MemoryMap can quickly become outdated, as soon as any explicit
/// or hidden allocation happens.
///
/// As soon as boot services are excited, all previous obtained memory maps must
/// be considered as outdated, except if the [`MemoryMapKey`] equals the one
/// returned by `exit_boot_services()`.
///
/// **Please note** that when working with memory maps, the `entry_size` is
/// usually larger than `size_of::<MemoryDescriptor` [[0]]. So to be safe,
/// always use `entry_size` as step-size when interfacing with the memory map on
/// a low level.
///
/// [0]: https://github.com/tianocore/edk2/blob/7142e648416ff5d3eac6c6d607874805f5de0ca8/MdeModulePkg/Core/PiSmmCore/Page.c#L1059
pub trait MemoryMap: Debug {
    // TODO also require IntoIterator?! :)

    /// Returns the associated [`MemoryMapMeta`].
    #[must_use]
    fn meta(&self) -> MemoryMapMeta;

    /// Returns the associated [`MemoryMapKey`].
    #[must_use]
    fn key(&self) -> MemoryMapKey;

    /// Returns the number of keys in the map.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns if the memory map is empty.
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the [`MemoryDescriptor`] at the given index, if
    /// present.
    #[must_use]
    fn get(&self, index: usize) -> Option<&MemoryDescriptor> {
        if index >= self.len() {
            None
        } else {
            let offset = index * self.meta().desc_size;
            unsafe {
                self.buffer()
                    .as_ptr()
                    .add(offset)
                    .cast::<MemoryDescriptor>()
                    .as_ref()
            }
        }
    }

    /// Returns a reference to the underlying memory.
    fn buffer(&self) -> &[u8];

    /// Returns an Iterator of type [`MemoryMapIter`].
    fn entries(&self) -> MemoryMapIter<'_>;
}

/// Extension to [`MemoryMap`] that adds mutable operations. This also includes
/// the ability to sort the memory map.
pub trait MemoryMapMut: MemoryMap {
    /// Returns a mutable reference to the [`MemoryDescriptor`] at the given
    /// index, if present.
    #[must_use]
    fn get_mut(&mut self, index: usize) -> Option<&mut MemoryDescriptor> {
        if index >= self.len() {
            None
        } else {
            let offset = index * self.meta().desc_size;
            unsafe {
                self.buffer_mut()
                    .as_mut_ptr()
                    .add(offset)
                    .cast::<MemoryDescriptor>()
                    .as_mut()
            }
        }
    }

    /// Sorts the memory map by physical address in place. This operation is
    /// optional and should be invoked only once.
    fn sort(&mut self);

    /// Returns a reference to the underlying memory.
    ///
    /// # Safety
    ///
    /// This is unsafe as there is a potential to create invalid entries.
    unsafe fn buffer_mut(&mut self) -> &mut [u8];
}

/// Implementation of [`MemoryMap`] for the given buffer.
#[derive(Debug)]
pub struct MemoryMapRef<'a> {
    buf: &'a [u8],
    key: MemoryMapKey,
    meta: MemoryMapMeta,
    len: usize,
}

impl<'a> MemoryMap for MemoryMapRef<'a> {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.key
    }

    fn len(&self) -> usize {
        self.len
    }

    fn buffer(&self) -> &[u8] {
        self.buf
    }

    fn entries(&self) -> MemoryMapIter<'_> {
        MemoryMapIter {
            memory_map: self,
            index: 0,
        }
    }
}

impl Index<usize> for MemoryMapRef<'_> {
    type Output = MemoryDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

/// Implementation of [`MemoryMapMut`] for the given buffer.
#[derive(Debug)]
pub struct MemoryMapRefMut<'a> {
    buf: &'a mut [u8],
    key: MemoryMapKey,
    meta: MemoryMapMeta,
    len: usize,
}

impl<'a> MemoryMap for MemoryMapRefMut<'a> {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.key
    }

    fn len(&self) -> usize {
        self.len
    }

    fn buffer(&self) -> &[u8] {
        self.buf
    }

    fn entries(&self) -> MemoryMapIter<'_> {
        MemoryMapIter {
            memory_map: self,
            index: 0,
        }
    }
}

impl<'a> MemoryMapMut for MemoryMapRefMut<'a> {
    fn sort(&mut self) {
        unsafe {
            self.qsort(0, self.len - 1);
        }
    }

    unsafe fn buffer_mut(&mut self) -> &mut [u8] {
        self.buf
    }
}

impl<'a> MemoryMapRefMut<'a> {
    /// Hoare partition scheme for quicksort.
    /// Must be called with `low` and `high` being indices within bounds.
    unsafe fn qsort(&mut self, low: usize, high: usize) {
        if low >= high {
            return;
        }

        let p = self.partition(low, high);
        self.qsort(low, p);
        self.qsort(p + 1, high);
    }

    unsafe fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.get_element_phys_addr(low + (high - low) / 2);

        let mut left_index = low.wrapping_sub(1);
        let mut right_index = high.wrapping_add(1);

        loop {
            while {
                left_index = left_index.wrapping_add(1);

                self.get_element_phys_addr(left_index) < pivot
            } {}

            while {
                right_index = right_index.wrapping_sub(1);

                self.get_element_phys_addr(right_index) > pivot
            } {}

            if left_index >= right_index {
                return right_index;
            }

            self.swap(left_index, right_index);
        }
    }

    /// Indices must be smaller than len.
    unsafe fn swap(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }

        let base = self.buf.as_mut_ptr();

        unsafe {
            ptr::swap_nonoverlapping(
                base.add(index1 * self.meta.desc_size),
                base.add(index2 * self.meta.desc_size),
                self.meta.desc_size,
            );
        }
    }

    fn get_element_phys_addr(&self, index: usize) -> PhysicalAddress {
        let offset = index.checked_mul(self.meta.desc_size).unwrap();
        let elem = unsafe { &*self.buf.as_ptr().add(offset).cast::<MemoryDescriptor>() };
        elem.phys_start
    }
}

impl Index<usize> for MemoryMapRefMut<'_> {
    type Output = MemoryDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for MemoryMapRefMut<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/// Implementation of [`MemoryMapMut`] that owns the buffer on the UEFI heap.
#[derive(Debug)]
pub struct MemoryMapOwned {
    /// Backing memory, properly initialized at this point.
    buf: MemoryMapBackingMemory,
    key: MemoryMapKey,
    meta: MemoryMapMeta,
    len: usize,
}

impl MemoryMapOwned {
    /// Creates a [`MemoryMapOwned`] from the give initialized memory map behind
    /// the buffer and the reported `desc_size` from UEFI.
    pub(crate) fn from_initialized_mem(buf: MemoryMapBackingMemory, meta: MemoryMapMeta) -> Self {
        assert!(meta.desc_size >= mem::size_of::<MemoryDescriptor>());
        let len = meta.entry_count();
        MemoryMapOwned {
            key: MemoryMapKey(0),
            buf,
            meta,
            len,
        }
    }

    #[cfg(test)]
    fn from_raw(buf: &mut [u8], desc_size: usize) -> Self {
        let mem = MemoryMapBackingMemory::from_slice(buf);
        Self::from_initialized_mem(
            mem,
            MemoryMapMeta {
                map_size: buf.len(),
                desc_size,
                map_key: MemoryMapKey(0),
                desc_version: MemoryDescriptor::VERSION,
            },
        )
    }
}

impl MemoryMap for MemoryMapOwned {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.key
    }

    fn len(&self) -> usize {
        self.len
    }

    fn buffer(&self) -> &[u8] {
        self.buf.as_slice()
    }

    fn entries(&self) -> MemoryMapIter<'_> {
        MemoryMapIter {
            memory_map: self,
            index: 0,
        }
    }
}

impl MemoryMapMut for MemoryMapOwned {
    fn sort(&mut self) {
        let mut reference = MemoryMapRefMut {
            buf: self.buf.as_mut_slice(),
            key: self.key,
            meta: self.meta,
            len: self.len,
        };
        reference.sort();
    }

    unsafe fn buffer_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut_slice()
    }
}

impl Index<usize> for MemoryMapOwned {
    type Output = MemoryDescriptor;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for MemoryMapOwned {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/// An iterator of [`MemoryDescriptor`]. The underlying memory map is always
/// associated with a unique [`MemoryMapKey`].
#[derive(Debug, Clone)]
pub struct MemoryMapIter<'a> {
    memory_map: &'a dyn MemoryMap,
    index: usize,
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryDescriptor;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.memory_map.len() - self.index;

        (sz, Some(sz))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let desc = self.memory_map.get(self.index)?;

        self.index += 1;

        Some(desc)
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {
    fn len(&self) -> usize {
        self.memory_map.len()
    }
}

#[cfg(test)]
mod tests_mmap_real {
    use super::*;
    use alloc::vec::Vec;
    use core::mem::size_of;
    use core::slice;

    const MMAP_META: MemoryMapMeta = MemoryMapMeta {
        map_size: MMAP_RAW.len() * size_of::<u64>(),
        desc_size: 48,
        map_key: MemoryMapKey(0),
        desc_version: 1,
    };
    /// Sample with 10 entries of a real UEFI memory map extracted from our
    /// UEFI test runner.
    const MMAP_RAW: [u64; 60] = [
        3, 0, 0, 1, 15, 0, 7, 4096, 0, 134, 15, 0, 4, 552960, 0, 1, 15, 0, 7, 557056, 0, 24, 15, 0,
        7, 1048576, 0, 1792, 15, 0, 10, 8388608, 0, 8, 15, 0, 7, 8421376, 0, 3, 15, 0, 10, 8433664,
        0, 1, 15, 0, 7, 8437760, 0, 4, 15, 0, 10, 8454144, 0, 240, 15, 0,
    ];

    #[test]
    fn basic_functionality() {
        let mut buf = MMAP_RAW;
        let buf =
            unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<u8>(), MMAP_META.map_size) };
        let mut mmap = MemoryMapOwned::from_raw(buf, MMAP_META.desc_size);
        mmap.sort();

        let entries = mmap.entries().copied().collect::<Vec<_>>();

        let expected = [
            MemoryDescriptor {
                ty: MemoryType::BOOT_SERVICES_CODE,
                phys_start: 0x0,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x1000,
                virt_start: 0x0,
                page_count: 0x86,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::BOOT_SERVICES_DATA,
                phys_start: 0x87000,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x88000,
                virt_start: 0x0,
                page_count: 0x18,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x100000,
                virt_start: 0x0,
                page_count: 0x700,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x800000,
                virt_start: 0x0,
                page_count: 0x8,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x808000,
                virt_start: 0x0,
                page_count: 0x3,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x80b000,
                virt_start: 0x0,
                page_count: 0x1,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::CONVENTIONAL,
                phys_start: 0x80c000,
                virt_start: 0x0,
                page_count: 0x4,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
            MemoryDescriptor {
                ty: MemoryType::ACPI_NON_VOLATILE,
                phys_start: 0x810000,
                virt_start: 0x0,
                page_count: 0xf0,
                att: MemoryAttribute::UNCACHEABLE
                    | MemoryAttribute::WRITE_COMBINE
                    | MemoryAttribute::WRITE_THROUGH
                    | MemoryAttribute::WRITE_BACK,
            },
        ];
        assert_eq!(entries.as_slice(), &expected);
    }
}
