// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Root Bridge protocol.

use super::{PciIoAddress, PciIoUnit, encode_io_mode_and_unit};
use crate::StatusExt;
use crate::proto::pci::buffer::PciBuffer;
use crate::proto::pci::region::{PciMappedRegion, PciRegion};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr;
use core::ptr::NonNull;
use log::debug;
use uefi_macros::unsafe_protocol;
use uefi_raw::Status;
use uefi_raw::protocol::pci::root_bridge::{PciRootBridgeIoAccess, PciRootBridgeIoProtocol, PciRootBridgeIoProtocolAttribute, PciRootBridgeIoProtocolOperation, PciRootBridgeIoProtocolWidth};
use uefi_raw::table::boot::{AllocateType, MemoryType, PAGE_SIZE};

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
    ) -> crate::Result<PciBuffer<MaybeUninit<T>>> {
        let mut address = 0usize;
        let original_alignment = align_of::<T>();
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
                let base = NonNull::new(address as *mut MaybeUninit<T>).unwrap();
                debug!("Allocated {} pages at 0x{:X}", pages.get(), address);
                Ok(PciBuffer::new(base, pages, &self.0))
            }
            error
            @ (Status::INVALID_PARAMETER | Status::UNSUPPORTED | Status::OUT_OF_RESOURCES) => {
                Err(error.into())
            }
            _ => unreachable!(),
        }
    }

    /// Map given variable's address into PCI Controller-specific address
    /// required to access it from a DMA bus master.
    /// # Arguments
    /// - `operation` - Indicates if bus master is going to read, write or do both to given variable.
    /// - `to_map` - Variable to map.
    ///
    /// # Returns
    /// - PciMappedRegion capturing lifetime of passed variable
    pub fn map<'p, 'r, T>(
        &'p self,
        operation: PciRootBridgeIoProtocolOperation,
        to_map: &'r T,
    ) -> PciMappedRegion<'p, 'r>
    where
        'p: 'r,
    {
        let host_address = ptr::from_ref(to_map);
        let mut bytes = size_of_val(to_map);
        let mut mapped_address = 0u64;
        let mut mapping: *mut c_void = ptr::null_mut();

        let status = unsafe {
            (self.0.map)(
                &self.0,
                operation,
                host_address.cast(),
                ptr::from_mut(&mut bytes),
                ptr::from_mut(&mut mapped_address).cast(),
                ptr::from_mut(&mut mapping),
            )
        };

        match status {
            Status::SUCCESS => {
                PciMappedRegion::new(mapped_address, bytes, mapping, to_map, &self.0)
            }
            _ => unreachable!(),
        }
    }

    /// Copies a region in PCI root bridge memory space onto the other.
    /// Two regions must have same length. Functionally, this is the same as
    /// `<[T]>::copy_from_slice` which is effectively memcpy.
    /// And the same safety requirements as the above method apply.
    pub fn copy(
        &mut self,
        width: PciRootBridgeIoProtocolWidth,
        destination: PciRegion,
        source: PciRegion,
    ) -> crate::Result<()> {
        assert_eq!(destination.length, source.length);

        let status = unsafe {
            (self.0.copy_mem)(
                &mut self.0,
                width,
                destination.device_address,
                source.device_address,
                destination.length,
            )
        };

        match status {
            Status::SUCCESS => Ok(()),
            error => Err(error.into()),
        }
    }

    // TODO: poll I/O
    // TODO: mem I/O access
    // TODO: io I/O access
    // TODO: get/set attributes
    // TODO: configuration / resource settings
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
