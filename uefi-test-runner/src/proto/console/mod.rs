// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::prelude::*;

pub fn test() {
    info!("Testing console protocols");

    system::with_stdout(stdout::test);

    unsafe {
        serial::test();
        gop::test();
    }
    pointer::test_pointer();
    pointer::test_absolute_pointer();
}

mod gop;
mod pointer;
mod serial;
mod stdout;
