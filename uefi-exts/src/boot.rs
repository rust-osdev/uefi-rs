use uefi::{Result, Handle};
use uefi::table::boot::{BootServices, SearchType};
use uefi::proto::Protocol;

use crate::alloc::vec::Vec;
use core::ptr::NonNull;

/// Utility functions for the UEFI boot services.
pub trait BootServicesExt {
    /// Returns all the handles implementing a certain protocol.
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>>;

    /// Returns a protocol implementation, if present on the system.
    fn find_protocol<P: Protocol>(&self) -> Option<NonNull<P>>;
}

impl BootServicesExt for BootServices {
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>> {
        // Search by protocol.
        let search_type = SearchType::from_proto::<P>();

        // Determine how much we need to allocate.
        let buffer_size = self.locate_handle(search_type, None)?;

        // Allocate a large enough buffer.
        let mut buffer = Vec::with_capacity(buffer_size);

        unsafe {
            buffer.set_len(buffer_size);
        }

        // Perform the search.
        let buffer_size = self.locate_handle(search_type, Some(&mut buffer))?;

        // Once the vector has been filled, update its size.
        unsafe {
            buffer.set_len(buffer_size);
        }

        Ok(buffer)
    }

    fn find_protocol<P: Protocol>(&self) -> Option<NonNull<P>> {
        // Retrieve all handles implementing this.
        self.find_handles::<P>()
            // Convert to an option.
            .ok()?
            // Using the `find_handles` function might not return _only_ compatible protocols.
            // We have to retrieve them all and find one that works.
            .iter()
            .map(|&handle| self.handle_protocol::<P>(handle))
            // Only choose a handle which implements the protocol.
            .filter(Option::is_some)
            // Pick the first one that works.
            .next()
            // Filter itself returns an option, we need to lift it out.
            .unwrap_or(None)
    }
}
