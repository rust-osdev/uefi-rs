// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::protocol::network::ip4::Ip4RouteTable;
use crate::{guid, Char16, Event, Guid, Ipv4Address, MacAddress, Status};
use core::ffi::c_void;

newtype_enum! {
    pub enum Ip4Config2DataType: i32 => {
        INTERFACE_INFO = 0,
        POLICY         = 1,
        MANUAL_ADDRESS = 2,
        GATEWAY        = 3,
        DNS_SERVER     = 4,
        MAXIMUM        = 5,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4Config2InterfaceInfo {
    pub name: [Char16; 32],
    pub if_type: u8,
    pub hw_addr_size: u32,
    pub hw_addr: MacAddress,
    pub station_addr: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub route_table_size: u32,
    pub route_table: *mut Ip4RouteTable,
}

newtype_enum! {
    pub enum Ip4Config2Policy: i32 => {
        STATIC = 0,
        DHCP   = 1,
        MAX    = 2,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4Config2ManualAddress {
    pub address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
}

#[derive(Debug)]
#[repr(C)]
pub struct Ip4Config2Protocol {
    pub set_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        data_type: Ip4Config2DataType,
        data_size: usize,
        data: *const c_void,
    ) -> Status,

    pub get_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        data_type: Ip4Config2DataType,
        data_size: *mut usize,
        data: *mut c_void,
    ) -> Status,

    pub register_data_notify: unsafe extern "efiapi" fn(
        this: *mut Self,
        data_type: Ip4Config2DataType,
        event: Event,
    ) -> Status,

    pub unregister_data_notify: unsafe extern "efiapi" fn(
        this: *mut Self,
        data_type: Ip4Config2DataType,
        event: Event,
    ) -> Status,
}

impl Ip4Config2Protocol {
    pub const GUID: Guid = guid!("5b446ed1-e30b-4faa-871a-3654eca36080");
}
