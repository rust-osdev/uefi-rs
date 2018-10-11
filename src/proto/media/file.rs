//! This module provides the `File` structure, representing an opaque handle to a
//! directory / file, as well as providing functions for opening new files.
//!
//! Usually a file system implementation will return a "root" file, representing
//! `/` on that volume, and with that file it is possible to enumerate and open
//! all the other files on that volume.

use bitflags::bitflags;
use core::mem;
use crate::{Result, Status};
use ucs2;

/// A file represents an abstraction of some contiguous block of data residing
/// on a volume.
///
/// Dropping this structure will result in the file handle being closed.
///
/// Files have names, and a fixed size.
pub struct File<'a> {
    inner: &'a mut FileImpl,
}

impl<'a> File<'a> {
    pub(super) fn new(ptr: usize) -> Self {
        let ptr = ptr as *mut FileImpl;

        let inner = unsafe { &mut *ptr };

        File { inner }
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
            let mut ptr = 0usize;

            ucs2::encode(filename, &mut buf)?;
            (self.inner.open)(self.inner, &mut ptr, buf.as_ptr(), open_mode, attributes).into_with(
                || File {
                    inner: unsafe { &mut *(ptr as *mut FileImpl) },
                },
            )
        }
    }

    /// Close this file handle. Same as dropping this structure.
    pub fn close(self) {}

    /// Closes and deletes this file
    ///
    /// # Errors
    /// * `uefi::Status::WARN_DELETE_FAILURE` The file was closed, but deletion failed
    pub fn delete(self) -> Result<()> {
        let result = (self.inner.delete)(self.inner).into();

        mem::forget(self);

        result
    }

    /// Read data from file
    ///
    /// Try to read as much as possible into `buffer`. Returns the number of bytes read
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.inner.read)(self.inner, &mut buffer_size, buffer.as_mut_ptr())
            .into_with(|| buffer_size)
    }

    /// Write data to file
    ///
    /// Write `buffer` to file, increment the file pointer and return number of bytes written
    ///
    /// # Arguments
    /// * `buffer`  Buffer to write to file
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`    Attempt to write to readonly file
    /// * `uefi::Status::ACCESS_DENIED`      The file was opened read only.
    /// * `uefi::Status::VOLUME_FULL`        The volume is full
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.inner.write)(self.inner, &mut buffer_size, buffer.as_ptr()).into_with(|| buffer_size)
    }

    /// Get the file's current position
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to get the position of a deleted file
    pub fn get_position(&mut self) -> Result<u64> {
        let mut pos = 0u64;
        (self.inner.get_position)(self.inner, &mut pos).into_with(|| pos)
    }

    /// Sets the file's current position
    ///
    /// Set the position of this file handle to the absolute position specified by `position`.
    /// Seeking is not permitted outside the bounds of the file, except in the case
    /// of 0xFFFFFFFFFFFFFFFF, in which case the position is set to the end of the file
    ///
    /// # Arguments
    /// * `position` The new absolution position of the file handle
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to set the position of a deleted file
    pub fn set_position(&mut self, position: u64) -> Result<()> {
        (self.inner.set_position)(self.inner, position).into()
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
        (self.inner.flush)(self.inner).into()
    }
}

impl<'a> Drop for File<'a> {
    fn drop(&mut self) {
        let result: Result<()> = (self.inner.close)(self.inner).into();
        // The spec says this always succeeds.
        result.expect("Failed to close file").unwrap();
    }
}

/// The function pointer table for the File protocol.
#[repr(C)]
struct FileImpl {
    revision: u64,
    open: extern "win64" fn(
        this: &mut FileImpl,
        new_handle: &mut usize,
        filename: *const u16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Status,
    close: extern "win64" fn(this: &mut FileImpl) -> Status,
    delete: extern "win64" fn(this: &mut FileImpl) -> Status,
    read:
        extern "win64" fn(this: &mut FileImpl, buffer_size: &mut usize, buffer: *mut u8) -> Status,
    write: extern "win64" fn(this: &mut FileImpl, buffer_size: &mut usize, buffer: *const u8)
        -> Status,
    get_position: extern "win64" fn(this: &mut FileImpl, position: &mut u64) -> Status,
    set_position: extern "win64" fn(this: &mut FileImpl, position: u64) -> Status,
    get_info: usize,
    set_info: usize,
    flush: extern "win64" fn(this: &mut FileImpl) -> Status,
}

bitflags! {
    /// Usage flags describing what is possible to do with the file.
    pub struct FileMode: u64 {
        /// The file can be read from.
        const READ = 1;
        /// The file can be written to.
        const WRITE = 1 << 1;
        /// The file will be created if not found.
        const CREATE = 1 << 63;
    }
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
