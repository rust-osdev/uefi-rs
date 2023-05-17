//! Standard UEFI tables.

mod header;
mod revision;

pub mod boot;
pub mod runtime;

pub use header::Header;
pub use revision::Revision;
