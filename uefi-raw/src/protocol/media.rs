use crate::protocol::device_path::DevicePathProtocol;
use crate::{guid, Guid, Status};
use core::ffi::c_void;

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
#[repr(C)]
pub struct LoadFileProtocol {
    /// Causes the driver to load a specified file.
    ///
    /// # Parameters
    /// - `this` pointer to self
    /// - `file_path` The device specific path of the file to load.
    /// - `boot_policy` If TRUE, indicates that the request originates from the
    ///   boot manager, and that the boot manager is attempting to load FilePath
    ///   as a boot selection. If FALSE, then FilePath must match an exact file
    ///   to be loaded.
    /// - `buffer_size` On input the size of Buffer in bytes. On output with a
    ///   return code of EFI_SUCCESS, the amount of data transferred to Buffer.
    ///   On output with a return code of EFI_BUFFER_TOO_SMALL, the size of
    ///   Buffer required to retrieve the requested file.
    /// - `buffer` The memory buffer to transfer the file to. If Buffer is NULL,
    ///   then the size of the requested file is returned in BufferSize.
    ///
    /// # Errors
    /// - `uefi::status::EFI_SUCCESS` The file was loaded.
    /// - `uefi::status::EFI_UNSUPPORTED` The device does not support the
    ///   provided BootPolicy.
    /// - `uefi::status::EFI_INVALID_PARAMETER` FilePath is not a valid device
    ///   path, or BufferSize is NULL.
    /// - `uefi::status::EFI_NO_MEDIA` No medium was present to load the file.
    /// - `uefi::status::EFI_DEVICE_ERROR` The file was not loaded due to a
    ///   device error.
    /// - `uefi::status::EFI_NO_RESPONSE` The remote system did not respond.
    /// - `uefi::status::EFI_NOT_FOUND` The file was not found.
    /// - `uefi::status::EFI_ABORTED` The file load process was manually
    ///   cancelled.
    /// - `uefi::status::EFI_BUFFER_TOO_SMALL` The BufferSize is too small to
    ///   read the current directory entry. BufferSize has been updated with the
    ///   size needed to complete the request.
    /// - `uefi::status::EFI_WARN_FILE_SYSTEM` The resulting Buffer contains
    ///   UEFI-compliant file system.
    pub load_file: unsafe extern "efiapi" fn(
        this: *mut LoadFileProtocol,
        file_path: *const DevicePathProtocol,
        boot_policy: bool,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFileProtocol {
    pub const GUID: Guid = guid!("56ec3091-954c-11d2-8e3f-00a0c969723b");
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
#[repr(C)]
pub struct LoadFile2Protocol {
    /// Causes the driver to load a specified file.
    ///
    /// # Parameters
    /// - `this` pointer to self
    /// - `file_path` The device specific path of the file to load.
    /// - `boot_policy` Should always be FALSE.
    /// - `buffer_size` On input the size of Buffer in bytes. On output with a
    ///   return code of EFI_SUCCESS, the amount of data transferred to Buffer.
    ///   On output with a return code of EFI_BUFFER_TOO_SMALL, the size of
    ///   Buffer required to retrieve the requested file.
    /// - `buffer` The memory buffer to transfer the file to. If Buffer is NULL,
    ///   then the size of the requested file is returned in BufferSize.
    ///
    /// # Errors
    /// - `uefi::status::EFI_SUCCESS` The file was loaded.
    /// - `uefi::status::EFI_UNSUPPORTED` BootPolicy is TRUE.
    /// - `uefi::status::EFI_INVALID_PARAMETER` FilePath is not a valid device
    ///   path, or BufferSize is NULL.
    /// - `uefi::status::EFI_NO_MEDIA` No medium was present to load the file.
    /// - `uefi::status::EFI_DEVICE_ERROR` The file was not loaded due to a
    ///   device error.
    /// - `uefi::status::EFI_NO_RESPONSE` The remote system did not respond.
    /// - `uefi::status::EFI_NOT_FOUND` The file was not found.
    /// - `uefi::status::EFI_ABORTED` The file load process was manually
    ///   cancelled.
    /// - `uefi::status::EFI_BUFFER_TOO_SMALL` The BufferSize is too small to
    ///   read the current directory entry. BufferSize has been updated with the
    ///   size needed to complete the request.
    pub load_file: unsafe extern "efiapi" fn(
        this: *mut LoadFile2Protocol,
        file_path: *const DevicePathProtocol,
        boot_policy: bool,
        buffer_size: *mut usize,
        buffer: *mut c_void,
    ) -> Status,
}

impl LoadFile2Protocol {
    pub const GUID: Guid = guid!("4006c0c1-fcb3-403e-996d-4a6c8724e06d");
}
