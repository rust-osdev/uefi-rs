// SPDX-License-Identifier: MIT OR Apache-2.0

//! Standard UEFI tables.

mod header;
mod revision;

pub mod boot;
pub mod configuration;
pub mod runtime;
pub mod system;

pub use header::Header;
pub use revision::Revision;
