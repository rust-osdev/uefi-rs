use crate::fs::{PathBuf, PathError};
use alloc::string::FromUtf8Error;
use core::fmt::{self, Debug, Display, Formatter};

/// All errors that can happen when working with the [`FileSystem`].
///
/// [`FileSystem`]: super::FileSystem
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// IO (low-level UEFI-errors) errors. See [`IoError`].
    Io(IoError),
    /// Path-related errors. See [`PathError`].
    Path(PathError),
    /// Can't parse file content as UTF-8. See [`FromUtf8Error`].
    Utf8Encoding(FromUtf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => write!(f, "IO error"),
            Self::Path(_) => write!(f, "path error"),
            Self::Utf8Encoding(_) => write!(f, "UTF-8 encoding error"),
        }
    }
}

/// UEFI-error with context when working with the underlying UEFI file protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoError {
    /// The path that led to the error.
    pub path: PathBuf,
    /// The context in which the path was used.
    pub context: IoErrorContext,
    /// The underlying UEFI error.
    pub uefi_error: crate::Error,
}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "IO error for path {}: {}", self.path, self.context)
    }
}

/// Enum that further specifies the context in that an [`Error`] occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for IoErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CantDeleteDirectory => "failed to delete directory",
            Self::CantDeleteFile => "failed to delete file",
            Self::FlushFailure => "failed to flush file",
            Self::CantOpenVolume => "failed to open volume",
            Self::Metadata => "failed to read metadata",
            Self::OpenError => "failed to open file",
            Self::ReadFailure => "failed to read file",
            Self::WriteFailure => "failed to write file",
            Self::NotADirectory => "expected a directory",
            Self::NotAFile => "expected a file",
        };
        write!(f, "{s}")
    }
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
