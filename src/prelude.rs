//! This module is used to simplify importing the most common UEFI types.
//!
//! This includes the system table types, `Status` codes, etc.

pub use crate::{Handle, ResultExt, Status};

// Import the basic table types.
pub use crate::table::boot::BootServices;
pub use crate::table::runtime::RuntimeServices;
pub use crate::table::{Boot, SystemTable};

// Import the macro for creating the custom entry point, as well as the cstr macros.
pub use uefi_macros::{cstr16, cstr8, entry};
