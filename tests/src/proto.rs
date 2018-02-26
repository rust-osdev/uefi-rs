use uefi::{Result, Handle};
use uefi::table::boot;

use uefi::proto;
use uefi::proto::Protocol;

use alloc::Vec;

fn find_protocol_handles<P: Protocol>(bt: &boot::BootServices) -> Result<Vec<Handle>> {
    // What to search for.
    let search_type = boot::SearchType::from_proto::<P>();

    let buffer_size = bt.locate_handle(search_type, None)
        .expect("Failed to retrieve size of buffer to allocate");

    // Allocate a large enough buffer.
    let mut buffer = Vec::with_capacity(buffer_size);

    // Perform the search.
    bt.locate_handle(search_type, Some(&mut buffer))?;

    // Once the vector has been filled, update its size.
    unsafe {
        buffer.set_len(buffer_size);
    }

    Ok(buffer)
}

pub fn protocol_test(bt: &boot::BootServices) -> Result<()> {
    type SearchedProtocol = proto::console::text::Output;

    let handles = find_protocol_handles::<SearchedProtocol>(bt)
        .expect("Failed to retrieve the list of handles");

    info!("Number of handles which implement the SimpleTextOutput protocol: {}", handles.len());

    Ok(())
}
