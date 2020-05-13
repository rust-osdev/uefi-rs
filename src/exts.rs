//! Utility functions for the most common UEFI patterns.

use alloc_api::{
    alloc::{handle_alloc_error, AllocInit, AllocRef, Global, MemoryBlock},
    boxed::Box,
};
use core::{alloc::Layout, slice};

/// Creates a boxed byte buffer using the standard allocator
///
/// # Panics
///
/// Calls `handle_alloc_error` if the layout has a size of zero or allocation fails.
pub fn allocate_buffer(layout: Layout) -> Box<[u8]> {
    if layout.size() == 0 {
        handle_alloc_error(layout);
    }
    unsafe {
        let MemoryBlock { ptr, size } = match Global.alloc(layout, AllocInit::Uninitialized) {
            Ok(block) => block,
            Err(_) => handle_alloc_error(layout),
        };
        Box::from_raw(slice::from_raw_parts_mut(ptr.as_ptr(), size))
    }
}
