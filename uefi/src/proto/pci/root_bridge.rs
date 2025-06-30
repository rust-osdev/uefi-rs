// SPDX-License-Identifier: MIT OR Apache-2.0

//! PCI Root Bridge protocol.

use super::{PciIoUnit, encode_io_mode_and_unit};
use crate::StatusExt;
use crate::proto::pci::buffer::PciBuffer;
use crate::proto::pci::region::PciMappedRegion;
use core::ffi::c_void;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::num::NonZeroUsize;
use core::ptr;
use core::ptr::NonNull;
use core::time::Duration;
use log::debug;
use uefi::proto::pci::PciIoMode;
use uefi::proto::pci::root_bridge::io_access::IoAccessType;
use uefi_macros::unsafe_protocol;
use uefi_raw::Status;
use uefi_raw::protocol::pci::resource::QWordAddressSpaceDescriptor;
use uefi_raw::protocol::pci::root_bridge::{
    PciRootBridgeIoAccess, PciRootBridgeIoProtocol, PciRootBridgeIoProtocolAttribute,
    PciRootBridgeIoProtocolOperation,
};
use uefi_raw::table::boot::{AllocateType, MemoryType, PAGE_SIZE};

#[cfg(doc)]
use crate::Status;

/// Protocol that provides access to the PCI Root Bridge I/O protocol.
///
/// # UEFI Spec Description
/// Provides the basic Memory, I/O, PCI configuration, and DMA interfaces that are
/// used to abstract accesses to PCI controllers behind a PCI Root Bridge Controller.
#[repr(transparent)]
#[unsafe_protocol(PciRootBridgeIoProtocol::GUID)]
#[derive(Debug)]
pub struct PciRootBridgeIo(PciRootBridgeIoProtocol);

impl PciRootBridgeIo {
    /// Get the segment number where this PCI root bridge resides.
    #[must_use]
    pub fn segment_nr(&self) -> u32 {
        self.0.segment_number
    }

