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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::missing_safety_doc,
    clippy::must_use_candidate,
    clippy::ptr_as_ptr,
    clippy::undocumented_unsafe_blocks,
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

use core::cmp;
pub use net::*;
pub use status::Status;
pub use uguid::{Guid, guid};

use core::ffi::c_void;
use core::hash::{Hash, Hasher};

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
/// Any non-zero value is treated as logical `true`. The comparison, ordering,
/// and hashing implementations follow that logical interpretation, so all
/// non-zero values compare equal and hash the same way. The original byte is
/// still preserved in the public field; compare the `.0` values directly when
/// the exact raw bit pattern matters.
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Boolean(pub u8);

impl Boolean {
    /// [`Boolean`] representing `true`.
    ///
    /// # Caution
    ///
    /// This is only one possible true bit pattern. In UEFI, every non-zero
    /// bit pattern is treated as logical `true`.
    pub const TRUE: Self = Self(1);

    /// [`Boolean`] representing `false`.
    pub const FALSE: Self = Self(0);
}

impl From<u8> for Boolean {
    fn from(value: u8) -> Self {
        Self(value)
    }
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
    #[expect(clippy::match_like_matches_macro)]
    fn from(value: Boolean) -> Self {
        // We handle it as in C: Any bit-pattern != 0 equals true
        match value.0 {
            0 => false,
            _ => true,
        }
    }
}

impl PartialEq for Boolean {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (0, 0) => true,
            (0, _) => false,
            (_, 0) => false,
            // We handle it as in C: Any bit-pattern != 0 equals true
            (_, _) => true,
        }
    }
}

impl Eq for Boolean {}

impl PartialOrd for Boolean {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Boolean {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self.0, other.0) {
            (0, 0) => cmp::Ordering::Equal,
            (0, _) => cmp::Ordering::Less,
            (_, 0) => cmp::Ordering::Greater,
            // We handle it as in C: Any bit-pattern != 0 equals true
            (_, _) => cmp::Ordering::Equal,
        }
    }
}

impl Hash for Boolean {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let seed = if self.0 == 0 { 0 } else { 1 };
        state.write_u8(seed);
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

    #[test]
    fn test_order() {
        assert!(Boolean::FALSE < Boolean::TRUE);
        assert!(Boolean::TRUE > Boolean::FALSE);
    }

    #[test]
    fn test_equality() {
        assert_eq!(Boolean(0), Boolean(0));
        assert_ne!(Boolean(0), Boolean(3));
        assert_ne!(Boolean(7), Boolean(0));
        assert_eq!(Boolean(1), Boolean(1));
        assert_eq!(Boolean(13), Boolean(7));
    }

    // Tests that hash impl matches equal impl
    #[test]
    fn test_hash() {
        #[derive(Default)]
        struct TestHasher(u64);

        impl Hasher for TestHasher {
            fn finish(&self) -> u64 {
                self.0
            }

            fn write(&mut self, bytes: &[u8]) {
                for byte in bytes {
                    self.0 = self.0.wrapping_mul(257).wrapping_add(u64::from(*byte));
                }
            }
        }

        fn calculate_hash(value: Boolean) -> u64 {
            let mut hasher = TestHasher::default();
            value.hash(&mut hasher);
            hasher.finish()
        }

        assert_eq!(calculate_hash(Boolean(1)), calculate_hash(Boolean(13)));
        assert_eq!(calculate_hash(Boolean(13)), calculate_hash(Boolean(255)));
        assert_ne!(calculate_hash(Boolean(0)), calculate_hash(Boolean(13)));
    }
}
