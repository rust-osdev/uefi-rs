//! Tests functionality from the `uefi::fs` module. See function [`test`].

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::fs::{FileSystem, FileSystemError};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::ScopedProtocol;

/// Tests functionality from the `uefi::fs` module. This test relies on a
/// working File System Protocol, which is tested at a dedicated place.
pub fn test(sfs: ScopedProtocol<SimpleFileSystem>) -> Result<(), FileSystemError> {
    let mut fs = FileSystem::new(sfs);

    fs.create_dir("test_file_system_abs")?;

    // slash is transparently transformed to backslash
    fs.write("test_file_system_abs/foo", "hello")?;
    // absolute or relative paths are supported; ./ is ignored
    fs.copy("\\test_file_system_abs/foo", "\\test_file_system_abs/./bar")?;
    let read = fs.read("\\test_file_system_abs\\bar")?;
    let read = String::from_utf8(read).expect("Should be valid utf8");
    assert_eq!(read, "hello");

    assert_eq!(
        fs.try_exists("test_file_system_abs\\barfoo"),
        Err(FileSystemError::OpenError(
            "\\test_file_system_abs\\barfoo".to_string()
        ))
    );
    fs.rename("test_file_system_abs\\bar", "test_file_system_abs\\barfoo")?;
    assert!(fs.try_exists("test_file_system_abs\\barfoo").is_ok());

    let entries = fs
        .read_dir("test_file_system_abs")?
        .map(|e| {
            e.expect("Should return boxed file info")
                .file_name()
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(&[".", "..", "foo", "barfoo"], entries.as_slice());

    fs.create_dir("/deeply_nested_test")?;
    fs.create_dir("/deeply_nested_test/1")?;
    fs.create_dir("/deeply_nested_test/1/2")?;
    fs.create_dir("/deeply_nested_test/1/2/3")?;
    fs.create_dir("/deeply_nested_test/1/2/3/4")?;
    fs.create_dir_all("/deeply_nested_test/1/2/3/4/5/6/7")?;
    fs.try_exists("/deeply_nested_test/1/2/3/4/5/6/7")?;
    // TODO
    // fs.remove_dir_all("/deeply_nested_test/1/2/3/4/5/6/7")?;
    fs.remove_dir("/deeply_nested_test/1/2/3/4/5/6/7")?;
    let exists = matches!(fs.try_exists("/deeply_nested_test/1/2/3/4/5/6/7"), Ok(_));
    assert!(!exists);

    Ok(())
}
