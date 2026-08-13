// SPDX-License-Identifier: MIT OR Apache-2.0

//! A high-level file system API for UEFI applications close to the `std::fs`
//! module from Rust's standard library. The main export of this module is
//! [`FileSystem`].
//!
//! # Differences from Typical File System Abstractions
//!
//! Operations act on dedicated volumes, such as a boot volume on a CD-ROM, USB
//! drive, or other storage device.
//!
//! Unlike typical UNIX file system APIs, UEFI has no virtual file system.
//! Unlike Windows, UEFI has no dedicated volume names.
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
//! ## Use `&str` as Path
//! A `&str` known at compile time can be converted to a [`Path`] using the
//! [`cstr16!`] macro. During runtime, you can create a path like this:
//!
//! ```no_run
//! use uefi::CString16;
//! use uefi::fs::{FileSystem, FileSystemResult};
//! use uefi::proto::media::fs::SimpleFileSystem;
//! use uefi::boot::{self, ScopedProtocol};
//!
//! fn read_file(path: &str) -> FileSystemResult<Vec<u8>> {
//!     let path: CString16 = CString16::try_from(path).unwrap();
//!     let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
//!     let mut fs = FileSystem::new(fs);
//!     fs.read(path.as_ref())
//! }
//! ```
//!
//! # API Notes
//! There is no `File` abstraction as in the Rust `std` library. Instead, it is
//! intended to work with the file system via dedicated functions, similar to
//! the public functions of the `std::fs` module.
//!
//! The file system does not automatically synchronize concurrent access; the
//! caller is responsible for synchronization.
//!
//! [`cstr16!`]: crate::cstr16

mod dir_entry_iter;
mod file_system;
mod path;
mod uefi_types;

pub use dir_entry_iter::*;
pub use file_system::*;
pub use path::*;

use uefi_types::*;
