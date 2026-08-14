// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{File, FileHandle, FileInfo, FromUefi, RegularFile};
use crate::Result;
use crate::data_types::Align;
use core::ffi::c_void;
#[cfg(feature = "alloc")]
use {crate::mem::make_boxed, alloc::boxed::Box};

/// A `FileHandle` that is also a directory.
///
/// Use `File::into_type` or `Directory::new` to create a `Directory`. In
/// addition to supporting the normal `File` operations, `Directory`
/// supports iterating over its contained files.
#[repr(transparent)]
#[derive(Debug)]
pub struct Directory(RegularFile);

impl Directory {
    /// Converts a [`FileHandle`] without checking that it is a directory.
    ///
    /// # Safety
    ///
    /// `handle` must represent a directory.
    #[must_use]
    pub const unsafe fn new(handle: FileHandle) -> Self {
        // SAFETY: The memory is valid.
        Self(unsafe { RegularFile::new(handle) })
    }

    /// Reads the next directory entry.
    ///
    /// If `buffer` is too small, the error contains the required size. Returns
    /// `None` after the final entry.
    ///
    /// The buffer must satisfy [`FileInfo`]'s [`Align`] requirement.
    ///
    /// # Arguments
    ///
    /// - `buffer`: Destination buffer for the next directory entry.
    ///
    /// # Errors
    ///
    /// All errors come from calls to [`RegularFile::read`].
    pub fn read_entry<'buf>(
        &mut self,
        buffer: &'buf mut [u8],
    ) -> Result<Option<&'buf mut FileInfo>, Option<usize>> {
        // Make sure that the storage is properly aligned
        FileInfo::assert_aligned(buffer);

        // Read the directory entry into the aligned storage
        self.0.read_unchunked(buffer).map(|read_bytes| {
            // 0 read bytes signals that the last directory entry was read
            let last_directory_entry_read = read_bytes == 0;
            if last_directory_entry_read {
                None
            } else {
                // SAFETY: The memory is valid.
                unsafe { Some(FileInfo::from_uefi(buffer.as_mut_ptr().cast::<c_void>())) }
            }
        })
    }

    /// Reads the next directory entry into an owned allocation.
    ///
    /// This has the same behavior as [`Self::read_entry`], but discards the
    /// required-size error payload.
    #[cfg(feature = "alloc")]
    pub fn read_entry_boxed(&mut self) -> Result<Option<Box<FileInfo>>> {
        let read_entry_res = self.read_entry(&mut []);

        // If no more entries are available, return early.
        if read_entry_res == Ok(None) {
            return Ok(None);
        }

        let fetch_data_fn = |buf| {
            self.read_entry(buf)
                // this is safe, as above, we checked that there are more entries
                .map(|maybe_info: Option<&mut FileInfo>| {
                    maybe_info.expect("Should have more entries")
                })
        };
        let file_info = make_boxed::<FileInfo, _>(fetch_data_fn)?;
        Ok(Some(file_info))
    }

    /// Restarts directory entry enumeration.
    ///
    /// # Errors
    ///
    /// All errors come from calls to [`RegularFile::set_position`].
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
