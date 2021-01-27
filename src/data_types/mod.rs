//! Data type definitions
//!
//! This module defines the basic data types that are used throughout uefi-rs

use core::{ffi::c_void, mem::MaybeUninit};

/// Opaque handle to an UEFI entity (protocol, image...)
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Handle(pub(crate) *mut c_void);

impl Handle {
    pub(crate) unsafe fn uninitialized() -> Self {
        MaybeUninit::zeroed().assume_init()
    }
}

/// Handle to an event structure
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Event(pub(crate) *mut c_void);

mod guid;
pub use self::guid::Guid;
pub use self::guid::{unsafe_guid, Identify};

pub mod chars;
pub use self::chars::{Char16, Char8};

#[macro_use]
mod enums;

mod strs;
pub use self::strs::{CStr16, CStr8};
