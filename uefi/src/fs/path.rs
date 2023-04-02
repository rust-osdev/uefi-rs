//! Module for handling file-system paths in [`super::FileSystem`].
//! See [`Path`].

use alloc::string::String;
use core::fmt::{Display, Formatter};

/// Path abstraction similar to `std::path::Path` but adapted to the platform-
/// agnostic `no_std` use case. It is up to the file-system implementation to
/// verify if a path is valid.
#[repr(transparent)]
#[derive(Debug)]
pub struct Path(str);

impl Path {
    /// Directly wraps a string slice as a `Path` slice.
    pub fn new<S: AsRef<str> + ?Sized>(str: &S) -> &Self {
        unsafe { &*(str.as_ref() as *const str as *const Path) }
    }

    /// Returns the underlying `str`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    /// Returns an Iterator of type [`Components`].
    pub fn components(&self, separator: char) -> Components<'_> {
        let split = self.0.split(separator);
        Components::new(self, split)
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        self.as_str().as_ref()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// [`Iterator`] over the [`Component`]s of a [`Path`].
#[derive(Debug)]
pub struct Components<'a> {
    path: &'a Path,
    split: core::str::Split<'a, char>,
    i: usize,
}

impl<'a> Components<'a> {
    fn new(path: &'a Path, split: core::str::Split<'a, char>) -> Self {
        Self { path, split, i: 0 }
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.0.is_empty() {
            return None;
        };

        self.split.next().map(|str| match str {
            "." => Component::CurDir,
            ".." => Component::ParentDir,
            "" if self.i == 0 => Component::RootDir,
            normal => Component::Normal(normal),
        })
    }
}

/// Components of a [`Path`].
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Component<'a> {
    /// Current dir: `.`
    CurDir,
    /// Parent dir: `..`
    ParentDir,
    /// Root directory: `/`
    RootDir,
    /// Normal directory or filename.
    Normal(&'a str),
}

impl<'a> Display for Component<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Component::CurDir => f.write_str(".\\"),
            Component::ParentDir => f.write_str("..\\"),
            Component::RootDir => f.write_str("\\"),
            Component::Normal(normal) => f.write_str(normal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn path_creation() {
        let path_str = "/foo/bar/foobar";
        let _path = Path::new(path_str);
        let path: &Path = path_str.as_ref();
        let _path: &Path = path.as_ref();
    }

    #[test]
    fn path_components() {
        let path_str = "/foo/./../bar/foobar";
        let path = Path::new(path_str);
        assert_eq!(path_str, path.as_str());
        let components = path.components('/').collect::<Vec<_>>();
        let expected = [
            Component::RootDir,
            Component::Normal("foo"),
            Component::CurDir,
            Component::ParentDir,
            Component::Normal("bar"),
            Component::Normal("foobar"),
        ];
        assert_eq!(components.as_slice(), expected.as_slice());

        let path = Path::new("./foo");
        let components = path.components('/').collect::<Vec<_>>();
        let expected = [Component::CurDir, Component::Normal("foo")];
        assert_eq!(components.as_slice(), expected.as_slice());

        let path = Path::new("");
        assert_eq!(path.components('/').count(), 0);
    }
}
