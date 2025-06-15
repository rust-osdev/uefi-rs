// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI network types.
//!
//! The main exports of this module are:
//! - [`MacAddress`]
//! - [`IpAddress`]
//! - [`Ipv4Address`]
//! - [`Ipv6Address`]
//!
//! ## Relation to Rust Standard Library Net Types
//!
//! Some of these types may overlap with those in the Rust standard library
//! ([`core::net`]). To ensure a streamlined API and maintain ABI compatibility,
//! this results in some necessary duplication.
//!
//! All types are tightly integrated with the corresponding [`core::net`] types
//! and other convenient conversions using [`From`] implementations:
//! - `[u8; 4]` -> [`Ipv4Address`], [`IpAddress`]
//! - `[u8; 16]` -> [`Ipv6Address`], [`IpAddress`]
//! - [`core::net::Ipv4Addr`] -> [`Ipv4Address`], [`IpAddress`]
//! - [`core::net::Ipv6Addr`] -> [`Ipv6Address`], [`IpAddress`]
//! - [`core::net::IpAddr`] -> [`IpAddress`]
//! - [`Ipv4Address`] -> [`core::net::Ipv4Addr`]
//! - [`Ipv6Address`] -> [`core::net::Ipv6Addr`]
//!
//! Further, these [`From`] implementations exist:
//! - `[u8; 6]` -> [`MacAddress`]
//! - `[u8; 32]` -> [`MacAddress`]

use core::fmt::{Debug, Formatter};
use core::net::{IpAddr as StdIpAddr, Ipv4Addr as StdIpv4Addr, Ipv6Addr as StdIpv6Addr};
use core::{fmt, mem};

/// An IPv4 internet protocol address.
///
/// See the [module documentation] to get an overview over the relation to the
/// types from [`core::net`].
///
/// [module documentation]: self
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    /// Returns the octets of the IP address.
    #[must_use]
    pub const fn octets(self) -> [u8; 4] {
        self.0
    }
}

impl From<StdIpv4Addr> for Ipv4Address {
    fn from(ip: StdIpv4Addr) -> Self {
        Self(ip.octets())
    }
}

impl From<Ipv4Address> for StdIpv4Addr {
    fn from(ip: Ipv4Address) -> Self {
        Self::from(ip.0)
    }
}

/// An IPv6 internet protocol address.
///
/// See the [module documentation] to get an overview over the relation to the
/// types from [`core::net`].
///
/// [module documentation]: self
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Ipv6Address(pub [u8; 16]);

impl Ipv6Address {
    /// Returns the octets of the IP address.
    #[must_use]
    pub const fn octets(self) -> [u8; 16] {
        self.0
    }
}

impl From<StdIpv6Addr> for Ipv6Address {
    fn from(ip: StdIpv6Addr) -> Self {
        Self(ip.octets())
    }
}

impl From<Ipv6Address> for StdIpv6Addr {
    fn from(ip: Ipv6Address) -> Self {
        Self::from(ip.0)
    }
}

/// EFI ABI-compatible union of an IPv4 or IPv6 internet protocol address.
///
/// Corresponds to the `EFI_IP_ADDRESS` type in the UEFI specification. This
/// type is defined in the same way as edk2 for compatibility with C code. Note
/// that this is an **untagged union**, so there's no way to tell which type of
/// address an `IpAddress` value contains without additional context.
///
/// See the [module documentation] to get an overview over the relation to the
/// types from [`core::net`].
///
/// [module documentation]: self
#[derive(Clone, Copy)]
#[repr(C)]
pub union IpAddress {
    /// An IPv4 internet protocol address.
    pub v4: Ipv4Address,

    /// An IPv6 internet protocol address.
    pub v6: Ipv6Address,

    /// This member serves to align the whole type to 4 bytes as required by
    /// the spec. Note that this is slightly different from `repr(align(4))`,
    /// which would prevent placing this type in a packed structure.
    align_helper: [u32; 4],
}

impl IpAddress {
    /// Construct a new zeroed address.
    #[must_use]
    pub const fn new_zeroed() -> Self {
        // SAFETY: All bit patterns are valid.
        unsafe { mem::zeroed() }
    }

    /// Construct a new IPv4 address.
    ///
    /// The type won't know that it is an IPv6 address and additional context
    /// is needed.
    #[must_use]
    pub const fn new_v4(octets: [u8; 4]) -> Self {
        // Initialize all bytes to zero first.
        let mut obj = Self::new_zeroed();
        obj.v4 = Ipv4Address(octets);
        obj
    }

