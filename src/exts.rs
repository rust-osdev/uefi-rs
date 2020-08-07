//! Utility functions for the most common UEFI patterns.

use alloc_api::{
    alloc::{handle_alloc_error, AllocRef, Global},
    boxed::Box,
};
use core::alloc::Layout;

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
        let mut slice = match Global.alloc(layout) {
            Ok(slice) => slice,
            Err(_) => handle_alloc_error(layout),
        };
        Box::from_raw(slice.as_mut())
    }
}
