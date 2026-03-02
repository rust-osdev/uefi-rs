// SPDX-License-Identifier: MIT OR Apache-2.0

//! Defines wrapper for a region mapped by PCI Root Bridge I/O protocol.
use core::ffi::c_void;
use core::fmt::Debug;
use core::marker::PhantomData;
use log::trace;
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
    region: PciRegion,
    _lifetime_holder: PhantomData<&'r ()>,
    key: *const c_void,
    proto: &'p PciRootBridgeIoProtocol,
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
    pub length: usize,
}

impl<'p, 'r> PciMappedRegion<'p, 'r>
where
    'p: 'r,
{
    #[allow(dead_code)] // TODO Implement Map function
    pub(crate) fn new<T: ?Sized>(
        device_address: u64,
        length: usize,
        key: *const c_void,
        _to_map: &'r T,
        proto: &'p PciRootBridgeIoProtocol,
    ) -> Self {
        let end = device_address + length as u64;
        trace!("Mapped new region [0x{device_address:X}..0x{end:X}]");
        Self {
            region: PciRegion {
                device_address,
                length,
            },
            _lifetime_holder: PhantomData,
            key,
            proto,
        }
    }

    /// Returns mapped address and length of a region.
    ///
    /// # Warning
    /// **Returned address cannot be used to reference memory from CPU!**
    /// **Do not cast it back to pointer or reference**
    #[must_use]
    pub const fn region(&self) -> PciRegion {
        self.region
    }
}

impl Drop for PciMappedRegion<'_, '_> {
    fn drop(&mut self) {
        let status = unsafe { (self.proto.unmap)(self.proto, self.key) };
        match status {
            Status::SUCCESS => {
                let end = self.region.device_address + self.region.length as u64;
                trace!(
                    "Region [0x{:X}..0x{:X}] was unmapped",
                    self.region.device_address, end
                );
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