    /// Access PCI operations on this root bridge.
    pub fn pci(&self) -> PciIoAccessPci<'_, io_access::Pci> {
        PciIoAccessPci {
            proto: ptr::from_ref(&self.0).cast_mut(),
            io_access: &self.0.pci,
            _type: PhantomData,
        }
    }

    /// Access I/O operations on this root bridge.
    pub fn io(&self) -> PciIoAccessPci<'_, io_access::Io> {
        PciIoAccessPci {
            proto: ptr::from_ref(&self.0).cast_mut(),
            io_access: &self.0.io,
            _type: PhantomData,
        }
    }

    /// Access memory operations on this root bridge.
    pub fn mem(&self) -> PciIoAccessPci<'_, io_access::Mem> {
        PciIoAccessPci {
            proto: ptr::from_ref(&self.0).cast_mut(),
            io_access: &self.0.mem,
            _type: PhantomData,
        }
    }

    /// Flush all PCI posted write transactions from a PCI host bridge to system memory.
    ///
    /// # Errors
    /// - [`Status::DEVICE_ERROR`] The PCI posted write transactions were not flushed from the PCI host bridge
    ///   due to a hardware error.
    pub fn flush(&self) -> crate::Result<()> {
        let this = (&self.0) as *const PciRootBridgeIoProtocol;
        unsafe { (self.0.flush)(this.cast_mut()).to_result() }
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
    #[cfg(feature = "alloc")]
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
    #[cfg(feature = "alloc")]
    pub fn map<'p, 'r, T>(
        &'p self,
        operation: PciRootBridgeIoProtocolOperation,
        to_map: &'r T,
    ) -> crate::Result<PciMappedRegion<'p, 'r>>
    where
        'p: 'r,
        T: ?Sized,
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
            Status::SUCCESS => Ok(PciMappedRegion::new(
                mapped_address,
                bytes,
                mapping,
                to_map,
                &self.0,
            )),
            e => Err(e.into()),
        }
    }

    /// Copies a region in PCI root bridge memory space onto the other.
    /// Two regions must have same length. Functionally, this is the same as
    /// `<[T]>::copy_from_slice` which is effectively memcpy.
    /// And the same safety requirements as the above method apply.
    ///
    /// # Returns
    /// [`Ok`] on successful copy.
    ///
    /// [`Err`] otherwise.
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`]: The request could not be completed due to a lack of resources.
    /// # Question
    /// Should this support other types than just primitives?
    #[cfg(feature = "alloc")]
    pub fn copy<U: PciIoUnit>(&self, destination: &[U], source: &[U]) -> crate::Result<()> {
        assert_eq!(destination.len(), source.len());
        let width = encode_io_mode_and_unit::<U>(PciIoMode::Normal);

        let status = unsafe {
            (self.0.copy_mem)(
                ((&self.0) as *const PciRootBridgeIoProtocol).cast_mut(),
                width,
                destination.as_ptr().addr() as u64,
                source.as_ptr().addr() as u64,
                destination.len(),
            )
        };

        status.to_result()
    }

    /// Retrieves the current resource settings of this PCI root bridge
    /// in the form of a set of ACPI resource descriptors.
    ///
    /// # Returns
    /// [`Ok`] when it successfully retrieved current configuration.
    ///
    /// [`Err`] when it failed to retrieve current configuration.
    /// - Its Status value will be [`Status::UNSUPPORTED`]
    ///
    /// # Panic
    /// It may panic if pci devices or drivers for those provided by boot service misbehave.
    /// There are multiple verifications put in place, and they will panic if invariants
    /// are broken, such as when invalid enum variant value was received
    /// or reserved bits are not 0
    pub fn configuration(&self) -> crate::Result<&[QWordAddressSpaceDescriptor]> {
        let mut configuration_address = 0u64;
        let configuration_status = unsafe {
            (self.0.configuration)(
                &self.0,
                ((&mut configuration_address) as *mut u64).cast::<*const c_void>(),
            )
        };
        match configuration_status {
            Status::SUCCESS => {
                let head = configuration_address as *const QWordAddressSpaceDescriptor;
                let mut count = 0;

                unsafe {
                    loop {
                        let cursor = head.add(count);
                        match cursor.cast::<u8>().read() {
                            0x8A => {
                                let cursor_ref = cursor.as_ref().unwrap();
                                cursor_ref.verify();
                                count += 1;
                                if count >= 1024 {
                                    panic!(
                                        "Timed out while fetching configurations:\
                                     There are more than 1024 configuration spaces"
                                    );
                                }
                            }
                            0x79 => {
                                let checksum_ptr = cursor.cast::<u8>().byte_add(1);
                                if checksum_ptr.read() != 0 {
                                    panic!(
                                        "Checksum failed for QWordAddressSpaceDescriptor list starting at 0x{:X} with size {}",
                                        configuration_address, count
                                    );
                                }
                                break;
                            }
                            _ => panic!(
                                "Invalid Tag value for entry in QWordAddressSpaceDescriptor list starting at 0x{:X} with index {}",
                                configuration_address, count
                            ),
                        }
                    }
                };
                let list: &[QWordAddressSpaceDescriptor] =
                    unsafe { ptr::slice_from_raw_parts(head, count).as_ref().unwrap() };
                Ok(list)
            }
            e => Err(e.into()),
        }
    }

    /// Polls a same memory location until criteria is met.
    /// The criteria in question is met when value read from provided reference
    /// equals to provided value when masked:
    /// `(*to_poll) & mask == value`
    ///
    /// Refer to [`Self::poll_io`] for polling io port instead.
    ///
    /// # Returns
    /// [`Ok`]: Criteria was met before timeout.
    ///
    /// [`Err`]: One of below error happened:
    /// - [`Status::TIMEOUT`]: Delay expired before a match occurred.
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`]: The request could not be completed due to a lack of resources.
    ///
    /// # Panic
    /// Panics when delay is too large (longer than 58494 years).
    pub fn poll_mem<U: PciIoUnit>(
        &self,
        to_poll: &U,
        mask: U,
        value: U,
        delay: Duration,
    ) -> crate::Result<(), u64> {
        let mut result = 0u64;
        let delay = delay.as_nanos().div_ceil(100).try_into().unwrap();
        let status = unsafe {
            (self.0.poll_mem)(
                ptr::from_ref(&self.0).cast_mut(),
                encode_io_mode_and_unit::<U>(PciIoMode::Normal),
                ptr::from_ref(to_poll).addr() as u64,
                mask.into(),
                value.into(),
                delay,
                &mut result,
            )
        };

        status.to_result_with_err(|_| result)
    }

    /// Polls a same io port until criteria is met.
    /// The criteria in question is met when value read from provided reference
    /// equals to provided value when masked:
    /// `(*to_poll) & mask == value`
    ///
    /// Refer to [`Self::poll_mem`] for polling memory instead.
    ///
    /// # Returns
    /// [`Ok`]: Criteria was met before timeout.
    ///
    /// [`Err`]: One of below error happened:
    /// - [`Status::TIMEOUT`]: Delay expired before a match occurred.
    /// - [`Status::INVALID_PARAMETER`] The requested width is invalid for this PCI root bridge.
    /// - [`Status::OUT_OF_RESOURCES`]: The request could not be completed due to a lack of resources.
    ///
    /// # Panic
    /// Panics when delay is too large (longer than 58494 years).
    pub fn poll_io<U: PciIoUnit>(
        &self,
        to_poll: &U,
        mask: U,
        value: U,
        delay: Duration,
    ) -> crate::Result<(), u64> {
        let mut result = 0u64;
        let delay = delay.as_nanos().div_ceil(100).try_into().unwrap();
        let status = unsafe {
            (self.0.poll_io)(
                ptr::from_ref(&self.0).cast_mut(),
                encode_io_mode_and_unit::<U>(PciIoMode::Normal),
                ptr::from_ref(to_poll).addr() as u64,
                mask.into(),
                value.into(),
                delay,
                &mut result,
            )
        };

        status.to_result_with_err(|_| result)
    }

    /// Returns available and used attributes of this root bridge.
    ///
    /// # Returns
    /// Both supported and used attribute will be returned in struct [`AttributeReport`]
    pub fn get_attributes(&self) -> AttributeReport {
        let mut supports = PciRootBridgeIoProtocolAttribute::empty();
        let mut attributes = PciRootBridgeIoProtocolAttribute::empty();
        let status = unsafe {
            (self.0.get_attributes)(
                &self.0,
                ptr::from_mut(&mut supports).cast(),
                ptr::from_mut(&mut attributes).cast(),
            )
        };

        match status {
            Status::SUCCESS => AttributeReport {
                supported: supports,
                used: attributes,
            },
            Status::INVALID_PARAMETER => unreachable!(),
            e => panic!("Unexpected error occurred: {:?}", e),
        }
    }

    /// Sets attributes to use for this root bridge.
    /// Specified attributes must be supported. Otherwise, it will return error.
    /// Supported attributes can be requested with [`Self::get_attributes`]
    ///
    /// # Returns
    /// [`Ok`]: Optional resource range. It will only be available when resource
    /// parameter is Some and one of:
    /// - [`PciRootBridgeIoProtocolAttribute::MEMORY_WRITE_COMBINE`]
    /// - [`PciRootBridgeIoProtocolAttribute::MEMORY_CACHED`]
    /// - [`PciRootBridgeIoProtocolAttribute::MEMORY_DISABLE`]
    /// is set.
    ///
    /// [`Err`]: Possible error cases:
    /// - [`Status::UNSUPPORTED`]: A bit is set in Attributes that is not supported by the PCI Root Bridge.
    ///   The supported attribute bits are reported by [`Self::get_attributes`]
    /// - [`Status::INVALID_PARAMETER`]: More than one attribute bit is set in Attributes that requires a resource parameter.
    /// - [`Status::OUT_OF_RESOURCES`]: There are not enough resources to set the attributes on the resource range specified by resource parameter.
    pub fn set_attributes<'a, 'p>(
        &'p self,
        attributes: PciRootBridgeIoProtocolAttribute,
        resource: Option<&'a [u64]>,
    ) -> crate::Result<Option<&'a [u64]>>
    where
        'p: 'a,
    {
        let (mut base, mut length) = match resource {
            Some(v) => {
                let ptr: *const [u64] = v;
                let base = ptr.addr() as u64;
                let length = ptr.len() as u64;
                (base, length)
            }
            None => (0, 0),
        };
        let status = unsafe {
            (self.0.set_attributes)(
                ptr::from_ref(&self.0).cast_mut(),
                attributes.bits(),
                &mut base,
                &mut length,
            )
        };

        match status {
            Status::SUCCESS => {
                let to_return = if length != 0 {
                    unsafe {
                        Some(
                            ptr::slice_from_raw_parts(base as *const u64, length as usize)
                                .as_ref()
                                .unwrap(),
                        )
                    }
                } else {
                    None
                };
                Ok(to_return)
            }
            e => Err(e.into()),
        }
    }
}

