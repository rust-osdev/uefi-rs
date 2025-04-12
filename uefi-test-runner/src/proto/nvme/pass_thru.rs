// SPDX-License-Identifier: MIT OR Apache-2.0

use core::time::Duration;
use uefi::boot;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::device_path::DevicePath;
use uefi::proto::media::block::BlockIO;
use uefi::proto::nvme::pass_thru::NvmePassThru;
use uefi::proto::nvme::{NvmeQueueType, NvmeRequestBuilder};

pub fn test() {
    info!("Running NVMe PassThru tests");

    assert!(has_nvme_drive());
}

fn has_nvme_drive() -> bool {
    let block_io_handles = boot::find_handles::<BlockIO>().unwrap();
    for handle in block_io_handles {
        let Ok(device_path) = boot::open_protocol_exclusive::<DevicePath>(handle) else {
            continue;
        };
        let mut device_path = &*device_path;

        let Ok(nvme_pt_handle) = boot::locate_device_path::<NvmePassThru>(&mut device_path) else {
            continue;
        };
        let nvme_pt = boot::open_protocol_exclusive::<NvmePassThru>(nvme_pt_handle).unwrap();
        let device_path_str = device_path
            .to_string(DisplayOnly(true), AllowShortcuts(false))
            .unwrap();
        info!("- Successfully opened NVMe: {}", device_path_str);
        let mut nvme_ctrl = nvme_pt.controller();

        let request = NvmeRequestBuilder::new(nvme_pt.io_align(), 0x06, NvmeQueueType::ADMIN)
            .with_timeout(Duration::from_millis(500))
            .with_cdw10(1) // we want info about controller
            .with_transfer_buffer(4096)
            .unwrap()
            .build();
        let result = nvme_ctrl.execute_command(request);
        if let Ok(result) = result {
            let bfr = result.transfer_buffer().unwrap();
            let serial = core::str::from_utf8(&bfr[4..24]).unwrap().trim();
            info!("Found NVMe with serial: '{}'", serial);
            if serial == "uefi-rsNvmePassThru" {
                return true;
            }
        }
    }

    false
}
