//! `LoadedImage` protocol.

use crate::{
    data_types::FromSliceWithNulError,
    proto::device_path::{DevicePath, FfiDevicePath},
    proto::Protocol,
    table::boot::MemoryType,
    unsafe_guid, CStr16, Handle, Status,
};
use core::{ffi::c_void, mem, slice};

/// The LoadedImage protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("5b1b31a1-9562-11d2-8e3f-00a0c969723b")]
#[derive(Protocol)]
pub struct LoadedImage {
    revision: u32,
    parent_handle: Handle,
    system_table: *const c_void,

    // Source location of the image
    device_handle: Handle,
    file_path: *const FfiDevicePath,
    _reserved: *const c_void,

    // Image load options
    load_options_size: u32,
    load_options: *const u8,

    // Location where image was loaded
    image_base: *const c_void,
    image_size: u64,
    image_code_type: MemoryType,
    image_data_type: MemoryType,
    /// This is a callback that a loaded image can use to do cleanup. It is called by the
    /// `UnloadImage` boot service.
    unload: extern "efiapi" fn(image_handle: Handle) -> Status,
}

/// Errors that can be raised during parsing of the load options.
#[derive(Debug)]
pub enum LoadOptionsError {
    /// Load options are not set.
    NotSet,

    /// The start and/or length of the load options is not [`u16`]-aligned.
    NotAligned,

    /// Not a valid null-terminated UCS-2 string.
    InvalidString(FromSliceWithNulError),
}

impl LoadedImage {
    /// Returns a handle to the storage device on which the image is located.
    pub fn device(&self) -> Handle {
        self.device_handle
    }

    /// Get a reference to the `file_path`.
    ///
    /// Return `None` if the pointer to the file path portion specific to
    /// DeviceHandle that the EFI Image was loaded from is null.
    pub fn file_path(&self) -> Option<&DevicePath> {
        if self.file_path.is_null() {
            None
        } else {
            unsafe { Some(DevicePath::from_ffi_ptr(self.file_path)) }
        }
    }

    /// Get the load options of the image as a [`&CStr16`].
    ///
    /// Load options are typically used to pass command-line options as
    /// a null-terminated UCS-2 string. This format is not required
    /// though; use [`load_options_as_bytes`] to access the raw bytes.
    ///
    /// [`&CStr16`]: `CStr16`
    /// [`load_options_as_bytes`]: `Self::load_options_as_bytes`
    pub fn load_options_as_cstr16(&self) -> Result<&CStr16, LoadOptionsError> {
        let load_options_size = usize::try_from(self.load_options_size).unwrap();

        if self.load_options.is_null() {
            Err(LoadOptionsError::NotSet)
        } else if (load_options_size % mem::size_of::<u16>() != 0)
            || (((self.load_options as usize) % mem::align_of::<u16>()) != 0)
        {
            Err(LoadOptionsError::NotAligned)
        } else {
            let s = unsafe {
                slice::from_raw_parts(
                    self.load_options.cast::<u16>(),
                    load_options_size / mem::size_of::<u16>(),
                )
            };
            CStr16::from_u16_with_nul(s).map_err(LoadOptionsError::InvalidString)
        }
    }

    /// Get the load options of the image as raw bytes.
    ///
    /// UEFI allows arbitrary binary data in load options, but typically
    /// the data is a null-terminated UCS-2 string. Use
    /// [`load_options_as_cstr16`] to more conveniently access the load
    /// options as a string.
    ///
    /// Returns `None` if load options are not set.
    ///
    /// [`load_options_as_cstr16`]: `Self::load_options_as_cstr16`
    pub fn load_options_as_bytes(&self) -> Option<&[u8]> {
        if self.load_options.is_null() {
            None
        } else {
            unsafe {
                Some(slice::from_raw_parts(
                    self.load_options,
                    usize::try_from(self.load_options_size).unwrap(),
                ))
            }
        }
    }

    /// Set the image data address and size.
    ///
    /// This is useful in the following scenario:
    /// 1. Secure boot is enabled, so images loaded with `LoadImage` must be
    ///    signed with an appropriate key known to the firmware.
    /// 2. The bootloader has its own key embedded, and uses that key to
    ///    verify the next stage. This key is not known to the firmware, so
    ///    the next stage's image can't be loaded with `LoadImage`.
    /// 3. Since image handles are created by `LoadImage`, which we can't
    ///    call, we have to make use of an existing image handle -- the one
    ///    passed into the bootloader's entry function. By modifying that
    ///    image handle (after appropriately verifying the signature of the
    ///    new data), we can repurpose the image handle for the next stage.
    ///
    /// See [shim] for an example of this scenario.
    ///
    /// # Safety
    ///
    /// This function takes `data` as a raw pointer because the data is not
    /// owned by `LoadedImage`. The caller must ensure that the memory lives
    /// long enough.
    ///
    /// [shim]: https://github.com/rhboot/shim/blob/4d64389c6c941d21548b06423b8131c872e3c3c7/pe.c#L1143
    pub unsafe fn set_image(&mut self, data: *const c_void, size: u64) {
        self.image_base = data;
        self.image_size = size;
    }

    /// Set the load options for the image. This can be used prior to
    /// calling `BootServices.start_image` to control the command line
    /// passed to the image.
    ///
    /// `size` is in bytes.
    ///
    /// # Safety
    ///
    /// This function takes `options` as a raw pointer because the
    /// load options data is not owned by `LoadedImage`. The caller
    /// must ensure that the memory lives long enough.
    pub unsafe fn set_load_options(&mut self, options: *const u8, size: u32) {
        self.load_options = options;
        self.load_options_size = size;
    }

    /// Returns the base address and the size in bytes of the loaded image.
    pub fn info(&self) -> (*const c_void, u64) {
        (self.image_base, self.image_size)
    }
}
