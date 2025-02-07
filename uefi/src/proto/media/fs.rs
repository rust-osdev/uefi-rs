// SPDX-License-Identifier: MIT OR Apache-2.0

//! File system support protocols.

use super::file::{Directory, FileHandle};
use crate::proto::unsafe_protocol;
use crate::{Result, StatusExt};
use core::ptr;
use uefi_raw::protocol::file_system::SimpleFileSystemProtocol;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
///
/// # Accessing `SimpleFileSystem` protocol
///
/// Use [`boot::get_image_file_system`] to retrieve the `SimpleFileSystem`
/// protocol associated with a given image handle.
///
/// See the [`boot`] documentation for more details of how to open a protocol.
///
/// [`boot::get_image_file_system`]: crate::boot::get_image_file_system
/// [`boot`]: crate::boot#accessing-protocols
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(SimpleFileSystemProtocol::GUID)]
pub struct SimpleFileSystem(SimpleFileSystemProtocol);

impl SimpleFileSystem {
    /// Open the root directory on a volume.
    ///
    /// # Errors
    ///
    /// See section `EFI_SIMPLE_FILE_SYSTEM_PROTOCOL.OpenVolume()` in the UEFI Specification
    /// for more details.
    ///
    /// If you can't find the function definition, try searching for
    /// `EFI_SIMPLE_FILE SYSTEM_PROTOCOL.OpenVolume()` (this has a space in between FILE and
    /// SYSTEM; it could be a typo in the UEFI spec).
    ///
    /// * [`uefi::Status::UNSUPPORTED`]
    /// * [`uefi::Status::NO_MEDIA`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    /// * [`uefi::Status::VOLUME_CORRUPTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::MEDIA_CHANGED`]
    pub fn open_volume(&mut self) -> Result<Directory> {
        let mut ptr = ptr::null_mut();
        unsafe { (self.0.open_volume)(&mut self.0, &mut ptr) }
            .to_result_with_val(|| unsafe { Directory::new(FileHandle::new(ptr.cast())) })
    }
}
