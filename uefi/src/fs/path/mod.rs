// SPDX-License-Identifier: MIT OR Apache-2.0

//! This module offers the [`Path`] and [`PathBuf`] abstractions.
//!
//! # Interoperability with Rust Strings
//!
//! To use Rust strings - `String` and `str` - with this API, first convert them
//! to [`CString16`] or [`CStr16`]. Rust strings do not directly convert to
//! [`Path`] or [`PathBuf`].
//!
//! # Path Structure
//!
//! Paths use the [`SEPARATOR`] character as separator. Paths are absolute and
//! do not contain `.` or `..` components. However, this can be implemented in
//! the future.
//!
//! [`CString16`]: uefi::data_types::CString16

mod path;
mod pathbuf;
mod validation;

pub use path::{Components, Path};
pub use pathbuf::PathBuf;

use crate::data_types::chars::NUL_16;
use crate::{CStr16, Char16, cstr16};
pub use validation::PathError;
pub(super) use validation::validate_path;

/// The default separator for paths.
// SAFETY: The memory is valid.
pub const SEPARATOR: Char16 = unsafe { Char16::from_u16_unchecked('\\' as u16) };

/// Stringified version of [`SEPARATOR`].
pub const SEPARATOR_STR: &CStr16 = cstr16!("\\");

/// Deny list of characters for path components. UEFI supports FAT-like file
/// systems. According to <https://en.wikipedia.org/wiki/Comparison_of_file_systems>,
/// paths should not contain these symbols.
// SAFETY: The memory is valid.
pub const CHARACTER_DENY_LIST: [Char16; 10] = unsafe {
    [
        NUL_16,
        Char16::from_u16_unchecked('"' as u16),
        Char16::from_u16_unchecked('*' as u16),
        Char16::from_u16_unchecked('/' as u16),
        Char16::from_u16_unchecked(':' as u16),
        Char16::from_u16_unchecked('<' as u16),
        Char16::from_u16_unchecked('>' as u16),
        Char16::from_u16_unchecked('?' as u16),
        SEPARATOR,
        Char16::from_u16_unchecked('|' as u16),
    ]
};