    /// Construct a new IPv6 address.
    ///
    /// The type won't know that it is an IPv6 address and additional context
    /// is needed.
    #[must_use]
    pub const fn new_v6(octets: [u8; 16]) -> Self {
        // Initialize all bytes to zero first.
        let mut obj = Self::new_zeroed();
        obj.v6 = Ipv6Address(octets);
        obj
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
    pub const fn as_ptr_mut(&mut self) -> *mut Self {
        core::ptr::addr_of_mut!(*self)
    }

    /// Transforms this EFI type to the Rust standard library's type.
    ///
    /// # Arguments
    /// - `is_ipv6`: Whether the internal data should be interpreted as IPv6 or
    ///   IPv4 address.
    ///
    /// # Panics
    /// Panics if `is_ipv6` is false but there are additional bytes
    /// indicating it's a IPv6 address.
    #[must_use]
    pub fn to_std_ip_addr(self, is_ipv6: bool) -> StdIpAddr {
        if is_ipv6 {
            StdIpAddr::V6(StdIpv6Addr::from(unsafe { self.v6.octets() }))
        } else {
            let has_extra_bytes = self.octets()[4..].iter().any(|&x| x != 0);
            assert!(!has_extra_bytes);
            StdIpAddr::V4(StdIpv4Addr::from(unsafe { self.v4.octets() }))
        }
    }

    /// Returns the underlying data as [`Ipv4Address`], if only the first four
    /// octets are used.
    ///
    /// # Safety
    /// This function is not unsafe memory-wise but callers need to ensure with
    /// additional context that the IP is indeed an IPv4 address.
    pub unsafe fn try_as_ipv4(&self) -> Result<Ipv4Address, Ipv6Address> {
        let extra = self.octets()[4..].iter().any(|&x| x != 0);
        if !extra {
            let octets: [u8; 4] = self.octets()[..4].try_into().unwrap();
            Ok(Ipv4Address(octets))
        } else {
            Err(Ipv6Address(self.octets()))
        }
    }

    /// Returns the underlying data as [`Ipv6Address`].
    ///
    /// # Safety
    /// This function is not unsafe memory-wise but callers need to ensure with
    /// additional context that the IP is indeed an IPv6 address.
    #[must_use]
    pub unsafe fn as_ipv6(&self) -> Ipv6Address {
        Ipv6Address::from(unsafe { self.v6.octets() })
    }
}

impl Debug for IpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpAddress")
            // SAFETY: All constructors ensure that all bytes are always
            // initialized.
            .field("v4", &unsafe { self.v4 })
            // SAFETY: All constructors ensure that all bytes are always
            // initialized.
            .field("v6", &unsafe { self.v6 })
            .finish()
    }
}

