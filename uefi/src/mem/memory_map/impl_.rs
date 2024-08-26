//! Module for [`MemoryMapOwned`], [`MemoryMapRef`], and [`MemoryMapRefMut`],
//! as well as relevant helper types, such as [`MemoryMapBackingMemory`].

use super::*;
use crate::boot;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Index, IndexMut};
use core::ptr::NonNull;
use core::{mem, ptr};
use uefi_raw::PhysicalAddress;

/// Errors that may happen when constructing a [`MemoryMapRef`] or
/// [`MemoryMapRefMut`].
#[derive(Copy, Clone, Debug)]
pub enum MemoryMapError {
    /// The buffer is not 8-byte aligned.
    Misaligned,
    /// The memory map size is invalid.
    InvalidSize,
}

impl Display for MemoryMapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for MemoryMapError {}

/// Implementation of [`MemoryMap`] for the given buffer.
#[derive(Debug)]
pub struct MemoryMapRef<'a> {
    buf: &'a [u8],
    meta: MemoryMapMeta,
    len: usize,
}

impl<'a> MemoryMapRef<'a> {
    /// Constructs a new [`MemoryMapRef`].
    ///
    /// The underlying memory might contain an invalid/malformed memory map
    /// which can't be checked during construction of this type. The entry
    /// iterator might yield unexpected results.
    pub fn new(buffer: &'a [u8], meta: MemoryMapMeta) -> Result<Self, MemoryMapError> {
        if buffer.as_ptr().align_offset(8) != 0 {
            return Err(MemoryMapError::Misaligned);
        }
        if buffer.len() < meta.map_size {
            return Err(MemoryMapError::InvalidSize);
        }
        Ok(Self {
            buf: buffer,
            meta,
            len: meta.entry_count(),
        })
    }
}

impl<'a> MemoryMap for MemoryMapRef<'a> {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.meta.map_key
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
    meta: MemoryMapMeta,
    len: usize,
}

impl<'a> MemoryMapRefMut<'a> {
    /// Constructs a new [`MemoryMapRefMut`].
    ///
    /// The underlying memory might contain an invalid/malformed memory map
    /// which can't be checked during construction of this type. The entry
    /// iterator might yield unexpected results.
    pub fn new(buffer: &'a mut [u8], meta: MemoryMapMeta) -> Result<Self, MemoryMapError> {
        if buffer.as_ptr().align_offset(8) != 0 {
            return Err(MemoryMapError::Misaligned);
        }
        if buffer.len() < meta.map_size {
            return Err(MemoryMapError::InvalidSize);
        }
        Ok(Self {
            buf: buffer,
            meta,
            len: meta.entry_count(),
        })
    }
}

impl<'a> MemoryMap for MemoryMapRefMut<'a> {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.meta.map_key
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
///
/// [`BootServices::get_memory_map`]: crate::table::boot::BootServices::get_memory_map
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
        let memory_map_meta = boot::memory_map_size();
        let len = Self::safe_allocation_size_hint(memory_map_meta);
        let ptr = boot::allocate_pool(memory_type, len)?.as_ptr();

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

    /// INTERNAL, for unit tests.
    ///
    /// Creates an instance from the provided memory, which is not necessarily
    /// on the UEFI heap.
    #[cfg(test)]
    pub(crate) fn from_slice(buffer: &mut [u8]) -> Self {
        let len = buffer.len();
        unsafe { Self::from_raw(buffer.as_mut_ptr(), len) }
    }

