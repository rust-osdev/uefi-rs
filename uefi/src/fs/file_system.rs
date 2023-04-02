//! Module for [`FileSystem`].

use super::*;
use crate::proto::media::file::{FileAttribute, FileInfo, FileType};
use crate::table::boot::ScopedProtocol;
use alloc::boxed::Box;
use alloc::string::{FromUtf8Error, String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ops::Deref;
use derive_more::Display;
use log::info;

/// All errors that can happen when working with the [`FileSystem`].
#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum FileSystemError {
    /// Can't open the root directory of the underlying volume.
    CantOpenVolume,
    /// The path is invalid because of the underlying [`PathError`].
    IllegalPath(PathError),
    /// The file or directory was not found in the underlying volume.
    FileNotFound(String),
    /// The path is existent but does not correspond to a directory when a
    /// directory was expected.
    NotADirectory(String),
    /// The path is existent but does not correspond to a file when a file was
    /// expected.
    NotAFile(String),
    /// Can't delete the file.
    CantDeleteFile(String),
    /// Can't delete the directory.
    CantDeleteDirectory(String),
    /// Error writing bytes.
    WriteFailure,
    /// Error flushing file.
    FlushFailure,
    /// Error reading file.
    ReadFailure,
    /// Can't parse file content as UTF-8.
    Utf8Error(FromUtf8Error),
    /// Could not open the given path.
    OpenError(String),
}

#[cfg(feature = "unstable")]
impl core::error::Error for FileSystemError {}

impl From<PathError> for FileSystemError {
    fn from(err: PathError) -> Self {
        Self::IllegalPath(err)
    }
}

/// Return type for public [`FileSystem`] operations.
pub type FileSystemResult<T> = Result<T, FileSystemError>;

/// High-level file-system abstraction for UEFI volumes with an API that is
/// close to `std::fs`. It acts as convenient accessor around the
/// [`SimpleFileSystemProtocol`].
pub struct FileSystem<'a>(ScopedProtocol<'a, SimpleFileSystemProtocol>);

impl<'a> FileSystem<'a> {
    /// Constructor.
    #[must_use]
    pub fn new(proto: ScopedProtocol<'a, SimpleFileSystemProtocol>) -> Self {
        Self(proto)
    }

    /// Tests if the underlying file exists. If this returns `Ok`, the file
    /// exists.
    pub fn try_exists(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        self.metadata(path).map(|_| ())
    }

    /// Copies the contents of one file to another. Creates the destination file
    /// if it doesn't exist and overwrites any content, if it exists.
    pub fn copy(
        &mut self,
        src_path: impl AsRef<Path>,
        dest_path: impl AsRef<Path>,
    ) -> FileSystemResult<()> {
        let read = self.read(src_path)?;
        self.write(dest_path, read)
    }

