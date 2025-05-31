// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use crate::proto::unsafe_protocol;
use crate::{CStr16, Char16, Error, Result, Status, StatusExt};
use core::ptr;
use uefi_raw::protocol::shell::ShellProtocol;
use alloc::vec::Vec;

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

    /// Gets the environment variable or list of environment variables
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable name of which to retrieve the
    ///   value
    ///   If None, will return all defined shell environment
    ///   variables
    ///
    /// # Returns
    ///
    /// * `Some(Vec<env_value>)` - Value of the environment variable
    /// * `Some(Vec<env_names>)` - Vector of environment variable names
    /// * `None` - Environment variable doesn't exist
    #[must_use]
    pub fn get_env<'a>(&'a self, name: Option<&CStr16>) -> Option<Vec<&'a CStr16>> {
        let mut env_vec = Vec::new();
        match name {
            Some(n) => {
                let name_ptr: *const Char16 = core::ptr::from_ref::<CStr16>(n).cast();
                let var_val = unsafe { (self.0.get_env)(name_ptr.cast()) };
                if var_val.is_null() {
                    return None;
                } else {
                    unsafe { env_vec.push(CStr16::from_ptr(var_val.cast())) };
                }
            }
            None => {
                let cur_env_ptr = unsafe { (self.0.get_env)(ptr::null()) };

                let mut cur_start = cur_env_ptr;
                let mut cur_len = 0;

                let mut i = 0;
                let mut null_count = 0;
                unsafe {
                    while null_count <= 1 {
                        if (*(cur_env_ptr.add(i))) == Char16::from_u16_unchecked(0).into() {
                            if cur_len > 0 {
                                env_vec.push(CStr16::from_char16_with_nul_unchecked(
                                    &(*ptr::slice_from_raw_parts(cur_start.cast(), cur_len + 1)),
                                ));
                            }
                            cur_len = 0;
                            null_count += 1;
                        } else {
                            if null_count > 0 {
                                cur_start = cur_env_ptr.add(i);
                            }
                            null_count = 0;
                            cur_len += 1;
                        }
                        i += 1;
                    }
                }
            }
        }
        Some(env_vec)
    }

    /// Sets the environment variable
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable for which to set the value
    /// * `value` - The new value of the environment variable
    /// * `volatile` - Indicates whether or not the variable is volatile or
    ///   not
    ///
    /// # Returns
    ///
    /// * `Status::SUCCESS` The variable was successfully set
    pub fn set_env(&self, name: &CStr16, value: &CStr16, volatile: bool) -> Status {
        let name_ptr: *const Char16 = core::ptr::from_ref::<CStr16>(name).cast();
        let value_ptr: *const Char16 = core::ptr::from_ref::<CStr16>(value).cast();
        unsafe { (self.0.set_env)(name_ptr.cast(), value_ptr.cast(), volatile) }
    }
}
