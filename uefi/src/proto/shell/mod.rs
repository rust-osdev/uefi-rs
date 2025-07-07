// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use uefi_macros::unsafe_protocol;

use core::marker::PhantomData;
use core::ptr;

use uefi_raw::protocol::shell::ShellProtocol;

use crate::{CStr16, Char16, Result, StatusExt};

/// Shell Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellProtocol::GUID)]
pub struct Shell(ShellProtocol);

/// Iterator over the names of environmental variables obtained from the Shell protocol.
#[derive(Debug)]
pub struct Vars<'a> {
    /// Char16 containing names of environment variables
    inner: *const Char16,
    /// Placeholder to attach a lifetime to `Vars`
    placeholder: PhantomData<&'a CStr16>,
}

impl<'a> Iterator for Vars<'a> {
    type Item = &'a CStr16;
    // We iterate a list of NUL terminated CStr16s.
    // The list is terminated with a double NUL.
    fn next(&mut self) -> Option<Self::Item> {
        let s = unsafe { CStr16::from_ptr(self.inner) };
        if s.is_empty() {
            None
        } else {
            self.inner = unsafe { self.inner.add(s.num_chars() + 1) };
            Some(s)
        }
    }
}

impl Shell {
    /// Gets the value of the specified environment variable
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable name of which to retrieve the
    ///   value.
    ///
    /// # Returns
    ///
    /// * `Some(<env_value>)` - &CStr16 containing the value of the
    ///   environment variable
    /// * `None` - If environment variable does not exist
    #[must_use]
    pub fn var(&self, name: &CStr16) -> Option<&CStr16> {
        let name_ptr: *const Char16 = name.as_ptr();
        let var_val = unsafe { (self.0.get_env)(name_ptr.cast()) };
        if var_val.is_null() {
            None
        } else {
            unsafe { Some(CStr16::from_ptr(var_val.cast())) }
        }
    }

    /// Gets an iterator over the names of all environment variables
    #[must_use]
    pub fn vars(&self) -> Vars<'_> {
        let env_ptr = unsafe { (self.0.get_env)(ptr::null()) };
        Vars {
            inner: env_ptr.cast::<Char16>(),
            placeholder: PhantomData,
        }
    }

    /// Sets the environment variable
    ///
    /// # Arguments
    ///
    /// * `name` - The environment variable for which to set the value
    /// * `value` - The new value of the environment variable
    /// * `volatile` - Indicates whether the variable is volatile or
    ///   not
    ///
    /// # Returns
    ///
    /// * `Status::SUCCESS` - The variable was successfully set
    pub fn set_var(&self, name: &CStr16, value: &CStr16, volatile: bool) -> Result {
        let name_ptr: *const Char16 = name.as_ptr();
        let value_ptr: *const Char16 = value.as_ptr();
        unsafe { (self.0.set_env)(name_ptr.cast(), value_ptr.cast(), volatile) }.to_result()
    }

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
        let fs_ptr: *const Char16 = file_system.map_or(ptr::null(), |x| (x.as_ptr()));
        let dir_ptr: *const Char16 = directory.map_or(ptr::null(), |x| (x.as_ptr()));
        unsafe { (self.0.set_cur_dir)(fs_ptr.cast(), dir_ptr.cast()) }.to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use uefi::cstr16;

    /// Testing Vars struct
    #[test]
    fn test_vars() {
        // Empty Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.push(0);
        vars_mock.push(0);
        let mut vars = Vars {
            inner: vars_mock.as_ptr().cast(),
            placeholder: PhantomData,
        };
        assert!(vars.next().is_none());

        // One environment variable in Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.push(b'f' as u16);
        vars_mock.push(b'o' as u16);
        vars_mock.push(b'o' as u16);
        vars_mock.push(0);
        vars_mock.push(0);
        let vars = Vars {
            inner: vars_mock.as_ptr().cast(),
            placeholder: PhantomData,
        };
        assert_eq!(vars.collect::<Vec<_>>(), Vec::from([cstr16!("foo")]));

        // Multiple environment variables in Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.push(b'f' as u16);
        vars_mock.push(b'o' as u16);
        vars_mock.push(b'o' as u16);
        vars_mock.push(b'1' as u16);
        vars_mock.push(0);
        vars_mock.push(b'b' as u16);
        vars_mock.push(b'a' as u16);
        vars_mock.push(b'r' as u16);
        vars_mock.push(0);
        vars_mock.push(b'b' as u16);
        vars_mock.push(b'a' as u16);
        vars_mock.push(b'z' as u16);
        vars_mock.push(b'2' as u16);
        vars_mock.push(0);
        vars_mock.push(0);

        let vars = Vars {
            inner: vars_mock.as_ptr().cast(),
            placeholder: PhantomData,
        };
        assert_eq!(
            vars.collect::<Vec<_>>(),
            Vec::from([cstr16!("foo1"), cstr16!("bar"), cstr16!("baz2")])
        );
    }
}
