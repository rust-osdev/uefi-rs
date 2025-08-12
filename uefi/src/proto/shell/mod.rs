// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use uefi_macros::unsafe_protocol;

use core::ptr;

use uefi_raw::protocol::shell::ShellProtocol;

use crate::{CStr16, Char16, Result, StatusExt};

/// Shell Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellProtocol::GUID)]
pub struct Shell(ShellProtocol);
impl Shell {
    /// Returns the current directory on the specified device
    ///
    /// # Arguments
    ///
    /// * `file_system_mapping` - The file system mapping for which to get
    ///   the current directory
    ///
    /// # Returns
    ///
    /// * `Some(cwd)` - CStr16 containing the current working directory
    /// * `None` - Could not retrieve current directory
    #[must_use]
    pub fn current_dir(&self, file_system_mapping: Option<&CStr16>) -> Option<&CStr16> {
        let mapping_ptr: *const Char16 = file_system_mapping.map_or(ptr::null(), CStr16::as_ptr);
        let cur_dir = unsafe { (self.0.get_cur_dir)(mapping_ptr.cast()) };
        if cur_dir.is_null() {
            None
        } else {
            unsafe { Some(CStr16::from_ptr(cur_dir.cast())) }
        }
    }

    /// Changes the current directory on the specified device
    ///
    /// # Arguments
    ///
    /// * `file_system` - Pointer to the file system's mapped name.
    /// * `directory` - Points to the directory on the device specified by
    ///   `file_system`.
    ///
    /// # Returns
    ///
    /// * `Status::SUCCESS` - The directory was successfully set
    ///
    /// # Errors
    ///
    /// * `Status::EFI_NOT_FOUND` - The directory does not exist
    pub fn set_current_dir(
        &self,
        file_system: Option<&CStr16>,
        directory: Option<&CStr16>,
    ) -> Result {
        let fs_ptr: *const Char16 = file_system.map_or(ptr::null(), |x| x.as_ptr());
        let dir_ptr: *const Char16 = directory.map_or(ptr::null(), |x| x.as_ptr());
        unsafe { (self.0.set_cur_dir)(fs_ptr.cast(), dir_ptr.cast()) }.to_result()
    }
}
