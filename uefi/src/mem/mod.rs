// SPDX-License-Identifier: MIT OR Apache-2.0

//! Types, functions, traits, and other helpers to work with memory in UEFI
//! libraries and applications.

use crate::boot;
use core::ptr::NonNull;

pub mod memory_map;

#[cfg(feature = "alloc")]
pub(crate) mod util;

#[cfg(feature = "alloc")]
pub(crate) use util::*;

#[cfg(feature = "alloc")]
mod aligned_buffer;
#[cfg(feature = "alloc")]
pub use aligned_buffer::{AlignedBuffer, AlignmentError};

/// Wrapper for memory allocated with UEFI's pool allocator. The memory is freed
/// on drop.
#[derive(Debug)]
pub(crate) struct PoolAllocation(NonNull<u8>);

impl PoolAllocation {
    pub(crate) const fn new(ptr: NonNull<u8>) -> Self {
        Self(ptr)
    }

    pub(crate) const fn as_ptr(&self) -> NonNull<u8> {
        self.0
    }
}

impl Drop for PoolAllocation {
    fn drop(&mut self) {
        // Ignore errors returned by `free_pool` since we can't propagate them
        // from `drop`.
        let _ = unsafe { boot::free_pool(self.0) };
    }
}
