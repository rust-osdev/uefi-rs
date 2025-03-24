// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::proto::scsi::pass_thru::ExtScsiPassThru;
use uefi::proto::scsi::ScsiRequestBuilder;

pub fn test() {
    info!("Running extended SCSI Pass Thru tests");
    test_allocating_api();
    test_reusing_buffer_api();
}

fn test_allocating_api() {
    let scsi_ctrl_handles = uefi::boot::find_handles::<ExtScsiPassThru>().unwrap();

    // On I440FX and Q35 (both x86 machines), Qemu configures an IDE and a SATA controller
    // by default respectively. We manually configure an additional SCSI controller.
    // Thus, we should see two controllers with support for EXT_SCSI_PASS_THRU on this platform
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    assert_eq!(scsi_ctrl_handles.len(), 2);
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    assert_eq!(scsi_ctrl_handles.len(), 1);

    let mut found_drive = false;
    for handle in scsi_ctrl_handles {
        let scsi_pt = uefi::boot::open_protocol_exclusive::<ExtScsiPassThru>(handle).unwrap();
        for mut device in scsi_pt.iter_devices() {
            // see: https://www.seagate.com/files/staticfiles/support/docs/manual/Interface%20manuals/100293068j.pdf
            // 3.6 INQUIRY command
            let request = ScsiRequestBuilder::read(scsi_pt.io_align())
                .with_timeout(core::time::Duration::from_millis(500))
                .with_command_data(&[0x12, 0x00, 0x00, 0x00, 0xFF, 0x00])
                .unwrap()
                .with_read_buffer(255)
                .unwrap()
                .build();
            let Ok(response) = device.execute_command(request) else {
                continue; // no device
            };
            let bfr = response.read_buffer().unwrap();
            // more no device checks
            if bfr.len() < 32 {
                continue;
            }
            if bfr[0] & 0b00011111 == 0x1F {
                continue;
            }

            // found device
            let vendor_id = core::str::from_utf8(&bfr[8..16]).unwrap().trim();
            let product_id = core::str::from_utf8(&bfr[16..32]).unwrap().trim();
            if vendor_id == "uefi-rs" && product_id == "ExtScsiPassThru" {
                info!(
                    "Found Testdisk at: {:?} | {}",
                    device.target(),
                    device.lun()
                );
                found_drive = true;
            }
        }
    }

    assert!(found_drive);
}

fn test_reusing_buffer_api() {
    let scsi_ctrl_handles = uefi::boot::find_handles::<ExtScsiPassThru>().unwrap();

    let mut found_drive = false;
    for handle in scsi_ctrl_handles {
        let scsi_pt = uefi::boot::open_protocol_exclusive::<ExtScsiPassThru>(handle).unwrap();
        let mut cmd_bfr = scsi_pt.alloc_io_buffer(6).unwrap();
        cmd_bfr.copy_from_slice(&[0x12, 0x00, 0x00, 0x00, 0xFF, 0x00]);
        let mut read_bfr = scsi_pt.alloc_io_buffer(255).unwrap();

        for mut device in scsi_pt.iter_devices() {
            // see: https://www.seagate.com/files/staticfiles/support/docs/manual/Interface%20manuals/100293068j.pdf
            // 3.6 INQUIRY command
            let request = ScsiRequestBuilder::read(scsi_pt.io_align())
                .with_timeout(core::time::Duration::from_millis(500))
                .use_command_buffer(&mut cmd_bfr)
                .unwrap()
                .use_read_buffer(&mut read_bfr)
                .unwrap()
                .build();
            let Ok(response) = device.execute_command(request) else {
                continue; // no device
            };
            let bfr = response.read_buffer().unwrap();
            // more no device checks
            if bfr.len() < 32 {
                continue;
            }
            if bfr[0] & 0b00011111 == 0x1F {
                continue;
            }

            // found device
            let vendor_id = core::str::from_utf8(&bfr[8..16]).unwrap().trim();
            let product_id = core::str::from_utf8(&bfr[16..32]).unwrap().trim();
            if vendor_id == "uefi-rs" && product_id == "ExtScsiPassThru" {
                info!(
                    "Found Testdisk at: {:?} | {}",
                    device.target(),
                    device.lun()
                );
                found_drive = true;
            }
        }
    }

    assert!(found_drive);
}
