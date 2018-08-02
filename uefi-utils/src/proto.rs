//! Protocol handling utility functions.

use crate::boot_services;

use uefi::{Result, Handle};
use uefi::table::boot;
use uefi::proto::Protocol;

use alloc::vec::Vec;

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
    // Note: using the `find_handles` function might not return _only_ compatible protocols.
    // We have to retrieve them all and find one that works.
    let handles = find_handles::<P>().ok()?;

    handles.iter()
        .map(|&handle| bt.handle_protocol::<P>(handle))
        .filter(Option::is_some)
        .next()
        .unwrap_or(None)
}
