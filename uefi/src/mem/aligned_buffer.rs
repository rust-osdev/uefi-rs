// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::alloc::{Layout, LayoutError, alloc_zeroed, dealloc};
use core::error::Error;
use core::num::NonZero;
use core::ptr::NonNull;
use core::{fmt, slice};

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
    /// The memory is zero-initialized.
    ///
    /// # Panics
    /// This method panics when the allocation fails (e.g. due to an out of memory situation).
    pub fn from_size_align(len: usize, alignment: usize) -> Result<Self, LayoutError> {
        let layout = Layout::from_size_align(len, alignment)?;
        Ok(Self::from_layout(layout))
    }

    /// Allocate a new memory region with the requested layout.
    ///
    /// The memory is zero-initialized.
    ///
    /// # Panics
    /// This method panics when the allocation fails (e.g. due to an out of memory situation).
    #[must_use]
    pub fn from_layout(layout: Layout) -> Self {
        let ptr = if layout.size() == 0 {
            // `GlobalAlloc` forbids zero-size layouts. A dangling but aligned
            // pointer is all that a zero-length slice needs.
            let align = NonZero::new(layout.align()).expect("layout alignment should be non-zero");
            NonNull::without_provenance(align)
        } else {
            // The safe accessors (`as_slice`, `iter`, ...) hand out `&[u8]`
            // over the whole region, so it must be initialized from the start.
            // SAFETY: The layout has a non-zero size.
            let ptr = unsafe { alloc_zeroed(layout) };
            NonNull::new(ptr).expect("Allocation failed")
        };
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
    pub const fn ptr_mut(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get the underlying memory region as immutable slice.
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        // SAFETY: The pointer is valid for the requested slice length.
        unsafe { slice::from_raw_parts(self.ptr(), self.size()) }
    }

    /// Get the underlying memory region as mutable slice.
    #[must_use]
    pub const fn as_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: The pointer is valid for the requested slice length.
        unsafe { slice::from_raw_parts_mut(self.ptr_mut(), self.size()) }
    }

    /// Get the size of the aligned memory region managed by this instance.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.layout.size()
    }

    /// Returns an iterator over the aligned buffer contents.
    pub fn iter(&self) -> impl Iterator<Item = &u8> {
        self.as_slice().iter()
    }

    /// Returns a mutable iterator over the aligned buffer contents.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut u8> {
        self.as_slice_mut().iter_mut()
    }

    /// Fill the aligned memory region with data from the given buffer.
    ///
    /// The length of `src` must be the same as `self`.
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert_eq!(self.size(), src.len());
        // SAFETY: The memory is valid.
        unsafe {
            self.ptr_mut().copy_from(src.as_ptr(), src.len());
        }
    }

    /// Fill the aligned memory region with data from the given iterator.
    /// If the given iterator is shorter than the buffer, the remaining area will be left untouched.
    pub fn copy_from_iter(&mut self, src: impl Iterator<Item = u8>) {
        self.iter_mut()
            .zip(src)
            .for_each(|(dst, src_byte)| *dst = src_byte);
    }

    /// Check the buffer's alignment against the `required_alignment`.
    pub fn check_alignment(&self, required_alignment: usize) -> Result<(), AlignmentError> {
        //TODO: use bfr.addr() when it's available
        if !(self.ptr() as usize).is_multiple_of(required_alignment) {
            return Err(AlignmentError); //TODO: use >is_aligned_to< when it's available
        }
        Ok(())
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        // Zero-size buffers were never allocated, see `from_layout`.
        if self.layout.size() == 0 {
            return;
        }
        // SAFETY: The memory was allocated with this layout.
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
            for request_len in [1_usize, 32, 64, 128, 1024] {
                let buffer =
                    AlignedBuffer::from_size_align(request_len, request_alignment).unwrap();
                assert_eq!(buffer.ptr() as usize % request_alignment, 0);
                assert_eq!(buffer.size(), request_len);
            }
        }
    }

    /// A fresh buffer must be readable through the safe accessors, so its
    /// memory must be initialized (Miri catches a read of uninitialized
    /// memory here otherwise).
    #[test]
    fn test_fresh_buffer_is_zeroed() {
        let bfr = AlignedBuffer::from_size_align(8, 8).unwrap();
        assert_eq!(bfr.as_slice(), [0; 8]);
    }

    /// A zero-size buffer must not hit the allocator (zero-size layouts are
    /// not allowed by `GlobalAlloc`), but must still behave like an empty,
    /// aligned buffer.
    #[test]
    fn test_zero_size() {
        for request_alignment in [1, 8, 64] {
            let mut bfr = AlignedBuffer::from_size_align(0, request_alignment).unwrap();
            assert_eq!(bfr.size(), 0);
            assert!(bfr.as_slice().is_empty());
            assert!(bfr.as_slice_mut().is_empty());
            bfr.check_alignment(request_alignment).unwrap();
            bfr.copy_from_slice(&[]);
        }
    }

    #[test]
    fn test_copy_from_iter() {
        let src8: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        {
            // src as large as dst
            let mut bfr = AlignedBuffer::from_size_align(8, 8).unwrap();
            bfr.copy_from_iter(src8.iter().cloned());
            assert_eq!(bfr.as_slice(), src8);
        }
        {
            // src larger than dst
            let mut bfr = AlignedBuffer::from_size_align(7, 8).unwrap();
            bfr.copy_from_iter(src8.iter().cloned());
            assert_eq!(bfr.as_slice(), [1, 2, 3, 4, 5, 6, 7]);
        }
        {
            // src smaller than dst
            let mut bfr = AlignedBuffer::from_size_align(9, 8).unwrap();
            bfr.iter_mut().for_each(|dst| *dst = 0); // fill with 0s
            bfr.copy_from_iter(src8.iter().cloned());
            assert_eq!(bfr.as_slice(), [1, 2, 3, 4, 5, 6, 7, 8, 0]);
        }
    }
}
