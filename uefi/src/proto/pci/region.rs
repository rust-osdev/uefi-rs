// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for region mapped by PCI Root Bridge I/O protocol.

use core::cell::RefCell;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ops::Deref;
use ghost_cell::{GhostCell, GhostToken};
use log::debug;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;
use uefi_raw::Status;

/// Represents a region of memory mapped by PCI Root Bridge I/O protocol.
/// The region will be unmapped automatically when it is dropped.
///
/// # Lifetime
/// `'p` is the lifetime for Protocol.
/// `'r` is the lifetime for Mapped Region.
/// Protocol must outlive the mapped region
/// as unmap function can only be accessed through the protocol.
pub struct PciMappedRegion<'p, 'r, 'id>
where
    'p: 'r,
{
    region: PciRegion,
    _lifetime_holder: PhantomData<&'r ()>,
    key: *const c_void,
    proto: &'p GhostCell<'id, PciRootBridgeIoProtocol>,
    token: &'p RefCell<GhostToken<'id>>,
}

/// Represents a region of memory in PCI root bridge memory space.
/// CPU cannot use address in this struct to deference memory.
/// This is effectively the same as rust's slice type.
/// This type only exists to prevent users from accidentally dereferencing it.
#[derive(Debug, Copy, Clone)]
pub struct PciRegion {
    /// Starting address of the memory region
    pub device_address: u64,

    /// Byte length of the memory region.
    pub length: usize
}

impl<'p, 'r, 'id> PciMappedRegion<'p, 'r, 'id> where 'p: 'r {
    pub(crate) fn new<T: ?Sized>(
        device_address: u64,
        length: usize,
        key: *const c_void,
        _to_map: &'r T,
        proto: &'p GhostCell<'id, PciRootBridgeIoProtocol>,
        token: &'p RefCell<GhostToken<'id>>,
    ) -> Self {
        let end = device_address + length as u64;
        debug!("Mapped new region [0x{:X}..0x{:X}]", device_address, end);
        Self {
            region: PciRegion {
                device_address,
                length,
            },
            _lifetime_holder: PhantomData,
            key,
            proto,
            token,
        }
    }

    /// Returns mapped address and length of mapped region.
    ///
    /// # Warning
    /// **Returned address cannot be used to reference memory from CPU!**
    /// **Do not cast it back to pointer or reference**
    #[must_use]
    pub const fn region(&self) -> PciRegion {
        self.region
    }
}

impl<'p, 'r, 'id> Drop for PciMappedRegion<'p, 'r, 'id> {
    fn drop(&mut self) {
        let token = self.token.borrow();
        let protocol = self.proto.borrow(token.deref());
        let status = unsafe {
            (protocol.unmap)(protocol, self.key)
        };
        match status {
            Status::SUCCESS => {
                let end = self.region.device_address + self.region.length as u64;
                debug!("Region [0x{:X}..0x{:X}] was unmapped", self.region.device_address, end);
            }
            Status::INVALID_PARAMETER => {
                panic!("This region was not mapped using PciRootBridgeIo::map");
            }
            Status::DEVICE_ERROR => {
                panic!("The data was not committed to the target system memory.");
            }
            _ => unreachable!(),
        }
    }
}

impl<'p, 'r, 'id> Debug for PciMappedRegion<'p, 'r, 'id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_struct("PciMappedRegion");
        debug.field("region", &self.region);
        debug.field("key", &self.key);

        if let Ok(token) = self.token.try_borrow() {
            debug.field("proto", self.proto.borrow(token.deref()));
        } else {
            debug.field("proto", &"unavailable");
        }
        debug.finish()
    }
}

impl PciRegion {
    /// Creates a new region of memory with different length.
    /// The new region must have shorter length to ensure
    /// it won't contain invalid memory address.
    #[must_use]
    pub fn with_length(self, new_length: usize) -> Self {
        assert!(new_length <= self.length);
        Self {
            device_address: self.device_address,
            length: new_length,
        }
    }
}
