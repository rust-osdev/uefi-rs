use uefi::proto::console::text::Output;
use uefi::table::boot::{BootServices, SearchType};
use uefi::table::{Boot, SystemTable};
use uefi::Identify;

pub fn test(st: &SystemTable<Boot>) {
    let bt = st.boot_services();
    info!("Testing boot services");
    memory::test(bt);
    misc::test(st);
    test_locate_handle_buffer(bt);
}

mod memory;
mod misc;

fn test_locate_handle_buffer(bt: &BootServices) {
    info!("Testing the `locate_handle_buffer` function");

    {
        // search all handles
        let handles = bt
            .locate_handle_buffer(SearchType::AllHandles)
            .expect("Failed to locate handle buffer");
        assert!(!handles.is_empty(), "Could not find any handles");
    }

    {
        // search by protocol
        let handles = bt
            .locate_handle_buffer(SearchType::ByProtocol(&Output::GUID))
            .expect("Failed to locate handle buffer");
        assert!(
            !handles.is_empty(),
            "Could not find any OUTPUT protocol handles"
        );
    }
}