/// Struct for performing PCI I/O operations on a root bridge.
#[derive(Debug)]
pub struct PciIoAccessPci<'a, T: IoAccessType> {
    proto: *mut PciRootBridgeIoProtocol,
    io_access: &'a PciRootBridgeIoAccess,
    _type: PhantomData<T>,
}

/// Defines all 3 PCI_ROOT_BRIDGE_IO_PROTOCOL_ACCESS types according to UEFI protocol 2.10
/// Currently there are 3: Mem, Io, Pci
pub mod io_access {
    use uefi::proto::pci::PciIoAddress;

    /// One of PCI_ROOT_BRIDGE_IO_PROTOCOL_ACCESS types that provides PCI configurations space access.
    #[derive(Debug)]
    pub struct Pci;
    impl IoAccessType for Pci {
        type Address = PciIoAddress;
    }

    /// One of PCI_ROOT_BRIDGE_IO_PROTOCOL_ACCESS types that provides I/O space access.
    #[derive(Debug)]
    pub struct Io;
    impl IoAccessType for Io {
        type Address = u64;
    }

    /// One of PCI_ROOT_BRIDGE_IO_PROTOCOL_ACCESS types that provides memory mapped I/O space access.
    #[derive(Debug)]
    pub struct Mem;
    impl IoAccessType for Mem {
        type Address = u64;
    }

