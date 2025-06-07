// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for region mapped by PCI Root Bridge I/O protocol.

use core::ffi::c_void;
use core::ptr;
use log::debug;
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;

/// Represents a region of memory mapped by PCI Root Bridge I/O protocol.
/// The region will be unmapped automatically when it is dropped.
///
/// # Lifetime
/// `'p` is the lifetime for Protocol.
/// `'r` is the lifetime for Mapped Region.
/// Protocol must outlive the mapped region
/// as unmap function can only be accessed through the protocol.
#[derive(Debug)]
pub struct PciMappedRegion<'p, 'r>
where
    'p: 'r,
{
    device_address: u64,
    length: usize,
    _lifetime_holder: &'r (),
    key: *const c_void,
    proto: &'p PciRootBridgeIoProtocol,
}

impl<'p, 'r> PciMappedRegion<'p, 'r> where 'p: 'r {
    pub(crate) fn new<T>(
        device_address: u64,
        length: usize,
        key: *const c_void,
        to_map: &'r T,
        proto: &'p PciRootBridgeIoProtocol,
    ) -> Self {
        let _lifetime_holder: &'r () = unsafe {
            let ptr = ptr::from_ref(to_map);
            ptr.cast::<()>().as_ref().unwrap()
        };

        let end = device_address + length as u64;
        debug!("Mapped new region [0x{:X}..0x{:X}]", device_address, end);
        Self {
            device_address,
            length,
            _lifetime_holder,
            key,
            proto,
        }
    }

    /// Returns mapped address and length of mapped region.
    ///
    /// # Warning
    /// **Returned address cannot be used to reference memory from CPU!**
    /// **Do not cast it back to pointer or reference**
    #[must_use]
    pub const fn region(&self) -> (u64, usize) {
        (self.device_address, self.length)
    }
}

impl<'p, 'r> Drop for PciMappedRegion<'p, 'r> {
    fn drop(&mut self) {
        let status = unsafe { (self.proto.unmap)(self.proto, self.key) };
        match status {
            Status::SUCCESS => {
                let end = self.device_address + self.length as u64;
                debug!("Region [0x{:X}..0x{:X}] was unmapped", self.device_address, end);
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
