use {Status, Result, ucs2};
use core::mem;

pub const FILE_MODE_READ    : u64 = 0x0000000000000001;
pub const FILE_MODE_WRITE   : u64 = 0x0000000000000002;
pub const FILE_MODE_CREATE  : u64 = 0x8000000000000000;

pub const FILE_READ_ONLY    : u64 = 0x0000000000000001;
pub const FILE_HIDDEN       : u64 = 0x0000000000000002;
pub const FILE_SYSTEM       : u64 = 0x0000000000000004;
pub const FILE_RESERVED     : u64 = 0x0000000000000008;
pub const FILE_DIRECTORY    : u64 = 0x0000000000000010;
pub const FILE_ARCHIVE      : u64 = 0x0000000000000020;
pub const FILE_VALID_ATTR   : u64 = 0x0000000000000037;

#[repr(C)]
pub struct File {
    revision: u64,
    open: extern "C" fn(this: &mut File, new_handle: &mut usize, filename: *const u16, open_mode: u64, attributes: u64) -> Status,
    close: extern "C" fn(this: &mut File) -> Status,
    delete: extern "C" fn(this: &mut File) -> Status,
    read: extern "C" fn(this: &mut File, buffer_size: &mut usize, buffer: *mut u8) -> Status,
    write: extern "C" fn(this: &mut File, buffer_size: &mut usize, buffer: *const u8) -> Status,
    get_position: extern "C" fn(this: &mut File, position: &mut u64) -> Status,
    set_position: extern "C" fn(this: &mut File, position: u64) -> Status,
    get_info: usize,
    set_info: usize,
    flush: extern "C" fn(this: &mut File) -> Status,
}

#[repr(C)]
pub struct SimpleFileSystem {
    revision: u64,
    open_volume: extern "C" fn(this: &mut SimpleFileSystem, root: &mut usize) -> Status, 
}

impl File {
    /// Try to open a file relative to this file/directory.
    ///
    /// # Arguments
    /// * `filename`    Path of file to open, relative to this File
    /// * `open_mode`   The mode to open the file with. Valid
    ///     combinations are FILE_MODE_READ, FILE_MODE_READ | FILE_MODE_WRITE and
    ///     FILE_MODE_READ | FILE_MODE_WRITE | FILE_MODE_CREATE
    /// * `attributes`  Only valid when FILE_MODE_CREATE is used as a mode
    /// 
    /// # Errors
    /// * `uefi::Status::NotFound`          Could not find file
    /// * `uefi::Status::NoMedia`           The device has no media
    /// * `uefi::Status::MediaChanged`      The device has a different medium in it
    /// * `uefi::Status::DeviceError`       The device reported an error
    /// * `uefi::Status::VolumeCorrupted`   The filesystem structures are corrupted
    /// * `uefi::Status::WriteProtected`    Write/Create attempted on readonly file
    /// * `uefi::Status::AccessDenied`      The service denied access to the file
    /// * `uefi::Status::OutOfResources`    Not enough resources to open file
    /// * `uefi::Status::VolumeFull`        The volume is full
    pub fn open(&mut self, filename: &str, open_mode: u64, attributes: u64) -> Result<&mut File> {
        const BUF_SIZE : usize = 128;
        if filename.len() > BUF_SIZE {
            Err(Status::InvalidParameter)
        }
        else {
            let mut buf = [0u16; BUF_SIZE+1];
            let mut ptr = 0usize;

            ucs2::encode_ucs2(filename, &mut buf)?;
            (self.open)(self, &mut ptr, buf.as_ptr(), open_mode, attributes).into_with(|| unsafe {
                &mut *(ptr as *mut File)
            })
        }
    }

    /// Close this file handle
    ///
    /// This MUST be called when you are done with the file
    pub fn close(&mut self) -> Result<()> {
        (self.close)(self).into()
    }

    /// Closes and deletes this file
    ///
    /// # Errors
    /// * `uefi::Status::WarnDeleteFailure` The file was closed, but deletion failed
    pub fn delete(&mut self) -> Result<()> {
        (self.delete)(self).into()
    }

    /// Read data from file
    ///
    /// Try to read as much as possible into `buffer`. Returns the number of bytes read
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::NoMedia`           The device has no media
    /// * `uefi::Status::DeviceError`       The device reported an error 
    /// * `uefi::Status::VolumeCorrupted`   The filesystem structures are corrupted
    pub fn read(&mut self, buffer: &mut[u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.read)(self, &mut buffer_size, buffer.as_mut_ptr()).into_with(|| buffer_size)
    }

    /// Write data to file
    ///
    /// Write `buffer` to file, increment the file pointer and return number of bytes read
    ///
    /// # Arguments
    /// * `buffer`  Buffer to write to file
    ///
    /// # Errors
    /// * `uefi::Status::NoMedia`           The device has no media
    /// * `uefi::Status::DeviceError`       The device reported an error
    /// * `uefi::Status::VolumeCorrupted`   The filesystem structures are corrupted
    /// * `uefi::Status::WriteProtected`    Attempt to write to readonly file
    /// * `uefi::Status::AccessDenied`      The file was opened read only.
    /// * `uefi::Status::VolumeFull`        The volume is full
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.write)(self, &mut buffer_size, buffer.as_ptr()).into_with(|| buffer_size)
    }

    /// Get the file's current position
    ///
    /// # Errors
    /// * `uefi::Status::DeviceError`   An attempt was made to get the position of a deleted file
    pub fn get_position(&mut self) -> Result<u64> {
        let mut pos = 0u64;
        (self.get_position)(self, &mut pos).into_with(|| pos)
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
    /// * `uefi::Status::DeviceError`   An attempt was made to set the position of a deleted file
    pub fn set_position(&mut self, position: u64) -> Result<()> {
        (self.set_position)(self, position).into()
    }

    /// Flushes all modified data associated with the file handle to the device
    ///
    /// # Errors
    /// * `uefi::Status::NoMedia`           The device has no media
    /// * `uefi::Status::DeviceError`       The device reported an error
    /// * `uefi::Status::VolumeCorrupted`   The filesystem structures are corrupted
    /// * `uefi::Status::WriteProtected`    The file or medium is write protected
    /// * `uefi::Status::AccessDenied`      The file was opened read only
    /// * `uefi::Status::VolumeFull`        The volume is full
    pub fn flush(&mut self) -> Result<()> {
        (self.flush)(self).into()
    }
}

impl SimpleFileSystem {
    /// Open the root directory on a volume
    ///
    /// # Errors
    /// * `uefi::Status::Unsupported`   The volume does not support the requested filesystem type
    /// * `uefi::Status::NoMedia`       The device has no media
    /// * `uefi::Status::DeviceError`   The device reported an error
    /// * `uefi::Status::VolumeCorrupted`   The file system structures are corrupted
    /// * `uefi::Status::AccessDenied`  The service denied access to the file
    /// * `uefi::Status::OutOfResources`    The volume was not opened
    /// * `uefi::Status::MediaChanged`  The device has a different medium in it
    pub fn open_volume(&mut self) -> Result<&mut File> {
        let mut ptr = 0usize;
        (self.open_volume)(self, &mut ptr).into_with(|| unsafe { &mut *(ptr as *mut File)})
    }
}

impl_proto! {
    protocol SimpleFileSystem {
        GUID = 0x0964e5b22,0x6459,0x11d2,[0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b];
    }
}
