//! Loaded image protocol.
//!
//! This module also contains the corollary type DevicePath, which is
//! used to emulate `EFI_DEVICE_PATH_PROTOCOL`.

mod device_path;
pub use self::device_path::DevicePath;

use crate::{data_types::CStr16, proto::Protocol, unsafe_guid, Handle};
use core::str;
use uefi_sys::EFI_LOADED_IMAGE_PROTOCOL;

/// The Loaded Image protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("5b1b31a1-9562-11d2-8e3f-00a0c969723b")]
#[derive(Protocol)]
pub struct LoadedImage {
    /// Unsafe raw type extracted from EDK2
    pub raw: EFI_LOADED_IMAGE_PROTOCOL,
}

/// Errors that can be raised during parsing of the load options.
#[derive(Debug)]
pub enum LoadOptionsError {
    /// The passed buffer is not large enough to contain the load options.
    BufferTooSmall,
    /// The load options are not valid UTF-8.
    NotValidUtf8,
}

impl LoadedImage {
    /// Returns a handle to the storage device on which the image is located.
    pub fn device(&self) -> Handle {
        Handle(self.raw.DeviceHandle)
    }

    /// Get the load options of the given image. If the image was executed from the EFI shell, or from a boot
    /// option, this is the command line that was used to execute it as a string. If no options were given, this
    /// returns `Ok("")`.
    pub fn load_options<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a str, LoadOptionsError> {
        if self.raw.LoadOptions.is_null() {
            Ok("")
        } else {
            let ucs2_slice =
                unsafe { CStr16::from_ptr(self.raw.LoadOptions as *mut _).to_u16_slice() };
            let length =
                ucs2::decode(ucs2_slice, buffer).map_err(|_| LoadOptionsError::BufferTooSmall)?;
            str::from_utf8(&buffer[0..length]).map_err(|_| LoadOptionsError::NotValidUtf8)
        }
    }

    /// Returns the base address and the size in bytes of the loaded image.
    pub fn info(&self) -> (usize, u64) {
        (self.raw.ImageBase as _, self.raw.ImageSize)
    }
}
