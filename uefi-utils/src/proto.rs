//! Protocol handling utility functions.

use boot_services;

use uefi::{Result, Handle};

use uefi::table::boot;

use uefi::proto::Protocol;

use alloc::Vec;

/// Returns all the handles implementing a certain protocol.
pub fn find_handles<P: Protocol>() -> Result<Vec<Handle>> {
    let bt = boot_services();

    // Search by protocol.
    let search_type = boot::SearchType::from_proto::<P>();

    // Determine how much we need to allocate.
    let buffer_size = bt.locate_handle(search_type, None)?;

    // Allocate a large enough buffer.
    let mut buffer = Vec::with_capacity(buffer_size);

    unsafe {
        buffer.set_len(buffer_size);
    }

    // Perform the search.
    let buffer_size = bt.locate_handle(search_type, Some(&mut buffer))?;

    // Once the vector has been filled, update its size.
    unsafe {
        buffer.set_len(buffer_size);
    }

    Ok(buffer)
}
