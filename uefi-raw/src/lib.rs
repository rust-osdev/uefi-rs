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
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(
    clippy::all,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::ptr_as_ptr,
    clippy::use_self,
    missing_debug_implementations,
    unused
)]

#[macro_use]
mod enums;

pub mod capsule;
pub mod firmware_storage;
pub mod protocol;
pub mod table;
pub mod time;

mod status;

use core::ffi::c_void;
use core::fmt::{self, Debug, Formatter};
pub use status::Status;
pub use uguid::{guid, Guid};

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

/// An IPv4 internet protocol address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

/// An IPv6 internet protocol address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv6Address(pub [u8; 16]);

/// An IPv4 or IPv6 internet protocol address.
///
/// Corresponds to the `EFI_IP_ADDRESS` type in the UEFI specification. This
/// type is defined in the same way as edk2 for compatibility with C code. Note
/// that this is an untagged union, so there's no way to tell which type of
/// address an `IpAddress` value contains without additional context.
#[derive(Clone, Copy)]
#[repr(C)]
pub union IpAddress {
    /// This member serves to align the whole type to a 4 bytes as required by
    /// the spec. Note that this is slightly different from `repr(align(4))`,
    /// which would prevent placing this type in a packed structure.
    pub addr: [u32; 4],

    /// An IPv4 internet protocol address.
    pub v4: Ipv4Address,

    /// An IPv6 internet protocol address.
    pub v6: Ipv6Address,
}

impl IpAddress {
    /// Construct a new IPv4 address.
    #[must_use]
    pub const fn new_v4(ip_addr: [u8; 4]) -> Self {
        Self {
            v4: Ipv4Address(ip_addr),
        }
    }

    /// Construct a new IPv6 address.
    #[must_use]
    pub const fn new_v6(ip_addr: [u8; 16]) -> Self {
        Self {
            v6: Ipv6Address(ip_addr),
        }
    }
}

impl Debug for IpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // The type is an untagged union, so we don't know whether it contains
        // an IPv4 or IPv6 address. It's also not safe to just print the whole
        // 16 bytes, since they might not all be initialized.
        f.debug_struct("IpAddress").finish()
    }
}

impl Default for IpAddress {
    fn default() -> Self {
        Self { addr: [0u32; 4] }
    }
}

/// A Media Access Control (MAC) address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 32]);
