use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileMode, FileType};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::partition::PartitionInfo;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::CString16;

/// Open and read a test file in the boot directory.
pub fn test_open_and_read(directory: &mut Directory) {
    let test_input_path = CString16::try_from("EFI\\BOOT\\test_input.txt").unwrap();
    match directory.open(&test_input_path, FileMode::Read, FileAttribute::empty()) {
        Ok(file) => {
            let file = file.unwrap().into_type().unwrap_success();
            if let FileType::Regular(mut file) = file {
                let mut buffer = vec![0; 128];
                let size = file
                    .read(&mut buffer)
                    .expect_success(&format!("failed to read {}", test_input_path));
                let buffer = &buffer[..size];
                info!("Successfully read {}", test_input_path);
                assert_eq!(buffer, b"test input data");
            } else {
                panic!("{} is not a regular file", test_input_path);
            }
        }
        Err(err) => {
            let msg = format!("Failed to open {}: {:?}", test_input_path, err);
            // The file might reasonably not be present when running on real
            // hardware, so only panic on failure under qemu.
            if cfg!(feature = "qemu") {
                panic!("{}", msg);
            } else {
                warn!("{}", msg);
            }
        }
    }
}

pub fn test(image: Handle, bt: &BootServices) {
    info!("Testing Media Access protocols");

    if let Ok(sfs) = bt.locate_protocol::<SimpleFileSystem>() {
        let sfs = sfs.expect("Cannot open `SimpleFileSystem` protocol");
        let sfs = unsafe { &mut *sfs.get() };
        let mut directory = sfs.open_volume().unwrap().unwrap();
        let mut buffer = vec![0; 128];
        loop {
            let file_info = match directory.read_entry(&mut buffer) {
                Ok(completion) => {
                    if let Some(info) = completion.unwrap() {
                        info
                    } else {
                        // We've reached the end of the directory
                        break;
                    }
                }
                Err(error) => {
                    // Buffer is not big enough, allocate a bigger one and try again.
                    let min_size = error.data().unwrap();
                    buffer.resize(min_size, 0);
                    continue;
                }
            };
            info!("Root directory entry: {:?}", file_info);
        }
        directory.reset_entry_readout().unwrap().unwrap();

        test_open_and_read(&mut directory);
    } else {
        warn!("`SimpleFileSystem` protocol is not available");
    }

    let handles = bt
        .find_handles::<PartitionInfo>()
        .expect_success("Failed to get handles for `PartitionInfo` protocol");

    for handle in handles {
        let pi = bt
            .open_protocol::<PartitionInfo>(
                OpenProtocolParams {
                    handle,
                    agent: image,
                    controller: None,
                },
                OpenProtocolAttributes::Exclusive,
            )
            .expect_success("Failed to get partition info");
        let pi = unsafe { &*pi.interface.get() };

        if let Some(mbr) = pi.mbr_partition_record() {
            info!("MBR partition: {:?}", mbr);
        } else if let Some(gpt) = pi.gpt_partition_entry() {
            info!("GPT partition: {:?}", gpt);
        } else {
            info!("Unknown partition");
        }
    }
}
