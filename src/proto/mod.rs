//! Protocol definitions.
//!
//! Protocols are sets of related functionality.
//!
//! Protocols are identified by a unique ID.
//!
//! Protocols can be implemented by a UEFI driver,
//! and are usually retrieved from a standard UEFI table or
//! by querying a handle.

use crate::Guid;

/// Common trait implemented by all standard UEFI protocols.
pub trait Protocol {
    /// Unique protocol identifier.
    const GUID: Guid;
}

#[macro_use]
mod macros;

pub mod console;
pub mod media;
pub mod debug;
