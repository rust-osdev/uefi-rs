//! This module provides the `File` structure, representing an opaque handle to a
//! directory / file, as well as providing functions for opening new files.
//!
//! Usually a file system implementation will return a "root" file, representing
//! `/` on that volume, and with that file it is possible to enumerate and open
//! all the other files on that volume.

use crate::data_types::chars::NUL_16;
use crate::prelude::*;
use crate::table::runtime::Time;
use crate::{CStr16, Char16, Guid, Identify, Result, Status};
use bitflags::bitflags;
use core::convert::TryInto;
use core::mem;
use core::ptr;
use core::result;
use core::slice;
use ucs2;

/// A file represents an abstraction of some contiguous block of data residing
/// on a volume.
///
/// Dropping this structure will result in the file handle being closed.
///
/// Files have names, and a fixed size.
///
/// FIXME: Currently, directories also map into this, but they are different
///        enough to warrant a dedicated API.
pub struct File<'a>(&'a mut FileImpl);

impl<'a> File<'a> {
    pub(super) unsafe fn new(ptr: *mut FileImpl) -> Self {
        File(&mut *ptr)
    }

    /// Try to open a file relative to this file/directory.
    ///
    /// # Arguments
    /// * `filename`    Path of file to open, relative to this File
    /// * `open_mode`   The mode to open the file with. Valid
    ///     combinations are READ, READ | WRITE and READ | WRITE | CREATE
    /// * `attributes`  Only valid when FILE_MODE_CREATE is used as a mode
    ///
    /// # Errors
    /// * `uefi::Status::INVALID_PARAMETER`  The filename exceeds the maximum length of 255 chars
    /// * `uefi::Status::NOT_FOUND`          Could not find file
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::MEDIA_CHANGED`      The device has a different medium in it
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`    Write/Create attempted on readonly file
    /// * `uefi::Status::ACCESS_DENIED`      The service denied access to the file
    /// * `uefi::Status::OUT_OF_RESOURCES`    Not enough resources to open file
    /// * `uefi::Status::VOLUME_FULL`        The volume is full
    pub fn open(
        &mut self,
        filename: &str,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Result<File> {
        const BUF_SIZE: usize = 255;
        if filename.len() > BUF_SIZE {
            Err(Status::INVALID_PARAMETER)
        } else {
            let mut buf = [0u16; BUF_SIZE + 1];
            let mut ptr = ptr::null_mut();

            let len = ucs2::encode(filename, &mut buf)?;
            let filename = unsafe { CStr16::from_u16_with_nul_unchecked(&buf[..=len]) };

            unsafe { (self.0.open)(self.0, &mut ptr, filename.as_ptr(), open_mode, attributes) }
                .into_with(|| unsafe { File::new(ptr) })
        }
    }

    /// Close this file handle. Same as dropping this structure.
    pub fn close(self) {}

    /// Closes and deletes this file
    ///
    /// # Warnings
    /// * `uefi::Status::WARN_DELETE_FAILURE` The file was closed, but deletion failed
    pub fn delete(self) -> Result<()> {
        let result = (self.0.delete)(self.0).into();

        mem::forget(self);

        result
    }

    /// Read data from file
    ///
    /// Try to read as much as possible into `buffer`. Returns the number of bytes read.
    ///
    /// When the File is actually a directory, the behaviour of this function changes completely,
    /// and directory entries (type EFI_FILE_INFO) are read into the buffer one by one instead.
    ///
    /// FIXME: Currently, there is no nice API for this latter mechanism.
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error, the file was deleted,
    ///                                      or the end of the file was reached before the `read()`.
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::BUFFER_TOO_SMALL`   The buffer is too small to hold a directory entry.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        unsafe { (self.0.read)(self.0, &mut buffer_size, buffer.as_mut_ptr()) }
            .into_with(|| buffer_size)
    }

    /// Write data to file
    ///
    /// Write `buffer` to file, increment the file pointer.
    ///
    /// If an error occurs, returns the number of bytes that were actually written. If no error
    /// occured, the entire buffer is guaranteed to have been written successfully.
    ///
    /// Opened directories cannot be written to.
    ///
    /// # Arguments
    /// * `buffer`  Buffer to write to file
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED`        Attempted to write in a directory.
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error or the file was deleted.
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`    Attempt to write to readonly file
    /// * `uefi::Status::ACCESS_DENIED`      The file was opened read only.
    /// * `uefi::Status::VOLUME_FULL`        The volume is full
    pub fn write(&mut self, buffer: &[u8]) -> result::Result<(), (Status, usize)> {
        let mut buffer_size = buffer.len();
        match unsafe { (self.0.write)(self.0, &mut buffer_size, buffer.as_ptr()) } {
            Status::SUCCESS => Ok(()),
            error => Err((error, buffer_size)),
        }
    }

    /// Get the file's current position
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED`    Attempted to get the position of an opened directory
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to get the position of a deleted file
    pub fn get_position(&mut self) -> Result<u64> {
        let mut pos = 0u64;
        (self.0.get_position)(self.0, &mut pos).into_with(|| pos)
    }

    /// Sets the file's current position
    ///
    /// Set the position of this file handle to the absolute position specified by `position`.
    ///
    /// Seeking past the end of the file is allowed, it will trigger file growth on the next write.
    ///
    /// The special value 0xFFFF_FFFF_FFFF_FFFF may be used to seek to the end of the file.
    ///
    /// Seeking directories is only allowed with the special value 0, which has the effect of
    /// resetting the enumeration of directory entries.
    ///
    /// # Arguments
    /// * `position` The new absolution position of the file handle
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED`    Attempted a nonzero seek on a directory.
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to set the position of a deleted file
    pub fn set_position(&mut self, position: u64) -> Result<()> {
        (self.0.set_position)(self.0, position).into()
    }

    /// Flushes all modified data associated with the file handle to the device
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`    The file or medium is write protected
    /// * `uefi::Status::ACCESS_DENIED`      The file was opened read only
    /// * `uefi::Status::VOLUME_FULL`        The volume is full
    pub fn flush(&mut self) -> Result<()> {
        (self.0.flush)(self.0).into()
    }
}

impl<'a> Drop for File<'a> {
    fn drop(&mut self) {
        let result: Result<()> = (self.0.close)(self.0).into();
        // The spec says this always succeeds.
        result.expect_success("Failed to close file");
    }
}

/// The function pointer table for the File protocol.
#[repr(C)]
pub(super) struct FileImpl {
    revision: u64,
    open: unsafe extern "win64" fn(
        this: &mut FileImpl,
        new_handle: &mut *mut FileImpl,
        filename: *const Char16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Status,
    close: extern "win64" fn(this: &mut FileImpl) -> Status,
    delete: extern "win64" fn(this: &mut FileImpl) -> Status,
    read: unsafe extern "win64" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    write: unsafe extern "win64" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *const u8,
    ) -> Status,
    get_position: extern "win64" fn(this: &mut FileImpl, position: &mut u64) -> Status,
    set_position: extern "win64" fn(this: &mut FileImpl, position: u64) -> Status,
    get_info: usize,
    set_info: usize,
    flush: extern "win64" fn(this: &mut FileImpl) -> Status,
}

/// Usage flags describing what is possible to do with the file.
///
/// SAFETY: Using a repr(C) enum is safe here because this type is only sent to
///         the UEFI implementation, and never received from it.
///
#[repr(u64)]
pub enum FileMode {
    /// The file can be read from
    Read = 1,

