// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::ata::pass_thru::AtaPassThru;
use uefi::proto::ata::AtaRequestBuilder;

pub fn test() {
    info!("Running ATA PassThru tests");

    assert!(is_testdrive_present());
}

const ATACMD_IDENTIFY: u8 = 0xEC;

fn is_testdrive_present() -> bool {
    let ata_ctrl_handles = boot::find_handles::<AtaPassThru>().unwrap();
    assert_eq!(ata_ctrl_handles.len(), 1);

    for handle in ata_ctrl_handles {
        let params = OpenProtocolParams {
            handle,
            agent: boot::image_handle(),
            controller: None,
        };
        let ata_pt = unsafe {
            // don't open exclusive! That would break other tests
            boot::open_protocol::<AtaPassThru>(params, OpenProtocolAttributes::GetProtocol).unwrap()
        };
        for mut device in ata_pt.iter_devices() {
            // ATA IDENTIFY command
            let request = AtaRequestBuilder::read_udma(ata_pt.io_align(), ATACMD_IDENTIFY)
                .unwrap()
                .with_timeout(core::time::Duration::from_millis(500))
                .with_read_buffer(255)
                .unwrap()
                .build();
            if let Ok(result) = device.execute_command(request) {
                let bfr = result.read_buffer().unwrap();
                // ATA uses wchar16 big endian strings for serial numbers
                let mut serial_bfr = [0u8; 20];
                bfr[20..40]
                    .chunks_exact(2)
                    .zip(serial_bfr.chunks_exact_mut(2))
                    .for_each(|(src, dst)| {
                        dst[0] = src[1];
                        dst[1] = src[0];
                    });
                let serial = core::str::from_utf8(&serial_bfr).unwrap().trim();
                if serial == "AtaPassThru" {
                    info!("Found Testdisk at handle: {:?}", handle);
                    return true; // found our testdrive!
                }
            }
        }
    }

    false
}
