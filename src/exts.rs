//! Utility functions for the most common UEFI patterns.

use alloc_api::{
    alloc::{alloc, handle_alloc_error},
    boxed::Box,
};
use core::alloc::Layout;
use core::slice;

/// Creates a boxed byte buffer using the standard allocator.
///
/// # Panics
///
/// Calls `handle_alloc_error` if the layout has a size of zero or allocation fails.
pub fn allocate_buffer(layout: Layout) -> Box<[u8]> {
    if layout.size() == 0 {
        handle_alloc_error(layout);
    }
    unsafe {
        let data = alloc(layout);
        if data.is_null() {
            handle_alloc_error(layout);
        }
        let len = layout.size();
        let slice = slice::from_raw_parts_mut(data, len);
        Box::from_raw(slice)
    }
}
