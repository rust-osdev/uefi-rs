// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use core::ffi::c_void;

use crate::{Boolean, Char8, Char16, Event, Guid, Handle, Status, guid};

use super::device_path::DevicePathProtocol;
use super::file_system::FileInfo;
use super::shell_params::ShellFileHandle;

use bitflags::bitflags;

/// List Entry for File Lists
#[derive(Debug)]
#[repr(C)]
pub struct ListEntry {
    pub f_link: *mut Self,
    pub b_link: *mut Self,
}

/// ShellFileInfo for File Lists
#[derive(Debug)]
#[repr(C)]
pub struct ShellFileInfo {
    pub link: ListEntry,
    pub status: Status,
    pub full_name: *mut Char16,
    pub file_name: *mut Char16,
    pub handle: ShellFileHandle,
    pub info: FileInfo,
}

bitflags! {
    /// Specifies the source of the component name
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ShellDeviceNameFlags: u32 {
        /// Use Component Name
        const USE_COMPONENT_NAME = 0x0000001;
        /// Use Device Path
        const USE_DEVICE_PATH = 0x0000002;
    }
}

/// Shell Protocol
#[derive(Debug)]
#[repr(C)]
pub struct ShellProtocol {
    pub execute: unsafe extern "efiapi" fn(
        parent_image_handle: *const Handle,
        command_line: *const Char16,
        environment: *const *const Char16,
        status_code: *mut Status,
    ) -> Status,
    pub get_env: unsafe extern "efiapi" fn(name: *const Char16) -> *const Char16,
    pub set_env: unsafe extern "efiapi" fn(
        name: *const Char16,
        value: *const Char16,
        volatile: Boolean,
    ) -> Status,
    pub get_alias:
        unsafe extern "efiapi" fn(alias: *const Char16, volatile: Boolean) -> *const Char16,
    pub set_alias: unsafe extern "efiapi" fn(
        command: *const Char16,
        alias: *const Char16,
        replace: Boolean,
        volatile: Boolean,
    ) -> Status,
    pub get_help_text: unsafe extern "efiapi" fn(
        command: *const Char16,
        sections: *const Char16,
        help_text: *mut *mut Char16,
    ) -> Status,
    pub get_device_path_from_map:
        unsafe extern "efiapi" fn(mapping: *const Char16) -> *const DevicePathProtocol,
    pub get_map_from_device_path:
        unsafe extern "efiapi" fn(device_path: *mut *mut DevicePathProtocol) -> *const Char16,
    pub get_device_path_from_file_path:
        unsafe extern "efiapi" fn(path: *const Char16) -> *const DevicePathProtocol,
    pub get_file_path_from_device_path:
        unsafe extern "efiapi" fn(path: *const DevicePathProtocol) -> *const Char16,
    pub set_map: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        mapping: *const Char16,
    ) -> Status,

    pub get_cur_dir: unsafe extern "efiapi" fn(file_system_mapping: *const Char16) -> *const Char16,
    pub set_cur_dir:
        unsafe extern "efiapi" fn(file_system: *const Char16, dir: *const Char16) -> Status,
    pub open_file_list: unsafe extern "efiapi" fn(
        path: *const Char16,
        open_mode: u64,
        file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    pub free_file_list: unsafe extern "efiapi" fn(file_list: *const *const ShellFileInfo) -> Status,
    pub remove_dup_in_file_list:
        unsafe extern "efiapi" fn(file_list: *const *const ShellFileInfo) -> Status,

    pub batch_is_active: unsafe extern "efiapi" fn() -> Boolean,
    pub is_root_shell: unsafe extern "efiapi" fn() -> Boolean,
    pub enable_page_break: unsafe extern "efiapi" fn(),
    pub disable_page_break: unsafe extern "efiapi" fn(),
    pub get_page_break: unsafe extern "efiapi" fn() -> Boolean,
    pub get_device_name: unsafe extern "efiapi" fn(
        device_handle: Handle,
        flags: ShellDeviceNameFlags,
        language: *const Char8,
        best_device_name: *mut *mut Char16,
    ) -> Status,

    pub get_file_info: unsafe extern "efiapi" fn(file_handle: ShellFileHandle) -> *const FileInfo,
    pub set_file_info: unsafe extern "efiapi" fn(
        file_handle: ShellFileHandle,
        file_info: *const FileInfo,
    ) -> Status,
    pub open_file_by_name: unsafe extern "efiapi" fn(
        file_name: *const Char16,
        file_handle: *mut ShellFileHandle,
        open_mode: u64,
    ) -> Status,
    pub close_file: unsafe extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    pub create_file: unsafe extern "efiapi" fn(
        file_name: *const Char16,
        file_attribs: u64,
        file_handle: *mut ShellFileHandle,
    ) -> Status,
    pub read_file: unsafe extern "efiapi" fn(
        file_handle: ShellFileHandle,
        read_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_file: unsafe extern "efiapi" fn(
        file_handle: ShellFileHandle,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub delete_file: unsafe extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    pub delete_file_by_name: unsafe extern "efiapi" fn(file_name: *const Char16) -> Status,
    pub get_file_position:
        unsafe extern "efiapi" fn(file_handle: ShellFileHandle, position: *mut u64) -> Status,
    pub set_file_position:
        unsafe extern "efiapi" fn(file_handle: ShellFileHandle, position: u64) -> Status,
    pub flush_file: unsafe extern "efiapi" fn(file_handle: ShellFileHandle) -> Status,
    pub find_files: unsafe extern "efiapi" fn(
        file_pattern: *const Char16,
        file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    pub find_files_in_dir: unsafe extern "efiapi" fn(
        file_dir_handle: ShellFileHandle,
        file_list: *mut *mut ShellFileInfo,
    ) -> Status,
    pub get_file_size:
        unsafe extern "efiapi" fn(file_handle: ShellFileHandle, size: *mut u64) -> Status,

    pub open_root: unsafe extern "efiapi" fn(
        device_path: *const DevicePathProtocol,
        file_handle: *mut ShellFileHandle,
    ) -> Status,
    pub open_root_by_handle: unsafe extern "efiapi" fn(
        device_handle: Handle,
        file_handle: *mut ShellFileHandle,
    ) -> Status,

    pub execution_break: Event,

    pub major_version: u32,
    pub minor_version: u32,
    pub register_guid_name:
        unsafe extern "efiapi" fn(guid: *const Guid, guid_name: *const Char16) -> Status,
    pub get_guid_name:
        unsafe extern "efiapi" fn(guid: *const Guid, guid_name: *mut *mut Char16) -> Status,
    pub get_guid_from_name:
        unsafe extern "efiapi" fn(guid_name: *const Char16, guid: *mut Guid) -> Status,
    pub get_env_ex:
        unsafe extern "efiapi" fn(name: *const Char16, attributes: *mut u32) -> *const Char16,
}

impl ShellProtocol {
    pub const GUID: Guid = guid!("6302d008-7f9b-4f30-87ac-60c9fef5da4e");
}
