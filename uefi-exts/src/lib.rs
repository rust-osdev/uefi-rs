//! Utility functions for the most common UEFI patterns.
//!
//! This crate simply exports some extension traits
//! which add utility functions to various UEFI objects.

#![no_std]
#![feature(alloc, alloc_layout_extra, allocator_api)]

extern crate alloc;

mod boot;
mod file;

pub use self::boot::BootServicesExt;
pub use self::file::FileExt;

use alloc::{
    alloc::{handle_alloc_error, Alloc, Global, Layout},
    boxed::Box,
};
use core::slice;

/// Creates a boxed byte buffer using the standard allocator
/// # Panics
/// Calls handle_alloc_error if the layout has a size of zero or allocation fails.
pub fn allocate_buffer(layout: Layout) -> Box<[u8]> {
    if layout.size() == 0 {
        handle_alloc_error(layout);
    }
    unsafe {
        match Global.alloc(layout) {
            Ok(mem) => Box::from_raw(slice::from_raw_parts_mut(mem.as_ptr(), layout.size())),
            Err(_) => handle_alloc_error(layout),
        }
    }
}
