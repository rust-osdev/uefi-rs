use super::{File, FileHandle, FileInternal};
use crate::{Error, Result, Status, StatusExt};

/// A `FileHandle` that is also a regular (data) file.
///
/// Use `FileHandle::into_type` or `RegularFile::new` to create a `RegularFile`.
/// In addition to supporting the normal `File` operations, `RegularFile`
/// supports direct reading and writing.
#[repr(transparent)]
#[derive(Debug)]
pub struct RegularFile(FileHandle);

impl RegularFile {
    /// A special position used to seek to the end of a file with `set_position()`.
    pub const END_OF_FILE: u64 = u64::MAX;

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
    ///
    /// # Quirks
    ///
    /// Some UEFI implementations have a bug where large reads will incorrectly
    /// return an error. This function avoids that bug by reading in chunks of
    /// no more than 1 MiB. This is handled internally within the function;
    /// callers can safely pass in a buffer of any size. See
    /// <https://github.com/rust-osdev/uefi-rs/issues/825> for more information.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let chunk_size = 1024 * 1024;

        read_chunked(buffer, chunk_size, |buf, buf_size| unsafe {
            (self.imp().read)(self.imp(), buf_size, buf)
        })
    }

    /// Internal method for reading without chunking. This is used to implement
    /// `Directory::read_entry`.
    pub(super) fn read_unchunked(&mut self, buffer: &mut [u8]) -> Result<usize, Option<usize>> {
        let mut buffer_size = buffer.len();
        let status =
            unsafe { (self.imp().read)(self.imp(), &mut buffer_size, buffer.as_mut_ptr()) };

        status.to_result_with(
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
    /// occurred, the entire buffer is guaranteed to have been written successfully.
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
            .to_result_with_err(|_| buffer_size)
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
        (self.imp().get_position)(self.imp(), &mut pos).to_result_with_val(|| pos)
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
        (self.imp().set_position)(self.imp(), position).to_result()
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

/// Read data into `buffer` in chunks of `chunk_size`. Reading is done by
/// calling `read`, which takes a pointer to a byte buffer and the buffer's
/// size.
///
/// See [`RegularFile::read`] for details of why reading in chunks is needed.
///
/// This separate function exists for easier unit testing.
fn read_chunked<F>(buffer: &mut [u8], chunk_size: usize, mut read: F) -> Result<usize>
where
    F: FnMut(*mut u8, &mut usize) -> Status,
{
    let mut remaining_size = buffer.len();
    let mut total_read_size = 0;
    let mut output_ptr = buffer.as_mut_ptr();

    while remaining_size > 0 {
        let requested_read_size = remaining_size.min(chunk_size);

        let mut read_size = requested_read_size;
        let status = read(output_ptr, &mut read_size);

        if status.is_success() {
            total_read_size += read_size;
            remaining_size -= read_size;
            output_ptr = unsafe { output_ptr.add(read_size) };

            // Exit the loop if there's nothing left to read.
            if read_size < requested_read_size {
                break;
            }
        } else {
            return Err(Error::new(status, ()));
        }
    }

    Ok(total_read_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::cell::RefCell;

    #[derive(Default)]
    struct TestFile {
        // Use `Rc<RefCell>` so that we can modify via an immutable ref, makes
        // the test simpler to implement.
        data: Rc<RefCell<Vec<u8>>>,
        offset: Rc<RefCell<usize>>,
    }

    impl TestFile {
        fn read(&self, buffer: *mut u8, buffer_size: &mut usize) -> Status {
            let mut offset = self.offset.borrow_mut();
            let data = self.data.borrow();

            let remaining_data_size = data.len() - *offset;
            let size_to_read = remaining_data_size.min(*buffer_size);
            unsafe { buffer.copy_from(data.as_ptr().add(*offset), size_to_read) };
            *offset += size_to_read;
            *buffer_size = size_to_read;
            Status::SUCCESS
        }

        fn reset(&self) {
            *self.data.borrow_mut() = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            *self.offset.borrow_mut() = 0;
        }
    }

    /// Test reading a regular file.
    #[test]
    fn test_file_read_chunked() {
        let file = TestFile::default();
        let read = |buf, buf_size: &mut usize| file.read(buf, buf_size);

        // Chunk size equal to the data size.
        file.reset();
        let mut buffer = [0; 10];
        assert_eq!(read_chunked(&mut buffer, 10, read), Ok(10));
        assert_eq!(buffer.as_slice(), *file.data.borrow());

        // Chunk size smaller than the data size.
        file.reset();
        let mut buffer = [0; 10];
        assert_eq!(read_chunked(&mut buffer, 2, read), Ok(10));
        assert_eq!(buffer.as_slice(), *file.data.borrow());

        // Chunk size bigger than the data size.
        file.reset();
        let mut buffer = [0; 10];
        assert_eq!(read_chunked(&mut buffer, 20, read), Ok(10));
        assert_eq!(buffer.as_slice(), *file.data.borrow());

        // Buffer smaller than the full file.
        file.reset();
        let mut buffer = [0; 4];
        assert_eq!(read_chunked(&mut buffer, 10, read), Ok(4));
        assert_eq!(buffer.as_slice(), [1, 2, 3, 4]);

        // Buffer bigger than the full file.
        file.reset();
        let mut buffer = [0; 20];
        assert_eq!(read_chunked(&mut buffer, 10, read), Ok(10));
        assert_eq!(
            buffer,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        // Empty buffer.
        file.reset();
        let mut buffer = [];
        assert_eq!(read_chunked(&mut buffer, 10, read), Ok(0));
        assert_eq!(buffer, []);

        // Empty file.
        file.reset();
        file.data.borrow_mut().clear();
        let mut buffer = [0; 10];
        assert_eq!(read_chunked(&mut buffer, 10, read), Ok(0));
        assert_eq!(buffer, [0; 10]);
    }
}
