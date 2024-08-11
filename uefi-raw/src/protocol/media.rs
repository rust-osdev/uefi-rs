// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::protocol::device_path::DevicePathProtocol;
use crate::{guid, Boolean, Guid, Status};
use core::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct LoadFileProtocol {
    pub load_file: unsafe extern "efiapi" fn(
        this: *mut LoadFileProtocol,
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
        this: *mut LoadFile2Protocol,
        file_path: *const DevicePathProtocol,
        boot_policy: Boolean,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFile2Protocol {
    pub const GUID: Guid = guid!("4006c0c1-fcb3-403e-996d-4a6c8724e06d");
}