    /// Defines base trait for all PCI_ROOT_BRIDGE_IO_PROTOCOL_ACCESS types
    pub trait IoAccessType {
        /// Specify what to use as address. This is needed as Pci IO Access uses special address format.
        type Address: Into<u64>;
    }
}

impl<T: IoAccessType> PciIoAccessPci<'_, T> {
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
    pub fn read_one<U: PciIoUnit>(&self, addr: T::Address) -> crate::Result<U> {
        let width_mode = encode_io_mode_and_unit::<U>(PciIoMode::Normal);
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
    pub fn write_one<U: PciIoUnit>(&self, addr: T::Address, data: U) -> crate::Result<()> {
        let width_mode = encode_io_mode_and_unit::<U>(PciIoMode::Normal);
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
    pub fn read<U: PciIoUnit>(&self, addr: T::Address, data: &mut [U]) -> crate::Result<()> {
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
    pub fn write<U: PciIoUnit>(&self, addr: T::Address, data: &[U]) -> crate::Result<()> {
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
        addr: T::Address,
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
    pub fn fifo_read<U: PciIoUnit>(&self, addr: T::Address, data: &mut [U]) -> crate::Result<()> {
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
    pub fn fifo_write<U: PciIoUnit>(&self, addr: T::Address, data: &[U]) -> crate::Result<()> {
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

/// Struct containing return value for [`PciRootBridgeIo::get_attributes`]
/// This is to minimize confusion by giving both of them names.
#[derive(Debug)]
pub struct AttributeReport {
    /// Attributes supported by this bridge.
    /// Only attributes in this set can be used as parameter for [`PciRootBridgeIo::set_attributes`]
    pub supported: PciRootBridgeIoProtocolAttribute,

    /// Attributes currently being used.
    pub used: PciRootBridgeIoProtocolAttribute,
}
