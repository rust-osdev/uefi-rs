use uefi::prelude::*;
use uefi::proto::Protocol;
use uefi::table::boot::{BootServices, SearchType};
use uefi::{Handle, Result};

use crate::alloc::vec::Vec;

/// Utility functions for the UEFI boot services.
pub trait BootServicesExt {
    /// Returns all the handles implementing a certain protocol.
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>>;

    /// Returns a protocol implementation, if present on the system.
    fn find_protocol<P: Protocol>(&self) -> Option<&mut P>;
}

impl BootServicesExt for BootServices {
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>> {
        // Search by protocol.
        let search_type = SearchType::from_proto::<P>();

        // Determine how much we need to allocate.
        let (buffer_size, status1) = self.locate_handle(search_type, None)?.split();

        // Allocate a large enough buffer.
        let mut buffer = Vec::with_capacity(buffer_size);

        unsafe {
            buffer.set_len(buffer_size);
        }

        // Perform the search.
        let (buffer_size, status2) = self.locate_handle(search_type, Some(&mut buffer))?.split();

        // Once the vector has been filled, update its size.
        unsafe {
            buffer.set_len(buffer_size);
        }

        status1
            .into_with(|| buffer)
            .map(|completion| completion.with_status(status2))
    }

    fn find_protocol<P: Protocol>(&self) -> Option<&mut P> {
        // Retrieve all handles implementing this.
        self.find_handles::<P>()
            // Convert to an option.
            .warning_as_error()
            .ok()?
            // Using the `find_handles` function might not return _only_ compatible protocols.
            // We have to retrieve them all and find one that works.
            .iter()
            .map(|&handle| self.handle_protocol::<P>(handle))
            // Find a handle which implements the protocol.
            .find(Option::is_some)
            // Filter itself returns an option, we need to lift it out.
            .unwrap_or(None)
    }
}
