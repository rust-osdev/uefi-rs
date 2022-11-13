use super::{File, FileHandle, FileInfo, FromUefi, RegularFile};
use crate::data_types::Align;
use crate::Result;
use core::ffi::c_void;

/// A `FileHandle` that is also a directory.
///
/// Use `File::into_type` or `Directory::new` to create a `Directory`. In
/// addition to supporting the normal `File` operations, `Directory`
/// supports iterating over its contained files.
#[repr(transparent)]
pub struct Directory(RegularFile);

impl Directory {
    /// Coverts a `FileHandle` into a `Directory` without checking the file type.
    /// # Safety
    /// This function should only be called on files which ARE directories,
    /// doing otherwise is unsafe.
    pub unsafe fn new(handle: FileHandle) -> Self {
        Self(RegularFile::new(handle))
    }

    /// Read the next directory entry
    ///
    /// Try to read the next directory entry into `buffer`. If the buffer is too small, report the
    /// required buffer size as part of the error. If there are no more directory entries, return
    /// an empty optional.
    ///
    /// The input buffer must be correctly aligned for a `FileInfo`. You can query the required
    /// alignment through the `Align` trait (`<FileInfo as Align>::alignment()`).
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
    ///                                      the required buffer size is provided into the error.
    pub fn read_entry<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<&'buf mut FileInfo>, Option<usize>> {
        // Make sure that the storage is properly aligned
        FileInfo::assert_aligned(buffer);

        // Read the directory entry into the aligned storage
        self.0.read(buffer).map(|size| {
            if size != 0 {
                unsafe { Some(FileInfo::from_uefi(buffer.as_mut_ptr().cast::<c_void>())) }
            } else {
                None
            }
        })
    }

    /// Start over the process of enumerating directory entries
    pub fn reset_entry_readout(&mut self) -> Result {
        self.0.set_position(0)
    }
}

impl File for Directory {
    #[inline]
    fn handle(&mut self) -> &mut FileHandle {
        self.0.handle()
    }

    fn is_regular_file(&self) -> Result<bool> {
        Ok(false)
    }

    fn is_directory(&self) -> Result<bool> {
        Ok(true)
    }
}
