// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use crate::proto::unsafe_protocol;
use crate::{CStr16, Char16, Error, Result, Status, StatusExt};
use core::ptr;
use uefi_raw::protocol::shell::ShellProtocol;

/// Shell Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellProtocol::GUID)]
pub struct Shell(ShellProtocol);

impl Shell {
    /// Returns the current directory on the specified device.
    ///
    /// # Arguments
    ///
    /// * `file_system_mapping` - The file system mapping for which to get
    ///   the current directory
    ///
    /// # Errors
    ///
    /// * [`Status::NOT_FOUND`] - Could not retrieve current directory
    pub fn current_dir(&self, file_system_mapping: Option<&CStr16>) -> Result<&CStr16> {
        let mapping_ptr: *const Char16 = file_system_mapping.map_or(ptr::null(), CStr16::as_ptr);
        let cur_dir = unsafe { (self.0.get_cur_dir)(mapping_ptr.cast()) };
        if cur_dir.is_null() {
            Err(Error::new(Status::NOT_FOUND, ()))
        } else {
            unsafe { Ok(CStr16::from_ptr(cur_dir.cast())) }
        }
    }

    /// Changes the current directory on the specified device
    ///
    /// # Arguments
    ///
    /// * `file_system` - File system's mapped name.
    /// * `directory` - Directory on the device specified by
    ///   `file_system`.
    ///
    /// # Errors
    ///
    /// * [`Status::NOT_FOUND`] - The directory does not exist
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
