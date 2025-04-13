// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi;

use crate::{guid, Boolean, Char16, Guid, Status};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct UsbTransferStatus(pub u32);

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

#[derive(Debug)]
#[repr(C)]
pub struct UsbIoProtocol {
    pub control_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        request: *mut DeviceRequest,
        direction: DataDirection,
        timeout: u32,
        data: *mut ffi::c_void,
        data_length: usize,
        status: *mut UsbTransferStatus,
    ) -> Status,
    pub bulk_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_endpoint: u8,
        data: *mut ffi::c_void,
        data_length: *mut usize,
        timeout: usize,
        status: *mut UsbTransferStatus,
    ) -> Status,
    pub async_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_endpoint: u8,
        is_new_transfer: Boolean,
        polling_interval: usize,
        data_length: usize,
        interrupt_callback: AsyncUsbTransferCallback,
        context: *mut ffi::c_void,
    ) -> Status,
    pub sync_interrupt_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_endpoint: u8,
        data: *mut ffi::c_void,
        data_length: *mut usize,
        timeout: usize,
        status: *mut UsbTransferStatus,
    ) -> Status,
    pub isochronous_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_endpoint: u8,
        data: *mut ffi::c_void,
        data_length: usize,
        status: *mut UsbTransferStatus,
    ) -> Status,
    pub async_isochronous_transfer: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_endpoint: u8,
        data: *mut ffi::c_void,
        data_length: usize,
        isochronous_callback: AsyncUsbTransferCallback,
        context: *mut ffi::c_void,
    ) -> Status,
    pub get_device_descriptor: unsafe extern "efiapi" fn(
        this: *mut Self,
        device_descriptor: *mut DeviceDescriptor,
    ) -> Status,
    pub get_config_descriptor: unsafe extern "efiapi" fn(
        this: *mut Self,
        config_descriptor: *mut ConfigDescriptor,
    ) -> Status,
    pub get_interface_descriptor: unsafe extern "efiapi" fn(
        this: *mut Self,
        interface_descriptor: *mut InterfaceDescriptor,
    ) -> Status,
    pub get_endpoint_descriptor: unsafe extern "efiapi" fn(
        this: *mut Self,
        endpoint_index: u8,
        endpoint_descriptor: *mut EndpointDescriptor,
    ) -> Status,
    pub get_string_descriptor: unsafe extern "efiapi" fn(
        this: *mut Self,
        lang_id: u16,
        string_id: u8,
        string: *mut *mut Char16,
    ) -> Status,
    pub get_supported_languages: unsafe extern "efiapi" fn(
        this: *mut Self,
        lang_id_table: *mut *mut u16,
        table_size: *mut u16,
    ) -> Status,
    pub port_reset: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

impl UsbIoProtocol {
    pub const GUID: Guid = guid!("2b2f68d6-0cd2-44cf-8e8b-bba20b1b5b75");
}
