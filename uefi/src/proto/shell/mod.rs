// SPDX-License-Identifier: MIT OR Apache-2.0

//! EFI Shell Protocol v2.2

use crate::proto::unsafe_protocol;

pub use uefi_raw::protocol::shell::ShellProtocol;

/// Shell Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(uefi_raw::protocol::shell::ShellProtocol::GUID)]
pub struct Shell(uefi_raw::protocol::shell::ShellProtocol);