    /// The file can be read from and written to
    ReadWrite = 2 | 1,

    /// The file can be read, written, and will be created if it does not exist
    CreateReadWrite = (1 << 63) | 2 | 1,
}

bitflags! {
    /// Attributes describing the properties of a file on the file system.
    pub struct FileAttribute: u64 {
        /// File can only be opened in [`FileMode::READ`] mode.
        const READ_ONLY = 1;
        /// Hidden file, not normally visible to the user.
        const HIDDEN = 1 << 1;
        /// System file, indicates this file is an internal operating system file.
        const SYSTEM = 1 << 2;
        /// This file is a directory.
        const DIRECTORY = 1 << 4;
        /// This file is compressed.
        const ARCHIVE = 1 << 5;
        /// Mask combining all the valid attributes.
        const VALID_ATTR = 0x37;
    }
}

/// Common trait for data structures that can be used with
/// EFI_FILE_PROTOCOL.GetInfo() or EFI_FILE_PROTOCOL.SetInfo()
trait FileProtocolInfo: Identify {}

/// Generic file information
#[repr(C)]
pub struct FileInfo {
    size: u64,
    file_size: u64,
    physical_size: u64,
    create_time: Time,
    last_access_time: Time,
    modification_time: Time,
    attribute: FileAttribute,
    file_name: [Char16],
}

impl FileInfo {
    /// Create a FileInfo structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the complete FileInfo
    /// structure, including a null-terminated UCS-2 version of the file_name
    /// string. A [u64] is requested because we need 64-bit alignment.
    ///
    pub fn new<'a>(
        storage: &'a mut [u64],
        file_size: u64,
        physical_size: u64,
        create_time: Time,
        last_access_time: Time,
        modification_time: Time,
        attribute: FileAttribute,
        file_name: &str,
    ) -> result::Result<&'a mut FileInfo, FileInfoCreationError> {
        // First, make sure that the user-provided storage is large enough
        const HEADER_SIZE: usize = 3 * mem::size_of::<u64>()
            + 3 * mem::size_of::<Time>()
            + mem::size_of::<FileAttribute>();
        let file_name_length_ucs2 = file_name.chars().count() + 1;
        let file_name_size = file_name_length_ucs2 * mem::size_of::<u16>();
        let file_info_size = HEADER_SIZE + file_name_size;
        if file_info_size > storage.len() * mem::size_of::<u64>() {
            return Err(FileInfoCreationError::InsufficientStorage(file_info_size));
        }

