// SPDX-License-Identifier: MIT OR Apache-2.0

//! `ShellParams` protocol

use crate::proto::unsafe_protocol;
use crate::{data_types, Char16};
use core::slice::from_raw_parts;
use uefi_raw::protocol::shell_params::ShellParametersProtocol;

use crate::CStr16;

/// The ShellParameters protocol.
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
            .map(|x| unsafe { CStr16::from_ptr(*x) })
    }

    /// Get a slice of the args, as Char16 pointers
    #[must_use]
    const fn args_slice(&self) -> &[*const Char16] {
        unsafe {
            from_raw_parts(
                self.0.argv.cast::<*const data_types::chars::Char16>(),
                self.0.argc,
            )
        }
    }
}
