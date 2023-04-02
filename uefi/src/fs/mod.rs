//! A high-level file system API for UEFI applications close to the `fs` module
//! from Rust's standard library.
//!
//! # Difference to typical File System Abstractions
//! Users perform actions on dedicated volumes: For example, the boot volume,
//! such as a CD-rom, USB-stick, or any other storage device.
//!
//! Unlike in the API of typical UNIX file system abstractions, there is
//! no virtual file system.
//!
//! Unlike Windows, there is no way to access volumes by a dedicated name.
//!
//! # Paths
//! All paths are absolute and follow the FAT-like file system conventions for
//! paths. Thus, there is no current working directory and path components
//! like `.` and `..` are not supported. In other words, the current working
//! directory is always `/`, i.e., the root, of the opened volume.
//!
//! Symlinks or hard-links are not supported but only directories and regular
//! files with plain linear paths to them.
//!
//! # API Hints
//! There are no `File` and `Path` abstractions similar to those from `std` that
//! are publicly exported. Instead, paths to files are provided as `&str`, and
//! will be validated and transformed internally to the correct type.
//! Furthermore, there are no `File` objects that are exposed to users. Instead,
//! it is intended to work with the file system as in `std::fs`.
//!
//! There is no automatic synchronization of the file system for concurrent
//! accesses. This is in the responsibility of the user.

mod dir_entry_iter;
mod file_system;
mod normalized_path;
mod path;
mod uefi_types;

pub use file_system::{FileSystem, FileSystemError, FileSystemResult};
pub use normalized_path::{PathError, SEPARATOR, SEPARATOR_STR};

use dir_entry_iter::*;
use normalized_path::*;
use path::*;
use uefi_types::*;
