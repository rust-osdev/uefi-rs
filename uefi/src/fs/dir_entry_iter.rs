//! Module for directory iteration. See [`UefiDirectoryIter`].

use super::*;
use crate::{CStr16, Result};
use alloc::boxed::Box;
use uefi_macros::cstr16;

/// Common skip dirs in UEFI/FAT-style file systems.
pub const COMMON_SKIP_DIRS: &[&CStr16] = &[cstr16!("."), cstr16!("..")];

/// Iterates over the entries of an UEFI directory. It returns boxed values of
/// type [`UefiFileInfo`].
///
/// Note that on UEFI/FAT-style file systems, the root dir usually doesn't
/// return the entries `.` and `..`, whereas sub directories do.
#[derive(Debug)]
pub struct UefiDirectoryIter(UefiDirectoryHandle);

impl UefiDirectoryIter {
    /// Constructor.
    #[must_use]
    pub fn new(handle: UefiDirectoryHandle) -> Self {
        Self(handle)
    }
}

impl Iterator for UefiDirectoryIter {
    type Item = Result<Box<UefiFileInfo>, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let e = self.0.read_entry_boxed();
        match e {
            // no more entries
            Ok(None) => None,
            Ok(Some(e)) => Some(Ok(e)),
            Err(e) => Some(Err(e)),
        }
    }
}
