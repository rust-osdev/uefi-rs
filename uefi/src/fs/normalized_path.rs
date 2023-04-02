//! Module for path normalization. See [`NormalizedPath`].

use super::*;
use crate::data_types::FromStrError;
use crate::CString16;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Deref;
use derive_more::Display;

/// The default separator for paths.
pub const SEPARATOR: char = '\\';

/// Stringifyed version of [`SEPARATOR`].
pub const SEPARATOR_STR: &str = "\\";

/// Errors that may happen during path normalization..
#[derive(Debug, Clone, Eq, PartialEq, Display)]
pub enum PathError {
    /// The specified present working directory is not absolute.
    PwdNotAbsolute,
    /// The path is empty.
    Empty,
    /// There are illegal characters in the path.
    IllegalCharacters(CharactersError),
}

#[cfg(feature = "unstable")]
impl core::error::Error for PathError {}

#[derive(Debug, Clone, Eq, PartialEq, Display)]
pub enum CharactersError {
    ProhibitedSymbols,
    NonUCS2Compatible(FromStrError),
}

#[cfg(feature = "unstable")]
impl core::error::Error for CharactersError {}

/// **Internal API (so far).**
///
/// Unlike a [`Path`], which is close to the implementation of the Rust
/// standard library, this abstraction is an absolute path that is valid in
/// FAT-like file systems (which are supported by UEFI and can be accessed via
/// the file system protocol).
///
/// Hence, it is called normalized path. Another term might be canonicalized
/// path.
///
/// For compatibility with the UEFI file-system protocol, this is a
/// [`CString16`]. The separator is `\`. For convenience, all occurrences of `/`
/// are transparently replaced by `\`.
///
/// A normalized path is always absolute, i.e., starts at the root directory.
#[derive(Debug, Eq, PartialEq, Display)]
pub struct NormalizedPath(CString16);

impl NormalizedPath {
    /// Deny list of characters for path components. UEFI supports FAT-like file
    /// systems. According to <https://en.wikipedia.org/wiki/Comparison_of_file_systems>,
    /// paths should not contain the following symbols.
    pub const CHARACTER_DENY_LIST: [char; 10] =
        ['\0', '"', '*', '/', ':', '<', '>', '?', '\\', '|'];

