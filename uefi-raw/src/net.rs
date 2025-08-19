// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI network types.
//!
//! The main exports of this module are:
//! - [`MacAddress`]
//! - [`IpAddress`]
//! - [`Ipv4Address`]
//! - [`Ipv6Address`]

use core::fmt::{self, Debug, Formatter};

/// An IPv4 internet protocol address.
///
/// # Conversions and Relation to [`core::net`]
///
/// The following [`From`] implementations exist:
///   - `[u8; 4]` -> [`Ipv4Address`]
///   - [`core::net::Ipv4Addr`] -> [`Ipv4Address`]
///   - [`core::net::IpAddr`] -> [`Ipv4Address`]
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

impl From<[u8; 4]> for Ipv4Address {
    fn from(octets: [u8; 4]) -> Self {
        Self(octets)
    }
}

/// An IPv6 internet protocol address.
///
/// # Conversions and Relation to [`core::net`]
///
/// The following [`From`] implementations exist:
///   - `[u8; 16]` -> [`Ipv6Address`]
///   - [`core::net::Ipv6Addr`] -> [`Ipv6Address`]
///   - [`core::net::IpAddr`] -> [`Ipv6Address`]
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

impl From<[u8; 16]> for Ipv6Address {
    fn from(octets: [u8; 16]) -> Self {
        Self(octets)
    }
}

/// An IPv4 or IPv6 internet protocol address that is ABI compatible with EFI.
///
/// Corresponds to the `EFI_IP_ADDRESS` type in the UEFI specification. This
/// type is defined in the same way as edk2 for compatibility with C code. Note
/// that this is an untagged union, so there's no way to tell which type of
/// address an `IpAddress` value contains without additional context.
///
/// # Conversions and Relation to [`core::net`]
///
/// The following [`From`] implementations exist:
///   - `[u8; 4]` -> [`IpAddress`]
///   - `[u8; 16]` -> [`IpAddress`]
///   - [`core::net::Ipv4Addr`] -> [`IpAddress`]
///   - [`core::net::Ipv6Addr`] -> [`IpAddress`]
///   - [`core::net::IpAddr`] -> [`IpAddress`]
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
    /// Zeroed variant where all bytes are guaranteed to be initialized to zero.
    pub const ZERO: Self = Self { addr: [0; 4] };

    /// Construct a new IPv4 address.
    ///
    /// The type won't know that it is an IPv6 address and additional context
    /// is needed.
    ///
    /// # Safety
    /// The constructor only initializes the bytes needed for IPv4 addresses.
    #[must_use]
    pub const fn new_v4(octets: [u8; 4]) -> Self {
        Self {
            v4: Ipv4Address(octets),
        }
    }

    /// Construct a new IPv6 address.
    ///
    /// The type won't know that it is an IPv6 address and additional context
    /// is needed.
    #[must_use]
    pub const fn new_v6(octets: [u8; 16]) -> Self {
        Self {
            v6: Ipv6Address(octets),
        }
    }

    /// Transforms this EFI type to the Rust standard library's type
    /// [`core::net::IpAddr`].
    ///
    /// # Arguments
    /// - `is_ipv6`: Whether the internal data should be interpreted as IPv6 or
    ///   IPv4 address.
    ///
    /// # Safety
    /// Callers must ensure that the `v4` field is valid if `is_ipv6` is false,
    /// and that the `v6` field is valid if `is_ipv6` is true
    #[must_use]
    pub unsafe fn into_core_ip_addr(self, is_ipv6: bool) -> core::net::IpAddr {
        if is_ipv6 {
            // SAFETY: Caller assumes that the underlying data is initialized.
            core::net::IpAddr::V6(core::net::Ipv6Addr::from(unsafe { self.v6.octets() }))
        } else {
            // SAFETY: Caller assumes that the underlying data is initialized.
            core::net::IpAddr::V4(core::net::Ipv4Addr::from(unsafe { self.v4.octets() }))
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
        Self::ZERO
    }
}

impl From<core::net::IpAddr> for IpAddress {
    fn from(t: core::net::IpAddr) -> Self {
        match t {
            core::net::IpAddr::V4(ip) => Self::new_v4(ip.octets()),
            core::net::IpAddr::V6(ip) => Self::new_v6(ip.octets()),
        }
    }
}

impl From<core::net::Ipv4Addr> for IpAddress {
    fn from(value: core::net::Ipv4Addr) -> Self {
        Self::new_v4(value.octets())
    }
}

