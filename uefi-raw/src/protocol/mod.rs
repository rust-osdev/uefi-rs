//! Protocol definitions.
//!
//! Protocols are sets of related functionality identified by a unique
//! ID. They can be implemented by a UEFI driver or occasionally by a
//! UEFI application.

pub mod block;
pub mod console;
pub mod device_path;
pub mod disk;
pub mod driver;
pub mod loaded_image;
pub mod memory_protection;
pub mod rng;
