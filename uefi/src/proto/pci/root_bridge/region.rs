// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for a region mapped by PCI Root Bridge I/O protocol.

use crate::StatusExt;
use crate::proto::pci::root_bridge::buffer::PciBuffer;
use core::ffi::c_void;
use core::fmt::Debug;
use core::mem::ManuallyDrop;
use core::ptr;
use log::{error, trace};
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::PciRootBridgeIoProtocol;

/// Represents a region of memory mapped and made visible to devices
/// by PCI Root Bridge I/O protocol.
/// The region will be unmapped automatically when it is dropped.
///
/// # Lifetime
/// `'p` is the lifetime for Protocol.
/// Protocol must be available for the entire lifetime of this struct
/// as it provides its unmap function.
///
/// # Invariant
/// Value stored in its internal buffer cannot have a larger alignment requirement than page size,
/// which is 4096.
///
/// # Safety
/// Value stored in its internal buffer cannot be larger than the buffer's size,
/// which is 4096 * `pages`
#[derive(Debug)]
pub struct PciMappedRegion<'p: 'r, 'r, T> {
    pub(super) host: &'r PciBuffer<'p, T>,
    /// Bytes size of the mapped region.
    pub(super) length: usize,
    pub(super) device_address: usize,
    pub(super) key: *const c_void,
    pub(super) proto: &'p PciRootBridgeIoProtocol,
}

/// Represents a region of memory that a PCI device can use.
/// CPU cannot use address in this struct to deference memory.
/// This is effectively the same as rust's slice type.
/// This type only exists to prevent users from accidentally dereferencing it.
#[derive(Debug, Copy, Clone)]
pub struct DeviceRegion {
    /// Starting address of the memory region
    pub device_address: usize,

    /// Byte length of the memory region.
    pub length: usize,
}

impl<'p: 'r, 'r, T> PciMappedRegion<'p, 'r, T> {
    /// Returns access to the underlying buffer.
    #[must_use]
    pub const fn host(&'r self) -> &'r PciBuffer<'p, T> {
        self.host
    }

    /// Returns mapped address and length of a region.
    ///
    /// # Safety
    /// **Returned address cannot be used to reference memory from CPU!**
    /// **Do not cast it back to pointer or reference**
    #[must_use]
    pub const fn region(&self) -> DeviceRegion {
        DeviceRegion {
            device_address: self.device_address,
            length: self.length,
        }
    }

    /// Unmaps underlying memory region.
    /// It is recommended to use this over its Drop implementation, which will only log an error
    /// if unmapping fails.
    pub fn unmap(self) -> crate::Result<PciBuffer<'p, T>> {
        let region = ManuallyDrop::new(self);
        match region.unmap_inner() {
            // SAFETY:
            // This technically creates an alias to its underlying ExclusivePtr value,
            // but we don't do any read/writes through it.
            // And the original is discarded right away.
            Ok(_) => unsafe { Ok(ptr::read(region.host)) },
            Err(e) => Err(e),
        }
    }

    fn unmap_inner(&self) -> crate::Result {
        unsafe { (self.proto.unmap)(self.proto, self.key) }.to_result_with_val(|| {
            let host_start = self.host.base_ptr().addr();
            let host_end = host_start + self.length;
            let device_start = self.device_address;
            let device_end = device_start + self.length;
            trace!(
                "Region [Host 0x{:X}..0x{:X}] -> [Device 0x{:}..0x{:X}] was unmapped",
                host_start, host_end, device_start, device_end
            );
        })
    }
}

impl<'p: 'r, 'r, T> Drop for PciMappedRegion<'p, 'r, T> {
    fn drop(&mut self) {
        let Err(status) = self.unmap_inner() else {
            return;
        };
        match status.status() {
            // Effectively unreachable path
            Status::SUCCESS => {}

            Status::INVALID_PARAMETER => {
                error!("This region was not mapped using PciRootBridgeIo::map");
            }
            Status::DEVICE_ERROR => {
                error!("The data was not committed to the target system memory.");
            }
            etc => {
                error!(
                    "Unexpected error occurred when unmapping device memory: {:?}",
                    etc
                );
            }
        }
    }
}

impl DeviceRegion {
    /// Changes length of a given region.
    /// The new region must have a shorter length to ensure
    /// it won't contain invalid memory address.
    #[must_use]
    pub fn with_length(mut self, new_length: usize) -> Self {
        assert!(new_length <= self.length);
        self.length = new_length;
        self
    }
}
