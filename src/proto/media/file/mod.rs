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

use crate::prelude::*;
use crate::{CStr16, Result, Status};
#[cfg(feature = "exts")]
use alloc_api::{alloc::Layout, boxed::Box};
use bitflags::bitflags;
use core::ffi::c_void;
use core::mem;
use core::ptr;
use uefi_sys::EFI_FILE_PROTOCOL;

pub use self::info::{
    FileInfo, FileProtocolInfo, FileSystemInfo, FileSystemVolumeLabel, FromUefi,
    NamedFileProtocolInfo,
};
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
        filename: &str,
        open_mode: FileMode,
        attributes: FileAttribute,
    ) -> Result<FileHandle> {
        const BUF_SIZE: usize = 255;
        if filename.len() > BUF_SIZE {
            Err(Status::INVALID_PARAMETER.into())
        } else {
            let mut buf = [0u16; BUF_SIZE + 1];
            let mut ptr = ptr::null_mut();

            let len = ucs2::encode(filename, &mut buf)?;
            let filename = unsafe { CStr16::from_u16_with_nul_unchecked(&buf[..=len]) };

            Status::from_raw_api(unsafe {
                self.imp().raw.Open.unwrap()(
                    &mut self.imp().raw,
                    &mut ptr,
                    filename.as_ptr() as *mut u16,
                    open_mode as _,
                    attributes.bits,
                )
            })
            .into_with_val(|| unsafe { FileHandle::new(ptr as *mut _ as *mut _) })
        }
    }

    /// Close this file handle. Same as dropping this structure.
    fn close(self) {}

    /// Closes and deletes this file
    ///
    /// # Warnings
    /// * `uefi::Status::WARN_DELETE_FAILURE` The file was closed, but deletion failed
    fn delete(mut self) -> Result {
        let result = Status::from_raw_api(unsafe {
            self.imp().raw.Delete.unwrap()(self.imp() as *mut _ as *mut _)
        })
        .into();
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
        Status::from_raw_api(unsafe {
            self.imp().raw.GetInfo.unwrap()(
                self.imp() as *mut _ as *mut _,
                &Info::UNIQUE_GUID as *const _ as *mut _,
                &mut buffer_size as *mut _ as *mut _,
                buffer.as_mut_ptr() as *mut _,
            )
        })
        .into_with(
            || unsafe { Info::from_uefi(buffer.as_ptr() as *mut c_void) },
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
        let info_ptr = info as *const Info as *const c_void;
        let info_size = mem::size_of_val(&info);
        Status(unsafe {
            self.imp().raw.SetInfo.unwrap()(
                self.imp() as *mut _ as *mut _,
                &Info::UNIQUE_GUID as *const _ as *mut _,
                info_size as _,
                info_ptr as *const _ as *mut _,
            )
        } as _)
        .into()
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
        Status(unsafe { self.imp().raw.Flush.unwrap()(self.imp() as *mut _ as *mut _) } as _).into()
    }

    #[cfg(feature = "exts")]
    /// Get the dynamically allocated info for a file
    fn get_boxed_info<Info: FileProtocolInfo + ?Sized>(&mut self) -> Result<Box<Info>> {
        // Initially try get_info with an empty array, this should always fail
        // as all Info types at least need room for a null-terminator.
        let size = match self
            .get_info::<Info>(&mut [])
            .expect_error("zero sized get_info unexpectedly succeeded")
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
        let mut buffer = crate::exts::allocate_buffer(layout);
        let buffer_start = buffer.as_ptr();

        let info = self
            .get_info(&mut buffer)
            .discard_errdata()?
            .map(|info_ref| {
                // This operation is safe because info uses the exact memory
                // of the provied buffer (so no memory is leaked), and the box
                // is created if and only if buffer is leaked (so no memory can
                // ever be freed twice).

                assert_eq!(mem::size_of_val(info_ref), layout.size());
                assert_eq!(info_ref as *const Info as *const u8, buffer_start);
                unsafe { Box::from_raw(info_ref as *mut _) }
            });
        mem::forget(buffer);

        Ok(info)
    }
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
    /// directory or not. It does this via a call to `get_position`.
    pub fn into_type(mut self) -> Result<FileType> {
        use FileType::*;

        // get_position fails with EFI_UNSUPPORTED on directories
        let mut pos = 0;
        match Status(unsafe {
            self.imp().raw.GetPosition.unwrap()(self.imp() as *mut _ as *mut _, &mut pos)
        } as _)
        {
            Status::SUCCESS => unsafe { Ok(Regular(RegularFile::new(self)).into()) },
            Status::UNSUPPORTED => unsafe { Ok(Dir(Directory::new(self)).into()) },
            s => Err(s.into()),
        }
    }
}

impl File for FileHandle {
    #[inline]
    fn handle(&mut self) -> &mut FileHandle {
        self
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        let result: Result = Status::from_raw_api(unsafe {
            self.imp().raw.Close.unwrap()(self.imp() as *mut _ as *mut _)
        })
        .into();
        // The spec says this always succeeds.
        result.expect_success("Failed to close file");
    }
}

/// The function pointer table for the File protocol.
#[repr(C)]
pub(super) struct FileImpl {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_FILE_PROTOCOL,
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
