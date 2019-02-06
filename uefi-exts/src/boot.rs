use uefi::proto::Protocol;
use uefi::table::boot::{BootServices, SearchType};
use uefi::{Handle, Result};

use crate::alloc::vec::Vec;

/// Utility functions for the UEFI boot services.
pub trait BootServicesExt {
    /// Returns all the handles implementing a certain protocol.
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>>;
}

impl BootServicesExt for BootServices {
    fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>> {
        // Search by protocol.
        let search_type = SearchType::from_proto::<P>();

        // Determine how much we need to allocate.
        let (status1, buffer_size) = self.locate_handle(search_type, None)?.split();

        // Allocate a large enough buffer.
        let mut buffer = Vec::with_capacity(buffer_size);

        unsafe {
            buffer.set_len(buffer_size);
        }

        // Perform the search.
        let (status2, buffer_size) = self.locate_handle(search_type, Some(&mut buffer))?.split();

        // Once the vector has been filled, update its size.
        unsafe {
            buffer.set_len(buffer_size);
        }

        // Emit output, with warnings
        status1
            .into_with_val(|| buffer)
            .map(|completion| completion.with_status(status2))
    }
}
