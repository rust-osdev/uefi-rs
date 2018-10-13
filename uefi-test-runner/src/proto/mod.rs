use uefi::prelude::*;
use uefi_exts::BootServicesExt;

use uefi::proto;

pub fn test(st: &SystemTable) {
    info!("Testing various protocols");

    let bt = st.boot;

    find_protocol(bt);

    console::test(st);
    debug::test(bt);
}

fn find_protocol(bt: &BootServices) {
    type SearchedProtocol = proto::console::text::Output;

    let handles = bt
        .find_handles::<SearchedProtocol>()
        .expect_success("Failed to retrieve list of handles");

    assert!(
        handles.len() > 1,
        "There should be at least one implementation of Simple Text Output (stdout)"
    );
}

mod console;
mod debug;
