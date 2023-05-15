use crate::fs::{PathBuf, PathError};
use alloc::string::FromUtf8Error;
use core::fmt::Debug;
use derive_more::Display;

/// All errors that can happen when working with the [`FileSystem`].
///
/// [`FileSystem`]: super::FileSystem
#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum Error {
    /// IO (low-level UEFI-errors) errors. See [`IoError`].
    Io(IoError),
    /// Path-related errors. See [`PathError`].
    Path(PathError),
    /// Can't parse file content as UTF-8. See [`FromUtf8Error`].
    Utf8Encoding(FromUtf8Error),
}

/// UEFI-error with context when working with the underlying UEFI file protocol.
#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display(fmt = "IoError({},{})", context, path)]
pub struct IoError {
    /// The path that led to the error.
    pub path: PathBuf,
    /// The context in which the path was used.
    pub context: IoErrorContext,
    /// The underlying UEFI error.
    pub uefi_error: crate::Error,
}

/// Enum that further specifies the context in that an [`Error`] occurred.
#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum IoErrorContext {
    /// Can't delete the directory.
    CantDeleteDirectory,
    /// Can't delete the file.
    CantDeleteFile,
    /// Error flushing file.
    FlushFailure,
    /// Can't open the root directory of the underlying volume.
    CantOpenVolume,
    /// Error while reading the metadata of the file.
    Metadata,
    /// Could not open the given path. One possible reason is that the file does
    /// not exist.
    OpenError,
    /// Error reading file.
    ReadFailure,
    /// Error writing bytes.
    WriteFailure,
    /// The path exists but does not correspond to a directory when a directory
    /// was expected.
    NotADirectory,
    /// The path exists but does not correspond to a file when a file was
    /// expected.
    NotAFile,
}

impl From<PathError> for Error {
    fn from(value: PathError) -> Self {
        Self::Path(value)
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Path(err) => Some(err),
            Error::Utf8Encoding(err) => Some(err),
        }
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for IoError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.uefi_error)
    }
}
