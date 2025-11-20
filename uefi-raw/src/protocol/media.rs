// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::protocol::device_path::DevicePathProtocol;
use crate::{Boolean, Guid, Status, guid};
use core::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct LoadFileProtocol {
    pub load_file: unsafe extern "efiapi" fn(
        this: *mut Self,
        file_path: *const DevicePathProtocol,
        boot_policy: Boolean,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFileProtocol {
    pub const GUID: Guid = guid!("56ec3091-954c-11d2-8e3f-00a0c969723b");
}

#[derive(Debug)]
#[repr(C)]
pub struct LoadFile2Protocol {
    pub load_file: unsafe extern "efiapi" fn(
        this: *mut Self,
        file_path: *const DevicePathProtocol,
        boot_policy: Boolean,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFile2Protocol {
    pub const GUID: Guid = guid!("4006c0c1-fcb3-403e-996d-4a6c8724e06d");
}

#[derive(Debug)]
#[repr(C)]
pub struct StorageSecurityCommandProtocol {
    pub receive_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        timeout: u64,
        security_protocol: u8,
        security_protocol_specific_data: u16,
        buffer_size: usize,
        buffer: *mut c_void,
        transfer_size: *mut usize,
    ) -> Status,

    pub send_data: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        timeout: u64,
        security_protocol: u8,
        security_protocol_specific_data: u16,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
}

impl StorageSecurityCommandProtocol {
    pub const GUID: Guid = guid!("c88b0b6d-0dfc-49a7-9cb4-49074b4c3a78");
}
