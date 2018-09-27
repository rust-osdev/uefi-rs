use crate::{Result, Status};

use super::file::File;

/// Allows access to a FAT-12/16/32 file system.
///
/// This interface is implemented by some storage devices
/// to allow file access to the contained file systems.
#[repr(C)]
pub struct SimpleFileSystem {
    revision: u64,
    open_volume: extern "win64" fn(this: &mut SimpleFileSystem, root: &mut usize) -> Status,
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
    pub fn open_volume(&mut self) -> Result<File> {
        let mut ptr = 0usize;
        (self.open_volume)(self, &mut ptr).into_with(|| File::new(ptr))
    }
}

impl_proto! {
    protocol SimpleFileSystem {
        GUID = 0x0964e5b22,0x6459,0x11d2,[0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b];
    }
}
