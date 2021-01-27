//! File system support protocols.

use super::file::{Directory, FileHandle};
use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};
use core::ptr;
use uefi_sys::EFI_SIMPLE_FILE_SYSTEM_PROTOCOL;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
#[repr(C)]
#[unsafe_guid("964e5b22-6459-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct SimpleFileSystem {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_SIMPLE_FILE_SYSTEM_PROTOCOL,
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
        Status(unsafe { self.raw.OpenVolume.unwrap()(self as *mut _ as *mut _, &mut ptr) } as _)
            .into_with_val(|| unsafe { Directory::new(FileHandle::new(ptr as *mut _)) })
    }
}
