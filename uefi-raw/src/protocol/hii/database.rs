// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bindings for HII Database Protocol

use super::{HiiHandle, HiiPackageHeader, HiiPackageListHeader, KeyDescriptor};
use crate::{guid, Guid, Handle, Status};

/// EFI_HII_KEYBOARD_LAYOUT
#[derive(Debug)]
#[repr(C)]
pub struct HiiKeyboardLayout {
    pub layout_length: u16,
    pub guid: Guid,
    pub layout_descriptor_string_offset: u32,
    pub descriptor_count: u8,
    pub descriptors: [KeyDescriptor; 0],
}

newtype_enum! {
    /// EFI_HII_DATABASE_NOTIFY_TYPE
    pub enum HiiDatabaseNotifyType: usize => {
        NEW_PACK = 1 << 0,
        REMOVE_PACK = 1 << 1,
        EXPORT_PACK = 1 << 2,
        ADD_PACK = 1 << 3,
    }
}

/// EFI_HII_DATABASE_NOTIFY
pub type HiiDatabaseNotifyFn = unsafe extern "efiapi" fn(
    package_type: u8,
    package_guid: *const Guid,
    package: *const HiiPackageHeader,
    handle: HiiHandle,
    notify_type: HiiDatabaseNotifyType,
) -> Status;

/// EFI_HII_DATABASE_PROTOCOL
#[derive(Debug)]
#[repr(C)]
pub struct HiiDatabaseProtocol {
    pub new_package_list: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list: *const HiiPackageListHeader,
        driver_handle: Handle,
        handle: *mut HiiHandle,
    ) -> Status,
    pub remove_package_list:
        unsafe extern "efiapi" fn(this: *const Self, handle: HiiHandle) -> Status,
    pub update_package_list: unsafe extern "efiapi" fn(
        this: *const Self,
        handle: HiiHandle,
        package_list: *const HiiPackageListHeader,
    ) -> Status,
    pub list_package_lists: unsafe extern "efiapi" fn(
        this: *const Self,
        package_type: u8,
        package_guid: *const Guid,
        handle_buffer_length: *mut usize,
        handle: *mut HiiHandle,
    ) -> Status,
    pub export_package_lists: unsafe extern "efiapi" fn(
        this: *const Self,
        handle: HiiHandle,
        buffer_size: *mut usize,
        buffer: *mut HiiPackageListHeader,
    ) -> Status,
    pub register_package_notify: unsafe extern "efiapi" fn(
        this: *const Self,
        package_type: u8,
        package_guid: *const Guid,
        package_notify_fn: HiiDatabaseNotifyFn,
        notify_type: HiiDatabaseNotifyType,
        notify_handle: *mut Handle,
    ) -> Status,
    pub unregister_package_notify:
        unsafe extern "efiapi" fn(this: *const Self, notification_handle: Handle) -> Status,
    pub find_keyboard_layouts: unsafe extern "efiapi" fn(
        this: *const Self,
        key_guid_buffer_length: *mut u16,
        key_guid_buffer: *mut Guid,
    ) -> Status,
    pub get_keyboard_layout: unsafe extern "efiapi" fn(
        this: *const Self,
        key_guid: *const Guid,
        leyboard_layout_length: *mut u16,
        keyboard_layout: *mut HiiKeyboardLayout,
    ) -> Status,
    pub set_keyboard_layout:
        unsafe extern "efiapi" fn(this: *const Self, key_guid: *const Guid) -> Status,
    pub get_package_list_handle: unsafe extern "efiapi" fn(
        this: *const Self,
        package_list_handle: HiiHandle,
        driver_handle: *mut Handle,
    ) -> Status,
}

impl HiiDatabaseProtocol {
    pub const GUID: Guid = guid!("ef9fc172-a1b2-4693-b327-6d32fc416042");
}
