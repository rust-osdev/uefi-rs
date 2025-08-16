// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI network types.
//!
//! The main exports of this module are:
//! - [`MacAddress`]
//! - [`IpAddress`]
//! - [`Ipv4Address`]
//! - [`Ipv6Address`]

use core::fmt;
use core::fmt::{Debug, Formatter};

/// An IPv4 internet protocol address.
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

impl Ipv6Address {
    /// Returns the octets of the IP address.
    #[must_use]
    pub const fn octets(self) -> [u8; 16] {
        self.0
    }
}

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

/// An IPv4 or IPv6 internet protocol address that is ABI compatible with EFI.
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
}

// Normal/typical MAC addresses, such as in Ethernet.
impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        let mut buffer = [0; 32];
        buffer[..6].copy_from_slice(&octets);
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

    // Ensure that our IpAddress type can be put into a packed struct,
    // even when it is normally 4 byte aligned.
    #[test]
    fn test_efi_ip_address_abi() {
        #[repr(C, packed)]
        struct PackedHelper<T>(T);

        assert_eq!(align_of::<IpAddress>(), 4);
        assert_eq!(size_of::<IpAddress>(), 16);

        assert_eq!(align_of::<PackedHelper<IpAddress>>(), 1);
        assert_eq!(size_of::<PackedHelper<IpAddress>>(), 16);
    }
}
