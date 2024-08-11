// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::time::Time;
use crate::{guid, Boolean, Char16, Event, Guid, Status};
use bitflags::bitflags;
use core::ffi::c_void;

#[derive(Debug)]
#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume:
        unsafe extern "efiapi" fn(this: *mut Self, root: *mut *mut FileProtocolV1) -> Status,
}

impl SimpleFileSystemProtocol {
    pub const GUID: Guid = guid!("964e5b22-6459-11d2-8e39-00a0c969723b");
}

newtype_enum! {
    pub enum FileProtocolRevision: u64 => {
        REVISION_1 = 0x00010000,
        REVISION_2 = 0x00020000,
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FileProtocolV1 {
    pub revision: FileProtocolRevision,
    pub open: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_handle: *mut *mut Self,
        file_name: *const Char16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Status,
    pub close: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub delete: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub read: unsafe extern "efiapi" fn(
        this: *mut Self,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: *mut Self,
        buffer_size: *mut usize,
        buffer: *const c_void,
    ) -> Status,
    pub get_position: unsafe extern "efiapi" fn(this: *const Self, position: *mut u64) -> Status,
    pub set_position: unsafe extern "efiapi" fn(this: *mut Self, position: u64) -> Status,
    pub get_info: unsafe extern "efiapi" fn(
        this: *mut Self,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
    pub set_info: unsafe extern "efiapi" fn(
        this: *mut Self,
        information_type: *const Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    pub flush: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct FileProtocolV2 {
    pub v1: FileProtocolV1,
    pub open_ex: unsafe extern "efiapi" fn(
        this: *mut Self,
        new_handle: *mut *mut Self,
        file_name: *const Char16,
        open_mode: FileMode,
        attributes: FileAttribute,
        token: *mut FileIoToken,
    ) -> Status,
    pub read_ex: unsafe extern "efiapi" fn(this: *mut Self, token: *mut FileIoToken) -> Status,
    pub write_ex: unsafe extern "efiapi" fn(this: *mut Self, token: *mut FileIoToken) -> Status,
    pub flush_ex: unsafe extern "efiapi" fn(this: *mut Self, token: *mut FileIoToken) -> Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct FileIoToken {
    pub event: Event,
    pub status: Status,
    pub buffer_size: usize,
    pub buffer: *mut c_void,
}

bitflags! {
    /// File attributes.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct FileAttribute: u64 {
        /// The file cannot be opened for modification.
        const READ_ONLY = 0x0000000000000001;

        /// The file is hidden from normal directory views.
        const HIDDEN = 0x0000000000000002;

        /// The file belongs to the system and must not be physically moved.
        const SYSTEM = 0x0000000000000004;

        /// The file is a directory.
        const DIRECTORY = 0x0000000000000010;

        /// The file is marked for archival by backup software.
        const ARCHIVE = 0x0000000000000020;

        /// Mask combining all the valid attributes.
        const VALID_ATTR = 0x0000000000000037;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct FileMode: u64 {
        const READ = 0x0000000000000001;
        const WRITE = 0x0000000000000002;
        const CREATE = 0x8000000000000000;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FileInfo {
    pub size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: Time,
    pub last_access_time: Time,
    pub modification_time: Time,
    pub attribute: FileAttribute,

    /// The null-terminated name of the file. For a root directory, this is an
    /// empty string.
    ///
    /// Note that this field is actually a variable-length array. In order to
    /// avoid making this struct a DST, the field is represented as a
    /// zero-length array here.
    pub file_name: [Char16; 0],
}

impl FileInfo {
    pub const ID: Guid = guid!("09576e92-6d3f-11d2-8e39-00a0c969723b");
}

#[derive(Debug)]
#[repr(C)]
pub struct FileSystemInfo {
    pub size: u64,
    pub read_only: Boolean,
    pub volume_size: u64,
    pub free_space: u64,
    pub block_size: u32,

    /// The null-terminated label of the volume.
    ///
    /// Note that this field is actually a variable-length array. In order to
    /// avoid making this struct a DST, the field is represented as a
    /// zero-length array here.
    pub volume_label: [Char16; 0],
}

impl FileSystemInfo {
    pub const ID: Guid = guid!("09576e93-6d3f-11d2-8e39-00a0c969723b");
}

#[derive(Debug)]
#[repr(C)]
pub struct FileSystemVolumeLabel {
    /// The null-terminated label of the volume.
    ///
    /// Note that this field is actually a variable-length array. In order to
    /// avoid making this struct a DST, the field is represented as a
    /// zero-length array here.
    pub volume_label: [Char16; 0],
}

impl FileSystemVolumeLabel {
    pub const ID: Guid = guid!("db47d7d3-fe81-11d3-9a35-0090273fc14d");
}
