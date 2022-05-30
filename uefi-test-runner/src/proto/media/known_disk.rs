use alloc::string::ToString;
use uefi::prelude::*;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileSystemInfo,
};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::table::runtime::{Daylight, Time, TimeParams};

/// Test directory entry iteration.
fn test_existing_dir(directory: &mut Directory) {
    info!("Testing existing directory");

    let input_dir_path = cstr16!("test_dir");
    let mut dir = directory
        .open(input_dir_path, FileMode::Read, FileAttribute::empty())
        .expect("failed to open directory")
        .into_directory()
        .expect("not a directory");

    // Collect and validate the directory entries.
    let mut entry_names = vec![];
    let mut buf = vec![0; 200];
    loop {
        let entry = dir.read_entry(&mut buf).expect("failed to read directory");
        if let Some(entry) = entry {
            entry_names.push(entry.file_name().to_string());
        } else {
            break;
        }
    }
    assert_eq!(entry_names, [".", "..", "test_input.txt"]);
}

/// Test that deleting a file opened in read-only mode fails with a
/// warning. This is mostly just an excuse to verify that warnings are
/// properly converted to errors.
fn test_delete_warning(directory: &mut Directory) {
    let input_file_path = cstr16!("test_dir\\test_input.txt");
    let file = directory
        .open(input_file_path, FileMode::Read, FileAttribute::empty())
        .expect("failed to open file")
        .into_regular_file()
        .expect("not a regular file");

    assert_eq!(
        file.delete().unwrap_err().status(),
        Status::WARN_DELETE_FAILURE
    );
}

/// Test operations on an existing file.
fn test_existing_file(directory: &mut Directory) {
    info!("Testing existing file");

    // Open an existing file.
    let input_file_path = cstr16!("test_dir\\test_input.txt");
    let mut file = directory
        .open(input_file_path, FileMode::ReadWrite, FileAttribute::empty())
        .expect("failed to open file")
        .into_regular_file()
        .expect("not a regular file");

    // Read the file.
    let mut buffer = vec![0; 128];
    let size = file.read(&mut buffer).expect("failed to read file");
    let buffer = &buffer[..size];
    info!("Successfully read {}", input_file_path);
    assert_eq!(buffer, b"test input data");

    // Check file metadata.
    let mut info_buffer = vec![0; 128];
    let info = file.get_info::<FileInfo>(&mut info_buffer).unwrap();
    assert_eq!(info.file_size(), 15);
    assert_eq!(info.physical_size(), 512);
    let tp = TimeParams {
        year: 2000,
        month: 1,
        day: 24,
        hour: 0,
        minute: 0,
        second: 0,
        nanosecond: 0,
        time_zone: None,
        daylight: Daylight::empty(),
    };
    assert_eq!(*info.create_time(), Time::new(tp).unwrap());
    assert_eq!(
        *info.last_access_time(),
        Time::new(TimeParams {
            year: 2001,
            month: 2,
            day: 25,
            ..tp
        })
        .unwrap()
    );
    assert_eq!(
        *info.modification_time(),
        Time::new(TimeParams {
            year: 2002,
            month: 3,
            day: 26,
            ..tp
        })
        .unwrap()
    );
    assert_eq!(info.attribute(), FileAttribute::empty());
    assert_eq!(info.file_name(), cstr16!("test_input.txt"));

    // Check that `get_boxed_info` returns the same info.
    let boxed_info = file.get_boxed_info::<FileInfo>().unwrap();
    assert_eq!(*info, *boxed_info);

    // Delete the file.
    file.delete().unwrap();

    // Verify the file is gone.
    assert!(directory
        .open(input_file_path, FileMode::Read, FileAttribute::empty())
        .is_err());
}

/// Test file creation.
fn test_create_file(directory: &mut Directory) {
    info!("Testing file creation");

    // Create a new file.
    let mut file = directory
        .open(
            cstr16!("new_test_file.txt"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .expect("failed to create file")
        .into_regular_file()
        .expect("not a regular file");
    file.write(b"test output data").unwrap();
}

/// Run various tests on a special test disk. The disk is created by
/// xtask/src/disk.rs.
pub fn test_known_disk(image: Handle, bt: &BootServices) {
    // This test is only valid when running in the specially-prepared
    // qemu with the test disk.
    if !cfg!(feature = "qemu") {
        return;
    }

    let handles = bt
        .find_handles::<SimpleFileSystem>()
        .expect("Failed to get handles for `SimpleFileSystem` protocol");
    assert_eq!(handles.len(), 2);

    let mut found_test_disk = false;
    for handle in handles {
        let mut sfs = bt
            .open_protocol::<SimpleFileSystem>(
                OpenProtocolParams {
                    handle,
                    agent: image,
                    controller: None,
                },
                OpenProtocolAttributes::Exclusive,
            )
            .expect("Failed to get simple file system");
        let mut directory = sfs.open_volume().unwrap();

        let mut fs_info_buf = vec![0; 128];
        let fs_info = directory
            .get_info::<FileSystemInfo>(&mut fs_info_buf)
            .unwrap();

        if fs_info.volume_label().to_string() == "MbrTestDisk" {
            info!("Checking MbrTestDisk");
            found_test_disk = true;
        } else {
            continue;
        }

        assert!(!fs_info.read_only());
        assert_eq!(fs_info.volume_size(), 512 * 1192);
        assert_eq!(fs_info.free_space(), 512 * 1190);
        assert_eq!(fs_info.block_size(), 512);

        // Check that `get_boxed_info` returns the same info.
        let boxed_fs_info = directory.get_boxed_info::<FileSystemInfo>().unwrap();
        assert_eq!(*fs_info, *boxed_fs_info);

        test_existing_dir(&mut directory);
        test_delete_warning(&mut directory);
        test_existing_file(&mut directory);
        test_create_file(&mut directory);
    }

    if !found_test_disk {
        panic!("MbrTestDisk not found");
    }
}
