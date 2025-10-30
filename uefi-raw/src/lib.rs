// SPDX-License-Identifier: MIT OR Apache-2.0

//! Raw interface for working with UEFI.
//!
//! This crate is intended for implementing UEFI services. It is also used for
//! implementing the [`uefi`] crate, which provides a safe wrapper around UEFI.
//!
//! For creating UEFI applications and drivers, consider using the [`uefi`]
//! crate instead of `uefi-raw`.
//!
//! [`uefi`]: https://crates.io/crates/uefi

#![no_std]
#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::ptr_as_ptr,
    clippy::use_self,
    missing_debug_implementations,
    unused
)]

mod enums;

pub mod capsule;
pub mod firmware_storage;
pub mod protocol;
pub mod table;
pub mod time;

mod net;
mod status;

pub use net::*;
pub use status::Status;
pub use uguid::{Guid, guid};

use core::ffi::c_void;

/// Handle to an event structure.
pub type Event = *mut c_void;

/// Handle to a UEFI entity (protocol, image, etc).
pub type Handle = *mut c_void;

/// One-byte character.
///
/// Most strings in UEFI use [`Char16`], but a few places use one-byte
/// characters. Unless otherwise noted, these are encoded as 8-bit ASCII using
/// the ISO-Latin-1 character set.
pub type Char8 = u8;

/// Two-byte character.
///
/// Unless otherwise noted, the encoding is UCS-2. The UCS-2 encoding was
/// defined by Unicode 2.1 and ISO/IEC 10646 standards, but is no longer part of
/// the modern Unicode standards. It is essentially UTF-16 without support for
/// surrogate pairs.
pub type Char16 = u16;

/// Physical memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type PhysicalAddress = u64;

/// Virtual memory address. This is always a 64-bit value, regardless
/// of target platform.
pub type VirtualAddress = u64;

/// ABI-compatible UEFI boolean.
///
/// This is similar to a `bool`, but allows values other than 0 or 1 to be
/// stored without it being undefined behavior.
///
/// Any non-zero value is treated as logically `true`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Ord, PartialOrd, Eq, Hash)]
#[repr(transparent)]
pub struct Boolean(pub u8);

impl Boolean {
    /// [`Boolean`] representing `true`.
    pub const TRUE: Self = Self(1);

    /// [`Boolean`] representing `false`.
    pub const FALSE: Self = Self(0);
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        match value {
            true => Self(1),
            false => Self(0),
        }
    }
}

impl From<Boolean> for bool {
    #[allow(clippy::match_like_matches_macro)]
    fn from(value: Boolean) -> Self {
        // We handle it as in C: Any bit-pattern != 0 equals true
        match value.0 {
            0 => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test the properties promised in [0]. This also applies for the other
    /// architectures.
    ///
    /// [0] https://github.com/tianocore/edk2/blob/b0f43dd3fdec2363e3548ec31eb455dc1c4ac761/MdePkg/Include/X64/ProcessorBind.h#L192
    fn test_boolean_abi() {
        assert_eq!(size_of::<Boolean>(), 1);
        assert_eq!(Boolean::from(true).0, 1);
        assert_eq!(Boolean::from(false).0, 0);
        assert_eq!(Boolean::TRUE.0, 1);
        assert_eq!(Boolean::FALSE.0, 0);
        assert!(!bool::from(Boolean(0b0)));
        assert!(bool::from(Boolean(0b1)));
        // We do it as in C: Every bit pattern not 0 is equal to true.
        assert!(bool::from(Boolean(0b11111110)));
        assert!(bool::from(Boolean(0b11111111)));
    }
}
