// SPDX-License-Identifier: MIT OR Apache-2.0

use super::device_path::DevicePathProtocol;
use crate::Status;
use core::ffi::c_void;
use uguid::{guid, Guid};

#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressPassThruMode {
    pub attributes: u32,
    pub io_align: u32,
    pub nvme_version: u32,
}

/// This structure maps to the NVM Express specification Submission Queue Entry
#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressCommand {
    pub cdw0: u32,
    pub flags: u8,
    pub nsid: u32,
    pub cdw2: u32,
    pub cdw3: u32,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

/// This structure maps to the NVM Express specification Completion Queue Entry
#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressCompletion {
    pub dw0: u32,
    pub dw1: u32,
    pub dw2: u32,
    pub dw3: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressPassThruCommandPacket {
    pub command_timeout: u64,
    pub transfer_buffer: *mut c_void,
    pub transfer_length: u32,
    pub meta_data_buffer: *mut c_void,
    pub meta_data_length: u32,
    pub queue_type: u8,
    pub nvme_cmd: *const NvmExpressCommand,
    pub nvme_completion: *mut NvmExpressCompletion,
}

#[derive(Debug)]
#[repr(C)]
pub struct NvmExpressPassThruProtocol {
    pub mode: *const NvmExpressPassThruMode,
    pub pass_thru: unsafe extern "efiapi" fn(
        this: *const Self,
        namespace_id: u32,
        packet: *mut NvmExpressPassThruCommandPacket,
        event: *mut c_void,
    ) -> Status,
    pub get_next_namespace:
        unsafe extern "efiapi" fn(this: *const Self, namespace_id: *mut u32) -> Status,
    pub build_device_path: unsafe extern "efiapi" fn(
        this: *const Self,
        namespace_id: u32,
        device_path: *mut *mut DevicePathProtocol,
    ) -> Status,
    pub get_namespace: unsafe extern "efiapi" fn(
        this: *const Self,
        device_path: *const DevicePathProtocol,
        namespace_id: *mut u32,
    ) -> Status,
}

impl NvmExpressPassThruProtocol {
    pub const GUID: Guid = guid!("52c78312-8edc-4233-98f2-1a1aa5e388a5");
}
