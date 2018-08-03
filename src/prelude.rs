//! This module is used to simplify importing the most common UEFI types.
//!
//! This includes the system table types, `Status` codes, etc.

pub use crate::Status;

// Import the basic table types.
pub use crate::table::{SystemTable, boot::BootServices, runtime::RuntimeServices};
