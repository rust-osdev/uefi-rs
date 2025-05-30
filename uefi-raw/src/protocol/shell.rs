// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use crate::{guid, Event, Guid};

/// Shell Protocol
#[derive(Debug)]
#[repr(C)]
pub struct ShellProtocol {
    pub execute: usize,
    pub get_env: usize,
    pub set_env: usize,
    pub get_alias: usize,
    pub set_alias: usize,
    pub get_help_text: usize,
    pub get_device_path_from_map: usize,
    pub get_map_from_device_path: usize,
    pub get_device_path_from_file_path: usize,
    pub get_file_path_from_device_path: usize,
    pub set_map: usize,

    pub get_cur_dir: usize,
    pub set_cur_dir: usize,
    pub open_file_list: usize,
    pub free_file_list: usize,
    pub remove_dup_in_file_list: usize,

    pub batch_is_active: usize,
    pub is_root_shell: usize,
    pub enable_page_break: usize,
    pub disable_page_break: usize,
    pub get_page_break: usize,
    pub get_device_name: usize,

    pub get_file_info: usize,
    pub set_file_info: usize,
    pub open_file_by_name: usize,
    pub close_file: usize,
    pub create_file: usize,
    pub read_file: usize,
    pub write_file: usize,
    pub delete_file: usize,
    pub delete_file_by_name: usize,
    pub get_file_position: usize,
    pub set_file_position: usize,
    pub flush_file: usize,
    pub find_files: usize,
    pub find_files_in_dir: usize,
    pub get_file_size: usize,

    pub open_root: usize,
    pub open_root_by_handle: usize,

    pub execution_break: Event,

    pub major_version: u32,
    pub minor_version: u32,
    pub register_guid_name: usize,
    pub get_guid_name: usize,
    pub get_guid_from_name: usize,
    pub get_env_ex: usize,
}

impl ShellProtocol {
    pub const GUID: Guid = guid!("6302d008-7f9b-4f30-87ac-60c9fef5da4e");
}
