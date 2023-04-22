//! File system support protocols.

use super::file::{Directory, FileHandle, FileImpl};
use crate::proto::unsafe_protocol;
use crate::{Result, Status, StatusExt};
use core::ptr;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
///
/// # Accessing `SimpleFileSystem` protocol
///
/// Use [`BootServices::get_image_file_system`] to retrieve the `SimpleFileSystem`
/// protocol associated with a given image handle.
///
/// See the [`BootServices`] documentation for more details of how to open a protocol.
///
/// [`BootServices::get_image_file_system`]: crate::table::boot::BootServices::get_image_file_system
/// [`BootServices`]: crate::table::boot::BootServices#accessing-protocols
#[repr(C)]
#[unsafe_protocol("964e5b22-6459-11d2-8e39-00a0c969723b")]
pub struct SimpleFileSystem {
    revision: u64,
    open_volume:
        extern "efiapi" fn(this: &mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status,
}

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
        (self.open_volume)(self, &mut ptr)
            .to_result_with_val(|| unsafe { Directory::new(FileHandle::new(ptr)) })
    }
}
