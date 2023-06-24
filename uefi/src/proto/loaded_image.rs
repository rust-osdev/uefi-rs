//! `LoadedImage` protocol.

use crate::data_types::FromSliceWithNulError;
use crate::proto::device_path::DevicePath;
use crate::proto::unsafe_protocol;
use crate::table::boot::MemoryType;
use crate::util::usize_from_u32;
use crate::{CStr16, Handle, Status};
use core::ffi::c_void;
use core::{mem, slice};
use uefi_raw::protocol::loaded_image::LoadedImageProtocol;

/// The LoadedImage protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(transparent)]
#[unsafe_protocol(LoadedImageProtocol::GUID)]
pub struct LoadedImage(LoadedImageProtocol);

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
    #[must_use]
    pub fn device(&self) -> Option<Handle> {
        unsafe { Handle::from_ptr(self.0.device_handle) }
    }

    /// Get a reference to the `file_path` portion of the DeviceHandle that the
    /// EFI image was loaded from.
    ///
    /// For a full device path, consider using the [`LoadedImageDevicePath`]
    /// protocol.
    ///
    /// Returns `None` if `file_path` is null.
    ///
    /// [`LoadedImageDevicePath`]: crate::proto::device_path::LoadedImageDevicePath
    #[must_use]
    pub fn file_path(&self) -> Option<&DevicePath> {
        if self.0.file_path.is_null() {
            None
        } else {
            unsafe { Some(DevicePath::from_ffi_ptr(self.0.file_path.cast())) }
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
        let load_options_size = usize_from_u32(self.0.load_options_size);

        if self.0.load_options.is_null() {
            Err(LoadOptionsError::NotSet)
        } else if (load_options_size % mem::size_of::<u16>() != 0)
            || (((self.0.load_options as usize) % mem::align_of::<u16>()) != 0)
        {
            Err(LoadOptionsError::NotAligned)
        } else {
            let s = unsafe {
                slice::from_raw_parts(
                    self.0.load_options.cast::<u16>(),
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
    #[must_use]
    pub fn load_options_as_bytes(&self) -> Option<&[u8]> {
        if self.0.load_options.is_null() {
            None
        } else {
            unsafe {
                Some(slice::from_raw_parts(
                    self.0.load_options.cast(),
                    usize_from_u32(self.0.load_options_size),
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
        self.0.image_base = data;
        self.0.image_size = size;
    }

    /// Set the callback handler to unload the image.
    ///
    /// Drivers that wish to support unloading have to register their unload handler
    /// using this protocol. It is responsible for cleaning up any resources the
    /// image is using before returning. Unloading a driver is done with
    /// [`BootServices::unload_image`].
    ///
    /// # Safety
    ///
    /// Only the driver that this [`LoadedImage`] is attached to should register an
    /// unload handler.
    ///
    /// [`BootServices::unload_image`]: crate::table::boot::BootServices
    pub unsafe fn set_unload(
        &mut self,
        unload: extern "efiapi" fn(image_handle: Handle) -> Status,
    ) {
        type RawFn = unsafe extern "efiapi" fn(image_handle: uefi_raw::Handle) -> uefi_raw::Status;
        let unload: RawFn = mem::transmute(unload);
        self.0.unload = Some(unload);
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
        self.0.load_options = options.cast();
        self.0.load_options_size = size;
    }

    /// Returns the base address and the size in bytes of the loaded image.
    #[must_use]
    pub const fn info(&self) -> (*const c_void, u64) {
        (self.0.image_base, self.0.image_size)
    }

    /// Get the memory type of the image's code sections.
    ///
    /// Normally the returned value is one of:
    ///  - `MemoryType::LOADER_CODE` for UEFI applications
    ///  - `MemoryType::BOOT_SERVICES_CODE` for UEFI boot drivers
    ///  - `MemoryType::RUNTIME_SERVICES_CODE` for UEFI runtime drivers
    #[must_use]
    pub fn code_type(&self) -> MemoryType {
        self.0.image_code_type
    }

    /// Get the memory type of the image's data sections.
    ///
    /// Normally the returned value is one of:
    ///  - `MemoryType::LOADER_DATA` for UEFI applications
    ///  - `MemoryType::BOOT_SERVICES_DATA` for UEFI boot drivers
    ///  - `MemoryType::RUNTIME_SERVICES_DATA` for UEFI runtime drivers
    #[must_use]
    pub fn data_type(&self) -> MemoryType {
        self.0.image_data_type
    }
}
