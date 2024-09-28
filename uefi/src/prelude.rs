//! This module is used to simplify importing the most common UEFI types.
//!
//! This includes the system table modules, `Status` codes, etc.

pub use crate::{
    boot, cstr16, cstr8, entry, runtime, system, Handle, ResultExt, Status, StatusExt,
};
