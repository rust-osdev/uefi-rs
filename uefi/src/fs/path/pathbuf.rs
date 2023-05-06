use crate::fs::path::Path;
use crate::fs::SEPARATOR;
use crate::{CStr16, CString16, Char16};
use core::fmt::{Display, Formatter};

/// A path buffer similar to the `PathBuf` of the standard library, but based on
/// [`CString16`] strings and [`SEPARATOR`] as separator.
///
/// `/` is replaced by [`SEPARATOR`] on the fly.
#[derive(Clone, Debug, Default, Eq, PartialOrd, Ord)]
pub struct PathBuf(CString16);

impl PathBuf {
    /// Constructor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructor that replaces all occurrences of `/` with `\`.
    fn new_from_cstring16(mut string: CString16) -> Self {
        const SEARCH: Char16 = unsafe { Char16::from_u16_unchecked('/' as u16) };
        string.replace_char(SEARCH, SEPARATOR);
        Self(string)
    }

    /// Extends self with path.
    ///
    /// UNIX separators (`/`) will be replaced by [`SEPARATOR`] on the fly.
    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        const SEARCH: Char16 = unsafe { Char16::from_u16_unchecked('/' as u16) };

        // do nothing on empty path
        if path.as_ref().is_empty() {
            return;
        }

        let empty = self.0.is_empty();
        let needs_sep = *self
            .0
            .as_slice_with_nul()
            .last()
            .expect("Should have at least null character")
            != SEPARATOR;
        if !empty && needs_sep {
            self.0.push(SEPARATOR)
        }

        self.0.push_str(path.as_ref().to_cstr16());
        self.0.replace_char(SEARCH, SEPARATOR);
    }
}

impl PartialEq for PathBuf {
    fn eq(&self, other: &Self) -> bool {
        let path1: &Path = self.as_ref();
        let path2: &Path = other.as_ref();
        path1 == path2
    }
}

impl Display for PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.to_cstr16(), f)
    }
}

mod convenience_impls {
    use super::*;
    use core::borrow::Borrow;
    use core::ops::Deref;

    impl From<CString16> for PathBuf {
        fn from(value: CString16) -> Self {
            Self::new_from_cstring16(value)
        }
    }

    impl From<&CStr16> for PathBuf {
        fn from(value: &CStr16) -> Self {
            Self::new_from_cstring16(CString16::from(value))
        }
    }

    impl Deref for PathBuf {
        type Target = Path;

        fn deref(&self) -> &Self::Target {
            Path::new(&self.0)
        }
    }

    impl AsRef<Path> for PathBuf {
        fn as_ref(&self) -> &Path {
            // falls back to deref impl
            self
        }
    }

    impl Borrow<Path> for PathBuf {
        fn borrow(&self) -> &Path {
            // falls back to deref impl
            self
        }
    }

    impl AsRef<CStr16> for PathBuf {
        fn as_ref(&self) -> &CStr16 {
            &self.0
        }
    }

    impl Borrow<CStr16> for PathBuf {
        fn borrow(&self) -> &CStr16 {
            &self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use uefi_macros::cstr16;

    #[test]
    fn from_cstr16() {
        let source: &CStr16 = cstr16!("\\hello\\foo\\bar");
        let _path: PathBuf = source.into();
    }

    #[test]
    fn from_cstring16() {
        let source = CString16::try_from("\\hello\\foo\\bar").unwrap();
        let _path: PathBuf = source.as_ref().into();
        let _path: PathBuf = source.clone().into();
        let _path: PathBuf = PathBuf::new_from_cstring16(source);
    }

    #[test]
    fn from_std_string() {
        let std_string = "\\hello\\foo\\bar".to_string();
        let _path = PathBuf::new_from_cstring16(CString16::try_from(std_string.as_str()).unwrap());
    }

    #[test]
    fn push() {
        let mut pathbuf = PathBuf::new();
        pathbuf.push(cstr16!("first"));
        pathbuf.push(cstr16!("second"));
        pathbuf.push(cstr16!("third"));
        assert_eq!(pathbuf.to_cstr16(), cstr16!("first\\second\\third"));

        let mut pathbuf = PathBuf::new();
        pathbuf.push(cstr16!("\\first"));
        pathbuf.push(cstr16!("second"));
        assert_eq!(pathbuf.to_cstr16(), cstr16!("\\first\\second"));

        // empty pushes should be ignored and have no effect
        let empty_cstring16 = CString16::try_from("").unwrap();
        let mut pathbuf = PathBuf::new();
        pathbuf.push(cstr16!("first"));
        pathbuf.push(empty_cstring16.as_ref());
        pathbuf.push(empty_cstring16.as_ref());
        pathbuf.push(empty_cstring16.as_ref());
        pathbuf.push(cstr16!("second"));
        assert_eq!(pathbuf.to_cstr16(), cstr16!("first\\second"));
    }

    #[test]
    fn partial_eq() {
        let mut pathbuf1 = PathBuf::new();
        pathbuf1.push(cstr16!("first"));
        pathbuf1.push(cstr16!("second"));
        pathbuf1.push(cstr16!("third"));

        assert_eq!(pathbuf1, pathbuf1);

        let mut pathbuf2 = PathBuf::new();
        pathbuf2.push(cstr16!("\\first"));
        pathbuf2.push(cstr16!("second"));

        assert_eq!(pathbuf2, pathbuf2);
        assert_ne!(pathbuf1, pathbuf2);
    }
}
