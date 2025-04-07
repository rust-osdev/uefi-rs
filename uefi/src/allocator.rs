// SPDX-License-Identifier: MIT OR Apache-2.0

//! This module exports [`Allocator`].
//!
//! The allocator can be used as global Rust allocator using the
//! `global_allocator` crate feature. See [`helpers`] for more info.
//!
//! [`helpers`]: uefi::helpers

use crate::boot::{self, AllocateType};
use crate::mem::memory_map::MemoryType;
use crate::proto::loaded_image::LoadedImage;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicU32, Ordering};
use uefi_raw::table::boot::PAGE_SIZE;

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

/// Helper to get a custom alignment out of an allocation with an alignment of
/// eight (UEFI default alignment). This works by allocating extra space and
/// storing a pointer to the actual allocation right above the allocation
/// handed out via the public API.
fn alloc_pool_aligned(memory_type: MemoryType, size: usize, align: usize) -> *mut u8 {
    let full_alloc_ptr = boot::allocate_pool(memory_type, size + align);
    let full_alloc_ptr = if let Ok(ptr) = full_alloc_ptr {
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
}

/// Returns whether the allocation is a multiple of a [`PAGE_SIZE`] and is
/// aligned to [`PAGE_SIZE`].
///
/// This does not only check the alignment but also the size. For types
/// allocated by Rust itself (e.g., `Box<T>`), the size is always at least the
/// alignment, as specified in the [Rust type layout]. However, to be also safe
/// when it comes to manual invocations, we additionally check if the size is
/// a multiple of [`PAGE_SIZE`].
///
/// [Rust type layout]: https://doc.rust-lang.org/reference/type-layout.html
const fn layout_allows_page_alloc_shortcut(layout: &Layout) -> bool {
    layout.size() % PAGE_SIZE == 0 && layout.align() == PAGE_SIZE
}

/// Allocator using UEFI boot services.
///
/// This type implements [`GlobalAlloc`] and can be marked with the
/// `#[global_allocator]` attribute to be used as global Rust allocator.
///
/// Note that if boot services are not active (anymore), [`Allocator::alloc`]
/// will return a null pointer and [`Allocator::dealloc`] will panic.
#[derive(Debug)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    /// Allocate memory using the UEFI boot services.
    ///
    /// The allocation's [memory type] matches the current image's [data type].
    ///
    /// [memory type]: MemoryType
    /// [data type]: LoadedImage::data_type
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !boot::are_boot_services_active() {
            return ptr::null_mut();
        }

        let memory_type = get_memory_type();
        let use_page_shortcut = layout_allows_page_alloc_shortcut(&layout);

        match (use_page_shortcut, layout.align()) {
            // Allocating pages is actually very expected in UEFI OS loaders, so
            // it makes sense to provide this optimization.
            (true, _) => {
                // To spammy, but useful for manual testing.
                // log::trace!("Taking PAGE_SIZE shortcut for layout={layout:?}");
                let count = layout.size().div_ceil(PAGE_SIZE);
                boot::allocate_pages(AllocateType::AnyPages, memory_type, count)
                    .map(|ptr| ptr.as_ptr())
                    .unwrap_or(ptr::null_mut())
            }
            (false, 0..=8 /* UEFI default alignment */) => {
                // The requested alignment is less than or equal to eight, and
                // `allocate_pool` always provides eight-byte alignment, so we can
                // use `allocate_pool` directly.
                boot::allocate_pool(memory_type, layout.size())
                    .map(|ptr| ptr.as_ptr())
                    .unwrap_or(ptr::null_mut())
            }
            (false, 9..) => alloc_pool_aligned(memory_type, layout.size(), layout.align()),
        }
    }

    /// Deallocate memory using the UEFI boot services.
    ///
    /// This will panic after exiting boot services.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();

        let use_page_shortcut = layout_allows_page_alloc_shortcut(&layout);

        match (use_page_shortcut, layout.align()) {
            (true, _) => {
                // To spammy, but useful for manual testing.
                // log::trace!("Taking PAGE_SIZE shortcut for layout={layout:?}");
                let count = layout.size().div_ceil(PAGE_SIZE);
                unsafe { boot::free_pages(ptr, count).unwrap() }
            }
            (false, 0..=8 /* UEFI default alignment */) => {
                // Warning: this will panic after exiting boot services.
                unsafe { boot::free_pool(ptr) }.unwrap();
            }
            (false, 9..) => {
                let ptr = ptr.as_ptr().cast::<*mut u8>();
                // Retrieve the pointer to the full allocation that was packed right
                // before the aligned allocation in `alloc`.
                let actual_alloc_ptr = unsafe { ptr.sub(1).read() };
                let ptr = NonNull::new(actual_alloc_ptr).unwrap();
                // Warning: this will panic after exiting boot services.
                unsafe { boot::free_pool(ptr) }.unwrap();
            }
        }
    }
}
