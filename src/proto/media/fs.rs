//! File system support protocols.

use super::file::{Directory, File, FileImpl};
use crate::{Result, Status};
use core::ptr;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
#[repr(C)]
#[derive(Identify, Protocol)]
#[unsafe_guid(0x0964_e5b22, 0x6459, 0x11d2, 0x8e39, 0x00a0_c969_723b)]
pub struct SimpleFileSystem {
    revision: u64,
    open_volume: extern "win64" fn(this: &mut SimpleFileSystem, root: &mut *mut FileImpl) -> Status,
}

impl SimpleFileSystem {
    /// Open the root directory on a volume.
    ///
    /// # Errors
    /// * `uefi::Status::UNSUPPORTED` - The volume does not support the requested filesystem type
    /// * `uefi::Status::NO_MEDIA` - The device has no media
    /// * `uefi::Status::DEVICE_ERROR` - The device reported an error
    /// * `uefi::Status::VOLUME_CORRUPTED` - The file system structures are corrupted
    /// * `uefi::Status::ACCESS_DENIED` - The service denied access to the file
    /// * `uefi::Status::OUT_OF_RESOURCES` - The volume was not opened
    /// * `uefi::Status::MEDIA_CHANGED` - The device has a different medium in it
    pub fn open_volume(&mut self) -> Result<Directory> {
        let mut ptr = ptr::null_mut();
        (self.open_volume)(self, &mut ptr)
            .into_with(|| unsafe { Directory::from_file(File::new(ptr)) })
    }
}
