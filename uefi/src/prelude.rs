// SPDX-License-Identifier: MIT OR Apache-2.0

//! This module is used to simplify importing the most common UEFI types.
//!
//! This includes the system table modules, `Status` codes, etc.

pub use crate::{
    Handle, ResultExt, Status, StatusExt, boot, cstr8, cstr16, entry, runtime, system,
};
