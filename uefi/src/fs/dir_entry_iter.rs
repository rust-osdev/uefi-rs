//! Module for directory iteration. See [`UefiDirectoryIter`].

use super::*;
use crate::Result;
use alloc::boxed::Box;

/// Iterates over the entries of an UEFI directory. It returns boxed values of
/// type [`UefiFileInfo`].
#[derive(Debug)]
pub struct UefiDirectoryIter(UefiDirectoryHandle);

impl UefiDirectoryIter {
    /// Constructor.
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
