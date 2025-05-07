// SPDX-License-Identifier: MIT OR Apache-2.0

use core::net::Ipv4Addr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct Ip4RouteTable {
    pub subnet_addr: Ipv4Addr,
    pub subnet_mask: Ipv4Addr,
    pub gateway_addr: Ipv4Addr,
}
