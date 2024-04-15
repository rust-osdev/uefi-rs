// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

extern crate alloc;

use log::{info, warn};

// ANCHOR: use
use uefi::prelude::*;
use uefi::proto::misc::Timestamp;

// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi::helpers::init(&mut system_table).unwrap();
    let boot_services = system_table.boot_services();
    // ANCHOR_END: services

    // ANCHOR: params
    test_timestamp(boot_services);
    // ANCHOR_END: params

    // ANCHOR: stall
    boot_services.stall(10_000_000);
    // ANCHOR_END: stall

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return

// ANCHOR: test_timestamp
pub fn test_timestamp(bt: &BootServices) {
    // ANCHOR_END: test_timestamp
    info!("Running loaded Timestamp Protocol test");

    let handle = bt.get_handle_for_protocol::<Timestamp>();

    match handle {
        Ok(handle) => {
            let timestamp_proto = bt
                .open_protocol_exclusive::<Timestamp>(handle)
                .expect("Founded Timestamp Protocol but open failed");
            // ANCHOR: text
            let timestamp = timestamp_proto.get_timestamp();
            info!("Timestamp Protocol's timestamp: {:?}", timestamp);

            let properties = timestamp_proto.get_properties();
            info!("Timestamp Protocol's properties: {:?}", properties);
            // ANCHOR_END: text
        }
        Err(err) => {
            warn!("Failed to found Timestamp Protocol: {:?}", err);
        }
    }
}
// ANCHOR_END: all
