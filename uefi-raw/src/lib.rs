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
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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

/// An IPv4 or IPv6 internet protocol address.
///
/// Corresponds to the `EFI_IP_ADDRESS` type in the UEFI specification. This
/// type is defined in the same way as edk2 for compatibility with C code. Note
/// that this is an **untagged union**, so there's no way to tell which type of
/// address an `IpAddress` value contains without additional context.
///
/// For convenience, this type is tightly integrated with the Rust standard
/// library types [`IpAddr`], [`Ipv4Addr`], and [`Ipv6Addr`].
///
/// The constructors ensure that all unused bytes of these type are always
/// initialized to zero.
#[derive(Clone, Copy)]
#[repr(C)]
pub union IpAddress {
    /// An IPv4 internet protocol address.
    pub v4: Ipv4Addr,

    /// An IPv6 internet protocol address.
    pub v6: Ipv6Addr,

    /// This member serves to align the whole type to 4 bytes as required by
    /// the spec. Note that this is slightly different from `repr(align(4))`,
    /// which would prevent placing this type in a packed structure.
    pub _align_helper: [u32; 4],
}

impl IpAddress {
    /// Construct a new IPv4 address.
    #[must_use]
    pub fn new_v4(ip_addr: [u8; 4]) -> Self {
        // Initialize all bytes to zero first.
        let mut obj = Self::default();
        obj.v4 = Ipv4Addr::from(ip_addr);
        obj
    }

    /// Construct a new IPv6 address.
    #[must_use]
    pub fn new_v6(ip_addr: [u8; 16]) -> Self {
        Self {
            v6: Ipv6Addr::from(ip_addr),
        }
    }

    /// Returns the octets of the union. Without additional context, it is not
    /// clear whether the octets represent an IPv4 or IPv6 address.
    #[must_use]
    pub const fn octets(&self) -> [u8; 16] {
        unsafe { self.v6.octets() }
    }

    /// Returns a raw pointer to the IP address.
    #[must_use]
    pub const fn as_ptr(&self) -> *const Self {
        core::ptr::addr_of!(*self)
    }

    /// Returns a raw mutable pointer to the IP address.
    #[must_use]
    pub fn as_ptr_mut(&mut self) -> *mut Self {
        core::ptr::addr_of_mut!(*self)
    }

    /// Transforms this EFI type to the Rust standard libraries type.
    ///
    /// # Arguments
    /// - `is_ipv6`: Whether the internal data should be interpreted as IPv6 or
    ///   IPv4 address.
    #[must_use]
    pub fn to_ip_addr(self, is_ipv6: bool) -> IpAddr {
        if is_ipv6 {
            IpAddr::V6(Ipv6Addr::from(unsafe { self.v6.octets() }))
        } else {
            IpAddr::V4(Ipv4Addr::from(unsafe { self.v4.octets() }))
        }
    }

    /// Returns the underlying data as [`Ipv4Addr`], if only the first four
    /// octets are used.
    ///
    /// # Safety
    /// This function is not unsafe memory-wise but callers need to ensure with
    /// additional context that the IP is indeed an IPv4 address.
    pub unsafe fn as_ipv4(&self) -> Result<Ipv4Addr, Ipv6Addr> {
        let extra = self.octets()[4..].iter().any(|&x| x != 0);
        if !extra {
            Ok(Ipv4Addr::from(unsafe { self.v4.octets() }))
        } else {
            Err(Ipv6Addr::from(unsafe { self.v6.octets() }))
        }
    }

    /// Returns the underlying data as [`Ipv6Addr`].
    ///
    /// # Safety
    /// This function is not unsafe memory-wise but callers need to ensure with
    /// additional context that the IP is indeed an IPv6 address.
    #[must_use]
    pub unsafe fn as_ipv6(&self) -> Ipv6Addr {
        Ipv6Addr::from(unsafe { self.v6.octets() })
    }
}

impl Debug for IpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IpAddress").field(&self.octets()).finish()
    }
}

impl Default for IpAddress {
    fn default() -> Self {
        Self {
            // Initialize all fields to zero
            _align_helper: [0u32; 4],
        }
    }
}

impl From<IpAddr> for IpAddress {
    fn from(t: IpAddr) -> Self {
        match t {
            IpAddr::V4(ip) => Self::new_v4(ip.octets()),
            IpAddr::V6(ip) => Self::new_v6(ip.octets()),
        }
    }
}

impl From<&IpAddr> for IpAddress {
    fn from(t: &IpAddr) -> Self {
        match t {
            IpAddr::V4(ip) => Self::new_v4(ip.octets()),
            IpAddr::V6(ip) => Self::new_v6(ip.octets()),
        }
    }
}

