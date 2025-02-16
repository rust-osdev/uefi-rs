// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Char8, IpAddress, MacAddress};
use core::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeArpEntry {
    pub ip_addr: IpAddress,
    pub mac_addr: MacAddress,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeRouteEntry {
    pub ip_addr: IpAddress,
    pub subnet_mask: IpAddress,
    pub gw_addr: IpAddress,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIcmpError {
    pub ty: u8,
    pub code: u8,
    pub checksum: u16,
    pub u: PxeBaseCodeIcmpErrorUnion,
    pub data: [u8; 494],
}

/// In the C API, this is an anonymous union inside the definition of
/// `EFI_PXE_BASE_CODE_ICMP_ERROR`.
#[derive(Clone, Copy)]
#[repr(C)]
pub union PxeBaseCodeIcmpErrorUnion {
    pub reserved: u32,
    pub mtu: u32,
    pub pointer: u32,
    pub echo: PxeBaseCodeIcmpErrorEcho,
}

impl Debug for PxeBaseCodeIcmpErrorUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PxeBaseCodeIcmpErrorUnion").finish()
    }
}

/// In the C API, this is an anonymous struct inside the definition of
/// `EFI_PXE_BASE_CODE_ICMP_ERROR`.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIcmpErrorEcho {
    pub identifier: u16,
    pub sequence: u16,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PxeBaseCodeTftpError {
    pub error_code: u8,
    pub error_string: [Char8; 127],
}
