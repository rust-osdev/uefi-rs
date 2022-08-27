use super::{File, FileHandle, FileInternal};
use crate::{Result, Status};

/// A `FileHandle` that is also a regular (data) file.
///
/// Use `FileHandle::into_type` or `RegularFile::new` to create a `RegularFile`.
/// In addition to supporting the normal `File` operations, `RegularFile`
/// supports direct reading and writing.
#[repr(transparent)]
pub struct RegularFile(FileHandle);

impl RegularFile {
    /// A special position used to seek to the end of a file with `set_position()`.
    pub const END_OF_FILE: u64 = core::u64::MAX;

    /// Coverts a `FileHandle` into a `RegularFile` without checking the file kind.
    /// # Safety
    /// This function should only be called on handles which ARE NOT directories,
    /// doing otherwise is unsafe.
    pub unsafe fn new(handle: FileHandle) -> Self {
        Self(handle)
    }

    /// Read data from file
    ///
    /// Try to read as much as possible into `buffer`. Returns the number of bytes that were
    /// actually read.
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error, the file was deleted,
    ///                                      or the end of the file was reached before the `read()`.
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::BUFFER_TOO_SMALL`   The buffer is too small to hold a directory entry,
    ///                                      and the required buffer size is provided as output.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Option<usize>> {
        let mut buffer_size = buffer.len();
        unsafe { (self.imp().read)(self.imp(), &mut buffer_size, buffer.as_mut_ptr()) }.into_with(
            || buffer_size,
            |s| {
                if s == Status::BUFFER_TOO_SMALL {
                    Some(buffer_size)
                } else {
                    None
                }
            },
        )
    }

    /// Write data to file
    ///
    /// Write `buffer` to file, increment the file pointer.
    ///
    /// If an error occurs, returns the number of bytes that were actually written. If no error
    /// occured, the entire buffer is guaranteed to have been written successfully.
    ///
    /// # Arguments
    /// * `buffer`  Buffer to write to file
    ///
    /// # Errors
    /// * `uefi::Status::NO_MEDIA`           The device has no media
    /// * `uefi::Status::DEVICE_ERROR`       The device reported an error or the file was deleted.
    /// * `uefi::Status::VOLUME_CORRUPTED`   The filesystem structures are corrupted
    /// * `uefi::Status::WRITE_PROTECTED`    Attempt to write to readonly file
    /// * `uefi::Status::ACCESS_DENIED`      The file was opened read only.
    /// * `uefi::Status::VOLUME_FULL`        The volume is full
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), usize> {
        let mut buffer_size = buffer.len();
        unsafe { (self.imp().write)(self.imp(), &mut buffer_size, buffer.as_ptr()) }
            .into_with_err(|_| buffer_size)
    }

    /// Get the file's current position
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to get the position of a deleted file
    pub fn get_position(&mut self) -> Result<u64> {
        let mut pos = 0u64;
        (self.imp().get_position)(self.imp(), &mut pos).into_with_val(|| pos)
    }

    /// Sets the file's current position
    ///
    /// Set the position of this file handle to the absolute position specified by `position`.
    ///
    /// Seeking past the end of the file is allowed, it will trigger file growth on the next write.
    /// Using a position of RegularFile::END_OF_FILE will seek to the end of the file.
    ///
    /// # Arguments
    /// * `position` The new absolution position of the file handle
    ///
    /// # Errors
    /// * `uefi::Status::DEVICE_ERROR`   An attempt was made to set the position of a deleted file
    pub fn set_position(&mut self, position: u64) -> Result {
        (self.imp().set_position)(self.imp(), position).into()
    }
}

impl File for RegularFile {
    #[inline]
    fn handle(&mut self) -> &mut FileHandle {
        &mut self.0
    }

    fn is_regular_file(&self) -> Result<bool> {
        Ok(true)
    }

    fn is_directory(&self) -> Result<bool> {
        Ok(false)
    }
}