    /// Returns a "safe" best-effort size hint for the memory map size with
    /// some additional bytes in buffer compared to the [`MemoryMapMeta`]. This
    /// takes into account that, as you go, more (small) allocations might
    /// happen.
    #[must_use]
    const fn safe_allocation_size_hint(mmm: MemoryMapMeta) -> usize {
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

// Don't drop when we use this in unit tests.
impl Drop for MemoryMapBackingMemory {
    fn drop(&mut self) {
        if boot::are_boot_services_active() {
            let res = unsafe { boot::free_pool(self.0.cast()) };
            if let Err(e) = res {
                log::error!("Failed to deallocate memory map: {e:?}");
            }
        } else {
            log::debug!("Boot services are exited. Memory map won't be freed using the UEFI boot services allocator.");
        }
    }
}

/// Implementation of [`MemoryMapMut`] that owns the buffer on the UEFI heap.
#[derive(Debug)]
pub struct MemoryMapOwned {
    /// Backing memory, properly initialized at this point.
    pub(crate) buf: MemoryMapBackingMemory,
    pub(crate) meta: MemoryMapMeta,
    pub(crate) len: usize,
}

impl MemoryMapOwned {
    /// Creates a [`MemoryMapOwned`] from the given **initialized** memory map
    /// (stored inside the provided buffer) and the corresponding
    /// [`MemoryMapMeta`].
    pub(crate) fn from_initialized_mem(buf: MemoryMapBackingMemory, meta: MemoryMapMeta) -> Self {
        assert!(meta.desc_size >= mem::size_of::<MemoryDescriptor>());
        let len = meta.entry_count();
        Self { buf, meta, len }
    }
}

impl MemoryMap for MemoryMapOwned {
    fn meta(&self) -> MemoryMapMeta {
        self.meta
    }

    fn key(&self) -> MemoryMapKey {
        self.meta.map_key
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use core::mem::size_of;

    const BASE_MMAP_UNSORTED: [MemoryDescriptor; 3] = [
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x3000,
            virt_start: 0x3000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x2000,
            virt_start: 0x2000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x1000,
            virt_start: 0x1000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
    ];

    /// Returns a copy of [`BASE_MMAP_UNSORTED`] owned on the stack.
    fn new_mmap_memory() -> [MemoryDescriptor; 3] {
        BASE_MMAP_UNSORTED
    }

    fn mmap_raw<'a>(memory: &mut [MemoryDescriptor]) -> (&'a mut [u8], MemoryMapMeta) {
        let desc_size = size_of::<MemoryDescriptor>();
        let len = memory.len() * desc_size;
        let ptr = memory.as_mut_ptr().cast::<u8>();
        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        let meta = MemoryMapMeta {
            map_size: len,
            desc_size,
            map_key: Default::default(),
            desc_version: MemoryDescriptor::VERSION,
        };
        (slice, meta)
    }

    /// Basic sanity checks for the type [`MemoryMapRef`].
    #[test]
    fn memory_map_ref() {
        let mut memory = new_mmap_memory();
        let (mmap, meta) = mmap_raw(&mut memory);
        let mmap = MemoryMapRef::new(mmap, meta).unwrap();

        assert_eq!(mmap.entries().count(), 3);
        assert_eq!(
            mmap.entries().copied().collect::<Vec<_>>().as_slice(),
            &BASE_MMAP_UNSORTED
        );
        assert!(!mmap.is_sorted());
    }

    /// Basic sanity checks for the type [`MemoryMapRefMut`].
    #[test]
    fn memory_map_ref_mut() {
        let mut memory = new_mmap_memory();
        let (mmap, meta) = mmap_raw(&mut memory);
        let mut mmap = MemoryMapRefMut::new(mmap, meta).unwrap();

        assert_eq!(mmap.entries().count(), 3);
        assert_eq!(
            mmap.entries().copied().collect::<Vec<_>>().as_slice(),
            &BASE_MMAP_UNSORTED
        );
        assert!(!mmap.is_sorted());
        mmap.sort();
        assert!(mmap.is_sorted());
    }

    /// Basic sanity checks for the type [`MemoryMapOwned`].
    #[test]
    fn memory_map_owned() {
        let mut memory = new_mmap_memory();
        let (mmap, meta) = mmap_raw(&mut memory);
        let mmap = MemoryMapBackingMemory::from_slice(mmap);
        let mut mmap = MemoryMapOwned::from_initialized_mem(mmap, meta);

        assert_eq!(mmap.entries().count(), 3);
        assert_eq!(
            mmap.entries().copied().collect::<Vec<_>>().as_slice(),
            &BASE_MMAP_UNSORTED
        );
        assert!(!mmap.is_sorted());
        mmap.sort();
        assert!(mmap.is_sorted());
    }
}