impl Default for IpAddress {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

impl From<StdIpAddr> for IpAddress {
    fn from(t: StdIpAddr) -> Self {
        match t {
            StdIpAddr::V4(ip) => Self::new_v4(ip.octets()),
            StdIpAddr::V6(ip) => Self::new_v6(ip.octets()),
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

impl From<[u8; 4]> for Ipv4Address {
    fn from(octets: [u8; 4]) -> Self {
        Self(octets)
    }
}

impl From<[u8; 16]> for Ipv6Address {
    fn from(octets: [u8; 16]) -> Self {
        Self(octets)
    }
}

impl From<StdIpv4Addr> for IpAddress {
    fn from(value: StdIpv4Addr) -> Self {
        Self::new_v4(value.octets())
    }
}

impl From<StdIpv6Addr> for IpAddress {
    fn from(value: StdIpv6Addr) -> Self {
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

impl MacAddress {
    /// Returns the octets of the MAC address.
    #[must_use]
    pub const fn octets(self) -> [u8; 32] {
        self.0
    }

    /// Tries to interpret the MAC address as normal 6-byte MAC address, as used
    /// in ethernet.
    pub fn try_as_ethernet_mac_addr(self) -> Result<[u8; 6], [u8; 32]> {
        let extra = self.octets()[4..].iter().any(|&x| x != 0);
        if extra {
            Err(self.0)
        } else {
            Ok(self.octets()[..4].try_into().unwrap())
        }
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        let mut buffer = [0; 32];
        buffer[..6].copy_from_slice(&octets);
        Self(buffer)
    }
}

impl From<[u8; 32]> for MacAddress {
    fn from(octets: [u8; 32]) -> Self {
        Self(octets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_IPV4: [u8; 4] = [91, 92, 93, 94];
    const TEST_IPV6: [u8; 16] = [
        101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
    ];

    /// Test round-trip conversion between `Ipv4Address` and `StdIpv4Addr`.
    #[test]
    fn test_ip_addr4_conversion() {
        let uefi_addr = Ipv4Address(TEST_IPV4);
        let core_addr = StdIpv4Addr::from(uefi_addr);
        assert_eq!(uefi_addr, Ipv4Address::from(core_addr));
    }

    /// Test round-trip conversion between [`Ipv6Address`] and [`StdIpv6Addr`].
    #[test]
    fn test_ip_addr6_conversion() {
        let uefi_addr = Ipv6Address(TEST_IPV6);
        let core_addr = StdIpv6Addr::from(uefi_addr);
        assert_eq!(uefi_addr, Ipv6Address::from(core_addr));
    }

    /// Test conversion from [`StdIpAddr`] to [`IpvAddress`].
    ///
    /// Note that conversion in the other direction is not possible.
    #[test]
    fn test_ip_addr_conversion() {
        let core_addr = StdIpAddr::V4(StdIpv4Addr::from(TEST_IPV4));
        let uefi_addr = IpAddress::from(core_addr);
        assert_eq!(unsafe { uefi_addr.v4.0 }, TEST_IPV4);

        let core_addr = StdIpAddr::V6(StdIpv6Addr::from(TEST_IPV6));
        let uefi_addr = IpAddress::from(core_addr);
        assert_eq!(unsafe { uefi_addr.v6.0 }, TEST_IPV6);
    }

    /// Tests the From-impls as described in the module description.
    #[test]
    fn test_module_description_from_impls() {
        {
            let octets = [0_u8, 1, 2, 3];
            assert_eq!(Ipv4Address::from(octets), Ipv4Address(octets));
            let uefi_addr = IpAddress::from(octets);
            assert_eq!(&octets, &uefi_addr.octets()[0..4]);
        }
        {
            let octets = [0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
            assert_eq!(Ipv6Address::from(octets), Ipv6Address(octets));
            let uefi_addr = IpAddress::from(octets);
            assert_eq!(&octets, &uefi_addr.octets());
        }
        {
            let octets = [7, 5, 3, 1];
            let core_ipv4_addr = StdIpv4Addr::from(octets);
            assert_eq!(Ipv4Address::from(core_ipv4_addr).octets(), octets);
            assert_eq!(IpAddress::from(core_ipv4_addr).octets()[0..4], octets);
        }
        {
            let octets = [7, 5, 3, 1, 6, 3, 8, 5, 2, 5, 2, 7, 3, 5, 2, 6];
            let core_ipv6_addr = StdIpv6Addr::from(octets);
            assert_eq!(Ipv6Address::from(core_ipv6_addr).octets(), octets);
            assert_eq!(IpAddress::from(core_ipv6_addr).octets(), octets);
        }
        {
            let octets = [8, 8, 2, 6];
            let core_ip_addr = StdIpAddr::from(octets);
            assert_eq!(IpAddress::from(core_ip_addr).octets()[0..4], octets);
        }
        {
            let octets = [8, 8, 2, 6, 6, 7];
            let uefi_mac_addr = MacAddress::from(octets);
            assert_eq!(uefi_mac_addr.octets()[0..6], octets);
        }
        {
            let octets = [
                8_u8, 8, 2, 6, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 7, 0, 0, 0,
                0, 0, 0, 0, 42,
            ];
            let uefi_mac_addr = MacAddress::from(octets);
            assert_eq!(uefi_mac_addr.octets(), octets);
        }
    }

    /// Tests that all bytes are initialized and that the Debug print doesn't
    /// produce errors, when Miri executes this.
    #[test]
    fn test_ip_address_debug_memory_safe() {
        let uefi_addr = IpAddress::new_v6(TEST_IPV6);
        std::eprintln!("{uefi_addr:#?}");
    }

    /// Tests the expected flow of types in a higher-level UEFI API.
    #[test]
    fn test_uefi_flow() {
        fn efi_retrieve_efi_ip_addr(addr: &mut IpAddress, is_ipv6: bool) {
            // SAFETY: Alignment is guaranteed and memory is initialized.
            unsafe {
                addr.v4.0[0] = 42;
                addr.v4.0[1] = 42;
                addr.v4.0[2] = 42;
                addr.v4.0[3] = 42;
            }
            if is_ipv6 {
                unsafe {
                    addr.v6.0[14] = 42;
                    addr.v6.0[15] = 42;
                }
            }
        }

        fn high_level_retrieve_ip(is_ipv6: bool) -> StdIpAddr {
            let mut efi_ip_addr = IpAddress::new_zeroed();
            efi_retrieve_efi_ip_addr(&mut efi_ip_addr, is_ipv6);
            efi_ip_addr.to_std_ip_addr(is_ipv6)
        }

        let ipv4_addr = high_level_retrieve_ip(false);
        let ipv4_addr: StdIpv4Addr = match ipv4_addr {
            StdIpAddr::V4(ipv4_addr) => ipv4_addr,
            StdIpAddr::V6(_) => panic!("should not happen"),
        };
        assert_eq!(ipv4_addr.octets(), [42, 42, 42, 42]);

        let ipv6_addr = high_level_retrieve_ip(true);
        let ipv6_addr: StdIpv6Addr = match ipv6_addr {
            StdIpAddr::V6(ipv6_addr) => ipv6_addr,
            StdIpAddr::V4(_) => panic!("should not happen"),
        };
        let expected = [42, 42, 42, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 42];
        assert_eq!(ipv6_addr.octets(), expected);
    }
}
