// SPDX-License-Identifier: MIT OR Apache-2.0

//! Very basic tests for the BlockIo and BlockIo2 protocols.
//!
//! We look for some well-known data on a few well-known disks
//! of our test environment.

use alloc::string::String;
use core::ffi::c_void;
use core::hint;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::Protocol;
use uefi::proto::device_path::DevicePath;
use uefi::proto::device_path::text::{AllowShortcuts, DisplayOnly};
use uefi::proto::media::block::{BlockIO, BlockIO2, BlockIO2Token};
use uefi::{CString16, Event, Handle, boot};
use uefi_raw::Status;
use uefi_raw::protocol::device_path::DeviceSubType;
use uefi_raw::table::boot::{EventType, Tpl};

fn verify_block_device(dvp: &DevicePath, first_block: &[u8]) {
    // We only look for storage technologies that we are interested in.
    let storage_device_types = [
        DeviceSubType::MESSAGING_SCSI,
        DeviceSubType::MESSAGING_NVME_NAMESPACE,
        DeviceSubType::MESSAGING_SATA,
    ];
    let maybe_storage_node = dvp
        .node_iter()
        .find(|x| storage_device_types.contains(&x.sub_type()));

    if maybe_storage_node.is_none() {
        // This happens on CI for the AArch64 target for a handle with
        // device path `PciRoot(0x0)/Pci(0x9,0x0)` only.
        return;
    }
    let storage_node = maybe_storage_node.unwrap();

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
            if is_valid_fat && storage_node.data() == [0x2, 0, 0xff, 0xff, 0x0, 0x0] {
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

        // sync test
        {
            let mut first_block = vec![0; 512];
            unsafe {
                proto
                    .read_blocks_ex(
                        proto.media().media_id(),
                        0,
                        None, /* sync */
                        first_block.len(),
                        first_block.as_mut_ptr(),
                    )
                    .unwrap();
            }

            verify_block_device(&dvp, first_block.as_slice());
        }
        // async test
        {
            static ASYNC_READ_LOCK: AtomicBool = AtomicBool::new(false);

            let mut first_block = vec![0; 512];
            extern "efiapi" fn callback(_event: Event, _context: Option<NonNull<c_void>>) {
                log::info!("event fired: block I/O 2 read_blocks_ex done");
                ASYNC_READ_LOCK.store(true, Ordering::SeqCst);
            }
            let event = unsafe {
                boot::create_event(
                    EventType::NOTIFY_SIGNAL,
                    Tpl::CALLBACK,
                    Some(callback),
                    None,
                )
                .expect("should create event")
            };
            let mut token = BlockIO2Token::new(event, Status::NOT_READY);
            let token_ptr = NonNull::new(&raw mut token).unwrap();
            unsafe {
                proto
                    .read_blocks_ex(
                        proto.media().media_id(),
                        0,
                        Some(token_ptr), /* sync */
                        first_block.len(),
                        first_block.as_mut_ptr(),
                    )
                    .unwrap();
            }

            // This works for some disks but the implementations behave
            // differently.
            // assert_ne!(token.transaction_status(), Status::SUCCESS);

            // Wait util callback notified us the read is done
            while !ASYNC_READ_LOCK.load(Ordering::SeqCst) {
                hint::spin_loop();
            }
            ASYNC_READ_LOCK.store(false, Ordering::SeqCst);

            // No boot::check_event() here, doesn't work, invalid parameter.
            // Instead, one must use the notify function to perform further
            // action.

            assert_eq!(token.transaction_status(), Status::SUCCESS);
            verify_block_device(&dvp, first_block.as_slice());
        }
    }
}

pub fn test() {
    test_blockio_protocol();
    test_blockio2_protocol();
}
