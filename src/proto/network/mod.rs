//! Network access protocols.
//!
//! These protocols can be used to interact with network resources.

pub mod pxe;

/// EFI_IP_ADDRESS
#[derive(Clone, Copy)]
#[repr(C, align(4))]
pub struct IpAddress(pub [u8; 16]);

impl IpAddress {
    /// Construct a new IPv4 address.
    pub const fn new_v4(ip_addr: [u8; 4]) -> Self {
        let mut buffer = [0; 16];
        buffer[0] = ip_addr[0];
        buffer[1] = ip_addr[1];
        buffer[2] = ip_addr[2];
        buffer[3] = ip_addr[3];
        Self(buffer)
    }

    /// Construct a new IPv6 address.
    pub const fn new_v6(ip_addr: [u8; 16]) -> Self {
        Self(ip_addr)
    }
}

/// EFI_MAC_ADDRESS
#[derive(Clone, Copy)]
#[repr(C)]
pub struct MacAddress(pub [u8; 32]);
