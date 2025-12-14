// SPDX-License-Identifier: MIT OR Apache-2.0

//! Very basic tests for the BlockIo and BlockIo2 protocols.
//!
//! We look for some well-known data on a few well-known disks
//! of our test environment.

use alloc::string::String;
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::Protocol;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::media::block::{BlockIO, BlockIO2};
use uefi::{CString16, Handle, boot};
use uefi_raw::protocol::device_path::DeviceSubType;

fn verify_block_device(dvp: &DevicePath, first_block: &[u8]) {
    // We only look for storage technologies that we are interested in.
    let storage_device_types = [
        DeviceSubType::MESSAGING_SCSI,
        DeviceSubType::MESSAGING_NVME_NAMESPACE,
        DeviceSubType::MESSAGING_SATA,
    ];
    let storage_node = dvp
        .node_iter()
        .skip_while(|x| !storage_device_types.contains(&x.sub_type()))
        .next()
        .unwrap();
    let storage_node_string = storage_node
        .to_string(DisplayOnly(true), AllowShortcuts(true))
        .unwrap();
    debug!("Storage technology: {storage_node_string}");

    //debug!("First 512 bytes: {first_block:?}");
    match storage_node.sub_type() {
        DeviceSubType::MESSAGING_SCSI => { /* empty disks so far, nothing to check for */ }
        DeviceSubType::MESSAGING_NVME_NAMESPACE => {
            /* empty disks so far, nothing to check for */
        }
        DeviceSubType::MESSAGING_SATA => {
            // We check that the right SATA disk indeed contains a correct
            // FAT16 volume.
            let expected = "MbrTestDisk";
            let contains_volume_label = first_block
                .windows(expected.len())
                .any(|w| w == expected.as_bytes());
            let oem_name = {
                let bytes = &first_block[3..10];
                String::from_utf8(bytes.to_vec())
            };
            let is_valid_fat = first_block[0] != 0 && oem_name.is_ok();
            if is_valid_fat && storage_node.data() == &[0x2, 0, 0xff, 0xff, 0x0, 0x0] {
                if !contains_volume_label {
                    panic!(
                        "Sata disk {storage_node_string} does not contain {expected} in its first 512 bytes"
                    )
                } else {
                    debug!(
                        "Found volume label {expected} with OEM name: {}",
                        oem_name.unwrap()
                    );
                }
            }
        }
        _ => unreachable!(),
    }
}

fn open_proto_and_dvp<P: Protocol>(
    handle: Handle,
) -> (ScopedProtocol<P>, ScopedProtocol<DevicePath>, CString16) {
    let proto = unsafe {
        boot::open_protocol::<P>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .unwrap()
    };
    let dvp = unsafe {
        boot::open_protocol::<DevicePath>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .unwrap()
    };
    let dvp_string = dvp
        .to_string(DisplayOnly(true), AllowShortcuts(true))
        .unwrap();

    (proto, dvp, dvp_string)
}

fn test_blockio_protocol() {
    info!("Testing BLOCKIO protocol");
    for handle in boot::find_handles::<BlockIO>().unwrap() {
        let (proto, dvp, dvp_string) = open_proto_and_dvp::<BlockIO>(handle);
        debug!("Found handle supporting protocol: {dvp_string}");
        debug!("media: {:?}", proto.media());
        let mut first_block = vec![0; 512];
        proto
            .read_blocks(proto.media().media_id(), 0, &mut first_block)
            .unwrap();

        verify_block_device(&dvp, first_block.as_slice());
    }
}

fn test_blockio2_protocol() {
    info!("Testing BLOCKIO 2 protocol");

    for handle in boot::find_handles::<BlockIO2>().unwrap() {
        let (proto, dvp, dvp_string) = open_proto_and_dvp::<BlockIO2>(handle);
        debug!("Found handle supporting protocol: {dvp_string}");
        debug!("media: {:?}", proto.media());
        let mut first_block = vec![0; 512];
        unsafe {
            proto
                .read_blocks_ex(proto.media().media_id(), 0, None, &mut first_block)
                .unwrap();
        }

        verify_block_device(&dvp, first_block.as_slice());
    }
}

pub fn test() {
    test_blockio_protocol();
    test_blockio2_protocol();
}
