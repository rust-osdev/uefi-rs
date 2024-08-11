// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{guid, Boolean, Char8, Event, Guid, Ipv4Address, MacAddress, Status};
use core::ffi::c_void;

newtype_enum! {
    pub enum Dhcp4Event: i32 => {
        SEND_DISCOVER   = 0x01,
        RCVD_OFFER      = 0x02,
        SELECT_OFFER    = 0x03,
        SEND_REQUEST    = 0x04,
        RCVD_ACK        = 0x05,
        RCVD_NAK        = 0x06,
        SEND_DECLINE    = 0x07,
        BOUND_COMPLETED = 0x08,
        ENTER_RENEWING  = 0x09,
        ENTER_REBINDING = 0x0a,
        ADDRESS_LOST    = 0x0b,
        FAIL            = 0x0c,
    }
}

newtype_enum! {
    pub enum Dhcp4State: i32 => {
        STOPPED     = 0x0,
        INIT        = 0x1,
        SELECTING   = 0x2,
        REQUESTING  = 0x3,
        BOUND       = 0x4,
        RENEWING    = 0x5,
        REBINDING   = 0x6,
        INIT_REBOOT = 0x7,
        REBOOTING   = 0x8,
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Dhcp4Packet {
    pub size: u32,
    pub length: u32,
    pub header: Dhcp4Header,
    pub magik: u32,

    /// Start of the DHCP packed option data.
    ///
    /// Note that this field is actually a variable-length array.
    pub option: [u8; 0],
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct Dhcp4Header {
    pub op_code: u8,
    pub hw_type: u8,
    pub hw_addr_len: u8,
    pub hops: u8,
    pub xid: u32,
    pub seconds: u16,
    pub reserved: u16,
    pub client_addr: Ipv4Address,
    pub your_addr: Ipv4Address,
    pub server_addr: Ipv4Address,
    pub gateway_addr: Ipv4Address,
    pub client_hw_addr: [u8; 16],
    pub server_name: [Char8; 64],
    pub boot_file_name: [Char8; 128],
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct Dhcp4PacketOption {
    pub op_code: u8,
    pub length: u8,

    /// Start of the DHCP option data.
    ///
    /// Note that this field is actually a variable-length array.
    pub data: [u8; 0],
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcp4ConfigData {
    pub discover_try_count: u32,
    pub discover_timeout: *mut u32,
    pub request_try_count: u32,
    pub request_timeout: *mut u32,
    pub client_address: Ipv4Address,
    pub callback: Option<
        unsafe extern "efiapi" fn(
            this: *mut Dhcp4Protocol,
            context: *const c_void,
            current_state: Dhcp4State,
            dhcp4_event: Dhcp4Event,
            packet: *const Dhcp4Packet,
            new_packet: *mut *const Dhcp4Packet,
        ) -> Status,
    >,
    pub callback_context: *mut c_void,
    pub option_count: u32,
    pub option_list: *mut *const Dhcp4PacketOption,
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcp4ModeData {
    pub state: Dhcp4State,
    pub config_data: Dhcp4ConfigData,
    pub client_address: Ipv4Address,
    pub client_mac_address: MacAddress,
    pub server_address: Ipv4Address,
    pub router_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub lease_time: u32,
    pub reply_packet: *const Dhcp4Packet,
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcp4ListenPoint {
    pub listen_address: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub listen_port: u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcp4TransmitReceiveToken {
    pub status: Status,
    pub completion_event: Event,
    pub remote_address: Ipv4Address,
    pub remote_port: u16,
    pub gateway_address: Ipv4Address,
    pub listen_point_count: u32,
    pub listen_points: *mut Dhcp4ListenPoint,
    pub timeout_value: u32,
    pub packet: *mut Dhcp4Packet,
    pub response_count: u32,
    pub response_list: *mut Dhcp4Packet,
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcp4Protocol {
    pub get_mode_data:
        unsafe extern "efiapi" fn(this: *const Self, mode_data: *mut Dhcp4ModeData) -> Status,
    pub configure:
        unsafe extern "efiapi" fn(this: *mut Self, cfg_data: *const Dhcp4ConfigData) -> Status,
    pub start: unsafe extern "efiapi" fn(this: *mut Self, completion_event: Event) -> Status,
    pub renew_rebind: unsafe extern "efiapi" fn(
        this: *mut Self,
        rebind_request: Boolean,
        completion_event: Event,
    ) -> Status,
    pub release: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub stop: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub build: unsafe extern "efiapi" fn(
        this: *mut Self,
        seed_packet: *mut Dhcp4Packet,
        delete_count: u32,
        delete_list: *mut u8,
        append_count: u32,
        append_list: *const *const Dhcp4PacketOption,
        new_packet: *mut *mut Dhcp4Packet,
    ) -> Status,
    pub transmit_receive:
        unsafe extern "efiapi" fn(this: *mut Self, token: *mut Dhcp4TransmitReceiveToken) -> Status,
    pub parse: unsafe extern "efiapi" fn(
        this: *mut Self,
        packet: *mut Dhcp4Packet,
        option_count: *mut u32,
        packet_option_list: *mut *mut Dhcp4PacketOption,
    ) -> Status,
}

impl Dhcp4Protocol {
    pub const GUID: Guid = guid!("8a219718-4ef5-4761-91c8-c0f04bda9e56");
    pub const SERVICE_BINDING_GUID: Guid = guid!("9d9a39d8-bd42-4a73-a4d5-8ee94be11380");
}
