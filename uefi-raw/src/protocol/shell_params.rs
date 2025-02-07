// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{guid, Char16, Guid};
use core::ffi::c_void;

pub type ShellFileHandle = *const c_void;

#[derive(Debug)]
#[repr(C)]
pub struct ShellParametersProtocol {
    /// Pointer to a list of arguments.
    pub argv: *const *const Char16,
    /// Number of arguments.
    pub argc: usize,
    /// Handle of the standard input.
    pub std_in: ShellFileHandle,
    /// Handle of the standard output.
    pub std_out: ShellFileHandle,
    /// Handle of the standard error output.
    pub std_err: ShellFileHandle,
}

impl ShellParametersProtocol {
    pub const GUID: Guid = guid!("752f3136-4e16-4fdc-a22a-e5f46812f4ca");
}
