// SPDX-License-Identifier: MIT OR Apache-2.0

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

use crate::{CStr16, Result, Status, StatusExt};
use core::ffi::c_void;
use core::fmt::Debug;
use core::{mem, ptr};
use uefi_raw::protocol::file_system::FileProtocolV1;

#[cfg(all(feature = "unstable", feature = "alloc"))]
use {alloc::alloc::Global, core::alloc::Allocator};

#[cfg(feature = "alloc")]
use {crate::mem::make_boxed, alloc::boxed::Box};

pub use dir::Directory;
pub use info::{
    FileInfo, FileInfoCreationError, FileProtocolInfo, FileSystemInfo, FileSystemVolumeLabel,
    FromUefi,
};
pub use regular::RegularFile;
pub use uefi_raw::protocol::file_system::FileAttribute;

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
    ///
    /// See section `EFI_FILE_PROTOCOL.Open()` in the UEFI Specification for more details.
    /// Note that [`INVALID_PARAMETER`] is not listed in the specification as one of the
    /// errors returned by this function, but some implementations (such as EDK2) perform
    /// additional validation and may return that status for invalid inputs.
    ///
    /// [`INVALID_PARAMETER`]: uefi::Status::INVALID_PARAMETER
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::NOT_FOUND`]
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::MEDIA_CHANGED`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::WRITE_PROTECTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::VOLUME_FULL`]
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
                filename.as_ptr().cast(),
                uefi_raw::protocol::file_system::FileMode::from_bits_truncate(open_mode as u64),
                attributes,
            )
        }
        .to_result_with_val(|| unsafe { FileHandle::new(ptr) })
    }

    /// Close this file handle. Same as dropping this structure.
    fn close(self) {}

    /// Closes and deletes this file
    ///
    /// # Warnings
    ///
    /// See section `EFI_FILE_PROTOCOL.Delete()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::WARN_DELETE_FAILURE`]
    fn delete(mut self) -> Result {
        let result = unsafe { (self.imp().delete)(self.imp()) }.to_result();
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
    ///
    /// See section `EFI_FILE_PROTOCOL.GetInfo()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::UNSUPPORTED`]
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::BUFFER_TOO_SMALL`]
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
                buffer.as_mut_ptr().cast(),
            )
        }
        .to_result_with(
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
    ///
    /// See section `EFI_FILE_PROTOCOL.SetInfo()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::UNSUPPORTED`]
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::WRITE_PROTECTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::VOLUME_FULL`]
    /// * [`uefi::Status::BAD_BUFFER_SIZE`]
    fn set_info<Info: FileProtocolInfo + ?Sized>(&mut self, info: &Info) -> Result {
        let info_ptr = ptr::from_ref(info).cast::<c_void>();
        let info_size = size_of_val(info);
        unsafe { (self.imp().set_info)(self.imp(), &Info::GUID, info_size, info_ptr).to_result() }
    }

    /// Flushes all modified data associated with the file handle to the device
    ///
    /// # Errors
    ///
    /// See section `EFI_FILE_PROTOCOL.Flush()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::WRITE_PROTECTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::VOLUME_FULL`]
    fn flush(&mut self) -> Result {
        unsafe { (self.imp().flush)(self.imp()) }.to_result()
    }

    /// Read the dynamically allocated info for a file.
    #[cfg(feature = "alloc")]
    fn get_boxed_info<Info: FileProtocolInfo + ?Sized + Debug>(&mut self) -> Result<Box<Info>> {
        let fetch_data_fn = |buf| self.get_info::<Info>(buf);
        #[cfg(not(feature = "unstable"))]
        let file_info = make_boxed::<Info, _>(fetch_data_fn)?;
        #[cfg(feature = "unstable")]
        let file_info = make_boxed::<Info, _, _>(fetch_data_fn, Global)?;
        Ok(file_info)
    }

    /// Read the dynamically allocated info for a file.
    #[cfg(all(feature = "unstable", feature = "alloc"))]
    fn get_boxed_info_in<Info: FileProtocolInfo + ?Sized + Debug, A: Allocator>(
        &mut self,
        allocator: A,
    ) -> Result<Box<Info>> {
        let fetch_data_fn = |buf| self.get_info::<Info>(buf);
        let file_info = make_boxed::<Info, _, A>(fetch_data_fn, allocator)?;
        Ok(file_info)
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

// Internal File helper methods to access the function pointer table.
trait FileInternal: File {
    fn imp(&mut self) -> &mut FileProtocolV1 {
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
#[derive(Debug)]
pub struct FileHandle(*mut FileProtocolV1);

impl FileHandle {
    pub(super) const unsafe fn new(ptr: *mut FileProtocolV1) -> Self {
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
    #[must_use]
    pub fn into_directory(self) -> Option<Directory> {
        if let Ok(FileType::Dir(dir)) = self.into_type() {
            Some(dir)
        } else {
            None
        }
    }

    /// If the handle represents a regular file, convert it into a
    /// [`RegularFile`]. Otherwise returns `None`.
    #[must_use]
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
        match unsafe { (this.get_position)(this, &mut pos) } {
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
        let result: Result = unsafe { (self.imp().close)(self.imp()) }.to_result();
        // The spec says this always succeeds.
        result.expect("Failed to close file");
    }
}

/// Disambiguate the file type. Returned by `File::into_type()`.
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Time;
    use crate::{CString16, Guid, Identify};
    use ::alloc::vec;
    use uefi_raw::protocol::file_system::FileProtocolRevision;

    // Test `get_boxed_info` by setting up a fake file, which is mostly
    // just function pointers. Most of the functions can be empty, only
    // get_info is actually implemented to return useful data.
    #[test]
    fn test_get_boxed_info() {
        let mut file_impl = FileProtocolV1 {
            revision: FileProtocolRevision::REVISION_1,
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

    unsafe extern "efiapi" fn stub_get_info(
        _this: *mut FileProtocolV1,
        information_type: *const Guid,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status {
        assert_eq!(unsafe { *information_type }, FileInfo::GUID);

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
        let required_size = size_of_val(info);
        if unsafe { *buffer_size } < required_size {
            unsafe {
                *buffer_size = required_size;
            }
            Status::BUFFER_TOO_SMALL
        } else {
            unsafe {
                ptr::copy_nonoverlapping((info as *const FileInfo).cast(), buffer, required_size);
            }
            unsafe {
                *buffer_size = required_size;
            }
            Status::SUCCESS
        }
    }

    extern "efiapi" fn stub_open(
        _this: *mut FileProtocolV1,
        _new_handle: *mut *mut FileProtocolV1,
        _filename: *const uefi_raw::Char16,
        _open_mode: uefi_raw::protocol::file_system::FileMode,
        _attributes: FileAttribute,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_close(_this: *mut FileProtocolV1) -> Status {
        Status::SUCCESS
    }

    extern "efiapi" fn stub_delete(_this: *mut FileProtocolV1) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_read(
        _this: *mut FileProtocolV1,
        _buffer_size: *mut usize,
        _buffer: *mut c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_write(
        _this: *mut FileProtocolV1,
        _buffer_size: *mut usize,
        _buffer: *const c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_get_position(
        _this: *const FileProtocolV1,
        _position: *mut u64,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_set_position(_this: *mut FileProtocolV1, _position: u64) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_set_info(
        _this: *mut FileProtocolV1,
        _information_type: *const Guid,
        _buffer_size: usize,
        _buffer: *const c_void,
    ) -> Status {
        Status::UNSUPPORTED
    }

    extern "efiapi" fn stub_flush(_this: *mut FileProtocolV1) -> Status {
        Status::UNSUPPORTED
    }
}