        // Next, build a suitably sized &mut FileInfo pointing into the storage.
        // It is okay to do this, even if the FileInfo fields will get random
        // invalid values, because no field has a nontrivial Drop impl, so we
        // can overwrite them in safe code without risking Rust code interaction
        // with the uninitialized value.
        let file_info_ptr = unsafe {
            slice::from_raw_parts_mut(storage.as_mut_ptr() as *mut u16, file_name_length_ucs2)
                as *mut [u16] as *mut FileInfo
        };
        let file_info = unsafe { &mut *file_info_ptr };
        debug_assert!(file_info.file_name.len() == file_name_length_ucs2);

        // Finally, we can initialize the resulting FileInfo
        file_info.size = file_info_size as u64;
        file_info.file_size = file_size;
        file_info.physical_size = physical_size;
        file_info.create_time = create_time;
        file_info.last_access_time = last_access_time;
        file_info.modification_time = modification_time;
        file_info.attribute = attribute;
        for (target, ch) in file_info.file_name.iter_mut().zip(file_name.chars()) {
            *target = ch
                .try_into()
                .map_err(|_| FileInfoCreationError::InvalidChar(ch))?;
        }
        file_info.file_name[file_name_length_ucs2 - 1] = NUL_16;
        Ok(file_info)
    }

    /// Query the file size (number of bytes stored in the file)
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Query the physical size (number of bytes used on the device)
    pub fn physical_size(&self) -> u64 {
        self.physical_size
    }

    /// Query the creation time
    pub fn create_time(&self) -> &Time {
        &self.create_time
    }

    /// Query the last access time
    pub fn last_access_time(&self) -> &Time {
        &self.last_access_time
    }

    /// Query the modification time
    pub fn modification_time(&self) -> &Time {
        &self.modification_time
    }

    /// Query the attributes
    pub fn attribute(&self) -> FileAttribute {
        self.attribute
    }
}

/// The enum enumerates the things that can go wrong when creating a FileInfo
pub enum FileInfoCreationError {
    /// The provided buffer was too small to hold the FileInfo. You need at
    /// least the indicated buffer size (in bytes).
    InsufficientStorage(usize),

    /// The suggested file name contains invalid code points (not in UCS-2)
    InvalidChar(char),
}

impl Identify for FileInfo {
    const GUID: Guid = Guid::from_values(
        0x0957_6e92,
        0x6d3f,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );
}

impl FileProtocolInfo for FileInfo {}
