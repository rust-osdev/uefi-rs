// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Root Bridge protocol.

use super::{PciIoAddress, PciIoUnit, encode_io_mode_and_unit};
#[cfg(feature = "alloc")]
use crate::proto::pci::configuration::QwordAddressSpaceDescriptor;
use crate::proto::pci::root_bridge::buffer::PciBuffer;
use crate::proto::pci::root_bridge::region::PciMappedRegion;
use crate::{Status, StatusExt};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr;
use core::ptr::{NonNull, null_mut};
use log::debug;
use uefi_macros::unsafe_protocol;
use uefi_raw::protocol::pci::root_bridge::{
    PciRootBridgeIoAccess, PciRootBridgeIoProtocol, PciRootBridgeIoProtocolAttribute,
    PciRootBridgeIoProtocolOperation,
};
use uefi_raw::table::boot::{AllocateType, MemoryType, PAGE_SIZE};

pub mod buffer;
pub mod region;

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

    /// Allocates pages suitable for communicating with PCI devices.
    ///
    /// # Errors
    /// - [`Status::INVALID_PARAMETER`] MemoryType is invalid.
    /// - [`Status::UNSUPPORTED`] Attributes is unsupported. The only legal attribute bits are:
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_MEMORY_WRITE_COMBINE`]
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_MEMORY_CACHED`]
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_DUAL_ADDRESS_CYCLE`]
    /// - [`Status::OUT_OF_RESOURCES`] The memory pages could not be allocated.
    pub fn allocate_buffer<T>(
        &self,
        memory_type: MemoryType,
        pages: Option<NonZeroUsize>,
        attributes: PciRootBridgeIoProtocolAttribute,
    ) -> crate::Result<PciBuffer<'_, MaybeUninit<T>>> {
        let original_alignment = align_of::<T>();
        // TODO switch to const block once it lands on stable. These checks should be done in compile time.
        assert_ne!(original_alignment, 0);
        assert!(PAGE_SIZE >= original_alignment);
        assert_eq!(PAGE_SIZE % original_alignment, 0);

        let alignment = PAGE_SIZE;

        let pages = if let Some(pages) = pages {
            pages
        } else {
            let size = size_of::<T>();
            assert_ne!(size, 0);

            NonZeroUsize::new(size.div_ceil(alignment)).unwrap()
        };
        let size = size_of::<T>();
        // TODO switch to const block once it lands on stable.
        assert!(pages.get() * PAGE_SIZE >= size);

        let mut address: *mut T = null_mut();
        let status = unsafe {
            (self.0.allocate_buffer)(
                &self.0,
                AllocateType(0),
                memory_type,
                pages.get(),
                ptr::from_mut(&mut address).cast(),
                attributes.bits(),
            )
        };

        match status {
            Status::SUCCESS => {
                let base = NonNull::new(address.cast()).unwrap();
                debug!("Allocated {} pages at 0x{:X}", pages.get(), address.addr());
                Ok(PciBuffer {
                    base,
                    pages,
                    proto: &self.0,
                })
            }
            error
            @ (Status::INVALID_PARAMETER | Status::UNSUPPORTED | Status::OUT_OF_RESOURCES) => {
                Err(error.into())
            }
            _ => unreachable!(),
        }
    }

    /// Allocates pages suitable for communicating with PCI devices and initialize it right away.
    ///
    /// # Errors
    /// Same as [`Self::allocate_buffer`]
    /// - [`Status::INVALID_PARAMETER`] MemoryType is invalid.
    /// - [`Status::UNSUPPORTED`] Attributes is unsupported. The only legal attribute bits are:
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_MEMORY_WRITE_COMBINE`]
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_MEMORY_CACHED`]
    ///   - [`PciRootBridgeIoProtocolAttribute::PCI_ATTRIBUTE_DUAL_ADDRESS_CYCLE`]
    /// - [`Status::OUT_OF_RESOURCES`] The memory pages could not be allocated.
    pub fn allocate_buffer_init<T>(
        &self,
        memory_type: MemoryType,
        value: T,
        attributes: PciRootBridgeIoProtocolAttribute,
    ) -> crate::Result<PciBuffer<'_, T>> {
        let buffer = self.allocate_buffer(memory_type, None, attributes)?;
        unsafe {
            buffer.base_ptr().write(MaybeUninit::new(value));
            Ok(buffer.assume_init())
        }
    }

    /// Map the given buffer into a PCI Controller-specific address
    /// so that devices can read system memory through it.
    ///
    /// # Arguments
    /// - `operation` - Indicates if bus master is going to read, write, or do both to the buffer.
    /// - `to_map` - Buffer to map.
    ///
    /// # Returns
    /// An mapped region and unmapped bytes. It can map up to one DMA operation. Meaning large values
    /// will require multiple calls to this function.
    pub fn map<'p: 'r, 'r, T>(
        &'p self,
        operation: PciRootBridgeIoProtocolOperation,
        to_map: &'r PciBuffer<'p, T>,
    ) -> crate::Result<(PciMappedRegion<'p, 'r, T>, usize)> {
        let host_address = to_map.base_ptr();
        let mut bytes = size_of::<T>();
        let requested_bytes = bytes;
        let mut device_address = 0usize;
        let mut mapping: *mut c_void = null_mut();

        let status = unsafe {
            (self.0.map)(
                &self.0,
                operation,
                host_address.cast(),
                ptr::from_mut(&mut bytes),
                ptr::from_mut(&mut device_address).cast(),
                ptr::from_mut(&mut mapping),
            )
        };

        status.to_result_with_val(|| {
            let left_over = requested_bytes - bytes;
            let region = PciMappedRegion {
                host: to_map,
                length: 0,
                device_address,
                key: mapping,
                proto: &self.0,
            };
            (region, left_over)
        })
    }

    // TODO: poll I/O
    // TODO: mem I/O access
    // TODO: io I/O access
    // TODO: copy memory
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
    /// This can basically fail with all the IO errors found in [`PciIoAccessPci`] methods.
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
