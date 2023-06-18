//! Data type definitions
//!
//! This module defines the basic data types that are used throughout uefi-rs

use core::ffi::c_void;
use core::ptr::{self, NonNull};

/// Opaque handle to an UEFI entity (protocol, image...), guaranteed to be non-null.
///
/// If you need to have a nullable handle (for a custom UEFI FFI for example) use `Option<Handle>`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Handle(NonNull<c_void>);

impl Handle {
    /// Creates a new [`Handle`] from a raw address. The address might
    /// come from the Multiboot2 information structure or something similar.
    ///
    /// # Example
    /// ```no_run
    /// use core::ffi::c_void;
    /// use uefi::Handle;
    ///
    /// let image_handle_addr = 0xdeadbeef as *mut c_void;
    ///
    /// let uefi_image_handle = unsafe {
    ///     Handle::from_ptr(image_handle_addr).expect("Pointer must not be null!")
    /// };
    /// ```
    ///
    /// # Safety
    /// This function is unsafe because the caller must be sure that the pointer
    /// is valid. Otherwise, further operations on the object might result in
    /// undefined behaviour, even if the methods aren't marked as unsafe.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        // shorthand for "|ptr| Self(ptr)"
        NonNull::new(ptr).map(Self)
    }

    /// Get the underlying raw pointer.
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub(crate) fn opt_to_ptr(handle: Option<Self>) -> *mut c_void {
        handle.map(|h| h.0.as_ptr()).unwrap_or(ptr::null_mut())
    }
}

/// Handle to an event structure, guaranteed to be non-null.
///
/// If you need to have a nullable event, use `Option<Event>`.
#[repr(transparent)]
#[derive(Debug)]
pub struct Event(NonNull<c_void>);

impl Event {
    /// Clone this `Event`
    ///
    /// # Safety
    /// When an event is closed by calling `BootServices::close_event`, that event and ALL references
    /// to it are invalidated and the underlying memory is freed by firmware. The caller must ensure
    /// that any clones of a closed `Event` are never used again.
    #[must_use]
    pub const unsafe fn unsafe_clone(&self) -> Self {
        Self(self.0)
    }

    /// Create an `Event` from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pointer is valid.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> Option<Self> {
        NonNull::new(ptr).map(Self)
    }

    /// Get the underlying raw pointer.
    #[must_use]
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

/// Trait for querying the alignment of a struct.
///
/// For a statically-sized type the alignment can be retrieved with
/// [`core::mem::align_of`]. For a dynamically-sized type (DST),
/// [`core::mem::align_of_val`] provides the alignment given a reference. But in
/// some cases it's helpful to know the alignment of a DST prior to having a
/// value, meaning there's no reference to pass to `align_of_val`. For example,
/// when using an API that creates a value using a `[u8]` buffer, the alignment
/// of the buffer must be checked. The `Align` trait makes that possible by
/// allowing the appropriate alignment to be manually specified.
pub trait Align {
    /// Required memory alignment for this type
    fn alignment() -> usize;

    /// Calculate the offset from `val` necessary to make it aligned,
    /// rounding up. For example, if `val` is 1 and the alignment is 8,
    /// this will return 7. Returns 0 if `val == 0`.
    #[must_use]
    fn offset_up_to_alignment(val: usize) -> usize {
        assert!(Self::alignment() != 0);
        let r = val % Self::alignment();
        if r == 0 {
            0
        } else {
            Self::alignment() - r
        }
    }

    /// Round `val` up so that it is aligned.
    #[must_use]
    fn round_up_to_alignment(val: usize) -> usize {
        val + Self::offset_up_to_alignment(val)
    }

    /// Get a subslice of `buf` where the address of the first element
    /// is aligned. Returns `None` if no element of the buffer is
    /// aligned.
    fn align_buf(buf: &mut [u8]) -> Option<&mut [u8]> {
        let addr = buf.as_ptr() as usize;
        let offset = Self::offset_up_to_alignment(addr);
        buf.get_mut(offset..)
    }

    /// Assert that some storage is correctly aligned for this type
    fn assert_aligned(storage: &mut [u8]) {
        if !storage.is_empty() {
            assert_eq!(
                (storage.as_ptr() as usize) % Self::alignment(),
                0,
                "The provided storage is not correctly aligned for this type"
            )
        }
    }
}

mod guid;
pub use self::guid::{Guid, Identify};

pub mod chars;
pub use self::chars::{Char16, Char8};

#[macro_use]
mod opaque;

mod strs;
pub use self::strs::{
    CStr16, CStr8, EqStrUntilNul, FromSliceWithNulError, FromStrWithBufError, UnalignedCStr16Error,
};

#[cfg(feature = "alloc")]
mod owned_strs;
#[cfg(feature = "alloc")]
pub use self::owned_strs::{CString16, FromStrError};

mod unaligned_slice;
pub use unaligned_slice::UnalignedSlice;

pub use uefi_raw::{PhysicalAddress, VirtualAddress};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment() {
        struct X {}

        impl Align for X {
            fn alignment() -> usize {
                4
            }
        }

        assert_eq!(X::offset_up_to_alignment(0), 0);
        assert_eq!(X::offset_up_to_alignment(1), 3);
        assert_eq!(X::offset_up_to_alignment(2), 2);
        assert_eq!(X::offset_up_to_alignment(3), 1);
        assert_eq!(X::offset_up_to_alignment(4), 0);
        assert_eq!(X::offset_up_to_alignment(5), 3);
        assert_eq!(X::offset_up_to_alignment(6), 2);
        assert_eq!(X::offset_up_to_alignment(7), 1);
        assert_eq!(X::offset_up_to_alignment(8), 0);

        assert_eq!(X::round_up_to_alignment(0), 0);
        assert_eq!(X::round_up_to_alignment(1), 4);
        assert_eq!(X::round_up_to_alignment(2), 4);
        assert_eq!(X::round_up_to_alignment(3), 4);
        assert_eq!(X::round_up_to_alignment(4), 4);
        assert_eq!(X::round_up_to_alignment(5), 8);
        assert_eq!(X::round_up_to_alignment(6), 8);
        assert_eq!(X::round_up_to_alignment(7), 8);
        assert_eq!(X::round_up_to_alignment(8), 8);

        // Get an intentionally misaligned buffer.
        let mut buffer = [0u8; 16];
        let mut buffer = &mut buffer[..];
        if (buffer.as_ptr() as usize) % X::alignment() == 0 {
            buffer = &mut buffer[1..];
        }

        let buffer = X::align_buf(buffer).unwrap();
        X::assert_aligned(buffer);
    }
}
