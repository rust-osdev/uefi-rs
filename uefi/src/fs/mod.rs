//! A high-level file system API for UEFI applications close to the `std::fs`
//! module from Rust's standard library. The main type by this module is
//! [`FileSystem`].
//!
//! # Difference to typical File System Abstractions
//! Users perform actions on dedicated volumes: For example, the boot volume,
//! such as a CD-rom, USB-stick, or any other storage device.
//!
//! Unlike in the API of typical UNIX file system abstractions, there is
//! no virtual file system. Unlike in Windows, there is no way to access volumes
//! by a dedicated name.
//!
//! # Paths
//! All paths are absolute and follow the FAT-like file system conventions for
//! paths. Thus, there is no current working directory and path components
//! like `.` and `..` are not supported. In other words, the current working
//! directory is always `/`, i.e., the root, of the opened volume.
//!
//! Symlinks or hard-links are not supported but only directories and regular
//! files with plain linear paths to them. For more information, see
//! [`Path`] and [`PathBuf`].
//!
//! # API Hints
//! There is no `File` abstraction as in the Rust `std` library. Instead, it is
//! intended to work with the file system via dedicated functions, similar to
//! the public functions of the `std::fs` module.
//!
//! There is no automatic synchronization of the file system for concurrent
//! accesses. This is in the responsibility of the user.

mod dir_entry_iter;
mod file_system;
mod path;
mod uefi_types;

pub use dir_entry_iter::*;
pub use file_system::*;
pub use path::*;

use uefi_types::*;
