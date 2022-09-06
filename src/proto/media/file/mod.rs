//! This module provides the `FileHandle` structure as well as the more specific `RegularFile` and
//! `Directory` structures. This module also provides the `File` trait for opening, querying,
//! creating, reading, and writing files.
//!
//! Usually a file system implementation will return a "root" directory, representing
//! `/` on that volume. With that directory, it is possible to enumerate and open
//! all the other files on that volume.

mod dir;
mod info;
mod regular;

use crate::{CStr16, Char16, Guid, Result, Status};
use bitflags::bitflags;
use core::ffi::c_void;
use core::fmt::Debug;
use core::mem;
use core::ptr;
#[cfg(feature = "exts")]
use {
    crate::ResultExt,
    alloc_api::{alloc, alloc::Layout, boxed::Box},
    core::slice,
};

pub use self::info::{FileInfo, FileProtocolInfo, FileSystemInfo, FileSystemVolumeLabel, FromUefi};
pub use self::{dir::Directory, regular::RegularFile};

/// Common interface to `FileHandle`, `RegularFile`, and `Directory`.
///
/// `File` contains all functionality that is safe to perform on any type of
/// file handle.
pub trait File: Sized {
    /// Access the underlying file handle.
    #[doc(hidden)]
    fn handle(&mut self) -> &mut FileHandle;

