// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ffi;

use crate::{guid, Boolean, Char16, Guid, Status};

use super::{
    AsyncUsbTransferCallback, ConfigDescriptor, DataDirection, DeviceDescriptor, DeviceRequest,
    EndpointDescriptor, InterfaceDescriptor, UsbTransferStatus,
};

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
