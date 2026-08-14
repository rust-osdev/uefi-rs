// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::table::boot::{AllocateType, MemoryType};
use crate::{Guid, Handle, Status, guid};
use bitflags::bitflags;
use core::ffi::c_void;

use crate::newtype_enum;

impl EdkiiIommuProtocol {
    /// EDK II IOMMU protocol GUID.
    pub const GUID: Guid = guid!("4e939de9-d948-4b0f-88ed-e6e1ce517c1e");
}

/// EDK II IOMMU protocol.
#[derive(Debug)]
#[repr(C)]
pub struct EdkiiIommuProtocol {
    pub revision: u64,
    pub set_attribute: unsafe extern "efiapi" fn(
        this: *const Self,
        device_handle: Handle,
        mapping: *mut c_void,
        iommu_access: EdkiiIommuAccess,
    ) -> Status,
    pub map: unsafe extern "efiapi" fn(
        this: *const Self,
        operation: EdkiiIommuOperation,
        host_address: *mut c_void,
        number_of_bytes: *mut usize,
        device_address: *mut u64,
        mapping: *mut *mut c_void,
    ) -> Status,
    pub unmap: unsafe extern "efiapi" fn(this: *const Self, mapping: *mut c_void) -> Status,
    pub allocate_buffer: unsafe extern "efiapi" fn(
        this: *const Self,
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
        host_address: *mut *mut c_void,
        attributes: EdkiiIommuAttribute,
    ) -> Status,
    pub free_buffer: unsafe extern "efiapi" fn(
        this: *const Self,
        pages: usize,
        host_address: *mut c_void,
    ) -> Status,
}

newtype_enum! {
    /// DMA operation passed to the IOMMU mapping function.
    pub enum EdkiiIommuOperation: u32 => {
        /// Reads system memory without PCI dual-address cycles.
        BUS_MASTER_READ = 0,
        /// Writes system memory without PCI dual-address cycles.
        BUS_MASTER_WRITE = 1,
        /// Shares a buffer without PCI dual-address cycles.
        BUS_MASTER_COMMON_BUFFER = 2,
        /// Reads system memory with PCI dual-address cycles.
        BUS_MASTER_READ64 = 3,
        /// Writes system memory with PCI dual-address cycles.
        BUS_MASTER_WRITE64 = 4,
        /// Shares a buffer with PCI dual-address cycles.
        BUS_MASTER_COMMON_BUFFER64 = 5,
        /// Sentinel used for bounds checking; not a valid operation.
        MAXIMUM = 6,
    }
}

/// EDK II IOMMU protocol revision.
pub const EDKII_IOMMU_PROTOCOL_REVISION: u64 = 0x0001_0000;

bitflags! {
    /// EDK II IOMMU memory attributes.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct EdkiiIommuAttribute: u64 {
        /// Uses write-combined memory.
        const MEMORY_WRITE_COMBINE   = 0x0080;
        /// Uses cached memory.
        const MEMORY_CACHED          = 0x0800;
        /// Supports PCI dual-address cycles.
        const DUAL_ADDRESS_CYCLE     = 0x8000;
    }
}

impl EdkiiIommuAttribute {
    /// Attributes accepted by `allocate_buffer`.
    pub const VALID_FOR_ALLOCATE_BUFFER: Self = Self::from_bits_truncate(
        Self::MEMORY_WRITE_COMBINE.bits()
            | Self::MEMORY_CACHED.bits()
            | Self::DUAL_ADDRESS_CYCLE.bits(),
    );

    /// Attributes rejected by `allocate_buffer`.
    pub const INVALID_FOR_ALLOCATE_BUFFER: Self =
        Self::from_bits_truncate(!Self::VALID_FOR_ALLOCATE_BUFFER.bits());
}

bitflags! {
    /// Access permissions passed to `set_attribute`.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct EdkiiIommuAccess: u64 {
        /// Grants read access.
        const READ  = 0x1;
        /// Grants write access.
        const WRITE = 0x2;
    }
}
