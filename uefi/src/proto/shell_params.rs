// SPDX-License-Identifier: MIT OR Apache-2.0

//! UEFI Shell Parameters protocol.

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
    /// Returns the number of shell arguments.
    #[must_use]
    pub const fn args_len(&self) -> usize {
        self.0.argc
    }

    /// Returns an iterator over the shell arguments.
    pub fn args(&self) -> impl Iterator<Item = &CStr16> {
        self.args_slice()
            .iter()
            // SAFETY: The memory is valid.
            .map(|x| unsafe { CStr16::from_ptr(*x) })
    }

    /// Returns the argument pointers as a slice.
    #[must_use]
    const fn args_slice(&self) -> &[*const Char16] {
        // SAFETY: The memory is valid.
        unsafe {
            from_raw_parts(
                self.0.argv.cast::<*const data_types::chars::Char16>(),
                self.0.argc,
            )
        }
    }
}
