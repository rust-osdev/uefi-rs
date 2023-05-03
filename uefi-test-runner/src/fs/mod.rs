//! Tests functionality from the `uefi::fs` module. See function [`test`].

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::cstr16;
use uefi::fs::{FileSystem, FileSystemError, PathBuf};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::ScopedProtocol;

/// Tests functionality from the `uefi::fs` module. This test relies on a
/// working File System Protocol, which is tested at a dedicated place.
pub fn test(sfs: ScopedProtocol<SimpleFileSystem>) -> Result<(), FileSystemError> {
    let mut fs = FileSystem::new(sfs);

    // test create dir
    fs.create_dir(cstr16!("foo_dir"))?;

    // test write, copy, and read
    let data_to_write = "hello world";
    fs.write(cstr16!("foo_dir\\foo"), data_to_write)?;
    // Here, we additionally check that absolute paths work.
    fs.copy(cstr16!("\\foo_dir\\foo"), cstr16!("\\foo_dir\\foo_cpy"))?;
    let read = fs.read(cstr16!("foo_dir\\foo_cpy"))?;
    let read = String::from_utf8(read).expect("Should be valid utf8");
    assert_eq!(read.as_str(), data_to_write);

    // test copy from non-existent file
    let err = fs.copy(cstr16!("not_found"), cstr16!("abc"));
    assert!(matches!(err, Err(FileSystemError::OpenError { .. })));

    // test rename file + path buf replaces / with \
    fs.rename(
        PathBuf::from(cstr16!("/foo_dir/foo_cpy")),
        cstr16!("foo_dir\\foo_cpy2"),
    )?;
    // file should not be available after rename
    let err = fs.read(cstr16!("foo_dir\\foo_cpy"));
    assert!(matches!(err, Err(FileSystemError::OpenError { .. })));

    // test read dir on a sub dir
    let entries = fs
        .read_dir(cstr16!("foo_dir"))?
        .map(|entry| entry.expect("Should be valid").file_name().to_string())
        .collect::<Vec<_>>();
    assert_eq!(&[".", "..", "foo", "foo_cpy2"], entries.as_slice());

    // test create dir recursively
    fs.create_dir_all(cstr16!("foo_dir\\1\\2\\3\\4\\5\\6\\7"))?;
    fs.create_dir_all(cstr16!("foo_dir\\1\\2\\3\\4\\5\\6\\7\\8"))?;
    fs.write(
        cstr16!("foo_dir\\1\\2\\3\\4\\5\\6\\7\\8\\foobar"),
        data_to_write,
    )?;
    let boxinfo = fs.metadata(cstr16!("foo_dir\\1\\2\\3\\4\\5\\6\\7\\8\\foobar"))?;
    assert_eq!(boxinfo.file_size(), data_to_write.len() as u64);

    // test remove dir all
    // TODO

    Ok(())
}
