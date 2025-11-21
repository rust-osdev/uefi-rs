// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Root Bridge protocol.

use super::{PciIoAddress, PciIoUnit, encode_io_mode_and_unit};
use crate::StatusExt;
#[cfg(feature = "alloc")]
use crate::proto::pci::configuration::QwordAddressSpaceDescriptor;
#[cfg(feature = "alloc")]
use alloc::collections::btree_set::BTreeSet;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::ffi::c_void;
use core::ptr;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::pci::root_bridge::{PciRootBridgeIoAccess, PciRootBridgeIoProtocol};

#[cfg(doc)]
use super::FullPciIoAddress;
#[cfg(doc)]
use crate::Status;

/// Protocol that provides access to the PCI Root Bridge I/O protocol.
///
/// # UEFI Spec Description
/// Provides the basic Memory, I/O, PCI configuration, and DMA interfaces that are
/// used to abstract accesses to PCI controllers behind a PCI Root Bridge Controller.
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(PciRootBridgeIoProtocol::GUID)]
pub struct PciRootBridgeIo(PciRootBridgeIoProtocol);

impl PciRootBridgeIo {
    /// Get the segment number where this PCI root bridge resides.
    #[must_use]
    pub const fn segment_nr(&self) -> u32 {
        self.0.segment_number
    }

    /// Access PCI I/O operations on this root bridge.
    pub const fn pci(&mut self) -> PciIoAccessPci<'_> {
        PciIoAccessPci {
            proto: &mut self.0,
            io_access: &mut self.0.pci,
        }
    }

    /// Flush all PCI posted write transactions from a PCI host bridge to system memory.
    ///
    /// # Errors
    /// - [`Status::DEVICE_ERROR`] The PCI posted write transactions were not flushed from the PCI host bridge
    ///   due to a hardware error.
    pub fn flush(&mut self) -> crate::Result<()> {
        unsafe { (self.0.flush)(&mut self.0).to_result() }
    }

    // TODO: poll I/O
    // TODO: mem I/O access
    // TODO: io I/O access
    // TODO: map & unmap & copy memory
    // TODO: buffer management
    // TODO: get/set attributes

    /// Retrieves the current resource settings of this PCI root bridge in the form of a set of ACPI resource descriptors.
    ///
    /// The returned list of descriptors contains information about bus, memory and io ranges that were set up
    /// by the firmware.
    ///
    /// # Errors
    /// - [`Status::UNSUPPORTED`] The current configuration of this PCI root bridge could not be retrieved.
    #[cfg(feature = "alloc")]
    pub fn configuration(&self) -> crate::Result<Vec<QwordAddressSpaceDescriptor>> {
        use crate::proto::pci::configuration;
        // The storage for the resource descriptors is allocated by this function. The caller must treat
        // the return buffer as read-only data, and the buffer must not be freed by the caller.
        let mut resources: *const c_void = ptr::null();
        unsafe {
            ((self.0.configuration)(&self.0, &mut resources))
                .to_result_with_val(|| configuration::parse(resources))
        }
    }

    // ###################################################
    // # Convenience functionality

    /// Recursively enumerate all devices, device functions and pci-to-pci bridges on this root bridge.
    ///
    /// The returned addresses might overlap with the addresses returned by another [`PciRootBridgeIo`] instance.
    /// Make sure to perform some form of cross-[`PciRootBridgeIo`] deduplication on the returned [`FullPciIoAddress`]es.
    /// **WARNING:** Only use the returned addresses with the respective [`PciRootBridgeIo`] instance that returned them.
    ///
    /// # Returns
    /// An ordered list of addresses containing all present devices below this RootBridge.
    ///
    /// # Errors
    /// This can basically fail with all the IO errors found in [`PciIoAccessPci`] methods.
    #[cfg(feature = "alloc")]
    pub fn enumerate(&mut self) -> crate::Result<BTreeSet<super::FullPciIoAddress>> {
        use crate::proto::pci::configuration::ResourceRangeType;
        use crate::proto::pci::enumeration;

        let mut devices = BTreeSet::new();
        // In the descriptors, the entry with range_type bus specifies the bus numbers that were
        // allocated to devices below this root bridge. The first bus number in this range is
        // the starting point. All subsequent numbers are reached via PCI bridge recursion during enumeration.
        if let Some(descriptor) = self
            .configuration()?
            .iter()
            .find(|d| d.resource_range_type == ResourceRangeType::Bus)
        {
            let addr = PciIoAddress::new(descriptor.address_min as u8, 0, 0);
            enumeration::visit_bus(self, addr, &mut devices)?;
        }

        Ok(devices)
    }
}

