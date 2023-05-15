//! Path validation for the purpose of the [`fs`] module. This is decoupled from
//! [`Path`] and [`PathBuf`], as the Rust standard library also does it this
//! way. Instead, the FS implementation is responsible for that.
//!
//! [`PathBuf`]: super::PathBuf
//! [`fs`]: crate::fs

use super::Path;
use crate::fs::CHARACTER_DENY_LIST;
use crate::Char16;
use core::fmt::{self, Display, Formatter};

/// Errors related to file paths.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PathError {
    /// The path is empty / points to nothing.
    Empty,
    /// A component of the path is empty, i.e., two separators without content
    /// in between were found.
    EmptyComponent,
    /// There are illegal characters in the path.
    IllegalChar(Char16),
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "path is empty"),
            Self::EmptyComponent => write!(f, "path contains an empty component"),
            Self::IllegalChar(c) => {
                write!(
                    f,
                    "path contains an illegal character (value {})",
                    u16::from(*c)
                )
            }
        }
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for PathError {}

/// Validates a path for the needs of the [`fs`] module.
///
/// [`fs`]: crate::fs
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<(), PathError> {
    let path = path.as_ref();
    if path.is_empty() {
        return Err(PathError::Empty);
    }
    for component in path.components() {
        if component.is_empty() {
            return Err(PathError::EmptyComponent);
        } else if let Some(char) = component
            .as_slice()
            .iter()
            .find(|c| CHARACTER_DENY_LIST.contains(c))
        {
            return Err(PathError::IllegalChar(*char));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::PathBuf;
    use crate::CString16;
    use uefi_macros::cstr16;

    #[test]
    fn test_validate_path() {
        validate_path(cstr16!("hello\\foo\\bar")).unwrap();

        let err = validate_path(cstr16!("hello\\f>oo\\bar")).unwrap_err();
        assert_eq!(err, PathError::IllegalChar(CHARACTER_DENY_LIST[6]));

        let err = validate_path(cstr16!("hello\\\\bar")).unwrap_err();
        assert_eq!(err, PathError::EmptyComponent);

        let empty_cstring16 = CString16::try_from("").unwrap();
        let path = PathBuf::from(empty_cstring16);
        let err = validate_path(path).unwrap_err();
        assert_eq!(err, PathError::Empty)
    }
}
