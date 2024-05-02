//! Standard UEFI tables.

pub mod boot;
pub mod cfg;
pub mod runtime;

mod header;
mod system;

pub use header::Header;
pub use system::{Boot, Runtime, SystemTable};
pub use uefi_raw::table::Revision;

/// Common trait implemented by all standard UEFI tables.
pub trait Table {
    /// A unique number assigned by the UEFI specification
    /// to the standard tables.
    const SIGNATURE: u64;
}
