// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use crate::proto::unsafe_protocol;
use crate::{CStr16, Char16, Error, Result, Status, StatusExt};

use core::marker::PhantomData;
use core::ptr;
use uefi_raw::protocol::shell::ShellProtocol;

/// Shell Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellProtocol::GUID)]
pub struct Shell(ShellProtocol);

/// Trait for implementing the var function
pub trait ShellVarProvider {
    /// Gets the value of the specified environment variable
    fn var(&self, name: &CStr16) -> Option<&CStr16>;
}

/// Iterator over the names of environmental variables obtained from the Shell protocol.
#[derive(Debug)]
pub struct Vars<'a, T: ShellVarProvider> {
    /// Char16 containing names of environment variables
    names: *const Char16,
    /// Reference to Shell Protocol
    protocol: *const T,
    /// Marker to attach a lifetime to `Vars`
    _marker: PhantomData<&'a CStr16>,
}

impl<'a, T: ShellVarProvider + 'a> Iterator for Vars<'a, T> {
    type Item = (&'a CStr16, Option<&'a CStr16>);
    // We iterate a list of NUL terminated CStr16s.
    // The list is terminated with a double NUL.
    fn next(&mut self) -> Option<Self::Item> {
        let s = unsafe { CStr16::from_ptr(self.names) };
        if s.is_empty() {
            None
        } else {
            self.names = unsafe { self.names.add(s.num_chars() + 1) };
            Some((s, unsafe { self.protocol.as_ref().unwrap().var(s) }))
        }
    }
}

impl ShellVarProvider for Shell {
    /// Gets the value of the specified environment variable
    fn var(&self, name: &CStr16) -> Option<&CStr16> {
        self.var(name)
    }
}

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
    ///
    /// # Returns
    ///
    /// * `Vars` - Iterator over the names of the environment variables
    #[must_use]
    pub fn vars(&self) -> Vars<'_, Self> {
        let env_ptr = unsafe { (self.0.get_env)(ptr::null()) };
        Vars {
            names: env_ptr.cast::<Char16>(),
            protocol: self,
            _marker: PhantomData,
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
        unsafe { (self.0.set_env)(name_ptr.cast(), value_ptr.cast(), volatile.into()) }.to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    use uefi::cstr16;

    struct ShellMock<'a> {
        inner: BTreeMap<&'a CStr16, &'a CStr16>,
    }

    impl<'a> ShellMock<'a> {
        fn new(pairs: impl IntoIterator<Item = (&'a CStr16, &'a CStr16)>) -> ShellMock<'a> {
            let mut inner_map = BTreeMap::new();
            for (name, val) in pairs.into_iter() {
                inner_map.insert(name, val);
            }
            ShellMock { inner: inner_map }
        }
    }
    impl<'a> ShellVarProvider for ShellMock<'a> {
        fn var(&self, name: &CStr16) -> Option<&CStr16> {
            if let Some(val) = self.inner.get(name) {
                Some(*val)
            } else {
                None
            }
        }
    }

    /// Testing Vars struct
    #[test]
    fn test_vars() {
        // Empty Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.extend_from_slice(
            b"\0\0"
                .into_iter()
                .map(|&x| x as u16)
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let mut vars = Vars {
            names: vars_mock.as_ptr().cast(),
            protocol: &ShellMock::new(Vec::new()),
            _marker: PhantomData,
        };

        assert!(vars.next().is_none());

        // One environment variable in Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.extend_from_slice(
            b"foo\0\0"
                .into_iter()
                .map(|&x| x as u16)
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let vars = Vars {
            names: vars_mock.as_ptr().cast(),
            protocol: &ShellMock::new(Vec::from([(cstr16!("foo"), cstr16!("value"))])),
            _marker: PhantomData,
        };
        assert_eq!(
            vars.collect::<Vec<_>>(),
            Vec::from([(cstr16!("foo"), Some(cstr16!("value")))])
        );

        // Multiple environment variables in Vars
        let mut vars_mock = Vec::<u16>::new();
        vars_mock.extend_from_slice(
            b"foo1\0bar\0baz2\0\0"
                .into_iter()
                .map(|&x| x as u16)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let vars = Vars {
            names: vars_mock.as_ptr().cast(),
            protocol: &ShellMock::new(Vec::from([
                (cstr16!("foo1"), cstr16!("value")),
                (cstr16!("bar"), cstr16!("one")),
                (cstr16!("baz2"), cstr16!("two")),
            ])),
            _marker: PhantomData,
        };
        assert_eq!(
            vars.collect::<Vec<_>>(),
            Vec::from([
                (cstr16!("foo1"), Some(cstr16!("value"))),
                (cstr16!("bar"), Some(cstr16!("one"))),
                (cstr16!("baz2"), Some(cstr16!("two")))
            ])
        );
    }
}
