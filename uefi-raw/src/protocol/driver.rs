// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::protocol::device_path::DevicePathProtocol;
use crate::{guid, Guid, Handle, Status};

#[derive(Debug)]
#[repr(C)]
pub struct DriverBindingProtocol {
    pub supported: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        remaining_device_path: *const DevicePathProtocol,
    ) -> Status,
    pub start: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        remaining_device_path: *const DevicePathProtocol,
    ) -> Status,
    pub stop: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        number_of_children: usize,
        child_handle_buffer: *const Handle,
    ) -> Status,
    pub version: u32,
    pub image_handle: Handle,
    pub driver_binding_handle: Handle,
}

impl DriverBindingProtocol {
    pub const GUID: Guid = guid!("18a031ab-b443-4d1a-a5c0-0c09261e9f71");
}

#[derive(Debug)]
#[repr(C)]
pub struct ComponentName2Protocol {
    pub get_driver_name: unsafe extern "efiapi" fn(
        this: *const Self,
        language: *const u8,
        driver_name: *mut *const u16,
    ) -> Status,
    pub get_controller_name: unsafe extern "efiapi" fn(
        this: *const Self,
        controller_handle: Handle,
        child_handle: Handle,
        language: *const u8,
        controller_name: *mut *const u16,
    ) -> Status,
    pub supported_languages: *const u8,
}

impl ComponentName2Protocol {
    pub const GUID: Guid = guid!("6a7a5cff-e8d9-4f70-bada-75ab3025ce14");

    /// GUID of the original `EFI_COMPONENT_NAME_PROTOCOL`. This protocol was
    /// deprecated in UEFI 2.1 in favor of the new
    /// `EFI_COMPONENT_NAME2_PROTOCOL`. The two protocols are identical
    /// except the encoding of supported languages changed from ISO 639-2 to RFC
    /// 4646.
    pub const DEPRECATED_COMPONENT_NAME_GUID: Guid = guid!("107a772c-d5e1-11d4-9a46-0090273fc14d");
}

#[derive(Debug)]
#[repr(C)]
pub struct ServiceBindingProtocol {
    pub create_child:
        unsafe extern "efiapi" fn(this: *mut Self, child_handle: *mut Handle) -> Status,
    pub destroy_child: unsafe extern "efiapi" fn(this: *mut Self, child_handle: Handle) -> Status,
}
