use crate::{guid, Event, Guid, Status};
use core::ffi::c_void;

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