impl From<[u8; 4]> for IpAddress {
    fn from(octets: [u8; 4]) -> Self {
        Self::new_v4(octets)
    }
}

impl From<[u8; 16]> for IpAddress {
    fn from(octets: [u8; 16]) -> Self {
        Self::new_v6(octets)
    }
}

impl From<IpAddress> for [u8; 16] {
    fn from(value: IpAddress) -> Self {
        value.octets()
    }
}

impl From<Ipv4Addr> for IpAddress {
    fn from(value: Ipv4Addr) -> Self {
        Self::new_v4(value.octets())
    }
}

impl From<Ipv6Addr> for IpAddress {
    fn from(value: Ipv6Addr) -> Self {
        Self::new_v6(value.octets())
    }
}

/// UEFI Media Access Control (MAC) address.
///
/// UEFI supports multiple network protocols and hardware types, not just
/// Ethernet. Some of them may use MAC addresses longer than 6 bytes. To be
/// protocol-agnostic and future-proof, the UEFI spec chooses a maximum size
/// that can hold any supported media access control address.
///
/// In most cases, this is just a typical `[u8; 6]` Ethernet style MAC
/// address with the rest of the bytes being zero.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct MacAddress(pub [u8; 32]);

impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        let mut buffer = [0; 32];
        buffer.copy_from_slice(&octets);
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

    /// We test that the core::net-types are ABI compatible with the EFI types.
    /// As long as this is the case, we can reuse core functionality and
    /// prevent type duplication.
    #[test]
    fn net_abi() {
        assert_eq!(size_of::<Ipv4Addr>(), 4);
        assert_eq!(align_of::<Ipv4Addr>(), 1);
        assert_eq!(size_of::<Ipv6Addr>(), 16);
        assert_eq!(align_of::<Ipv6Addr>(), 1);
    }

    #[test]
    fn ip_ptr() {
        let mut ip = IpAddress::new_v4([0; 4]);
        let ptr = ip.as_ptr_mut().cast::<u8>();
        unsafe {
            core::ptr::write(ptr, 192);
            core::ptr::write(ptr.add(1), 168);
            core::ptr::write(ptr.add(2), 42);
            core::ptr::write(ptr.add(3), 73);
        }
        unsafe { assert_eq!(ip.v4.octets(), [192, 168, 42, 73]) }
    }

    /// Test conversion from [`IpAddr`] to [`IpAddress`].
    #[test]
    fn test_ip_addr_conversion() {
        // Reference: std types
        let core_ipv4_v4 = Ipv4Addr::from(TEST_IPV4);
        let core_ipv4 = IpAddr::from(core_ipv4_v4);
        let core_ipv6_v6 = Ipv6Addr::from(TEST_IPV6);
        let core_ipv6 = IpAddr::from(core_ipv6_v6);

        // Test From [u8; N] constructors
        assert_eq!(IpAddress::from(TEST_IPV4).octets()[0..4], TEST_IPV4);
        assert_eq!(IpAddress::from(TEST_IPV6).octets(), TEST_IPV6);
        {
            let bytes: [u8; 16] = IpAddress::from(TEST_IPV6).into();
            assert_eq!(bytes, TEST_IPV6);
        }

        // Test From::from std type constructors
        let efi_ipv4 = IpAddress::from(core_ipv4);
        assert_eq!(efi_ipv4.octets()[0..4], TEST_IPV4);
        assert_eq!(unsafe { efi_ipv4.as_ipv4().unwrap() }, core_ipv4);

        let efi_ipv6 = IpAddress::from(core_ipv6);
        assert_eq!(efi_ipv6.octets(), TEST_IPV6);
        assert_eq!(unsafe { efi_ipv6.as_ipv4().unwrap_err() }, core_ipv6);
        assert_eq!(unsafe { efi_ipv6.as_ipv6() }, core_ipv6);

        // Test From::from std type constructors
        let efi_ipv4 = IpAddress::from(core_ipv4_v4);
        assert_eq!(efi_ipv4.octets()[0..4], TEST_IPV4);
        assert_eq!(unsafe { efi_ipv4.as_ipv4().unwrap() }, core_ipv4);

        let efi_ipv6 = IpAddress::from(core_ipv6_v6);
        assert_eq!(efi_ipv6.octets(), TEST_IPV6);
        assert_eq!(unsafe { efi_ipv6.as_ipv4().unwrap_err() }, core_ipv6);
        assert_eq!(unsafe { efi_ipv6.as_ipv6() }, core_ipv6);
    }
}
