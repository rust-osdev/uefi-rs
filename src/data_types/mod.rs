//! Data type definitions
//!
//! This module defines the basic data types that are used throughout uefi-rs

use core::{ffi::c_void, ptr::NonNull};

/// Opaque handle to an UEFI entity (protocol, image...), guaranteed to be non-null.
///
/// If you need to have a nullable handle (for a custom UEFI FFI for example) use `Option<Handle>`.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Handle(NonNull<c_void>);

/// Handle to an event structure
#[repr(transparent)]
pub struct Event(*mut c_void);

impl Event {
    /// Clone this `Event`
    ///
    /// # Safety
    /// When an event is closed by calling `BootServices::close_event`, that event and ALL references
    /// to it are invalidated and the underlying memory is freed by firmware. The caller must ensure
    /// that any clones of a closed `Event` are never used again.
    pub unsafe fn unsafe_clone(&self) -> Self {
        Self(self.0)
    }
}

/// Trait for querying the alignment of a struct
///
/// Needed for dynamic-sized types because `mem::align_of` has a `Sized` bound (due to `dyn Trait`)
pub trait Align {
    /// Required memory alignment for this type
    fn alignment() -> usize;

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
pub use self::guid::Guid;
pub use self::guid::{unsafe_guid, Identify};

pub mod chars;
pub use self::chars::{Char16, Char8};

#[macro_use]
mod enums;

mod strs;
pub use self::strs::{CStr16, CStr8, FromSliceWithNulError};

#[cfg(feature = "exts")]
mod owned_strs;
#[cfg(feature = "exts")]
pub use self::owned_strs::{CString16, FromStrError};