    /// Try to open a file relative to this file.
    ///
    /// # Arguments
    /// * `filename`    Path of file to open, relative to this file
    /// * `open_mode`   The mode to open the file with
    /// * `attributes`  Only valid when `FILE_MODE_CREATE` is used as a mode
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
    fn open(
        &mut self,
        filename: &CStr16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Result<FileHandle> {
        let mut ptr = ptr::null_mut();

        unsafe {
            (self.imp().open)(
                self.imp(),
                &mut ptr,
                filename.as_ptr(),
                open_mode,
                attributes,
            )
        }
        .into_with_val(|| unsafe { FileHandle::new(ptr) })
    }

    /// Close this file handle. Same as dropping this structure.
    fn close(self) {}

    /// Closes and deletes this file
    ///
    /// # Warnings
    /// * `uefi::Status::WARN_DELETE_FAILURE` The file was closed, but deletion failed
    fn delete(mut self) -> Result {
        let result = (self.imp().delete)(self.imp()).into();
        mem::forget(self);
        result
    }

    /// Queries some information about a file
    ///
    /// The information will be written into a user-provided buffer.
    /// If the buffer is too small, the required buffer size will be returned as part of the error.
    ///
    /// The buffer must be aligned on an `<Info as Align>::alignment()` boundary.
    ///
    /// # Arguments
    /// * `buffer`  Buffer that the information should be written into
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED`        The file does not possess this information type
    /// * `uefi::Status::NO_MEDIA`           The device has no medium
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`   The file system structures are corrupted
    /// * `uefi::Status::BUFFER_TOO_SMALL`   The buffer is too small for the requested
    fn get_info<'buf, Info: FileProtocolInfo + ?Sized>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<&'buf mut Info, Option<usize>> {
        let mut buffer_size = buffer.len();
        Info::assert_aligned(buffer);
        unsafe {
            (self.imp().get_info)(
                self.imp(),
                &Info::GUID,
                &mut buffer_size,
                buffer.as_mut_ptr(),
            )
        }
        .into_with(
            || unsafe { Info::from_uefi(buffer.as_mut_ptr().cast::<c_void>()) },
            |s| {
                if s == Status::BUFFER_TOO_SMALL {
                    Some(buffer_size)
                } else {
                    None
                }
            },
        )
    }

    /// Sets some information about a file
    ///
    /// There are various restrictions on the information that may be modified using this method.
    /// The simplest one is that it is usually not possible to call it on read-only media. Further
    /// restrictions specific to a given information type are described in the corresponding
    /// `FileProtocolInfo` type documentation.
    ///
    /// # Arguments
    /// * `info`  Info that should be set for the file
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED`       The file does not possess this information type
    /// * `uefi::Status::NO_MEDIA`          The device has no medium
    /// * `uefi::Status::DEVICE_ERROR`      The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED`  The file system structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`   Attempted to set information on a read-only media
    /// * `uefi::Status::ACCESS_DENIED`     Requested change is invalid for this information type
    /// * `uefi::Status::VOLUME_FULL`       Not enough space left on the volume to change the info
    fn set_info<Info: FileProtocolInfo + ?Sized>(&mut self, info: &Info) -> Result {
        let info_ptr = (info as *const Info).cast::<c_void>();
        let info_size = mem::size_of_val(&info);
        unsafe { (self.imp().set_info)(self.imp(), &Info::GUID, info_size, info_ptr).into() }
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
    fn flush(&mut self) -> Result {
        (self.imp().flush)(self.imp()).into()
    }

    #[cfg(feature = "exts")]
    /// Get the dynamically allocated info for a file
    fn get_boxed_info<Info: FileProtocolInfo + ?Sized + Debug>(&mut self) -> Result<Box<Info>> {
        // Initially try get_info with an empty array, this should always fail
        // as all Info types at least need room for a null-terminator.
        let size = match self
            .get_info::<Info>(&mut [])
            .expect_err("zero sized get_info unexpectedly succeeded")
            .split()
        {
            (s, None) => return Err(s.into()),
            (_, Some(size)) => size,
        };

        // We add trailing padding because the size of a rust structure must
        // always be a multiple of alignment.
        let layout = Layout::from_size_align(size, Info::alignment())
            .unwrap()
            .pad_to_align();

        // Allocate the buffer.
        let data: *mut u8 = unsafe {
            let data = alloc::alloc(layout);
            if data.is_null() {
                return Err(Status::OUT_OF_RESOURCES.into());
            }
            data
        };

        // Get the file info using the allocated buffer for storage.
        let info = {
            let buffer = unsafe { slice::from_raw_parts_mut(data, layout.size()) };
            self.get_info::<Info>(buffer).discard_errdata()
        };

        // If an error occurred, deallocate the memory before returning.
        let info = match info {
            Ok(info) => info,
            Err(err) => {
                unsafe { alloc::dealloc(data, layout) };
                return Err(err);
            }
        };

        // Wrap the file info in a box so that it will be deallocated on
        // drop. This is valid because the memory was allocated with the
        // global allocator.
        unsafe { Ok(Box::from_raw(info)) }
    }

    /// Returns if the underlying file is a regular file.
    /// The result is an error if the underlying file was already closed or deleted.
    ///
    /// UEFI file system protocol only knows "regular files" and "directories".
    fn is_regular_file(&self) -> Result<bool>;

    /// Returns if the underlying file is a directory.
    /// The result is an error if the underlying file was already closed or deleted.
    ///
    /// UEFI file system protocol only knows "regular files" and "directories".
    fn is_directory(&self) -> Result<bool>;
}

// Internal File helper methods to access the funciton pointer table.
trait FileInternal: File {
    fn imp(&mut self) -> &mut FileImpl {
        unsafe { &mut *self.handle().0 }
    }
}

impl<T: File> FileInternal for T {}

/// An opaque handle to some contiguous block of data on a volume.
///
/// A `FileHandle` is just a wrapper around a UEFI file handle. Under the hood, it can either be a
/// `RegularFile` or a `Directory`; use the `into_type()` or the unsafe
/// `{RegularFile, Directory}::new()` methods to perform the conversion.
///
/// Dropping this structure will result in the file handle being closed.
#[repr(transparent)]
pub struct FileHandle(*mut FileImpl);

impl FileHandle {
    pub(super) unsafe fn new(ptr: *mut FileImpl) -> Self {
        Self(ptr)
    }

    /// Converts `File` into a more specific subtype based on if it is a
    /// directory or not. Wrapper around [Self::is_regular_file].
    pub fn into_type(self) -> Result<FileType> {
        use FileType::*;

        self.is_regular_file().map(|is_file| {
            if is_file {
                unsafe { Regular(RegularFile::new(self)) }
            } else {
                unsafe { Dir(Directory::new(self)) }
            }
        })
    }

    /// If the handle represents a directory, convert it into a
    /// [`Directory`]. Otherwise returns `None`.
    pub fn into_directory(self) -> Option<Directory> {
        if let Ok(FileType::Dir(dir)) = self.into_type() {
            Some(dir)
        } else {
            None
        }
    }

    /// If the handle represents a regular file, convert it into a
    /// [`RegularFile`]. Otherwise returns `None`.
    pub fn into_regular_file(self) -> Option<RegularFile> {
        if let Ok(FileType::Regular(regular)) = self.into_type() {
            Some(regular)
        } else {
            None
        }
    }
}

impl File for FileHandle {
    #[inline]
    fn handle(&mut self) -> &mut FileHandle {
        self
    }

    fn is_regular_file(&self) -> Result<bool> {
        let this = unsafe { self.0.as_mut().unwrap() };

        // - get_position fails with EFI_UNSUPPORTED on directories
        // - result is an error if the underlying file was already closed or deleted.
        let mut pos = 0;
        match (this.get_position)(this, &mut pos) {
            Status::SUCCESS => Ok(true),
            Status::UNSUPPORTED => Ok(false),
            s => Err(s.into()),
        }
    }

    fn is_directory(&self) -> Result<bool> {
        self.is_regular_file().map(|b| !b)
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        let result: Result = (self.imp().close)(self.imp()).into();
        // The spec says this always succeeds.
        result.expect("Failed to close file");
    }
}

/// The function pointer table for the File protocol.
#[repr(C)]
pub(super) struct FileImpl {
    revision: u64,
    open: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        new_handle: &mut *mut FileImpl,
        filename: *const Char16,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Status,
    close: extern "efiapi" fn(this: &mut FileImpl) -> Status,
    delete: extern "efiapi" fn(this: &mut FileImpl) -> Status,
    read: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    write: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        buffer_size: &mut usize,
        buffer: *const u8,
    ) -> Status,
    get_position: extern "efiapi" fn(this: &mut FileImpl, position: &mut u64) -> Status,
    set_position: extern "efiapi" fn(this: &mut FileImpl, position: u64) -> Status,
    get_info: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    set_info: unsafe extern "efiapi" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
    flush: extern "efiapi" fn(this: &mut FileImpl) -> Status,
}

/// Disambiguates the file type. Returned by `File::into_type()`.
pub enum FileType {
    /// The file was a regular (data) file.
    Regular(RegularFile),
    /// The file was a directory.
    Dir(Directory),
}

/// Usage flags describing what is possible to do with the file.
///
/// SAFETY: Using a repr(C) enum is safe here because this type is only sent to
///         the UEFI implementation, and never received from it.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    #[repr(transparent)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::runtime::Time;
    use crate::{CString16, Identify};
    use alloc_api::vec;

    // Test `get_boxed_info` by setting up a fake file, which is mostly
    // just function pointers. Most of the functions can be empty, only
    // get_info is actually implemented to return useful data.
    #[test]
    fn test_get_boxed_info() {
        let mut file_impl = FileImpl {
            revision: 0,
            open: stub_open,
            close: stub_close,
            delete: stub_delete,
            read: stub_read,
            write: stub_write,
            get_position: stub_get_position,
            set_position: stub_set_position,
            get_info: stub_get_info,
            set_info: stub_set_info,
            flush: stub_flush,
        };
        let file_handle = FileHandle(&mut file_impl);

        let mut file = unsafe { RegularFile::new(file_handle) };
        let info = file.get_boxed_info::<FileInfo>().unwrap();
        assert_eq!(info.file_size(), 123);
        assert_eq!(info.file_name(), CString16::try_from("test_file").unwrap());
    }

    extern "efiapi" fn stub_get_info(
        _this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status {
        assert_eq!(*information_type, FileInfo::GUID);

        // Use a temporary buffer to get some file info, then copy that
        // data to the output buffer.
        let mut tmp = vec![0; 128];
        let file_size = 123;
        let physical_size = 456;
        let time = Time::invalid();
        let info = FileInfo::new(
            &mut tmp,
            file_size,
            physical_size,
            time,
            time,
            time,
            FileAttribute::empty(),
            &CString16::try_from("test_file").unwrap(),
        )
        .unwrap();
        let required_size = mem::size_of_val(info);
        if *buffer_size < required_size {
            *buffer_size = required_size;
            Status::BUFFER_TOO_SMALL
        } else {
            unsafe {
                ptr::copy_nonoverlapping((info as *const FileInfo).cast(), buffer, required_size);
            }
            *buffer_size = required_size;
            Status::SUCCESS
        }
    }

    extern "efiapi" fn stub_open(
        _this: &mut FileImpl,
        _new_handle: &mut *mut FileImpl,
        _filename: *const Char16,
        _open_mode: FileMode,
        _attributes: FileAttribute,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_close(_this: &mut FileImpl) -> Status {
        Status::SUCCESS
    }

    extern "efiapi" fn stub_delete(_this: &mut FileImpl) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_read(
        _this: &mut FileImpl,
        _buffer_size: &mut usize,
        _buffer: *mut u8,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_write(
        _this: &mut FileImpl,
        _buffer_size: &mut usize,
        _buffer: *const u8,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_get_position(_this: &mut FileImpl, _position: &mut u64) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_set_position(_this: &mut FileImpl, _position: u64) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_set_info(
        _this: &mut FileImpl,
        _information_type: &Guid,
        _buffer_size: usize,
        _buffer: *const c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_flush(_this: &mut FileImpl) -> Status {
        Status::UNSUPPORTED
    }
}
