// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::proto::console::pointer::Pointer;

pub fn test() {
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
        info!("New pointer State: {:#?}", state);
    } else {
        info!("Pointer state has not changed since the last query");
    }
}
