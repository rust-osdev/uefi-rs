use uefi::table::{SystemTable, boot::BootServices};

use uefi::proto;
use uefi_exts::BootServicesExt;

pub fn test(st: &SystemTable) {
    let bt = st.boot;

    find_protocol(bt);

    console::test(st);
    debug::test(bt);
}

fn find_protocol(bt: &BootServices) {
    type SearchedProtocol = proto::console::text::Output;

    let handles = bt.find_handles::<SearchedProtocol>().expect("Failed to retrieve list of handles");
    assert!(handles.len() > 1, "There should be at least one implementation of Simple Text Output (stdout)");
}

mod console;
mod debug;
