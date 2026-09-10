// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::prelude::*;

pub fn test() {
    info!("Testing console protocols");

    system::with_stdout(stdout::test);

    unsafe {
        serial::test();
        gop::gop_test();
    }
    pointer::test_pointer();
    pointer::test_absolute_pointer();
    gop::test_edid_discovered();
}

mod gop;
mod pointer;
mod serial;
mod stdout;
