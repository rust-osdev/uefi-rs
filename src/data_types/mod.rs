//! Data type definitions
//!
//! This module defines the basic data types that are used throughout uefi-rs

use core::{ffi::c_void, mem::MaybeUninit};

/// Opaque handle to an UEFI entity (protocol, image...)
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Handle(*mut c_void);

impl Handle {
    pub(crate) unsafe fn uninitialized() -> Self {
        MaybeUninit::zeroed().assume_init()
    }
}

/// Handle to an event structure
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Event(*mut c_void);

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
