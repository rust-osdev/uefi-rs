// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{guid, Event, Guid, Status};
use core::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct DiskIoProtocol {
    pub revision: u64,
    pub read_disk: unsafe extern "efiapi" fn(
        this: *const Self,
        media_id: u32,
        offset: u64,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_disk: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        offset: u64,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
}

impl DiskIoProtocol {
    pub const GUID: Guid = guid!("ce345171-ba0b-11d2-8e4f-00a0c969723b");
    pub const REVISION: u64 = 0x00010000;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DiskIo2Token {
    pub event: Event,
    pub transaction_status: Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct DiskIo2Protocol {
    pub revision: u64,
    pub cancel: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub read_disk_ex: unsafe extern "efiapi" fn(
        this: *const Self,
        media_id: u32,
        offset: u64,
        token: *mut DiskIo2Token,
        buffer_size: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write_disk_ex: unsafe extern "efiapi" fn(
        this: *mut Self,
        media_id: u32,
        offset: u64,
        token: *mut DiskIo2Token,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush_disk_ex:
        unsafe extern "efiapi" fn(this: *mut Self, token: *mut DiskIo2Token) -> Status,
}

impl DiskIo2Protocol {
    pub const GUID: Guid = guid!("151c8eae-7f2c-472c-9e54-9828194f6a88");
    pub const REVISION: u64 = 0x00020000;
}

/// DiskInfo protocol (EFI_DISK_INFO_PROTOCOL)
///
/// See: UEFI Platform Initialization Specification
#[derive(Debug)]
#[repr(C)]
pub struct DiskInfoProtocol {
    pub interface: Guid,
    pub inquiry: unsafe extern "efiapi" fn(
        this: *const Self,
        inquiry_data: *mut c_void,
        inquiry_data_size: *mut u32,
    ) -> Status,
    pub identify: unsafe extern "efiapi" fn(
        this: *const Self,
        identify_data: *mut c_void,
        identify_data_size: *mut u32,
    ) -> Status,
    pub sense_data: unsafe extern "efiapi" fn(
        this: *const Self,
        sense_data: *mut c_void,
        sense_data_size: *mut u32,
        sense_data_number: *mut u8,
    ) -> Status,
    pub which_ide: unsafe extern "efiapi" fn(
        this: *const Self,
        ide_channel: *mut u32,
        ide_device: *mut u32,
    ) -> Status,
}

impl DiskInfoProtocol {
    pub const GUID: Guid = guid!("d432a67f-14dc-484b-b3bb-3f0291849327");

    pub const IDE_INTERFACE_GUID: Guid = guid!("5e948fe3-26d3-42b5-af17-610287188dec");
    pub const UFS_INTERFACE_GUID: Guid = guid!("4b3029cc-6b98-47fb-bc96-76dcb80441f0");
    pub const USB_INTERFACE_GUID: Guid = guid!("cb871572-c11a-47b5-b492-675eafa77727");
    pub const AHCI_INTERFACE_GUID: Guid = guid!("9e498932-4abc-45af-a34d-0247787be7c6");
    pub const NVME_INTERFACE_GUID: Guid = guid!("3ab14680-5d3f-4a4d-bcdc-cc380018c7f7");
    pub const SCSI_INTERFACE_GUID: Guid = guid!("08f74baa-ea36-41d9-9521-21a70f8780bc");
    pub const SD_MMC_INTERFACE_GUID: Guid = guid!("8deec992-d39c-4a5c-ab6b-986e14242b9d");
}
