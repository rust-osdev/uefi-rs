use uefi::proto::console::text::Output;
use uefi::table::boot::{BootServices, SearchType};
use uefi::{prelude::*, Identify};

pub fn test(bt: &BootServices) {
    info!("Testing boot services");
    memory::test(bt);
    misc::test(bt);
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
            .expect_success("Failed to locate handle buffer");
        assert!(handles.handles().len() > 0, "Could not find any handles");
    }

    {
        // search by protocol
        let handles = bt
            .locate_handle_buffer(SearchType::ByProtocol(&Output::GUID))
            .expect_success("Failed to locate handle buffer");
        assert!(
            handles.handles().len() > 0,
            "Could not find any OUTPUT protocol handles"
        );
    }
}
