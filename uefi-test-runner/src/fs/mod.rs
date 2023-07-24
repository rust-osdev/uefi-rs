//! Tests functionality from the `uefi::fs` module. See function [`test`].

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::fs::{FileSystem, IoError, IoErrorContext, PathBuf};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::ScopedProtocol;
use uefi::{cstr16, fs, Status};

/// Tests functionality from the `uefi::fs` module. This test relies on a
/// working File System Protocol, which is tested at a dedicated place.
pub fn test(sfs: ScopedProtocol<SimpleFileSystem>) -> Result<(), fs::Error> {
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

    // test rename file + path buf replaces / with \
    fs.rename(
        PathBuf::from(cstr16!("/foo_dir/foo_cpy")),
        cstr16!("foo_dir\\foo_cpy2"),
    )?;
    // file should not be available after rename
    let err = fs.read(cstr16!("foo_dir\\foo_cpy"));
    assert!(err.is_err());

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
    fs.remove_dir_all(cstr16!("foo_dir\\1"))?;
    // file should not be available after remove all
    assert!(!fs.try_exists(cstr16!("foo_dir\\1"))?);

    test_copy_error(&mut fs)?;
    test_copy_success(&mut fs)?;
    test_copy_success_chunks(&mut fs)?;

    Ok(())
}

fn test_copy_error(fs: &mut FileSystem) -> Result<(), fs::Error> {
    let file1_path = cstr16!("file1");
    let dir_path = cstr16!("dir");

    // Test copy when the destination exists but the source does not. Verify
    // that the destination is not deleted or altered.
    fs.write(file1_path, "data1")?;
    assert_eq!(
        fs.copy(cstr16!("src"), file1_path),
        Err(fs::Error::Io(IoError {
            path: PathBuf::from(cstr16!("src")),
            context: IoErrorContext::OpenError,
            uefi_error: uefi::Error::new(Status::NOT_FOUND, ()),
        }))
    );
    assert_eq!(fs.read(file1_path)?, b"data1");

    // Test copy when the source is a directory. Verify that the destination is
    // not deleted or altered.
    fs.create_dir(dir_path)?;
    assert_eq!(
        fs.copy(dir_path, file1_path),
        Err(fs::Error::Io(IoError {
            path: PathBuf::from(dir_path),
            context: IoErrorContext::NotAFile,
            uefi_error: uefi::Error::new(Status::INVALID_PARAMETER, ()),
        }))
    );
    assert_eq!(fs.read(file1_path)?, b"data1");

    // Test copy when the source is valid but the destination is a
    // directory. Verify that the directory is not deleted.
    assert_eq!(
        fs.copy(file1_path, dir_path),
        Err(fs::Error::Io(IoError {
            path: PathBuf::from(dir_path),
            context: IoErrorContext::OpenError,
            uefi_error: uefi::Error::new(Status::INVALID_PARAMETER, ()),
        }))
    );
    assert_eq!(fs.try_exists(dir_path), Ok(true));

    // Clean up temporary files.
    fs.remove_file(file1_path)?;
    fs.remove_dir(dir_path)?;

    Ok(())
}

fn test_copy_success(fs: &mut FileSystem) -> Result<(), fs::Error> {
    let file1_path = cstr16!("file1");
    let file2_path = cstr16!("file2");

    // Test a successful copy where the destination does not already exist.
    fs.write(file1_path, "data1")?;
    assert_eq!(fs.try_exists(file2_path), Ok(false));
    fs.copy(file1_path, file2_path)?;
    assert_eq!(fs.read(file1_path)?, b"data1");
    assert_eq!(fs.read(file2_path)?, b"data1");

    // Test that when copying a smaller file over a larger file, the file is
    // properly truncated. Also check that the original file is unchanged.
    fs.write(file2_path, "some long data")?;
    fs.copy(file1_path, file2_path)?;
    assert_eq!(fs.read(file1_path)?, b"data1");
    assert_eq!(fs.read(file2_path)?, b"data1");

    // Clean up temporary files.
    fs.remove_file(file1_path)?;
    fs.remove_file(file2_path)?;

    Ok(())
}

fn test_copy_success_chunks(fs: &mut FileSystem) -> Result<(), fs::Error> {
    let file1_path = cstr16!("file1");
    let file2_path = cstr16!("file2");

    // Test copy of a large file, where the file's size is an even multiple of
    // the 1MiB chunk size.
    let chunk_size = 1024 * 1024;
    let mut big_buf = Vec::with_capacity(5 * chunk_size);
    for i in 0..(4 * chunk_size) {
        let byte = u8::try_from(i % 255).unwrap();
        big_buf.push(byte);
    }
    fs.write(file1_path, &big_buf)?;
    fs.copy(file1_path, file2_path)?;
    assert_eq!(fs.read(file1_path)?, big_buf);
    assert_eq!(fs.read(file2_path)?, big_buf);

    // Test copy of a large file, where the file's size is not an even multiple
    // of the 1MiB chunk size.
    big_buf.extend(b"some extra data");
    assert_ne!(big_buf.len() % chunk_size, 0);
    fs.write(file1_path, &big_buf)?;
    fs.copy(file1_path, file2_path)?;
    assert_eq!(fs.read(file1_path)?, big_buf);
    assert_eq!(fs.read(file2_path)?, big_buf);

    // Clean up temporary files.
    fs.remove_file(file1_path)?;
    fs.remove_file(file2_path)?;

    Ok(())
}
