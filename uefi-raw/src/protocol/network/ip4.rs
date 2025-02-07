// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Ipv4Address;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(C)]
pub struct Ip4RouteTable {
    pub subnet_addr: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub gateway_addr: Ipv4Address,
}
