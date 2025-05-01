// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::boot;
use uefi::proto::shell::Shell;

pub fn test() {
    info!("Running shell protocol tests");

    let handle = boot::get_handle_for_protocol::<Shell>().expect("No Shell handles");

    let mut _shell =
        boot::open_protocol_exclusive::<Shell>(handle).expect("Failed to open Shell protocol");
}
