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
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]

#![no_std]

// Custom allocators are currently unstable.
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]

extern crate alloc;
use alloc::allocator::{Alloc, AllocErr, Layout};

extern crate uefi;
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
    unsafe {
        BOOT_SERVICES.unwrap()
    }
}

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let mem_ty = MemoryType::LoaderData;
        let size = layout.size();
        let align = layout.align();

        // TODO: add support for other alignments.
        if align > 8 {
            let details = "Unsupported alignment for allocation, UEFI can only allocate 8-byte aligned addresses";
            Err(AllocErr::Unsupported { details })
        } else {
            boot_services()
                .allocate_pool(mem_ty, size)
                .map(|addr| addr as *mut u8)
                // This is the only possible error, according to the spec.
                .map_err(|_status| AllocErr::Exhausted { request: layout })
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        let addr = ptr as usize;
        boot_services()
            .free_pool(addr)
            .unwrap();
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