impl From<core::net::Ipv6Addr> for IpAddress {
    fn from(value: core::net::Ipv6Addr) -> Self {
        Self::new_v6(value.octets())
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

/// UEFI Media Access Control (MAC) address.
///
/// UEFI supports multiple network protocols and hardware types, not just
/// Ethernet. Some of them may use MAC addresses longer than 6 bytes. To be
/// protocol-agnostic and future-proof, the UEFI spec chooses a maximum size
/// that can hold any supported media access control address.
///
/// In most cases, this is just a typical `[u8; 6]` Ethernet style MAC
/// address with the rest of the bytes being zero.
///
/// # Conversions and Relation to [`core::net`]
///
/// There is no matching type in [`core::net`] but the following [`From`]
/// implementations exist:
///   - `[u8; 6]` -> [`MacAddress`]
///   - `[u8; 32]` -> [`MacAddress`]
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
    pub fn try_into_ethernet_mac_addr(self) -> Result<[u8; 6], [u8; 32]> {
        let extra = self.octets()[4..].iter().any(|&x| x != 0);
        if extra {
            Err(self.0)
        } else {
            Ok(self.octets()[..4].try_into().unwrap())
        }
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

// UEFI MAC addresses.
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

    /// Tests the From-impls from the documentation.
    #[test]
    fn test_promised_from_impls() {
        // octets -> Ipv4Address
        {
            let octets = [0_u8, 1, 2, 3];
            assert_eq!(Ipv4Address::from(octets), Ipv4Address(octets));
            let uefi_addr = IpAddress::from(octets);
            assert_eq!(&octets, &unsafe { uefi_addr.v4.octets() });
        }
        // octets -> Ipv6Address
        {
            let octets = [0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
            assert_eq!(Ipv6Address::from(octets), Ipv6Address(octets));
            let uefi_addr = IpAddress::from(octets);
            assert_eq!(&octets, &unsafe { uefi_addr.v6.octets() });
        }
        // StdIpv4Addr -> Ipv4Address
        {
            let octets = [7, 5, 3, 1];
            let core_ipv4_addr = core::net::Ipv4Addr::from(octets);
            assert_eq!(Ipv4Address::from(core_ipv4_addr).octets(), octets);
            assert_eq!(
                unsafe { IpAddress::from(core_ipv4_addr).v4.octets() },
                octets
            );
        }
        // StdIpv6Addr -> Ipv6Address
        {
            let octets = [7, 5, 3, 1, 6, 3, 8, 5, 2, 5, 2, 7, 3, 5, 2, 6];
            let core_ipv6_addr = core::net::Ipv6Addr::from(octets);
            assert_eq!(Ipv6Address::from(core_ipv6_addr).octets(), octets);
            assert_eq!(
                unsafe { IpAddress::from(core_ipv6_addr).v6.octets() },
                octets
            );
        }
        // StdIpAddr -> IpAddress
        {
            let octets = [8, 8, 2, 6];
            let core_ip_addr = core::net::IpAddr::from(octets);
            assert_eq!(unsafe { IpAddress::from(core_ip_addr).v4.octets() }, octets);
        }
        // octets -> MacAddress
        {
            let octets = [8, 8, 2, 6, 6, 7];
            let uefi_mac_addr = MacAddress::from(octets);
            assert_eq!(uefi_mac_addr.octets()[0..6], octets);
        }
        // octets -> MacAddress
        {
            let octets = [
                8_u8, 8, 2, 6, 6, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 7, 0, 0, 0,
                0, 0, 0, 0, 42,
            ];
            let uefi_mac_addr = MacAddress::from(octets);
            assert_eq!(uefi_mac_addr.octets(), octets);
        }
    }

    /// Tests the expected flow of types in a higher-level UEFI API.
    #[test]
    fn test_uefi_flow() {
        fn efi_retrieve_efi_ip_addr(addr: *mut IpAddress, is_ipv6: bool) {
            let addr = unsafe { addr.as_mut().unwrap() };
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

        fn high_level_retrieve_ip(is_ipv6: bool) -> core::net::IpAddr {
            let mut efi_ip_addr = IpAddress::ZERO;
            efi_retrieve_efi_ip_addr(&mut efi_ip_addr, is_ipv6);
            unsafe { efi_ip_addr.into_core_ip_addr(is_ipv6) }
        }

        let ipv4_addr = high_level_retrieve_ip(false);
        let ipv4_addr: core::net::Ipv4Addr = match ipv4_addr {
            core::net::IpAddr::V4(ipv4_addr) => ipv4_addr,
            core::net::IpAddr::V6(_) => panic!("should not happen"),
        };
        assert_eq!(ipv4_addr.octets(), [42, 42, 42, 42]);

        let ipv6_addr = high_level_retrieve_ip(true);
        let ipv6_addr: core::net::Ipv6Addr = match ipv6_addr {
            core::net::IpAddr::V6(ipv6_addr) => ipv6_addr,
            core::net::IpAddr::V4(_) => panic!("should not happen"),
        };
        let expected = [42, 42, 42, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 42];
        assert_eq!(ipv6_addr.octets(), expected);
    }
}
