// SPDX-License-Identifier: MIT OR Apache-2.0

//! Network access protocols.
//!
//! These protocols can be used to interact with network resources.

pub mod http;
pub mod ip4config2;
pub mod pxe;
pub mod snp;
pub mod tcpv4;

pub use uefi_raw::MacAddress;

/// Represents an IPv4/v6 address.
///
/// Corresponds to the `EFI_IP_ADDRESS` type in the C API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(C, align(4))]
pub struct IpAddress(pub [u8; 16]);

impl IpAddress {
    /// Construct a new IPv4 address.
    #[must_use]
    pub const fn new_v4(ip_addr: [u8; 4]) -> Self {
        let mut buffer = [0; 16];
        buffer[0] = ip_addr[0];
        buffer[1] = ip_addr[1];
        buffer[2] = ip_addr[2];
        buffer[3] = ip_addr[3];
        Self(buffer)
    }

    /// Construct a new IPv6 address.
    #[must_use]
    pub const fn new_v6(ip_addr: [u8; 16]) -> Self {
        Self(ip_addr)
    }

    /// Construct from a `uefi_raw::IpAddress` union.
    ///
    /// # Safety
    ///
    /// `is_ipv6` must accurately reflect how the union was initialized.
    #[must_use]
    const unsafe fn from_raw(ip_addr: uefi_raw::IpAddress, is_ipv6: bool) -> Self {
        if is_ipv6 {
            Self::new_v6(unsafe { ip_addr.v6.0 })
        } else {
            Self::new_v4(unsafe { ip_addr.v4.0 })
        }
    }

    #[must_use]
    const fn as_raw_ptr(&self) -> *const uefi_raw::IpAddress {
        // The uefi-raw type is defined differently, but the layout is
        // compatible.
        self.0.as_ptr().cast()
    }

    #[must_use]
    const fn as_raw_ptr_mut(&mut self) -> *mut uefi_raw::IpAddress {
        // The uefi-raw type is defined differently, but the layout is
        // compatible.
        self.0.as_mut_ptr().cast()
    }
}

impl From<core::net::Ipv4Addr> for IpAddress {
    fn from(t: core::net::Ipv4Addr) -> Self {
        Self::new_v4(t.octets())
    }
}

impl From<IpAddress> for core::net::Ipv4Addr {
    fn from(IpAddress(o): IpAddress) -> Self {
        Self::from([o[0], o[1], o[2], o[3]])
    }
}

impl From<core::net::Ipv6Addr> for IpAddress {
    fn from(t: core::net::Ipv6Addr) -> Self {
        Self::new_v6(t.octets())
    }
}

impl From<IpAddress> for core::net::Ipv6Addr {
    fn from(value: IpAddress) -> Self {
        Self::from(value.0)
    }
}

impl From<core::net::IpAddr> for IpAddress {
    fn from(t: core::net::IpAddr) -> Self {
        match t {
            core::net::IpAddr::V4(a) => a.into(),
            core::net::IpAddr::V6(a) => a.into(),
        }
    }
}

// NOTE: We cannot impl From<IpAddress> for core::net::IpAddr
// because IpAddress is a raw union, with nothing indicating
// whether it should be considered v4 or v6.
