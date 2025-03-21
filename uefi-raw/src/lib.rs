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

pub use status::Status;
pub use uguid::{guid, Guid};

use core::ffi::c_void;
use core::fmt::{self, Debug, Formatter};

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

/// An IPv4 internet protocol address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

impl From<core::net::Ipv4Addr> for Ipv4Address {
    fn from(ip: core::net::Ipv4Addr) -> Self {
        Self(ip.octets())
    }
}

impl From<Ipv4Address> for core::net::Ipv4Addr {
    fn from(ip: Ipv4Address) -> Self {
        Self::from(ip.0)
    }
}

/// An IPv6 internet protocol address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv6Address(pub [u8; 16]);

impl From<core::net::Ipv6Addr> for Ipv6Address {
    fn from(ip: core::net::Ipv6Addr) -> Self {
        Self(ip.octets())
    }
}

impl From<Ipv6Address> for core::net::Ipv6Addr {
    fn from(ip: Ipv6Address) -> Self {
        Self::from(ip.0)
    }
}

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

impl From<core::net::IpAddr> for IpAddress {
    fn from(t: core::net::IpAddr) -> Self {
        match t {
            core::net::IpAddr::V4(ip) => Self {
                v4: Ipv4Address::from(ip),
            },
            core::net::IpAddr::V6(ip) => Self {
                v6: Ipv6Address::from(ip),
            },
        }
    }
}

/// A Media Access Control (MAC) address.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 32]);

impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        let mut buffer = [0; 32];
        buffer[0] = octets[0];
        buffer[1] = octets[1];
        buffer[2] = octets[2];
        buffer[3] = octets[3];
        buffer[4] = octets[4];
        buffer[5] = octets[5];
        Self(buffer)
    }
}

impl From<MacAddress> for [u8; 6] {
    fn from(MacAddress(o): MacAddress) -> Self {
        [o[0], o[1], o[2], o[3], o[4], o[5]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_IPV4: [u8; 4] = [91, 92, 93, 94];
    const TEST_IPV6: [u8; 16] = [
        101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    ];

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

    /// Test round-trip conversion between `Ipv4Address` and `core::net::Ipv4Addr`.
    #[test]
    fn test_ip_addr4_conversion() {
        let uefi_addr = Ipv4Address(TEST_IPV4);
        let core_addr = core::net::Ipv4Addr::from(uefi_addr);
        assert_eq!(uefi_addr, Ipv4Address::from(core_addr));
    }

    /// Test round-trip conversion between `Ipv6Address` and `core::net::Ipv6Addr`.
    #[test]
    fn test_ip_addr6_conversion() {
        let uefi_addr = Ipv6Address(TEST_IPV6);
        let core_addr = core::net::Ipv6Addr::from(uefi_addr);
        assert_eq!(uefi_addr, Ipv6Address::from(core_addr));
    }

    /// Test conversion from `core::net::IpAddr` to `IpvAddress`.
    ///
    /// Note that conversion in the other direction is not possible.
    #[test]
    fn test_ip_addr_conversion() {
        let core_addr = core::net::IpAddr::V4(core::net::Ipv4Addr::from(TEST_IPV4));
        let uefi_addr = IpAddress::from(core_addr);
        assert_eq!(unsafe { uefi_addr.v4.0 }, TEST_IPV4);

        let core_addr = core::net::IpAddr::V6(core::net::Ipv6Addr::from(TEST_IPV6));
        let uefi_addr = IpAddress::from(core_addr);
        assert_eq!(unsafe { uefi_addr.v6.0 }, TEST_IPV6);
    }
}
