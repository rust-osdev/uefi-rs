// allow "path.rs" in "path"
#![allow(clippy::module_inception)]

use crate::fs::path::{PathBuf, SEPARATOR};
use crate::{CStr16, CString16};
use core::fmt::{Display, Formatter};

/// A path similar to the `Path` of the standard library, but based on
/// [`CStr16`] strings and [`SEPARATOR`] as separator.
///
/// [`SEPARATOR`]: super::SEPARATOR
#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Path(CStr16);

impl Path {
    /// Constructor.
    #[must_use]
    pub fn new<S: AsRef<CStr16> + ?Sized>(s: &S) -> &Self {
        unsafe { &*(s.as_ref() as *const CStr16 as *const Self) }
    }

    /// Returns the underlying string.
    #[must_use]
    pub fn to_cstr16(&self) -> &CStr16 {
        &self.0
    }

    /// Returns a path buf from that type.
    #[must_use]
    pub fn to_path_buf(&self) -> PathBuf {
        let cstring = CString16::from(&self.0);
        PathBuf::from(cstring)
    }

    /// Iterator over the components of a path.
    #[must_use]
    pub fn components(&self) -> Components {
        Components {
            path: self.as_ref(),
            i: 0,
        }
    }

    /// Returns the parent directory as [`PathBuf`].
    ///
    /// If the path is a top-level component, this returns None.
    #[must_use]
    pub fn parent(&self) -> Option<PathBuf> {
        let components_count = self.components().count();
        if components_count == 0 {
            return None;
        }

        // Return None, as we do not treat "\\" as dedicated component.
        let sep_count = self
            .0
            .as_slice()
            .iter()
            .filter(|char| **char == SEPARATOR)
            .count();
        if sep_count == 0 {
            return None;
        }

        let path =
            self.components()
                .take(components_count - 1)
                .fold(CString16::new(), |mut acc, next| {
                    // Add separator, as needed.
                    if !acc.is_empty() && *acc.as_slice().last().unwrap() != SEPARATOR {
                        acc.push(SEPARATOR);
                    }
                    acc.push_str(next.as_ref());
                    acc
                });
        let path = PathBuf::from(path);
        Some(path)
    }

    /// Returns of the path is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_cstr16().is_empty()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.to_cstr16(), f)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.components().count() == other.components().count()
            && !self
                .components()
                .zip(other.components())
                .any(|(c1, c2)| c1 != c2)
    }
}

/// Iterator over the components of a path. For example, the path `\\a\\b\\c`
/// has the components `[a, b, c]`. This is a more basic approach than the
/// components type of the standard library.
#[derive(Debug)]
pub struct Components<'a> {
    path: &'a CStr16,
    i: usize,
}

impl<'a> Iterator for Components<'a> {
    // Attention. We can't iterate over &'Ctr16, as we would break any guarantee
    // made for the terminating null character.
    type Item = CString16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            return None;
        }
        if self.path.num_chars() == 1 && self.path.as_slice()[0] == SEPARATOR {
            // The current implementation does not handle the root dir as
            // dedicated component so far. We just return nothing.
            return None;
        }

        // If the path is not empty and starts with a separator, skip it.
        if self.i == 0 && *self.path.as_slice().first().unwrap() == SEPARATOR {
            self.i = 1;
        }

        // Count how many characters are there until the next separator is
        // found.
        let len = self
            .path
            .iter()
            .skip(self.i)
            .take_while(|c| **c != SEPARATOR)
            .count();

        let progress = self.i + len;
        if progress > self.path.num_chars() {
            None
        } else {
            // select the next component and build an owned string
            let part = &self.path.as_slice()[self.i..self.i + len];
            let mut string = CString16::new();
            part.iter().for_each(|c| string.push(*c));

            // +1: skip the separator
            self.i = progress + 1;
            Some(string)
        }
    }
}

mod convenience_impls {
    use super::*;
    use core::borrow::Borrow;

    impl AsRef<Path> for &Path {
        fn as_ref(&self) -> &Path {
            self
        }
    }

