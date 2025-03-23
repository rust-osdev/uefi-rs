// SPDX-License-Identifier: MIT OR Apache-2.0

//! This module implements Rust's global allocator interface using UEFI's memory allocation functions.
//!
//! If the `global_allocator` feature is enabled, the [`Allocator`] will be used
//! as the global Rust allocator.
//!
//! This allocator can only be used while boot services are active. If boot
//! services are not active, `alloc` will return a null pointer, and `dealloc`
//! will panic.

use crate::boot;
use crate::mem::memory_map::MemoryType;
use crate::proto::loaded_image::LoadedImage;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU32, Ordering};

/// Get the memory type to use for allocation.
///
/// The first time this is called, the data type of the loaded image will be
/// retrieved. That value is cached in a static and reused on subsequent
/// calls. If the memory type of the loaded image cannot be retrieved for some
/// reason, a default of `LOADER_DATA` is used.
fn get_memory_type() -> MemoryType {
    // Initialize to a `RESERVED` to indicate the actual value hasn't been set yet.
    static MEMORY_TYPE: AtomicU32 = AtomicU32::new(MemoryType::RESERVED.0);

    let memory_type = MEMORY_TYPE.load(Ordering::Acquire);
    if memory_type == MemoryType::RESERVED.0 {
        let memory_type = if let Ok(loaded_image) =
            boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())
        {
            loaded_image.data_type()
        } else {
            MemoryType::LOADER_DATA
        };
        MEMORY_TYPE.store(memory_type.0, Ordering::Release);
        memory_type
    } else {
        MemoryType(memory_type)
    }
}

/// Allocator which uses the UEFI pool allocation functions.
///
/// Only valid for as long as the UEFI boot services are available.
#[derive(Debug)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    /// Allocate memory using [`boot::allocate_pool`]. The allocation's [memory
    /// type] matches the current image's [data type].
    ///
    /// [memory type]: MemoryType
    /// [data type]: LoadedImage::data_type
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !boot::are_boot_services_active() {
            return ptr::null_mut();
        }

        let size = layout.size();
        let align = layout.align();
        let memory_type = get_memory_type();

        if align > 8 {
            // The requested alignment is greater than 8, but `allocate_pool` is
            // only guaranteed to provide eight-byte alignment. Allocate extra
            // space so that we can return an appropriately-aligned pointer
            // within the allocation.
            let full_alloc_ptr = if let Ok(ptr) = boot::allocate_pool(memory_type, size + align) {
                ptr.as_ptr()
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
            unsafe {
                let aligned_ptr = full_alloc_ptr.add(offset);
                (aligned_ptr.cast::<*mut u8>()).sub(1).write(full_alloc_ptr);
                aligned_ptr
            }
        } else {
            // The requested alignment is less than or equal to eight, and
            // `allocate_pool` always provides eight-byte alignment, so we can
            // use `allocate_pool` directly.
            boot::allocate_pool(memory_type, size)
                .map(|ptr| ptr.as_ptr())
                .unwrap_or(ptr::null_mut())
        }
    }

    /// Deallocate memory using [`boot::free_pool`].
    unsafe fn dealloc(&self, mut ptr: *mut u8, layout: Layout) {
        if layout.align() > 8 {
            // Retrieve the pointer to the full allocation that was packed right
            // before the aligned allocation in `alloc`.
            ptr = unsafe { (ptr as *const *mut u8).sub(1).read() };
        }

        // OK to unwrap: `ptr` is required to be a valid allocation by the trait API.
        let ptr = NonNull::new(ptr).unwrap();

        // Warning: this will panic after exiting boot services.
        unsafe { boot::free_pool(ptr) }.unwrap();
    }
}
