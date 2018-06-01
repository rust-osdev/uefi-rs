//! Protocol handling utility functions.

use boot_services;

use uefi::{Result, Handle};
use uefi::table::boot;
use uefi::proto::Protocol;

use alloc::Vec;

use core::ptr;

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

/// Returns a reference to the requested protocol.
pub fn find_protocol<P: Protocol>() -> Option<ptr::NonNull<P>> {
    let bt = boot_services();

    // Retrieve a handle implementing the protocol.
    let handle = {
        // Allocate space for 1 handle.
        let mut buffer = [ptr::null_mut(); 1];

        let search_type = boot::SearchType::from_proto::<P>();

        bt.locate_handle(search_type, Some(&mut buffer)).ok()
            .and_then(|len| if len == 1 { Some(buffer[0]) } else { None })?
    };

    bt.handle_protocol::<P>(handle)
}
