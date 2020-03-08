//! Utility functions for the most common UEFI patterns.

use alloc_api::{
    alloc::{handle_alloc_error, AllocRef, Global},
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
        match Global.alloc(layout) {
            Ok((mem, len)) => Box::from_raw(slice::from_raw_parts_mut(mem.as_ptr(), len)),
            Err(_) => handle_alloc_error(layout),
        }
    }
}
