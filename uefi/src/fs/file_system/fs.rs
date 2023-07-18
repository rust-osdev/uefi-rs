//! Module for [`FileSystem`].

use crate::fs::{Path, PathBuf, UefiDirectoryIter, SEPARATOR_STR, *};
use crate::table::boot::ScopedProtocol;
use crate::Status;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ops::Deref;
use log::debug;

/// Return type for public [`FileSystem`] operations.
pub type FileSystemResult<T> = Result<T, Error>;

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

    /// Returns `Ok(true)` if the path points at an existing file.
    ///
    /// If the file does not exist, `Ok(false)` is returned. If it cannot be
    /// determined whether the file exists or not, an error is returned.
    pub fn try_exists(&mut self, path: impl AsRef<Path>) -> FileSystemResult<bool> {
        match self.open(path.as_ref(), UefiFileMode::Read, false) {
            Ok(_) => Ok(true),
            Err(Error::Io(err)) => {
                if err.uefi_error.status() == Status::NOT_FOUND {
                    Ok(false)
                } else {
                    Err(Error::Io(err))
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Copies the contents of one file to another. Creates the destination file
    /// if it doesn't exist and overwrites any content, if it exists.
    pub fn copy(
        &mut self,
        src_path: impl AsRef<Path>,
        dest_path: impl AsRef<Path>,
    ) -> FileSystemResult<()> {
        let src_path = src_path.as_ref();
        let dest_path = dest_path.as_ref();

        // Open the source file for reading.
        let mut src = self
            .open(src_path, UefiFileMode::Read, false)?
            .into_regular_file()
            .ok_or(Error::Io(IoError {
                path: src_path.to_path_buf(),
                context: IoErrorContext::NotAFile,
                uefi_error: Status::INVALID_PARAMETER.into(),
            }))?;

        // Get the source file's size in bytes.
        let src_size = {
            let src_info = src.get_boxed_info::<UefiFileInfo>().map_err(|err| {
                Error::Io(IoError {
                    path: src_path.to_path_buf(),
                    context: IoErrorContext::Metadata,
                    uefi_error: err,
                })
            })?;
            src_info.file_size()
        };

        // Try to delete the destination file in case it already exists. Allow
        // this to fail, since it might not exist. Or it might exist, but be a
        // directory, in which case the error will be caught when trying to
        // create the file.
        let _ = self.remove_file(dest_path);

        // Create and open the destination file.
        let mut dest = self
            .open(dest_path, UefiFileMode::CreateReadWrite, false)?
            .into_regular_file()
            .ok_or(Error::Io(IoError {
                path: dest_path.to_path_buf(),
                context: IoErrorContext::OpenError,
                uefi_error: Status::INVALID_PARAMETER.into(),
            }))?;

        // 1 MiB copy buffer.
        let mut chunk = vec![0; 1024 * 1024];

        // Read chunks from the source file and write to the destination file.
        let mut remaining_size = src_size;
        while remaining_size > 0 {
            // Read one chunk.
            let num_bytes_read = src.read(&mut chunk).map_err(|err| {
                Error::Io(IoError {
                    path: src_path.to_path_buf(),
                    context: IoErrorContext::ReadFailure,
                    uefi_error: err.to_err_without_payload(),
                })
            })?;

            // If the read returned no bytes, but `remaining_size > 0`, return
            // an error.
            if num_bytes_read == 0 {
                return Err(Error::Io(IoError {
                    path: src_path.to_path_buf(),
                    context: IoErrorContext::ReadFailure,
                    uefi_error: Status::ABORTED.into(),
                }));
            }

            // Copy the bytes read out to the destination file.
            dest.write(&chunk[..num_bytes_read]).map_err(|err| {
                Error::Io(IoError {
                    path: dest_path.to_path_buf(),
                    context: IoErrorContext::WriteFailure,
                    uefi_error: err.to_err_without_payload(),
                })
            })?;

            remaining_size -= u64::try_from(num_bytes_read).unwrap();
        }

        dest.flush().map_err(|err| {
            Error::Io(IoError {
                path: dest_path.to_path_buf(),
                context: IoErrorContext::FlushFailure,
                uefi_error: err,
            })
        })?;

        Ok(())
    }

    /// Creates a new, empty directory at the provided path
    pub fn create_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();
        self.open(path, UefiFileMode::CreateReadWrite, true)
            .map(|_| ())
    }

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    pub fn create_dir_all(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        // Collect all relevant sub paths in a vector.
        let mut dirs_to_create = vec![path.to_path_buf()];
        while let Some(parent) = dirs_to_create.last().unwrap().parent() {
            debug!("parent={parent}");
            dirs_to_create.push(parent)
        }
        // Now reverse, so that we have something like this:
        // - a
        // - a\\b
        // - a\\b\\c
        dirs_to_create.reverse();

        for parent in dirs_to_create {
            if !self.try_exists(&parent)? {
                self.create_dir(parent)?;
            }
        }

        Ok(())
    }

    /// Given a path, query the file system to get information about a file,
    /// directory, etc. Returns [`UefiFileInfo`].
    pub fn metadata(&mut self, path: impl AsRef<Path>) -> FileSystemResult<Box<UefiFileInfo>> {
        let path = path.as_ref();
        let mut file = self.open(path, UefiFileMode::Read, false)?;
        file.get_boxed_info().map_err(|err| {
            Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::Metadata,
                uefi_error: err,
            })
        })
    }

    /// Read the entire contents of a file into a bytes vector.
    pub fn read(&mut self, path: impl AsRef<Path>) -> FileSystemResult<Vec<u8>> {
        let path = path.as_ref();

        let mut file = self
            .open(path, UefiFileMode::Read, false)?
            .into_regular_file()
            .ok_or(Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::NotAFile,
                // We do not have a real UEFI error here as we have a logical
                // problem.
                uefi_error: Status::INVALID_PARAMETER.into(),
            }))?;

        let info = file.get_boxed_info::<UefiFileInfo>().map_err(|err| {
            Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::Metadata,
                uefi_error: err,
            })
        })?;

        let mut vec = vec![0; info.file_size() as usize];
        let read_bytes = file.read(vec.as_mut_slice()).map_err(|err| {
            Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::ReadFailure,
                uefi_error: err.to_err_without_payload(),
            })
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
            .ok_or(Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::NotADirectory,
                // We do not have a real UEFI error here as we have a logical
                // problem.
                uefi_error: Status::INVALID_PARAMETER.into(),
            }))?;
        Ok(UefiDirectoryIter::new(dir))
    }

    /// Read the entire contents of a file into a Rust string.
    pub fn read_to_string(&mut self, path: impl AsRef<Path>) -> FileSystemResult<String> {
        String::from_utf8(self.read(path)?).map_err(Error::Utf8Encoding)
    }

    /// Removes an empty directory.
    pub fn remove_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        let file = self
            .open(path, UefiFileMode::ReadWrite, false)?
            .into_type()
            .unwrap();

        match file {
            UefiFileType::Dir(dir) => dir.delete().map_err(|err| {
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::CantDeleteDirectory,
                    uefi_error: err,
                })
            }),
            UefiFileType::Regular(_) => {
                Err(Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::NotADirectory,
                    // We do not have a real UEFI error here as we have a logical
                    // problem.
                    uefi_error: Status::INVALID_PARAMETER.into(),
                }))
            }
        }
    }

    /// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    pub fn remove_dir_all(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();
        for file_info in self
            .read_dir(path)?
            .filter_map(|file_info_result| file_info_result.ok())
        {
            if COMMON_SKIP_DIRS.contains(&file_info.file_name()) {
                continue;
            }

            let mut abs_entry_path = PathBuf::new();
            abs_entry_path.push(path);
            abs_entry_path.push(file_info.file_name());
            if file_info.is_directory() {
                // delete all inner files
                // This recursion is fine as there are no links in UEFI/FAT file
                // systems. No cycles possible.
                self.remove_dir_all(&abs_entry_path)?;
            } else {
                self.remove_file(abs_entry_path)?;
            }
        }
        // Now that the dir is empty, we delete it as final step.
        self.remove_dir(path)?;
        Ok(())
    }

    /// Removes a file from the filesystem.
    pub fn remove_file(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        let file = self
            .open(path, UefiFileMode::ReadWrite, false)?
            .into_type()
            .unwrap();

        match file {
            UefiFileType::Regular(file) => file.delete().map_err(|err| {
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::CantDeleteFile,
                    uefi_error: err,
                })
            }),
            UefiFileType::Dir(_) => Err(Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::NotAFile,
                // We do not have a real UEFI error here as we have a logical
                // problem.
                uefi_error: Status::INVALID_PARAMETER.into(),
            })),
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
        if self.try_exists(path)? {
            self.remove_file(path)?;
        }

        let mut handle = self
            .open(path, UefiFileMode::CreateReadWrite, false)?
            .into_regular_file()
            .unwrap();

        handle.write(content.as_ref()).map_err(|err| {
            Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::WriteFailure,
                uefi_error: err.to_err_without_payload(),
            })
        })?;
        handle.flush().map_err(|err| {
            Error::Io(IoError {
                path: path.to_path_buf(),
                context: IoErrorContext::FlushFailure,
                uefi_error: err,
            })
        })?;
        Ok(())
    }

    /// Opens a fresh handle to the root directory of the volume.
    fn open_root(&mut self) -> FileSystemResult<UefiDirectoryHandle> {
        self.0.open_volume().map_err(|err| {
            Error::Io(IoError {
                path: {
                    let mut path = PathBuf::new();
                    path.push(SEPARATOR_STR);
                    path
                },
                context: IoErrorContext::CantOpenVolume,
                uefi_error: err,
            })
        })
    }

    /// Wrapper around [`Self::open_root`] that opens the provided path as
    /// absolute path.
    ///
    /// May create a file if [`UefiFileMode::CreateReadWrite`] is set. May
    /// create a directory if [`UefiFileMode::CreateReadWrite`] and `create_dir`
    /// is set. The parameter `create_dir` is ignored otherwise.
    fn open(
        &mut self,
        path: &Path,
        mode: UefiFileMode,
        create_dir: bool,
    ) -> FileSystemResult<UefiFileHandle> {
        validate_path(path)?;

        let attr = if mode == UefiFileMode::CreateReadWrite && create_dir {
            UefiFileAttribute::DIRECTORY
        } else {
            UefiFileAttribute::empty()
        };

        self.open_root()?
            .open(path.to_cstr16(), mode, attr)
            .map_err(|err| {
                log::trace!("Can't open file {path}: {err:?}");
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::OpenError,
                    uefi_error: err,
                })
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
