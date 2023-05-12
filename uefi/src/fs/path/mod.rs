//! This module offers the [`Path`] and [`PathBuf`] abstractions.
//!
//! # Interoperability with Rust strings
//!
//! For the interoperability with Rust strings, i.e., `String` and `str` from
//! the standard library, the API is intended to transform these types first to
//! `CString16` respectively `CStr16`. They do not directly translate to
//! [`Path`] and [`PathBuf`].
//!
//! # Path Structure
//!
//! Paths use the [`SEPARATOR`] character as separator. Paths are absolute and
//! do not contain `.` or `..` components. However, this can be implemented in
//! the future.

mod path;
mod pathbuf;
mod validation;

pub use path::{Components, Path};
pub use pathbuf::PathBuf;

use crate::data_types::chars::NUL_16;
use crate::{CStr16, Char16};
pub(super) use validation::validate_path;
pub use validation::PathError;

/// The default separator for paths.
pub const SEPARATOR: Char16 = unsafe { Char16::from_u16_unchecked('\\' as u16) };

/// Stringified version of [`SEPARATOR`].
pub const SEPARATOR_STR: &CStr16 = uefi_macros::cstr16!("\\");

/// Deny list of characters for path components. UEFI supports FAT-like file
/// systems. According to <https://en.wikipedia.org/wiki/Comparison_of_file_systems>,
/// paths should not contain these symbols.
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
