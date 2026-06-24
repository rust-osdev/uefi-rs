// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Root Bridge protocol.

use super::{PciIoAddress, PciIoUnit, encode_io_mode_and_unit};
use crate::StatusExt;
#[cfg(feature = "alloc")]
use crate::proto::pci::configuration::QwordAddressSpaceDescriptor;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::ffi::c_void;
use core::ptr;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::pci::root_bridge::{
    PciRootBridgeIoAccess, PciRootBridgeIoProtocol, PciRootBridgeIoProtocolAttributes,
};

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

    /// Access PCI controller registers in the configuration space on this root bridge.
    pub const fn pci(&mut self) -> PciIoAccess<'_, PciConfigurationSpace> {
        PciIoAccess {
            proto: &mut self.0,
            io_access: &mut self.0.pci,
            _address_space: PciConfigurationSpace,
        }
    }

    /// Access PCI controller registers in the memory space on this root bridge.
    pub const fn memory(&mut self) -> PciIoAccess<'_, PciMemorySpace> {
        PciIoAccess {
            proto: &mut self.0,
            io_access: &mut self.0.mem,
            _address_space: PciMemorySpace,
        }
    }

    /// Access PCI controller registers in the I/O space on this root bridge.
    pub const fn io(&mut self) -> PciIoAccess<'_, PciIoSpace> {
        PciIoAccess {
            proto: &mut self.0,
            io_access: &mut self.0.io,
            _address_space: PciIoSpace,
        }
    }

    /// Flush all PCI posted write transactions from a PCI host bridge to system memory.
    ///
    /// # Errors
    /// - [`Status::DEVICE_ERROR`] The PCI posted write transactions were not flushed from the PCI host bridge
    ///   due to a hardware error.
    pub fn flush(&mut self) -> crate::Result<()> {
        // SAFETY: The memory is valid.
        unsafe { (self.0.flush)(&mut self.0).to_result() }
    }

    /// Returns the set of [`PciRootBridgeIoProtocolAttributes`] that this PCI root bridge
    /// supports.
    pub fn supported_attributes(&self) -> crate::Result<PciRootBridgeIoProtocolAttributes> {
        let mut supported = 0;

        // SAFETY: The memory is valid.
        unsafe {
            (self.0.get_attributes)(&self.0, &mut supported, ptr::null_mut()).to_result_with_val(
                || PciRootBridgeIoProtocolAttributes::from_bits_retain(supported),
            )
        }
    }

    /// Returns the [`PciRootBridgeIoProtocolAttributes`] that this PCI root bridge is currently using.
    pub fn attributes(&self) -> crate::Result<PciRootBridgeIoProtocolAttributes> {
        let mut current = 0;

        // SAFETY: The memory is valid.
        unsafe {
            (self.0.get_attributes)(&self.0, ptr::null_mut(), &mut current)
                .to_result_with_val(|| PciRootBridgeIoProtocolAttributes::from_bits_retain(current))
        }
    }

    /// Sets [`PciRootBridgeIoProtocolAttributes`] for this PCI root bridge.
    ///
    /// # Safety
    ///
    /// The new [`PciRootBridgeIoProtocolAttributes`] must be valid for the current system
    /// configuration.
    pub unsafe fn set_attributes(
        &mut self,
        attributes: PciRootBridgeIoProtocolAttributes,
    ) -> crate::Result {
        // SAFETY: The memory is valid.
        unsafe {
            (self.0.set_attributes)(
                &mut self.0,
                attributes.bits(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
            .to_result()
        }
    }

    /// Sets [`PciRootBridgeIoProtocolAttributes`] for this PCI root bridge (supporting attributes
    /// that require a resource range). For instance, modifying the cache settings of a PCI
    /// memory range requires the use of this function.
    ///
    /// The provided base and length are set to the actual base and length of the region whose
    /// attributes were changed (due to granularity or other requirements).
    ///
    /// # Safety
    ///
    /// The new [`PciRootBridgeIoProtocolAttributes`] must be valid for the current system
    /// configuration.
    pub unsafe fn set_attributes_with_range(
        &mut self,
        attributes: PciRootBridgeIoProtocolAttributes,
        base: &mut u64,
        length: &mut u64,
    ) -> crate::Result {
        // SAFETY: The memory is valid.
        unsafe { (self.0.set_attributes)(&mut self.0, attributes.bits(), base, length).to_result() }
    }

    // TODO: poll I/O
    // TODO: map & unmap & copy memory
    // TODO: buffer management

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
        // SAFETY: The memory is valid.
        unsafe {
            ((self.0.configuration)(&self.0, &mut resources))
                .to_result_with_val(|| configuration::parse(resources))
        }
    }

    // ###################################################
    // # Convenience functionality

    /// Recursively enumerate all devices, device functions and pci(e)-to-pci(e) bridges, starting from this pci root.
    ///
    /// The returned addresses might overlap with the addresses returned by another [`PciRootBridgeIo`] instance.
    /// Make sure to perform some form of cross-[`PciRootBridgeIo`] deduplication on the discovered addresses.
    /// **WARNING:** Only use the returned addresses with the respective [`PciRootBridgeIo`] instance that returned them.
    ///
    /// # Returns
    /// An ordered list of addresses containing all present devices below this RootBridge.
    ///
    /// # Errors
    /// This can basically fail with all the IO errors found in [`PciIoAccess`] methods.
    #[cfg(feature = "alloc")]
    pub fn enumerate(&mut self) -> crate::Result<super::enumeration::PciTree> {
        use super::enumeration::{self, PciTree};
        use crate::proto::pci::configuration::ResourceRangeType;

        let mut tree = PciTree::new(self.segment_nr());
        for descriptor in self.configuration()? {
            // In the descriptors we can query for the current root bridge, Bus entries contain ranges of valid
            // bus addresses. These are all bus addresses found below ourselves. These are not only the busses
            // linked to **directly** from ourselves, but also recursively. Thus we use PciTree::push_bus() to
            // determine whether we have already visited a given bus number.
            if descriptor.resource_range_type == ResourceRangeType::Bus {
                for bus in (descriptor.address_min as u8)..=(descriptor.address_max as u8) {
                    if tree.should_visit_bus(bus) {
                        let addr = PciIoAddress::new(bus, 0, 0);
                        enumeration::visit_bus(self, addr, &mut tree)?;
                    }
                }
            }
        }

        Ok(tree)
    }
}

