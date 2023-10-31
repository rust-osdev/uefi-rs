//! This module implements Rust's global allocator interface using UEFI's memory allocation functions.
//!
//! If the `global_allocator` feature is enabled, the [`Allocator`] will be used
//! as the global Rust allocator.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use crate::boot::{allocate_pool, free_pool, image_handle, open_protocol_exclusive};
use crate::proto::loaded_image::LoadedImage;
use crate::table::boot::{BootServices, MemoryType};

/// The memory type used for pool memory allocations.
/// TODO: Use OnceCell when stablilized.
static mut MEMORY_TYPE: MemoryType = MemoryType::LOADER_DATA;

/// Initializes the allocator.
pub fn init(_boot_services: &BootServices) {
    if let Ok(loaded_image) = open_protocol_exclusive::<LoadedImage>(image_handle()) {
        unsafe { MEMORY_TYPE = loaded_image.data_type() }
    }
}

/// Notify the allocator library that boot services are not safe to call anymore
///
/// No longer needs to be called.
#[deprecated]
pub fn exit_boot_services() {}

/// Allocator which uses the UEFI pool allocation functions.
///
/// Only valid for as long as the UEFI boot services are available.
#[derive(Debug)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    /// Allocate memory using [`allocate_pool`]. The allocation is
    /// of type [`MemoryType::LOADER_DATA`] for UEFI applications, [`MemoryType::BOOT_SERVICES_DATA`]
    /// for UEFI boot drivers and [`MemoryType::RUNTIME_SERVICES_DATA`] for UEFI runtime drivers.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > 8 {
            // The requested alignment is greater than 8, but `allocate_pool` is
            // only guaranteed to provide eight-byte alignment. Allocate extra
            // space so that we can return an appropriately-aligned pointer
            // within the allocation.
            let full_alloc_ptr = if let Ok(ptr) = allocate_pool(MEMORY_TYPE, size + align) {
                ptr
            } else {
                return ptr::null_mut();
            };

            // Calculate the offset needed to get an aligned pointer within the
            // full allocation. If that offset is zero, increase it to `align`
            // so that we still have space to store the extra pointer described
            // below.
            let mut offset = full_alloc_ptr.align_offset(align);
            if offset == 0 {
                offset = align;
            }

            // Before returning the aligned allocation, store a pointer to the
            // full unaligned allocation in the bytes just before the aligned
            // allocation. We know we have at least eight bytes there due to
            // adding `align` to the memory allocation size. We also know the
            // write is appropriately aligned for a `*mut u8` pointer because
            // `align_ptr` is aligned, and alignments are always powers of two
            // (as enforced by the `Layout` type).
            let aligned_ptr = full_alloc_ptr.add(offset);
            (aligned_ptr.cast::<*mut u8>()).sub(1).write(full_alloc_ptr);
            aligned_ptr
        } else {
            // The requested alignment is less than or equal to eight, and
            // `allocate_pool` always provides eight-byte alignment, so we can
            // use `allocate_pool` directly.
            allocate_pool(MEMORY_TYPE, size).unwrap_or(ptr::null_mut())
        }
    }

    /// Deallocate memory using [`free_pool`].
    unsafe fn dealloc(&self, mut ptr: *mut u8, layout: Layout) {
        if layout.align() > 8 {
            // Retrieve the pointer to the full allocation that was packed right
            // before the aligned allocation in `alloc`.
            ptr = (ptr as *const *mut u8).sub(1).read();
        }
        free_pool(ptr).unwrap();
    }
}

#[cfg(feature = "global_allocator")]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
