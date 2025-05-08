// SPDX-License-Identifier: MIT OR Apache-2.0

// ANCHOR: all
// ANCHOR: features
#![no_main]
#![no_std]
// ANCHOR_END: features

extern crate alloc;

// ANCHOR: use
use core::time::Duration;
use log::{info, warn};
use uefi::boot;
use uefi::prelude::*;
use uefi::proto::misc::Timestamp;
// ANCHOR_END: use

// ANCHOR: entry
#[entry]
fn main() -> Status {
    // ANCHOR_END: entry
    // ANCHOR: services
    uefi::helpers::init().unwrap();
    // ANCHOR_END: services

    // ANCHOR: params
    test_timestamp();
    // ANCHOR_END: params

    // ANCHOR: stall
    boot::stall(Duration::from_secs(10));
    // ANCHOR_END: stall

    // ANCHOR: return
    Status::SUCCESS
}
// ANCHOR_END: return

// ANCHOR: test_timestamp
pub fn test_timestamp() {
    // ANCHOR_END: test_timestamp
    info!("Running loaded Timestamp Protocol test");

    let handle = boot::get_handle_for_protocol::<Timestamp>();

    match handle {
        Ok(handle) => {
            let timestamp_proto =
                boot::open_protocol_exclusive::<Timestamp>(handle)
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