    impl<'a> From<&'a CStr16> for &'a Path {
        fn from(value: &'a CStr16) -> Self {
            Path::new(value)
        }
    }

    impl AsRef<CStr16> for Path {
        fn as_ref(&self) -> &CStr16 {
            &self.0
        }
    }

    impl Borrow<CStr16> for Path {
        fn borrow(&self) -> &CStr16 {
            &self.0
        }
    }

    impl AsRef<Path> for CStr16 {
        fn as_ref(&self) -> &Path {
            Path::new(self)
        }
    }

    impl Borrow<Path> for CStr16 {
        fn borrow(&self) -> &Path {
            Path::new(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use uefi_macros::cstr16;

    #[test]
    fn from_cstr16() {
        let source: &CStr16 = cstr16!("\\hello\\foo\\bar");
        let _path: &Path = source.into();
        let _path: &Path = Path::new(source);
    }

    #[test]
    fn from_cstring16() {
        let source = CString16::try_from("\\hello\\foo\\bar").unwrap();
        let _path: &Path = source.as_ref().into();
        let _path: &Path = Path::new(source.as_ref());
    }

    #[test]
    fn components_iter() {
        let path = Path::new(cstr16!("foo\\bar\\hello"));
        let components = path.components().collect::<Vec<_>>();
        let components: Vec<&CStr16> = components.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        let expected: &[&CStr16] = &[cstr16!("foo"), cstr16!("bar"), cstr16!("hello")];
        assert_eq!(components.as_slice(), expected);

        // In case there is a leading slash, it should be ignored.
        let path = Path::new(cstr16!("\\foo\\bar\\hello"));
        let components = path.components().collect::<Vec<_>>();
        let components: Vec<&CStr16> = components.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        let expected: &[&CStr16] = &[cstr16!("foo"), cstr16!("bar"), cstr16!("hello")];
        assert_eq!(components.as_slice(), expected);

        // empty path iteration should be just fine
        let empty_cstring16 = CString16::try_from("").unwrap();
        let path = Path::new(empty_cstring16.as_ref());
        let components = path.components().collect::<Vec<_>>();
        let expected: &[CString16] = &[];
        assert_eq!(components.as_slice(), expected);

        // test empty path
        let _path = Path::new(cstr16!());
        let path = Path::new(cstr16!(""));
        let components = path.components().collect::<Vec<_>>();
        let components: Vec<&CStr16> = components.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        let expected: &[&CStr16] = &[];
        assert_eq!(components.as_slice(), expected);

        // test path that has only root component. Treated as empty path by
        // the components iterator.
        let path = Path::new(cstr16!("\\"));
        let components = path.components().collect::<Vec<_>>();
        let components: Vec<&CStr16> = components.iter().map(|x| x.as_ref()).collect::<Vec<_>>();
        let expected: &[&CStr16] = &[];
        assert_eq!(components.as_slice(), expected);
    }

    #[test]
    fn test_parent() {
        assert_eq!(None, Path::new(cstr16!("")).parent());
        assert_eq!(None, Path::new(cstr16!("\\")).parent());
        assert_eq!(
            Path::new(cstr16!("a\\b")).parent(),
            Some(PathBuf::from(cstr16!("a"))),
        );
        assert_eq!(
            Path::new(cstr16!("\\a\\b")).parent(),
            Some(PathBuf::from(cstr16!("a"))),
        );
        assert_eq!(
            Path::new(cstr16!("a\\b\\c\\d")).parent(),
            Some(PathBuf::from(cstr16!("a\\b\\c"))),
        );
        assert_eq!(Path::new(cstr16!("abc")).parent(), None,);
    }

    #[test]
    fn partial_eq() {
        let path1 = Path::new(cstr16!(r"a\b"));
        let path2 = Path::new(cstr16!(r"\a\b"));
        let path3 = Path::new(cstr16!(r"a\b\c"));

        assert_eq!(path1, path1);
        assert_eq!(path2, path2);
        assert_eq!(path3, path3);

        // Equal as currently, we only support absolute paths, so the leading
        // separator is obligatory.
        assert_eq!(path1, path2);
        assert_eq!(path2, path1);

        assert_ne!(path1, path3);
        assert_ne!(path3, path1);
    }
}
