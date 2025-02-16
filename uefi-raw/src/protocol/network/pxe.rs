// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Char8, IpAddress, MacAddress};
use bitflags::bitflags;
use core::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PxeBaseCodeIpFilter {
    pub filters: PxeBaseCodeIpFilterFlags,
    pub ip_cnt: u8,
    pub reserved: u16,
    pub ip_list: [IpAddress; 8],
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct PxeBaseCodeIpFilterFlags: u8 {
        const STATION_IP = 0x01;
        const BROADCAST = 0x02;
        const PROMISCUOUS = 0x04;
        const PROMISCUOUS_MULTICAST = 0x08;
    }
}

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
