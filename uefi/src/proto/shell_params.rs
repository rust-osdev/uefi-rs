// SPDX-License-Identifier: MIT OR Apache-2.0

//! `ShellParams` protocol

use crate::proto::unsafe_protocol;
use crate::{Char16, data_types};
use core::slice::from_raw_parts;
use uefi_raw::protocol::shell_params::ShellParametersProtocol;

use crate::CStr16;

/// The ShellParameters [`Protocol`].
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(ShellParametersProtocol::GUID)]
pub struct ShellParameters(ShellParametersProtocol);

impl ShellParameters {
    /// Get the number of shell parameter arguments
    #[must_use]
    pub const fn args_len(&self) -> usize {
        self.0.argc
    }

    /// Get an iterator of the shell parameter arguments
    pub fn args(&self) -> impl Iterator<Item = &CStr16> {
        self.args_slice()
            .iter()
            // SAFETY: The memory is valid.
            .map(|x| unsafe { CStr16::from_ptr(*x) })
    }

    /// Get a slice of the args, as Char16 pointers
    #[must_use]
    const fn args_slice(&self) -> &[*const Char16] {
        // The EDK2 shell sets `argv` to NULL for an empty command line.
        if self.0.argv.is_null() || self.0.argc == 0 {
            return &[];
        }
        // SAFETY: The memory is valid.
        unsafe {
            from_raw_parts(
                self.0.argv.cast::<*const data_types::chars::Char16>(),
                self.0.argc,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;

    #[test]
    fn test_args_null_argv() {
        let raw = ShellParametersProtocol {
            argv: ptr::null(),
            argc: 0,
            std_in: ptr::null_mut(),
            std_out: ptr::null_mut(),
            std_err: ptr::null_mut(),
        };
        // SAFETY: `ShellParameters` is a transparent wrapper.
        let params = unsafe { &*ptr::from_ref(&raw).cast::<ShellParameters>() };
        assert_eq!(params.args_len(), 0);
        assert_eq!(params.args().count(), 0);
    }
}
