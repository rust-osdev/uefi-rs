use alloc::string::ToString;
use core::ptr::NonNull;
use uefi::prelude::*;
use uefi::proto::media::block::BlockIO;
use uefi::proto::media::disk::{DiskIo, DiskIo2, DiskIo2Token};
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileSystemInfo,
};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{EventType, OpenProtocolAttributes, OpenProtocolParams, Tpl};
use uefi::table::runtime::{Daylight, Time, TimeParams};

/// Test directory entry iteration.
fn test_existing_dir(directory: &mut Directory) {
    info!("Testing existing directory");

    let input_dir_path = cstr16!("test_dir");
    let dir = directory
        .open(input_dir_path, FileMode::Read, FileAttribute::empty())
        .expect("failed to open directory");

    assert!(dir.is_directory().unwrap());

    let mut dir = dir.into_directory().expect("not a directory");

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
    let file = directory
        .open(
            cstr16!("new_test_file.txt"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .expect("failed to create file");

    assert!(file.is_regular_file().unwrap());

    let mut file = file.into_regular_file().expect("not a regular file");
    file.write(b"test output data").unwrap();
}

/// Get the media ID via the BlockIO protocol.
fn get_block_media_id(handle: Handle, bt: &BootServices) -> u32 {
    // This cannot be opened in `EXCLUSIVE` mode, as doing so
    // unregisters the `DiskIO` protocol from the handle.
    unsafe {
        let block_io = bt
            .open_protocol::<BlockIO>(
                OpenProtocolParams {
                    handle,
                    agent: bt.image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .expect("Failed to get block I/O protocol");
        block_io.media().media_id()
    }
}

/// Tests raw disk I/O.
fn test_raw_disk_io(handle: Handle, bt: &BootServices) {
    info!("Testing raw disk I/O");

    let media_id = get_block_media_id(handle, bt);

    // Open the disk I/O protocol on the input handle
    let disk_io = bt
        .open_protocol_exclusive::<DiskIo>(handle)
        .expect("Failed to get disk I/O protocol");

    // Read from the first sector of the disk into the buffer
    let mut buf = vec![0; 512];
    disk_io
        .read_disk(media_id, 0, &mut buf)
        .expect("Failed to read from disk");

    // Verify that the disk's MBR signature is correct
    assert_eq!(buf[510], 0x55);
    assert_eq!(buf[511], 0xaa);

    info!("Raw disk I/O succeeded");
}

/// Asynchronous disk I/O task context
#[repr(C)]
struct DiskIoTask {
    /// Token for the transaction
    token: DiskIo2Token,
    /// Buffer holding the read data
    buffer: [u8; 512],
}

/// Tests raw disk I/O through the DiskIo2 protocol.
fn test_raw_disk_io2(handle: Handle, bt: &BootServices) {
    info!("Testing raw disk I/O 2");

    // Open the disk I/O protocol on the input handle
    if let Ok(disk_io2) = bt.open_protocol_exclusive::<DiskIo2>(handle) {
        let media_id = get_block_media_id(handle, bt);

        unsafe {
            // Create the completion event
            let mut event = bt
                .create_event(EventType::empty(), Tpl::NOTIFY, None, None)
                .expect("Failed to create disk I/O completion event");

            // Initialise the task context
            let mut task = DiskIoTask {
                token: DiskIo2Token {
                    event: event.unsafe_clone(),
                    transaction_status: uefi::Status::NOT_READY,
                },
                buffer: [0; 512],
            };

            // Initiate the asynchronous read operation
            disk_io2
                .read_disk_raw(
                    media_id,
                    0,
                    NonNull::new(&mut task.token as _),
                    task.buffer.len(),
                    task.buffer.as_mut_ptr(),
                )
                .expect("Failed to initiate asynchronous disk I/O read");

            // Wait for the transaction to complete
            bt.wait_for_event(core::slice::from_mut(&mut event))
                .expect("Failed to wait on completion event");

            // Verify that the disk's MBR signature is correct
            assert_eq!(task.token.transaction_status, uefi::Status::SUCCESS);
            assert_eq!(task.buffer[510], 0x55);
            assert_eq!(task.buffer[511], 0xaa);

            info!("Raw disk I/O 2 succeeded");
        }
    }
}

/// Run various tests on a special test disk. The disk is created by
/// xtask/src/disk.rs.
pub fn test_known_disk(bt: &BootServices) {
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
        {
            let mut sfs = bt
                .open_protocol_exclusive::<SimpleFileSystem>(handle)
                .expect("Failed to get simple file system");
            let mut root_directory = sfs.open_volume().unwrap();

            // test is_directory() and is_regular_file() from the File trait which is the
            // base for into_type() used later in the test.
            {
                // because File is "Sized", we cannot cast it to &dyn
                fn test_is_directory(file: &impl File) {
                    assert_eq!(Ok(true), file.is_directory());
                    assert_eq!(Ok(false), file.is_regular_file());
                }
                test_is_directory(&root_directory);
            }

            let mut fs_info_buf = vec![0; 128];
            let fs_info = root_directory
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
            let boxed_fs_info = root_directory.get_boxed_info::<FileSystemInfo>().unwrap();
            assert_eq!(*fs_info, *boxed_fs_info);

            test_existing_dir(&mut root_directory);
            test_delete_warning(&mut root_directory);
            test_existing_file(&mut root_directory);
            test_create_file(&mut root_directory);
        }

        test_raw_disk_io(handle, bt);
        test_raw_disk_io2(handle, bt);
    }

    if !found_test_disk {
        panic!("MbrTestDisk not found");
    }
}
