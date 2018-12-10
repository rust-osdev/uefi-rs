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
use core::cmp;
use core::convert::TryInto;
use core::ffi::c_void;
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

    /// Queries some information about a file
    ///
    /// The information will be written into a user-provided buffer.
    /// If the buffer is too small, the required buffer size will be returned as part of the error.
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
    pub fn get_info<Info: FileProtocolInfo>(
        &mut self,
        buffer: &mut [u8],
    ) -> result::Result<&mut Info, (Status, usize)> {
        let mut buffer_size = buffer.len();
        match unsafe {
            (self.0.get_info)(self.0, &Info::GUID, &mut buffer_size, buffer.as_mut_ptr())
        } {
            Status::SUCCESS => Ok(unsafe { Info::from_uefi(buffer.as_ptr() as *mut c_void) }),
            Status::BUFFER_TOO_SMALL => Err((Status::BUFFER_TOO_SMALL, buffer_size)),
            other => Err((other, 0)),
        }
    }

    /// Sets some information about a file
    ///
    /// There are various restrictions on the information that may be modified using this method.
    /// The simplest one is that it is usually not possible to call it on read-only media. Further
    /// restrictions specific to a given given information type are described in the corresponding
    /// FileProtocolInfo type.
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
    pub fn set_info<Info: FileProtocolInfo>(&mut self, info: &Info) -> Result<()> {
        let info_ptr = info as *const Info as *const c_void;
        let info_size = mem::size_of_val(&info);
        unsafe { (self.0.set_info)(self.0, &Info::GUID, info_size, info_ptr).into() }
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
    get_info: unsafe extern "win64" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: &mut usize,
        buffer: *mut u8,
    ) -> Status,
    set_info: unsafe extern "win64" fn(
        this: &mut FileImpl,
        information_type: &Guid,
        buffer_size: usize,
        buffer: *const c_void,
    ) -> Status,
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
/// File::set_info() or File::set_info().
///
/// The long-winded name is needed because "FileInfo" is already taken by UEFI.
///
pub trait FileProtocolInfo: Identify {
    /// Turn an UEFI-provided pointer-to-base into a Rust-style fat reference
    unsafe fn from_uefi<'a>(ptr: *mut c_void) -> &'a mut Self;
}

/// Dynamically sized FileProtocolInfo with a header and an UCS-2 name
///
/// All struct that can currently be queried via Get/SetInfo can be described as
/// a (possibly empty) header followed by a variable-sized name.
///
/// Since such dynamic-sized types are a bit unpleasant to handle in Rust today,
/// this generic struct was created to deduplicate the relevant code.
///
/// The reason why this struct covers the whole DST, as opposed to the [Char16]
/// part only, is that pointers to DSTs are created in a rather unintuitive way
/// that is best kept centralized in one place.
///
#[repr(C)]
pub struct NamedFileProtocolInfo<Header: FileProtocolInfoHeader> {
    header: Header,
    name: [Char16],
}

/// Common properties of headers that can be used inside NamedFileProtocolInfo
///
/// The safety of NamedFileProtocolInfo relies on the following conditions:
///
/// - The GUID matches an info structure definition in the UEFI File Protocol
/// - The header type's data layout matches the UEFI specification
///
pub unsafe trait FileProtocolInfoHeader {
    /// GUID of the full NamedFileProtocolInfo
    const GUID: Guid;
}

