// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::proto::console::pointer::{AbsolutePointer, Pointer};

pub fn test_pointer() {
    info!("Running pointer protocol test");
    let handle = boot::get_handle_for_protocol::<Pointer>().expect("missing Pointer protocol");
    let mut pointer =
        boot::open_protocol_exclusive::<Pointer>(handle).expect("failed to open pointer protocol");

    pointer
        .reset(false)
        .expect("Failed to reset pointer device");

    let state = pointer
        .read_state()
        .expect("Failed to retrieve pointer state");

    if let Some(state) = state {
        info!("New pointer State: {state:#?}");
    } else {
        info!("Pointer state has not changed since the last query");
    }
}

pub fn test_absolute_pointer() {
    info!("Running absolute pointer protocol test");
    let handle = boot::get_handle_for_protocol::<AbsolutePointer>()
        .expect("missing AbsolutePointer protocol");
    let mut pointer = boot::open_protocol_exclusive::<AbsolutePointer>(handle)
        .expect("failed to open absolute pointer protocol");

    pointer
        .reset(false)
        .expect("Failed to reset absolute pointer device");

    let mode = pointer.mode();
    info!("Absolute pointer mode: {mode:#?}");

    let state = pointer
        .read_state()
        .expect("Failed to retrieve absolute pointer state");

    if let Some(state) = state {
        info!("New absolute pointer state: {state:#?}");
    } else {
        info!("Absolute pointer state has not changed since the last query");
    }
}
