use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::partition::PartitionInfo;

pub fn test(bt: &BootServices) {
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
    } else {
        warn!("`SimpleFileSystem` protocol is not available");
    }

    let handles = bt
        .find_handles::<PartitionInfo>()
        .expect_success("Failed to get handles for `PartitionInfo` protocol");

    for handle in handles {
        let pi = bt
            .handle_protocol::<PartitionInfo>(handle)
            .expect_success("Failed to get partition info");
        let pi = unsafe { &*pi.get() };

        if let Some(mbr) = pi.mbr_partition_record() {
            info!("MBR partition: {:?}", mbr);
        } else if let Some(gpt) = pi.gpt_partition_entry() {
            info!("GPT partition: {:?}", gpt);
        } else {
            info!("Unknown partition");
        }
    }
}