impl<Header: FileProtocolInfoHeader> NamedFileProtocolInfo<Header> {
    /// Create a NamedFileProtocolInfo structure in user-provided storage
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 version of the "name" string.
    ///
    /// The buffer should be suitably aligned for the full data structure. If
    /// it is not, some bytes at the beginning of the buffer will not be used,
    /// resulting in a reduction of effective storage capacity.
    ///
    #[allow(clippy::cast_ptr_alignment)]
    fn new_impl<'a>(
        mut storage: &'a mut [u8],
        header: Header,
        name: &str,
    ) -> result::Result<&'a mut Self, FileInfoCreationError> {
        // Compute the degree of storage misalignment. mem::align_of does not
        // support dynamically sized types, so we must help it a bit.
        let storage_address = storage.as_ptr() as usize;
        let info_alignment = cmp::max(mem::align_of::<Header>(), mem::align_of::<Char16>());
        let storage_misalignment = storage_address % info_alignment;
        let realignment_padding = info_alignment - storage_misalignment;

        // Make sure that the storage is large enough for our needs
        let name_length_ucs2 = name.chars().count() + 1;
        let name_size = name_length_ucs2 * mem::size_of::<Char16>();
        let info_size = mem::size_of::<Header>() + name_size;
        if realignment_padding + info_size > storage.len() {
            return Err(FileInfoCreationError::InsufficientStorage(info_size));
        }

        // Work on a correctly aligned subset of the storage
        storage = &mut storage[realignment_padding..];
        debug_assert_eq!((storage.as_ptr() as usize) % info_alignment, 0);

        // Write the header at the beginning of the storage
        let header_ptr = storage.as_mut_ptr() as *mut Header;
        unsafe {
            header_ptr.write(header);
        }

        // At this point, our storage contains a correct header, followed by
        // random rubbish. It is okay to reinterpret the rubbish as Char16s
        // because 1/we are going to overwrite it and 2/Char16 does not have a
        // Drop implementation. Thus, we are now ready to build a correctly
        // sized &mut Self and go back to the realm of safe code.
        debug_assert!(!mem::needs_drop::<Char16>());
        let info_ptr = unsafe {
            slice::from_raw_parts_mut(storage.as_mut_ptr() as *mut Char16, name_length_ucs2)
                as *mut [Char16] as *mut Self
        };
        let info = unsafe { &mut *info_ptr };
        debug_assert_eq!(info.name.len(), name_length_ucs2);

        // Write down the UCS-2 name before returning the storage reference
        for (target, ch) in info.name.iter_mut().zip(name.chars()) {
            *target = ch
                .try_into()
                .map_err(|_| FileInfoCreationError::InvalidChar(ch))?;
        }
        info.name[name_length_ucs2 - 1] = NUL_16;
        Ok(info)
    }
}

unsafe impl<Header: FileProtocolInfoHeader> Identify for NamedFileProtocolInfo<Header> {
    const GUID: Guid = Header::GUID;
}

impl<Header: FileProtocolInfoHeader> FileProtocolInfo for NamedFileProtocolInfo<Header> {
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn from_uefi<'a>(raw_ptr: *mut c_void) -> &'a mut Self {
        let byte_ptr = raw_ptr as *mut u8;
        let name_ptr = byte_ptr.add(mem::size_of::<Header>()) as *mut Char16;
        let name = CStr16::from_ptr(name_ptr);
        let name_len = name.to_u16_slice_with_nul().len();
        let fat_ptr = slice::from_raw_parts_mut(raw_ptr as *mut Char16, name_len);
        let self_ptr = fat_ptr as *mut [Char16] as *mut Self;
        &mut *self_ptr
    }
}

/// Errors that can occur when creating a FileProtocolInfo
pub enum FileInfoCreationError {
    /// The provided buffer was too small to hold the FileInfo. You need at
    /// least the indicated buffer size (in bytes). Please remember that using
    /// a misaligned buffer will cause a decrease of usable storage capacity.
    InsufficientStorage(usize),

    /// The suggested file name contains invalid code points (not in UCS-2)
    InvalidChar(char),
}

/// Header for generic file information
#[repr(C)]
pub struct FileInfoHeader {
    size: u64,
    file_size: u64,
    physical_size: u64,
    create_time: Time,
    last_access_time: Time,
    modification_time: Time,
    attribute: FileAttribute,
}

unsafe impl FileProtocolInfoHeader for FileInfoHeader {
    const GUID: Guid = Guid::from_values(
        0x0957_6e92,
        0x6d3f,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );
}

/// Generic file information
///
/// The following rules apply when using this struct with set_info():
///
/// - On directories, the file size is determined by the contents of the
///   directory and cannot be changed by setting file_size. On directories,
///   file_size is ignored during a set_info().
/// - The physical_size is determined by the file_size and cannot be changed.
///   This value is ignored during a set_info() request.
/// - The FileAttribute::DIRECTORY bit cannot be changed. It must match the
///   file’s actual type.
/// - A value of zero in create_time, last_access, or modification_time causes
///   the fields to be ignored (and not updated).
/// - It is forbidden to change the name of a file to the name of another
///   existing file in the same directory.
/// - If a file is read-only, the only allowed change is to remove the read-only
///   attribute. Other changes must be carried out in a separate transaction.
///
pub type FileInfo = NamedFileProtocolInfo<FileInfoHeader>;

