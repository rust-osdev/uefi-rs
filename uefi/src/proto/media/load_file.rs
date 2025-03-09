// SPDX-License-Identifier: MIT OR Apache-2.0

//! LoadFile and LoadFile2 protocols.

use crate::proto::unsafe_protocol;
#[cfg(doc)]
use crate::Status;
#[cfg(all(feature = "alloc", feature = "unstable"))]
use alloc::alloc::Global;
use uefi_raw::protocol::media::{LoadFile2Protocol, LoadFileProtocol};
#[cfg(feature = "alloc")]
use {
    crate::{mem::make_boxed, proto::device_path::DevicePath, Result, StatusExt},
    alloc::boxed::Box,
    uefi::proto::BootPolicy,
    uefi_raw::Boolean,
};

/// Load File Protocol.
///
/// Used to obtain files, that are primarily boot options, from arbitrary
/// devices.
///
/// # UEFI Spec Description
/// The EFI_LOAD_FILE_PROTOCOL is a simple protocol used to obtain files from
/// arbitrary devices.
///
/// When the firmware is attempting to load a file, it first attempts to use the
/// device’s Simple File System protocol to read the file. If the file system
/// protocol is found, the firmware implements the policy of interpreting the
/// File Path value of the file being loaded. If the device does not support the
/// file system protocol, the firmware then attempts to read the file via the
/// EFI_LOAD_FILE_PROTOCOL and the LoadFile() function. In this case the
/// LoadFile() function implements the policy of interpreting the File Path
/// value.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(LoadFileProtocol::GUID)]
pub struct LoadFile(LoadFileProtocol);

impl LoadFile {
    /// Causes the driver to load a specified file.
    ///
    /// # Parameters
    /// - `file_path` The device specific path of the file to load.
    /// - `boot_policy` The [`BootPolicy`] to use.
    ///
    /// # Errors
    /// - [`Status::SUCCESS`] The file was loaded.
    /// - [`Status::UNSUPPORTED`] The device does not support the
    ///   provided BootPolicy.
    /// - [`Status::INVALID_PARAMETER`] FilePath is not a valid device
    ///   path, or BufferSize is NULL.
    /// - [`Status::NO_MEDIA`] No medium was present to load the file.
    /// - [`Status::DEVICE_ERROR`] The file was not loaded due to a
    ///   device error.
    /// - [`Status::NO_RESPONSE`] The remote system did not respond.
    /// - [`Status::NOT_FOUND`] The file was not found.
    /// - [`Status::ABORTED`] The file load process was manually
    ///   cancelled.
    /// - [`Status::BUFFER_TOO_SMALL`] The BufferSize is too small to
    ///   read the current directory entry. BufferSize has been updated with the
    ///   size needed to complete the request.
    /// - [`Status::WARN_FILE_SYSTEM`] The resulting Buffer contains
    ///   UEFI-compliant file system.
    ///
    /// [`BootPolicy`]: uefi::proto::BootPolicy
    #[cfg(feature = "alloc")]
    #[allow(clippy::extra_unused_lifetimes)] // false positive, it is used
    pub fn load_file<'a>(
        &mut self,
        file_path: &DevicePath,
        boot_policy: BootPolicy,
    ) -> Result<Box<[u8]>> {
        let this = core::ptr::addr_of_mut!(*self).cast();

        let fetch_data_fn = |buf: &'a mut [u8]| {
            let mut size = buf.len();
            let status = unsafe {
                (self.0.load_file)(
                    this,
                    file_path.as_ffi_ptr().cast(),
                    boot_policy.into(),
                    &mut size,
                    buf.as_mut_ptr().cast(),
                )
            };
            status.to_result_with_err(|_| Some(size)).map(|_| buf)
        };

        #[cfg(not(feature = "unstable"))]
        let file: Box<[u8]> = make_boxed::<[u8], _>(fetch_data_fn)?;

        #[cfg(feature = "unstable")]
        let file = make_boxed::<[u8], _, _>(fetch_data_fn, Global)?;

        Ok(file)
    }
}

/// Load File2 Protocol.
///
/// The Load File2 protocol is used to obtain files from arbitrary devices that
/// are not boot options.
///
/// # UEFI Spec Description
///
/// The EFI_LOAD_FILE2_PROTOCOL is a simple protocol used to obtain files from
/// arbitrary devices that are not boot options. It is used by LoadImage() when
/// its BootOption parameter is FALSE and the FilePath does not have an instance
/// of the EFI_SIMPLE_FILE_SYSTEM_PROTOCOL.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(LoadFile2Protocol::GUID)]
pub struct LoadFile2(LoadFile2Protocol);

impl LoadFile2 {
    /// Causes the driver to load a specified file.
    ///
    /// # Parameters
    /// - `file_path` The device specific path of the file to load.
    ///
    /// # Errors
    /// - [`Status::SUCCESS`] The file was loaded.
    /// - [`Status::UNSUPPORTED`] BootPolicy is TRUE.
    /// - [`Status::INVALID_PARAMETER`] FilePath is not a valid device
    ///   path, or BufferSize is NULL.
    /// - [`Status::NO_MEDIA`] No medium was present to load the file.
    /// - [`Status::DEVICE_ERROR`] The file was not loaded due to a
    ///   device error.
    /// - [`Status::NO_RESPONSE`] The remote system did not respond.
    /// - [`Status::NOT_FOUND`] The file was not found.
    /// - [`Status::ABORTED`] The file load process was manually
    ///   cancelled.
    /// - [`Status::BUFFER_TOO_SMALL`] The BufferSize is too small to
    ///   read the current directory entry. BufferSize has been updated with the
    ///   size needed to complete the request.
    #[cfg(feature = "alloc")]
    #[allow(clippy::extra_unused_lifetimes)] // false positive, it is used
    pub fn load_file<'a>(&mut self, file_path: &DevicePath) -> Result<Box<[u8]>> {
        let this = core::ptr::addr_of_mut!(*self).cast();

        let fetch_data_fn = |buf: &'a mut [u8]| {
            let mut size = buf.len();
            let status = unsafe {
                (self.0.load_file)(
                    this,
                    file_path.as_ffi_ptr().cast(),
                    Boolean::FALSE, /* always false - see spec */
                    &mut size,
                    buf.as_mut_ptr().cast(),
                )
            };
            status.to_result_with_err(|_| Some(size)).map(|_| buf)
        };

        #[cfg(not(feature = "unstable"))]
        let file: Box<[u8]> = make_boxed::<[u8], _>(fetch_data_fn)?;

        #[cfg(feature = "unstable")]
        let file = make_boxed::<[u8], _, _>(fetch_data_fn, Global)?;

        Ok(file)
    }
}
