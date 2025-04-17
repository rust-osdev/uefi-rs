// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi;

use bitflags::bitflags;

use crate::Status;

pub mod host_controller;
pub mod io;

newtype_enum! {
    pub enum DataDirection: i32 => {
        DATA_IN = 0,
        DATA_OUT = 1,
        NO_DATA = 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DeviceRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct UsbTransferStatus: u32 {
        const NOT_EXECUTE = 0x0001;
        const STALL = 0x0002;
        const BUFFER = 0x0004;
        const BABBLE = 0x0008;
        const NAK = 0x0010;
        const CRC = 0x0020;
        const TIMEOUT = 0x0040;
        const BIT_STUFF = 0x0080;
        const SYSTEM = 0x0100;
    }
}

impl UsbTransferStatus {
    pub const SUCCESS: Self = Self::empty();
}

pub type AsyncUsbTransferCallback = unsafe extern "efiapi" fn(
    data: *mut ffi::c_void,
    data_length: usize,
    context: *mut ffi::c_void,
    status: UsbTransferStatus,
) -> Status;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub str_manufacturer: u8,
    pub str_product: u8,
    pub str_serial_number: u8,
    pub num_configurations: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ConfigDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration: u8,
    pub attributes: u8,
    pub max_power: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct EndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}
