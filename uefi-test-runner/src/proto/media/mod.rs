mod known_disk;

use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileSystemInfo, FileSystemVolumeLabel};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::partition::PartitionInfo;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};

/// Test `FileSystemInfo` and `FileSystemVolumeLabel`.
fn test_file_system_info(directory: &mut Directory) {
    let mut fs_info_buf = vec![0; 128];
    let fs_info = directory
        .get_info::<FileSystemInfo>(&mut fs_info_buf)
        .unwrap();
    info!("File system info: {:?}", fs_info);

    let mut fs_vol_buf = vec![0; 128];
    let fs_vol = directory
        .get_info::<FileSystemVolumeLabel>(&mut fs_vol_buf)
        .unwrap();
    info!("File system volume label: {:?}", fs_vol);

    // Both types should provide the same volume label.
    assert_eq!(fs_info.volume_label(), fs_vol.volume_label());
}

pub fn test(image: Handle, bt: &BootServices) {
    info!("Testing Media Access protocols");

    if let Ok(sfs) = bt.locate_protocol::<SimpleFileSystem>() {
        let sfs = unsafe { &mut *sfs.get() };
        let mut directory = sfs.open_volume().unwrap();
        let mut buffer = vec![0; 128];
        loop {
            let file_info = match directory.read_entry(&mut buffer) {
                Ok(info) => {
                    if let Some(info) = info {
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
        directory.reset_entry_readout().unwrap();

        test_file_system_info(&mut directory);
    } else {
        warn!("`SimpleFileSystem` protocol is not available");
    }

    let handles = bt
        .find_handles::<PartitionInfo>()
        .expect("Failed to get handles for `PartitionInfo` protocol");

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
            .expect("Failed to get partition info");
        let pi = unsafe { &*pi.interface.get() };

        if let Some(mbr) = pi.mbr_partition_record() {
            info!("MBR partition: {:?}", mbr);
        } else if let Some(gpt) = pi.gpt_partition_entry() {
            info!("GPT partition: {:?}", gpt);
        } else {
            info!("Unknown partition");
        }
    }

    known_disk::test_known_disk(image, bt);
}
