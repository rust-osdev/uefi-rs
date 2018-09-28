//! `uefi-alloc` implements Rust's global allocator interface using UEFI's memory allocation functions.
//!
//! Linking this crate in your app will allow you to use Rust's higher-level data structures,
//! like boxes, vectors, hash maps, linked lists and so on.
//!
//! # Usage
//!
//! Call the `init` function with a reference to the boot services table.
//! Failure to do so before calling a memory allocating function will panic.

// Enable additional lints.
#![warn(missing_docs)]
#![deny(clippy::all)]
#![no_std]
// Custom allocators are currently unstable.
#![feature(allocator_api)]
#![feature(tool_lints)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use uefi::table::boot::{BootServices, MemoryType};

/// Reference to the boot services table, used to call the pool memory allocation functions.
static mut BOOT_SERVICES: Option<&BootServices> = None;

/// Initializes the allocator.
pub fn init(boot_services: &'static BootServices) {
    unsafe {
        BOOT_SERVICES = Some(boot_services);
    }
}

fn boot_services() -> &'static BootServices {
    unsafe { BOOT_SERVICES.unwrap() }
}

/// Allocator which uses the UEFI pool allocation functions.
///
/// Only valid for as long as the UEFI runtime services are available.
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mem_ty = MemoryType::LOADER_DATA;
        let size = layout.size();
        let align = layout.align();

        // TODO: add support for other alignments.
        if align > 8 {
            // Unsupported alignment for allocation, UEFI can only allocate 8-byte aligned addresses
            ptr::null_mut()
        } else {
            boot_services()
                .allocate_pool(mem_ty, size)
                .map(|addr| addr as *mut _)
                .unwrap_or(ptr::null_mut())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let addr = ptr as usize;
        boot_services().free_pool(addr).unwrap();
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
