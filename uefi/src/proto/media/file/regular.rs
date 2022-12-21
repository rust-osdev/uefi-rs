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
    #[must_use]
    pub unsafe fn new(handle: FileHandle) -> Self {
        Self(handle)
    }

    /// Read data from file.
    ///
    /// Try to read as much as possible into `buffer`. Returns the number of bytes that were
    /// actually read.
    ///
    /// # Arguments
    /// * `buffer`  The target buffer of the read operation
    ///
    /// # Errors
    ///
    /// See section `EFI_FILE_PROTOCOL.Read()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::BUFFER_TOO_SMALL`]
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Option<usize>> {
        let mut buffer_size = buffer.len();
        let status =
            unsafe { (self.imp().read)(self.imp(), &mut buffer_size, buffer.as_mut_ptr()) };

        status.into_with(
            || buffer_size,
            |s| {
                if s == Status::BUFFER_TOO_SMALL {
                    // `buffer_size` was updated to the required buffer size by the underlying read
                    // function.
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
    ///
    /// See section `EFI_FILE_PROTOCOL.Write()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::WRITE_PROTECTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::VOLUME_FULL`]
    pub fn write(&mut self, buffer: &[u8]) -> Result<(), usize> {
        let mut buffer_size = buffer.len();
        unsafe { (self.imp().write)(self.imp(), &mut buffer_size, buffer.as_ptr()) }
            .into_with_err(|_| buffer_size)
    }

    /// Get the file's current position
    ///
    /// # Errors
    ///
    /// See section `EFI_FILE_PROTOCOL.GetPosition()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::DEVICE_ERROR`]
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
    ///
    /// See section `EFI_FILE_PROTOCOL.SetPosition()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::DEVICE_ERROR`]
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