impl FileInfo {
    /// Create a FileInfo structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 version of the "name" string.
    ///
    /// The buffer should be suitably aligned for the full data structure. If
    /// it is not, some bytes at the beginning of the buffer will not be used,
    /// resulting in a reduction of effective storage capacity.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        storage: &'a mut [u8],
        file_size: u64,
        physical_size: u64,
        create_time: Time,
        last_access_time: Time,
        modification_time: Time,
        attribute: FileAttribute,
        file_name: &str,
    ) -> result::Result<&'a mut Self, FileInfoCreationError> {
        let header = FileInfoHeader {
            size: 0,
            file_size,
            physical_size,
            create_time,
            last_access_time,
            modification_time,
            attribute,
        };
        let info = Self::new_impl(storage, header, file_name)?;
        info.header.size = mem::size_of_val(&info) as u64;
        Ok(info)
    }

    /// File size (number of bytes stored in the file)
    pub fn file_size(&self) -> u64 {
        self.header.file_size
    }

    /// Physical space consumed by the file on the file system volume
    pub fn physical_size(&self) -> u64 {
        self.header.physical_size
    }

    /// Time when the file was created
    pub fn create_time(&self) -> &Time {
        &self.header.create_time
    }

    /// Time when the file was last accessed
    pub fn last_access_time(&self) -> &Time {
        &self.header.last_access_time
    }

    /// Time when the file's contents were last modified
    pub fn modification_time(&self) -> &Time {
        &self.header.modification_time
    }

    /// Attribute bits for the file
    pub fn attribute(&self) -> FileAttribute {
        self.header.attribute
    }

    /// Name of the file
    pub fn file_name(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(&self.name[0]) }
    }
}

/// Header for system volume information
#[repr(C)]
pub struct FileSystemInfoHeader {
    size: u64,
    read_only: bool,
    volume_size: u64,
    free_space: u64,
    block_size: u32,
}

unsafe impl FileProtocolInfoHeader for FileSystemInfoHeader {
    const GUID: Guid = Guid::from_values(
        0x0957_6e93,
        0x6d3f,
        0x11d2,
        [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
    );
}

/// System volume information
///
/// May only be obtained on the root directory's file handle.
///
/// Please note that only the system volume's volume label may be set using
/// this information structure. Consider using FileSystemVolumeLabel instead.
///
pub type FileSystemInfo = NamedFileProtocolInfo<FileSystemInfoHeader>;

impl FileSystemInfo {
    /// Create a FileSystemInfo structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 version of the "name" string.
    ///
    /// The buffer should be suitably aligned for the full data structure. If
    /// it is not, some bytes at the beginning of the buffer will not be used,
    /// resulting in a reduction of effective storage capacity.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        storage: &'a mut [u8],
        read_only: bool,
        volume_size: u64,
        free_space: u64,
        block_size: u32,
        volume_label: &str,
    ) -> result::Result<&'a mut Self, FileInfoCreationError> {
        let header = FileSystemInfoHeader {
            size: 0,
            read_only,
            volume_size,
            free_space,
            block_size,
        };
        let info = Self::new_impl(storage, header, volume_label)?;
        info.header.size = mem::size_of_val(&info) as u64;
        Ok(info)
    }

    /// Truth that the volume only supports read access
    pub fn read_only(&self) -> bool {
        self.header.read_only
    }

    /// Number of bytes managed by the file system
    pub fn volume_size(&self) -> u64 {
        self.header.volume_size
    }

    /// Number of available bytes for use by the file system
    pub fn free_space(&self) -> u64 {
        self.header.free_space
    }

    /// Nominal block size by which files are typically grown
    pub fn block_size(&self) -> u32 {
        self.header.block_size
    }

    /// Volume label
    pub fn volume_label(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(&self.name[0]) }
    }
}

/// Header for system volume label information
#[repr(C)]
pub struct FileSystemVolumeLabelHeader {}

unsafe impl FileProtocolInfoHeader for FileSystemVolumeLabelHeader {
    const GUID: Guid = Guid::from_values(
        0xdb47_d7d3,
        0xfe81,
        0x11d3,
        [0x9a, 0x35, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
    );
}

/// System volume label
///
/// May only be obtained on the root directory's file handle.
///
pub type FileSystemVolumeLabel = NamedFileProtocolInfo<FileSystemVolumeLabelHeader>;

impl FileSystemVolumeLabel {
    /// Create a FileSystemVolumeLabel structure
    ///
    /// The structure will be created in-place within the provided storage
    /// buffer. The buffer must be large enough to hold the data structure,
    /// including a null-terminated UCS-2 version of the "name" string.
    ///
    /// The buffer should be suitably aligned for the full data structure. If
    /// it is not, some bytes at the beginning of the buffer will not be used,
    /// resulting in a reduction of effective storage capacity.
    ///
    pub fn new<'a>(
        storage: &'a mut [u8],
        volume_label: &str,
    ) -> result::Result<&'a mut Self, FileInfoCreationError> {
        let header = FileSystemVolumeLabelHeader {};
        Self::new_impl(storage, header, volume_label)
    }

    /// Volume label
    pub fn volume_label(&self) -> &CStr16 {
        unsafe { CStr16::from_ptr(&self.name[0]) }
    }
}
