use super::{File, FileInfo, FilesystemObject, FromUefi, RegularFile};
use crate::data_types::Align;
use crate::prelude::*;
use crate::Result;
use core::ffi::c_void;

/// A `File` that is also a directory.
///
/// Use `File::into_kind` or `File::into_directory` to create a `Directory`. In
/// addition to supporting the normal `FilesystemObject` operations, `Directory`
/// supports iterating over its contained files.
#[repr(transparent)]
pub struct Directory<'imp>(pub(super) RegularFile<'imp>);

impl Directory<'_> {
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
        self.0.read(buffer).map_inner(|size| {
            if size != 0 {
                unsafe { Some(FileInfo::from_uefi(buffer.as_mut_ptr() as *mut c_void)) }
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

impl<'imp> FilesystemObject<'imp> for Directory<'imp> {
    #[inline]
    fn file(&mut self) -> &mut File<'imp> {
        self.0.file()
    }
}
