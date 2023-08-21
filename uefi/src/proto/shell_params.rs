//! `ShellParams` protocol

use crate::proto::unsafe_protocol;
use crate::Char16;
use core::ffi::c_void;
use core::slice::from_raw_parts;

#[cfg(feature = "alloc")]
use crate::CStr16;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

type ShellFileHandle = *const c_void;

/// The ShellParameters protocol.
#[repr(C)]
#[unsafe_protocol("752f3136-4e16-4fdc-a22a-e5f46812f4ca")]
pub struct ShellParameters {
    /// Pointer to a list of arguments
    pub argv: *const *const Char16,
    /// Number of arguments
    pub argc: usize,
    /// Handle of the standard input
    std_in: ShellFileHandle,
    /// Handle of the standard output
    std_out: ShellFileHandle,
    /// Handle of the standard error output
    std_err: ShellFileHandle,
}

impl ShellParameters {
    /// Get an iterator of the shell parameter arguments
    #[cfg(feature = "alloc")]
    pub fn get_args(&self) -> impl Iterator<Item = String> {
        unsafe {
            from_raw_parts(self.argv, self.argc)
                .iter()
                .map(|x| CStr16::from_ptr(*x).to_string())
        }
    }

    /// Get a slice of the args, as Char16 pointers
    #[must_use]
    pub fn get_args_slice(&self) -> &[*const Char16] {
        unsafe { from_raw_parts(self.argv, self.argc) }
    }
}
