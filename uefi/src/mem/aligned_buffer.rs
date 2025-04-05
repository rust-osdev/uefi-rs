// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::alloc::{alloc, dealloc, Layout, LayoutError};
use core::error::Error;
use core::fmt;
use core::ptr::NonNull;

/// Helper class to maintain the lifetime of a memory region allocated with a non-standard alignment.
/// Facilitates RAII to properly deallocate when lifetime of the object ends.
///
/// Note: This uses the global Rust allocator under the hood.
#[derive(Debug)]
pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedBuffer {
    /// Allocate a new memory region with the requested len and alignment.
    ///
    /// # Panics
    /// This method panics when the allocation fails (e.g. due to an out of memory situation).
    pub fn from_size_align(len: usize, alignment: usize) -> Result<Self, LayoutError> {
        let layout = Layout::from_size_align(len, alignment)?;
        Ok(Self::from_layout(layout))
    }

    /// Allocate a new memory region with the requested layout.
    ///
    /// # Panics
    /// This method panics when the allocation fails (e.g. due to an out of memory situation).
    #[must_use]
    pub fn from_layout(layout: Layout) -> Self {
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).expect("Allocation failed");
        Self { ptr, layout }
    }

    // TODO: Add non-panicking method variants as soon as alloc::AllocError was stabilized (#32838).
    // - try_from_layout(layout: Layout) -> Result<Self, AllocError>;

    /// Get a pointer to the aligned memory region managed by this instance.
    #[must_use]
    pub const fn ptr(&self) -> *const u8 {
        self.ptr.as_ptr().cast_const()
    }

    /// Get a mutable pointer to the aligned memory region managed by this instance.
    #[must_use]
    pub fn ptr_mut(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get the size of the aligned memory region managed by this instance.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.layout.size()
    }

    /// Fill the aligned memory region with data from the given buffer.
    ///
    /// The length of `src` must be the same as `self`.
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert_eq!(self.size(), src.len());
        unsafe {
            self.ptr_mut().copy_from(src.as_ptr(), src.len());
        }
    }

    /// Check the buffer's alignment against the `required_alignment`.
    pub fn check_alignment(&self, required_alignment: usize) -> Result<(), AlignmentError> {
        //TODO: use bfr.addr() when it's available
        if (self.ptr() as usize) % required_alignment != 0 {
            return Err(AlignmentError); //TODO: use >is_aligned_to< when it's available
        }
        Ok(())
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr_mut(), self.layout);
        }
    }
}

/// The `AlignmentError` is returned if a user-provided buffer doesn't fulfill alignment requirements.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlignmentError;
impl Error for AlignmentError {}
impl fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Buffer alignment does not fulfill requirements.")
    }
}

#[cfg(test)]
mod tests {
    use super::AlignedBuffer;

    #[test]
    fn test_invalid_arguments() {
        // invalid alignments, valid len
        for request_alignment in [0, 3, 5, 7, 9] {
            for request_len in [1, 32, 64, 128, 1024] {
                assert!(AlignedBuffer::from_size_align(request_len, request_alignment).is_err());
            }
        }
    }

    #[test]
    fn test_allocation_alignment() {
        for request_alignment in [1, 2, 4, 8, 16, 32, 64, 128] {
            for request_len in [1 as usize, 32, 64, 128, 1024] {
                let buffer =
                    AlignedBuffer::from_size_align(request_len, request_alignment).unwrap();
                assert_eq!(buffer.ptr() as usize % request_alignment, 0);
                assert_eq!(buffer.size(), request_len);
            }
        }
    }
}