    /// Constructor. Combines the path with the present working directory (pwd)
    /// if the `path` is relative. The resulting path is technically valid so
    /// that it can be passed to the underlying file-system protocol. The
    /// resulting path doesn't contain `.` or `..`.
    ///
    /// `pwd` is expected to be valid.
    pub fn new(pwd: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<Self, PathError> {
        let pwd = pwd.as_ref();
        let path = path.as_ref();

        let path = Self::normalize_separator(path);
        let path = Path::new(path.as_str());

        Self::check_pwd_absolute(pwd)?;
        Self::check_prohibited_chars(path)?;

        let path = Self::combine_path_with_pwd(pwd, path);

        Self::build_normalized_path(path.as_str().as_ref())
    }

    /// Checks if the pwd is an absolute path.
    fn check_pwd_absolute(pwd: &Path) -> Result<(), PathError> {
        if !pwd.as_str().starts_with(SEPARATOR) {
            return Err(PathError::PwdNotAbsolute);
        }
        Ok(())
    }

    /// Replaces all occurrences of `/` with [`SEPARATOR`].
    fn normalize_separator(path: &Path) -> String {
        path.as_str().replace('/', SEPARATOR_STR)
    }

    /// Checks that each component of type [`Component::Normal`] doesn't contain
    /// any of the prohibited characters specified in
    /// [`Self::CHARACTER_DENY_LIST`].
    fn check_prohibited_chars(path: &Path) -> Result<(), PathError> {
        let prohibited_character_found = path
            .components(SEPARATOR)
            .filter_map(|c| match c {
                Component::Normal(n) => Some(n),
                _ => None,
            })
            .flat_map(|c| c.chars())
            .any(|c| Self::CHARACTER_DENY_LIST.contains(&c));

        (!prohibited_character_found)
            .then_some(())
            .ok_or(PathError::IllegalCharacters(
                CharactersError::ProhibitedSymbols,
            ))
    }

    /// Merges `pwd` and `path`, if `path` is not absolute.
    fn combine_path_with_pwd(pwd: &Path, path: &Path) -> String {
        let path_is_absolute = path.as_str().starts_with(SEPARATOR);
        if path_is_absolute {
            path.as_str().to_string()
        } else {
            // This concatenation is fine as pwd is an absolute path.
            if pwd.as_str() == SEPARATOR_STR {
                format!("{separator}{path}", separator = SEPARATOR)
            } else {
                format!("{pwd}{separator}{path}", separator = SEPARATOR)
            }
        }
    }

    /// Consumes an absolute path and builds a `Self` from it. At this point,
    /// the path is expected to have passed all sanity checks. The last step
    /// is only relevant to resolve `.` and `..`.
    fn build_normalized_path(path: &Path) -> Result<Self, PathError> {
        let component_count = path.components(SEPARATOR).count();
        let mut normalized_components = Vec::with_capacity(component_count);

        for component in path.components(SEPARATOR) {
            match component {
                Component::RootDir => {
                    normalized_components.push(SEPARATOR_STR);
                }
                Component::CurDir => continue,
                Component::ParentDir => {
                    normalized_components.remove(normalized_components.len() - 1);
                }
                Component::Normal(n) => {
                    let prev_has_sep = normalized_components
                        .last()
                        .map(|x| x.eq(&SEPARATOR_STR))
                        .unwrap_or(false);
                    if !prev_has_sep {
                        normalized_components.push(SEPARATOR_STR);
                    }
                    normalized_components.push(n);
                }
            }
        }

        let normalized_string: String = normalized_components.concat();
        CString16::try_from(normalized_string.as_str())
            .map(Self)
            .map_err(|x| PathError::IllegalCharacters(CharactersError::NonUCS2Compatible(x)))
    }
}

impl Deref for NormalizedPath {
    type Target = CString16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_must_be_absolute() {
        let path = NormalizedPath::new("", "");
        assert_eq!(Err(PathError::PwdNotAbsolute), path);

        let path = NormalizedPath::new(".", "");
        assert_eq!(Err(PathError::PwdNotAbsolute), path);

        let path = NormalizedPath::new("/", "");
        assert_eq!(Err(PathError::PwdNotAbsolute), path);
    }

    #[test]
    fn normalized_path() {
        let path = NormalizedPath::new("\\foo", "/bar/barfoo").map(|x| x.0);
        assert_eq!(path, Ok(CString16::try_from("\\bar\\barfoo").unwrap()));

        let path = NormalizedPath::new("\\foo", "bar/barfoo").map(|x| x.0);
        assert_eq!(path, Ok(CString16::try_from("\\foo\\bar\\barfoo").unwrap()));

        let path = NormalizedPath::new("\\foo", "./bar/barfoo").map(|x| x.0);
        assert_eq!(path, Ok(CString16::try_from("\\foo\\bar\\barfoo").unwrap()));

        let path = NormalizedPath::new("\\foo", "./bar/.././././barfoo").map(|x| x.0);
        assert_eq!(path, Ok(CString16::try_from("\\foo\\barfoo").unwrap()));

        let path = NormalizedPath::new("\\", "foo").map(|x| x.0);
        assert_eq!(path, Ok(CString16::try_from("\\foo").unwrap()));
    }

    #[test]
    fn check_components_for_allowed_chars() {
        fn check_fail(path: impl AsRef<Path>) {
            assert_eq!(
                NormalizedPath::check_prohibited_chars(path.as_ref()),
                Err(PathError::IllegalCharacters(
                    CharactersError::ProhibitedSymbols
                ))
            );
        }

        assert_eq!(
            NormalizedPath::check_prohibited_chars("\\foo".as_ref()),
            Ok(())
        );

        check_fail("\\foo\0");
        check_fail("\\foo:");
        check_fail("\\foo*");
        check_fail("\\foo/");
        check_fail("\\foo<");
        check_fail("\\foo>");
        check_fail("\\foo?");
        check_fail("\\foo|");
        check_fail("\\foo\"");
    }
}
