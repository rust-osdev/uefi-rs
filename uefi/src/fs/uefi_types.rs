//! Re-export of low-level UEFI types but prefixed with `Uefi`. This simplifies
//! to differ between high-level and low-level types and interfaces in this
//! module.

pub use crate::proto::media::file::{
    Directory as UefiDirectoryHandle, File as UefiFileTrait, FileAttribute as UefiFileAttribute,
    FileHandle as UefiFileHandle, FileInfo as UefiFileInfo, FileMode as UefiFileMode,
    FileType as UefiFileType, RegularFile as UefiRegularFileHandle,
};
pub use crate::proto::media::fs::SimpleFileSystem as SimpleFileSystemProtocol;