/// Struct for performing PCI I/O operations on a root bridge.
#[derive(Debug)]
pub struct PciIoAccessPci<'a> {
    proto: *mut PciRootBridgeIoProtocol,
    io_access: &'a mut PciRootBridgeIoAccess,
}

impl PciIoAccessPci<'_> {
    /// Reads a single value of type `U` from the specified PCI address.
    ///
    /// # Arguments
    /// - `addr` - The PCI address to read from.
    ///
    /// # Returns
    /// - The read value of type `U`.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The read request could not be completed due to a lack of resources.
    pub fn read_one<U: PciIoUnit>(&self, addr: PciIoAddress) -> crate::Result<U> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        let mut result = U::default();
        unsafe {
            (self.io_access.read)(
                self.proto,
                width_mode,
                addr.into(),
                1,
                ptr::from_mut(&mut result).cast(),
            )
            .to_result_with_val(|| result)
        }
    }

    /// Writes a single value of type `U` to the specified PCI address.
    ///
    /// # Arguments
    /// - `addr` - The PCI address to write to.
    /// - `data` - The value to write.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The write request could not be completed due to a lack of resources.
    pub fn write_one<U: PciIoUnit>(&self, addr: PciIoAddress, data: U) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        unsafe {
            (self.io_access.write)(
                self.proto,
                width_mode,
                addr.into(),
                1,
                ptr::from_ref(&data).cast(),
            )
            .to_result()
        }
    }

    /// Reads multiple values from the specified PCI address range.
    ///
    /// # Arguments
    /// - `addr` - The starting PCI address to read from.
    /// - `data` - A mutable slice to store the read values.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The read operation could not be completed due to a lack of resources.
    pub fn read<U: PciIoUnit>(&self, addr: PciIoAddress, data: &mut [U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        unsafe {
            (self.io_access.read)(
                self.proto,
                width_mode,
                addr.into(),
                data.len(),
                data.as_mut_ptr().cast(),
            )
            .to_result()
        }
    }

    /// Writes multiple values to the specified PCI address range.
    ///
    /// # Arguments
    /// - `addr` - The starting PCI address to write to.
    /// - `data` - A slice containing the values to write.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The write operation could not be completed due to a lack of resources.
    pub fn write<U: PciIoUnit>(&self, addr: PciIoAddress, data: &[U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        unsafe {
            (self.io_access.write)(
                self.proto,
                width_mode,
                addr.into(),
                data.len(),
                data.as_ptr().cast(),
            )
            .to_result()
        }
    }

    /// Fills a PCI address range with the specified value.
    ///
    /// # Arguments
    /// - `addr` - The starting PCI address to fill.
    /// - `count` - The number of units to write.
    /// - `data` - The value to fill the address range with.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The operation could not be completed due to a lack of resources.
    pub fn fill_write<U: PciIoUnit>(
        &self,
        addr: PciIoAddress,
        count: usize,
        data: U,
    ) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fill);
        unsafe {
            (self.io_access.write)(
                self.proto,
                width_mode,
                addr.into(),
                count,
                ptr::from_ref(&data).cast(),
            )
            .to_result()
        }
    }

    /// Reads a sequence of values of type `U` from the specified PCI address by repeatedly accessing it.
    ///
    /// # Arguments
    /// - `addr` - The PCI address to read from.
    /// - `data` - A mutable slice to store the read values.
    ///
    /// # Behavior
    /// This reads from the same memory region (starting at `addr` and ending at `addr + size_of::<U>()`) repeatedly.
    /// The resulting `data` buffer will contain the elements returned by reading the same address multiple times sequentially.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The read operation could not be completed due to a lack of resources.
    pub fn fifo_read<U: PciIoUnit>(&self, addr: PciIoAddress, data: &mut [U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fifo);
        unsafe {
            (self.io_access.read)(
                self.proto,
                width_mode,
                addr.into(),
                data.len(),
                data.as_mut_ptr().cast(),
            )
            .to_result()
        }
    }

    /// Writes a sequence of values of type `U` to the specified PCI address repeatedly.
    ///
    /// # Arguments
    /// - `addr` - The PCI address to write to.
    /// - `data` - A slice containing the values to write.
    ///
    /// # Behavior
    /// This sequentially writes all elements within the given `data` buffer to the same memory region
    /// (starting at `addr` and ending at `addr + size_of::<U>()`) sequentially.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`] The write operation could not be completed due to a lack of resources.
    pub fn fifo_write<U: PciIoUnit>(&self, addr: PciIoAddress, data: &[U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fifo);
        unsafe {
            (self.io_access.write)(
                self.proto,
                width_mode,
                addr.into(),
                data.len(),
                data.as_ptr().cast(),
            )
            .to_result()
        }
    }
}
