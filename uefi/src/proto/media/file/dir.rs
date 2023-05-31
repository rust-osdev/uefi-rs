use super::{File, FileHandle, FileInfo, FromUefi, RegularFile};
use crate::data_types::Align;
use crate::Result;
use core::ffi::c_void;
#[cfg(feature = "alloc")]
use {crate::mem::make_boxed, alloc::boxed::Box};
#[cfg(all(feature = "unstable", feature = "alloc"))]
use {alloc::alloc::Global, core::alloc::Allocator};

/// A `FileHandle` that is also a directory.
///
/// Use `File::into_type` or `Directory::new` to create a `Directory`. In
/// addition to supporting the normal `File` operations, `Directory`
/// supports iterating over its contained files.
#[repr(transparent)]
#[derive(Debug)]
pub struct Directory(RegularFile);

impl Directory {
    /// Coverts a `FileHandle` into a `Directory` without checking the file type.
    /// # Safety
    /// This function should only be called on files which ARE directories,
    /// doing otherwise is unsafe.
    #[must_use]
    pub unsafe fn new(handle: FileHandle) -> Self {
        Self(RegularFile::new(handle))
    }

    /// Read the next directory entry.
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
                unsafe { Some(FileInfo::from_uefi(buffer.as_mut_ptr().cast::<c_void>())) }
            }
        })
    }

    /// Wrapper around [`Self::read_entry`] that returns an owned copy of the data. It has the same
    /// implications and requirements. On failure, the payload of `Err` is `()´.
    #[cfg(feature = "alloc")]
    pub fn read_entry_boxed(&mut self) -> Result<Option<Box<FileInfo>>> {
        let read_entry_res = self.read_entry(&mut []);

        // If no more entries are available, return early.
        if let Ok(None) = read_entry_res {
            return Ok(None);
        }

        let fetch_data_fn = |buf| {
            self.read_entry(buf)
                // this is safe, as above, we checked that there are more entries
                .map(|maybe_info: Option<&mut FileInfo>| {
                    maybe_info.expect("Should have more entries")
                })
        };

        #[cfg(not(feature = "unstable"))]
        let file_info = make_boxed::<FileInfo, _>(fetch_data_fn)?;

        #[cfg(feature = "unstable")]
        let file_info = make_boxed::<FileInfo, _, _>(fetch_data_fn, Global)?;

        Ok(Some(file_info))
    }

    /// Wrapper around [`Self::read_entry`] that returns an owned copy of the data. It has the same
    /// implications and requirements. On failure, the payload of `Err` is `()´.
    ///
    /// It allows to use a custom allocator via the `allocator_api` feature.
    #[cfg(all(feature = "unstable", feature = "alloc"))]
    pub fn read_entry_boxed_in<A: Allocator>(
        &mut self,
        allocator: A,
    ) -> Result<Option<Box<FileInfo>>> {
        let read_entry_res = self.read_entry(&mut []);

        // If no more entries are available, return early.
        if let Ok(None) = read_entry_res {
            return Ok(None);
        }

        let fetch_data_fn = |buf| {
            self.read_entry(buf)
                // this is safe, as above, we checked that there are more entries
                .map(|maybe_info: Option<&mut FileInfo>| {
                    maybe_info.expect("Should have more entries")
                })
        };

        let file_info = make_boxed::<FileInfo, _, A>(fetch_data_fn, allocator)?;

        Ok(Some(file_info))
    }

    /// Start over the process of enumerating directory entries
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
