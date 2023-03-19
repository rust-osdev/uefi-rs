//! This module implements Rust's global allocator interface using UEFI's memory allocation functions.
//!
//! If the `global_allocator` feature is enabled, the [`Allocator`] will be used
//! as the global Rust allocator.
//!
//! # Usage
//!
//! Call the `init` function with a reference to the boot services table.
//! Failure to do so before calling a memory allocating function will panic.
//!
//! Call the `exit_boot_services` function before exiting UEFI boot services.
//! Failure to do so will turn subsequent allocation into undefined behaviour.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};

use crate::table::boot::{BootServices, MemoryType};

/// Reference to the boot services table, used to call the pool memory allocation functions.
///
/// The inner pointer is only safe to dereference if UEFI boot services have not been
/// exited by the host application yet.
static mut BOOT_SERVICES: Option<NonNull<BootServices>> = None;

/// Initializes the allocator.
///
/// # Safety
///
/// This function is unsafe because you _must_ make sure that exit_boot_services
/// will be called when UEFI boot services will be exited.
pub unsafe fn init(boot_services: &BootServices) {
    BOOT_SERVICES = NonNull::new(boot_services as *const _ as *mut _);
}

/// Access the boot services
fn boot_services() -> NonNull<BootServices> {
    unsafe { BOOT_SERVICES.expect("Boot services are unavailable or have been exited") }
}

/// Notify the allocator library that boot services are not safe to call anymore
///
/// You must arrange for this function to be called on exit from UEFI boot services
pub fn exit_boot_services() {
    unsafe {
        BOOT_SERVICES = None;
    }
}

/// Allocator which uses the UEFI pool allocation functions.
///
/// Only valid for as long as the UEFI boot services are available.
#[derive(Debug)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    /// Allocate memory using [`BootServices::allocate_pool`]. The allocation is
    /// of type [`MemoryType::LOADER_DATA`].
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mem_ty = MemoryType::LOADER_DATA;
        let size = layout.size();
        let align = layout.align();

        if align > 8 {
            // The requested alignment is greater than 8, but `allocate_pool` is
            // only guaranteed to provide eight-byte alignment. Allocate extra
            // space so that we can return an appropriately-aligned pointer
            // within the allocation.
            let full_alloc_ptr =
                if let Ok(ptr) = boot_services().as_ref().allocate_pool(mem_ty, size + align) {
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
            boot_services()
                .as_ref()
                .allocate_pool(mem_ty, size)
                .unwrap_or(ptr::null_mut())
        }
    }

    /// Deallocate memory using [`BootServices::free_pool`].
    unsafe fn dealloc(&self, mut ptr: *mut u8, layout: Layout) {
        if layout.align() > 8 {
            // Retrieve the pointer to the full allocation that was packed right
            // before the aligned allocation in `alloc`.
            ptr = (ptr as *const *mut u8).sub(1).read();
        }
        boot_services().as_ref().free_pool(ptr).unwrap();
    }
}

#[cfg(feature = "global_allocator")]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
