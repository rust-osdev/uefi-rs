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
pub mod file_system;
pub mod firmware_volume;
pub mod hii;
pub mod loaded_image;
pub mod media;
pub mod memory_protection;
pub mod misc;
pub mod network;
pub mod rng;
pub mod shell_params;
pub mod string;
pub mod tcg;