    /// Creates a new, empty directory at the provided path
    pub fn create_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();
        self.open(path, UefiFileMode::CreateReadWrite, true)
            .map(|_| ())
            .map_err(|err| {
                log::debug!("failed to fetch file info: {err:#?}");
                FileSystemError::OpenError(path.to_string())
            })
    }

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    pub fn create_dir_all(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        let normalized_path = NormalizedPath::new("\\", path)?;
        let normalized_path_string = normalized_path.to_string();
        let normalized_path_pathref = Path::new(&normalized_path_string);

        let iter = || normalized_path_pathref.components(SEPARATOR);
        iter()
            .scan(String::new(), |path_acc, component| {
                if component != Component::RootDir {
                    *path_acc += SEPARATOR_STR;
                    *path_acc += format!("{component}").as_str();
                }
                info!("path_acc: {path_acc}, component: {component}");
                Some((component, path_acc.clone()))
            })
            .try_for_each(|(_component, full_path)| self.create_dir(full_path.as_str()))
    }

    /// Given a path, query the file system to get information about a file,
    /// directory, etc. Returns [`UefiFileInfo`].
    pub fn metadata(&mut self, path: impl AsRef<Path>) -> FileSystemResult<Box<UefiFileInfo>> {
        let path = path.as_ref();
        let file = self.open(path, UefiFileMode::Read, false)?;
        log::debug!("{:#?}", &file.into_type().unwrap());
        let mut file = self.open(path, UefiFileMode::Read, false)?;
        file.get_boxed_info().map_err(|err| {
            log::debug!("failed to fetch file info: {err:#?}");
            FileSystemError::OpenError(path.to_string())
        })
    }

    /// Read the entire contents of a file into a bytes vector.
    pub fn read(&mut self, path: impl AsRef<Path>) -> FileSystemResult<Vec<u8>> {
        let path = path.as_ref();

        let mut file = self
            .open(path, UefiFileMode::Read, false)?
            .into_regular_file()
            .ok_or(FileSystemError::NotAFile(path.as_str().to_string()))?;
        let info = file.get_boxed_info::<FileInfo>().map_err(|e| {
            log::error!("get info failed: {e:?}");
            FileSystemError::OpenError(path.as_str().to_string())
        })?;

        let mut vec = vec![0; info.file_size() as usize];
        let read_bytes = file.read(vec.as_mut_slice()).map_err(|e| {
            log::error!("reading failed: {e:?}");
            FileSystemError::ReadFailure
        })?;

        // we read the whole file at once!
        if read_bytes != info.file_size() as usize {
            log::error!("Did only read {}/{} bytes", info.file_size(), read_bytes);
        }

        Ok(vec)
    }

    /// Returns an iterator over the entries within a directory.
    pub fn read_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<UefiDirectoryIter> {
        let path = path.as_ref();
        let dir = self
            .open(path, UefiFileMode::Read, false)?
            .into_directory()
            .ok_or(FileSystemError::NotADirectory(path.as_str().to_string()))?;
        Ok(UefiDirectoryIter::new(dir))
    }

    /// Read the entire contents of a file into a string.
    pub fn read_to_string(&mut self, path: impl AsRef<Path>) -> FileSystemResult<String> {
        String::from_utf8(self.read(path)?).map_err(FileSystemError::Utf8Error)
    }

    /// Removes an empty directory.
    pub fn remove_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        let file = self
            .open(path, UefiFileMode::ReadWrite, false)?
            .into_type()
            .unwrap();

        match file {
            FileType::Dir(dir) => dir.delete().map_err(|e| {
                log::error!("error removing dir: {e:?}");
                FileSystemError::CantDeleteDirectory(path.as_str().to_string())
            }),
            FileType::Regular(_) => Err(FileSystemError::NotADirectory(path.as_str().to_string())),
        }
    }

    /*/// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    pub fn remove_dir_all(&mut self, _path: impl AsRef<Path>) -> FileSystemResult<()> {
        todo!()
    }*/

    /// Removes a file from the filesystem.
    pub fn remove_file(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        let file = self
            .open(path, UefiFileMode::ReadWrite, false)?
            .into_type()
            .unwrap();

        match file {
            FileType::Regular(file) => file.delete().map_err(|e| {
                log::error!("error removing file: {e:?}");
                FileSystemError::CantDeleteFile(path.as_str().to_string())
            }),
            FileType::Dir(_) => Err(FileSystemError::NotAFile(path.as_str().to_string())),
        }
    }

    /// Rename a file or directory to a new name, replacing the original file if
    /// it already exists.
    pub fn rename(
        &mut self,
        src_path: impl AsRef<Path>,
        dest_path: impl AsRef<Path>,
    ) -> FileSystemResult<()> {
        self.copy(&src_path, dest_path)?;
        self.remove_file(src_path)
    }

    /// Write a slice as the entire contents of a file. This function will
    /// create a file if it does not exist, and will entirely replace its
    /// contents if it does.
    pub fn write(
        &mut self,
        path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> FileSystemResult<()> {
        let path = path.as_ref();

        // since there is no .truncate() in UEFI, we delete the file first it it
        // exists.
        if self.try_exists(path).is_ok() {
            self.remove_file(path)?;
        }

        let mut handle = self
            .open(path, UefiFileMode::CreateReadWrite, false)?
            .into_regular_file()
            .unwrap();

        handle.write(content.as_ref()).map_err(|e| {
            log::error!("only wrote {e:?} bytes");
            FileSystemError::WriteFailure
        })?;
        handle.flush().map_err(|e| {
            log::error!("flush failure: {e:?}");
            FileSystemError::FlushFailure
        })?;
        Ok(())
    }

    /// Opens a fresh handle to the root directory of the volume.
    fn open_root(&mut self) -> FileSystemResult<UefiDirectoryHandle> {
        self.0.open_volume().map_err(|e| {
            log::error!("Can't open root volume: {e:?}");
            FileSystemError::CantOpenVolume
        })
    }

    /// Wrapper around [`Self::open_root`] that opens the provided path as
    /// absolute path.
    ///
    /// May create a file if [`UefiFileMode::CreateReadWrite`] is set. May
    /// create a directory if [`UefiFileMode::CreateReadWrite`] and `is_dir`
    /// is set.
    fn open(
        &mut self,
        path: &Path,
        mode: UefiFileMode,
        is_dir: bool,
    ) -> FileSystemResult<UefiFileHandle> {
        let path = NormalizedPath::new("\\", path)?;
        log::debug!("normalized path: {path}");

        let attr = if mode == UefiFileMode::CreateReadWrite && is_dir {
            FileAttribute::DIRECTORY
        } else {
            FileAttribute::empty()
        };

        self.open_root()?.open(&path, mode, attr).map_err(|x| {
            log::trace!("Can't open file {path}: {x:?}");
            FileSystemError::OpenError(path.to_string())
        })
    }
}

impl<'a> Debug for FileSystem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FileSystem(<>))")
            .field(&(self.0.deref() as *const _))
            .finish()
    }
}
