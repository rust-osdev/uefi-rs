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
use core::{mem, ptr};
#[cfg(all(feature = "unstable", feature = "alloc"))]
use {alloc::alloc::Global, core::alloc::Allocator};
#[cfg(feature = "alloc")]
use {alloc::boxed::Box, uefi::mem::make_boxed};

pub use self::dir::Directory;
pub use self::info::{FileInfo, FileProtocolInfo, FileSystemInfo, FileSystemVolumeLabel, FromUefi};
pub use self::regular::RegularFile;

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
    ///
    /// See section `EFI_FILE_PROTOCOL.Delete()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::WARN_DELETE_FAILURE`]
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
        let info_ptr = (info as *const Info).cast::<c_void>();
        let info_size = mem::size_of_val(info);
        unsafe { (self.imp().set_info)(self.imp(), &Info::GUID, info_size, info_ptr).into() }
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
        (self.imp().flush)(self.imp()).into()
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
#[derive(Debug)]
pub struct FileHandle(*mut FileImpl);

impl FileHandle {
    pub(super) const unsafe fn new(ptr: *mut FileImpl) -> Self {
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
    /// # Read from Regular Files
    /// If `self` is not a directory, the function reads the requested number of bytes from the file
    /// at the file’s current position and returns them in `buffer`. If the read goes beyond the end
    /// of the file, the read length is truncated to the end of the file. The file’s current
    /// position is increased by the number of bytes returned.
    ///
    /// # Read from Directory
    /// If `self` is a directory, the function reads the directory entry at the file’s current
    /// position and returns the entry in `buffer`. If the `buffer` is not large enough to hold the
    /// current directory entry, then `EFI_BUFFER_TOO_SMALL` is returned and the current file
    /// position is not updated. `buffer_size` is set to be the size of the buffer needed to read
    /// the entry. On success, the current position is updated to the next directory entry. If there
    /// are no more directory entries, the read returns a zero-length buffer.
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
    use ::alloc::vec;

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