/// Struct for performing PCI I/O operations on a root bridge.
#[derive(Debug)]
pub struct PciIoAccess<'a, S: PciIoAddressSpace> {
    proto: *mut PciRootBridgeIoProtocol,
    io_access: &'a mut PciRootBridgeIoAccess,
    _address_space: S,
}

impl<S: PciIoAddressSpace> PciIoAccess<'_, S> {
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
    pub fn read_one<U: PciIoUnit>(&self, addr: S::Address) -> crate::Result<U> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        let mut result = U::default();
        // SAFETY: The memory is valid.
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
    pub fn write_one<U: PciIoUnit>(&self, addr: S::Address, data: U) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        // SAFETY: The memory is valid.
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
    pub fn read<U: PciIoUnit>(&self, addr: S::Address, data: &mut [U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        // SAFETY: The memory is valid.
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
    pub fn write<U: PciIoUnit>(&self, addr: S::Address, data: &[U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Normal);
        // SAFETY: The memory is valid.
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
        addr: S::Address,
        count: usize,
        data: U,
    ) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fill);
        // SAFETY: The memory is valid.
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
    pub fn fifo_read<U: PciIoUnit>(&self, addr: S::Address, data: &mut [U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fifo);
        // SAFETY: The memory is valid.
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
    pub fn fifo_write<U: PciIoUnit>(&self, addr: S::Address, data: &[U]) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(super::PciIoMode::Fifo);
        // SAFETY: The memory is valid.
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

/// Marker struct for the PCI memory space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PciMemorySpace;

impl private::Sealed for PciMemorySpace {}
impl PciIoAddressSpace for PciMemorySpace {
    type Address = u64;
}

/// Marker struct for the PCI I/O space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PciIoSpace;

impl private::Sealed for PciIoSpace {}
impl PciIoAddressSpace for PciIoSpace {
    type Address = u32;
}

/// Marker struct for the PCI configuration space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PciConfigurationSpace;

impl private::Sealed for PciConfigurationSpace {}
impl PciIoAddressSpace for PciConfigurationSpace {
    type Address = PciIoAddress;
}

/// Trait representing how to convert from the address type expected for the address space and the
/// raw address space.
pub trait PciIoAddressSpace: private::Sealed {
    /// Specifies the type of the address space addresses.
    type Address: Into<u64>;
}

mod private {
    pub trait Sealed {}
}
