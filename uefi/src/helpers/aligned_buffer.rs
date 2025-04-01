// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::alloc::{alloc, dealloc, Layout, LayoutError};
use core::error::Error;
use core::fmt;

/// Helper class to maintain the lifetime of a memory region allocated with a non-standard alignment.
/// Facilitates RAII to properly deallocate when lifetime of the object ends.
///
/// Note: This uses the global Rust allocator under the hood.
#[allow(clippy::len_without_is_empty)]
#[derive(Debug)]
pub struct AlignedBuffer {
    ptr: *mut u8,
    layout: Layout,
}

impl AlignedBuffer {
    /// Allocate a new memory region with the requested len and alignment.
    pub fn alloc(len: usize, alignment: usize) -> Result<Self, LayoutError> {
        let layout = Layout::from_size_align(len, alignment)?;
        let ptr = unsafe { alloc(layout) };
        Ok(Self { ptr, layout })
    }

    /// Get a pointer to the aligned memory region managed by this instance.
    #[must_use]
    pub const fn ptr(&self) -> *const u8 {
        self.ptr.cast_const()
    }

    /// Get a mutable pointer to the aligned memory region managed by this instance.
    #[must_use]
    pub fn ptr_mut(&mut self) -> *mut u8 {
        self.ptr
    }

    /// Get the size of the aligned memory region managed by this instance.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.layout.size()
    }

    /// Fill the aligned memory region with data from the given buffer.
    pub fn copy_from(&mut self, data: &[u8]) {
        let len = data.len().min(self.len());
        unsafe {
            self.ptr.copy_from(data.as_ptr(), len);
        }
    }

    /// Check the buffer's alignment against the `required_alignment`.
    pub fn check_alignment(&self, required_alignment: usize) -> Result<(), AlignmentError> {
        //TODO: use bfr.addr() when it's available
        if (self.ptr as usize) % required_alignment != 0 {
            return Err(AlignmentError); //TODO: use >is_aligned_to< when it's available
        }
        Ok(())
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

/// The `AlignmentError` is returned if a user-provided buffer doesn't fulfill alignment requirements.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlignmentError;
impl Error for AlignmentError {}
impl fmt::Display for AlignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid parameters to Layout::from_size_align")
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
                assert!(AlignedBuffer::alloc(request_len, request_alignment).is_err());
            }
        }
    }

    #[test]
    fn test_allocation_alignment() {
        for request_alignment in [1, 2, 4, 8, 16, 32, 64, 128] {
            for request_len in [1 as usize, 32, 64, 128, 1024] {
                let buffer = AlignedBuffer::alloc(request_len, request_alignment).unwrap();
                assert_eq!(buffer.ptr() as usize % request_alignment, 0);
                assert_eq!(buffer.len(), request_len);
            }
        }
    }
}
