// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module for [`FileSystem`].

use crate::Status;
use crate::fs::*;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Debug, Formatter};
use uefi::boot::ScopedProtocol;

/// Return type for public [`FileSystem`] operations.
pub type FileSystemResult<T> = Result<T, Error>;

/// High-level file-system abstraction for UEFI volumes.
///
/// Its API resembles `std::fs` and wraps [`SimpleFileSystemProtocol`].
///
/// Please refer to the [module documentation] for more information.
///
/// [module documentation]: uefi::fs
pub struct FileSystem(ScopedProtocol<SimpleFileSystemProtocol>);

impl FileSystem {
    /// Creates a file-system accessor for `proto`.
    #[must_use]
    pub fn new(proto: impl Into<Self>) -> Self {
        proto.into()
    }

    /// Returns `Ok(true)` if the path points at an existing file.
    ///
    /// If the file does not exist, `Ok(false)` is returned. If it cannot be
    /// determined whether the file exists or not, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot inspect the path.
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

    /// Copies one file to another.
    ///
    /// The destination is created if necessary and overwritten if it exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if a file
    /// operation fails.
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
            .ok_or_else(|| {
                Error::Io(IoError {
                    path: src_path.to_path_buf(),
                    context: IoErrorContext::NotAFile,
                    uefi_error: Status::INVALID_PARAMETER.into(),
                })
            })?;

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
            .ok_or_else(|| {
                Error::Io(IoError {
                    path: dest_path.to_path_buf(),
                    context: IoErrorContext::OpenError,
                    uefi_error: Status::INVALID_PARAMETER.into(),
                })
            })?;

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

    /// Creates an empty directory at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot create the directory.
    pub fn create_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();
        self.open(path, UefiFileMode::CreateReadWrite, true)
            .map(|_| ())
    }

    /// Creates a directory and any missing parents.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot inspect or create a directory.
    pub fn create_dir_all(&mut self, path: impl AsRef<Path>) -> FileSystemResult<()> {
        let path = path.as_ref();

        // Collect all relevant sub paths in a vector.
        let mut dirs_to_create = vec![path.to_path_buf()];
        while let Some(parent) = dirs_to_create.last().unwrap().parent() {
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

    /// Returns metadata for a file or directory.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot open the path or read its metadata.
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

    /// Reads an entire file into a byte vector.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot open, inspect, or read the file.
    pub fn read(&mut self, path: impl AsRef<Path>) -> FileSystemResult<Vec<u8>> {
        let path = path.as_ref();

        let mut file = self
            .open(path, UefiFileMode::Read, false)?
            .into_regular_file()
            .ok_or_else(|| {
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::NotAFile,
                    // We do not have a real UEFI error here as we have a logical
                    // problem.
                    uefi_error: Status::INVALID_PARAMETER.into(),
                })
            })?;

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
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot open the directory.
    pub fn read_dir(&mut self, path: impl AsRef<Path>) -> FileSystemResult<UefiDirectoryIter> {
        let path = path.as_ref();
        let dir = self
            .open(path, UefiFileMode::Read, false)?
            .into_directory()
            .ok_or_else(|| {
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::NotADirectory,
                    // We do not have a real UEFI error here as we have a logical
                    // problem.
                    uefi_error: Status::INVALID_PARAMETER.into(),
                })
            })?;
        Ok(UefiDirectoryIter::new(dir))
    }

    /// Reads an entire UTF-8 file into a [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path, [`Error::Io`] if firmware
    /// cannot read the file, or [`Error::Utf8Encoding`] for invalid UTF-8.
    pub fn read_to_string(&mut self, path: impl AsRef<Path>) -> FileSystemResult<String> {
        String::from_utf8(self.read(path)?).map_err(Error::Utf8Encoding)
    }

    /// Removes an empty directory.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if the path
    /// is not a directory or firmware cannot remove it.
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

    /// Recursively removes a directory and all of its contents.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot enumerate or remove an entry.
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if the path
    /// is not a file or firmware cannot remove it.
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

    /// Renames a file, replacing the destination if it exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot copy or remove the file.
    pub fn rename(
        &mut self,
        src_path: impl AsRef<Path>,
        dest_path: impl AsRef<Path>,
    ) -> FileSystemResult<()> {
        self.copy(&src_path, dest_path)?;
        self.remove_file(src_path)
    }

    /// Writes bytes as the entire contents of a file.
    ///
    /// The file is created if necessary and replaced if it exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Path`] for an invalid path or [`Error::Io`] if firmware
    /// cannot create, write, or flush the file.
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
                Error::Io(IoError {
                    path: path.to_path_buf(),
                    context: IoErrorContext::OpenError,
                    uefi_error: err,
                })
            })
    }
}

impl Debug for FileSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ptr: *const _ = &self.0;
        f.debug_tuple("FileSystem").field(&ptr).finish()
    }
}

impl From<uefi::boot::ScopedProtocol<SimpleFileSystemProtocol>> for FileSystem {
    fn from(proto: uefi::boot::ScopedProtocol<SimpleFileSystemProtocol>) -> Self {
        Self(proto)
    }
}
