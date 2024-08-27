//! This module is used to simplify importing the most common UEFI types.
//!
//! This includes the system table types, `Status` codes, etc.

pub use crate::{cstr16, cstr8, entry, Handle, ResultExt, Status, StatusExt};

// Import the basic table types.
#[allow(deprecated)]
pub use crate::table::boot::BootServices;
#[allow(deprecated)]
pub use crate::table::runtime::RuntimeServices;
#[allow(deprecated)]
pub use crate::table::{Boot, SystemTable};
