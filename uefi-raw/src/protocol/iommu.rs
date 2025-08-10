// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::table::boot::{AllocateType, MemoryType};
use crate::{Guid, Handle, Status, guid};
use bitflags::bitflags;
use core::ffi::c_void;

use crate::newtype_enum;

/// EDKII IOMMU Protocol GUID
impl EdkiiIommuProtocol {
    pub const GUID: Guid = guid!("4e939de9-d948-4b0f-88ed-e6e1ce517c1e");
}

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
    /// IOMMU Operation for Map (matches EDKII_IOMMU_OPERATION)
    pub enum EdkiiIommuOperation: u32 => {
        /// A read operation from system memory by a bus master that is not capable of producing PCI dual address cycles.
        BUS_MASTER_READ = 0,
        /// A write operation to system memory by a bus master that is not capable of producing PCI dual address cycles.
        BUS_MASTER_WRITE = 1,
        /// Provides both read and write access to system memory by both the processor and a bus master that is not capable of producing PCI dual address cycles.
        BUS_MASTER_COMMON_BUFFER = 2,
        /// A read operation from system memory by a bus master that is capable of producing PCI dual address cycles.
        BUS_MASTER_READ64 = 3,
        /// A write operation to system memory by a bus master that is capable of producing PCI dual address cycles.
        BUS_MASTER_WRITE64 = 4,
        /// Provides both read and write access to system memory by both the processor and a bus master that is capable of producing PCI dual address cycles.
        BUS_MASTER_COMMON_BUFFER64 = 5,
        /// Maximum value (not a valid operation, for bounds checking)
        MAXIMUM = 6,
    }
}

/// EDKII IOMMU protocol revision constant
pub const EDKII_IOMMU_PROTOCOL_REVISION: u64 = 0x0001_0000;

bitflags! {
    /// EDKII IOMMU attribute flags
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EdkiiIommuAttribute: u64 {
        /// Memory is write-combined
        const MEMORY_WRITE_COMBINE   = 0x0080;
        /// Memory is cached
        const MEMORY_CACHED          = 0x0800;
        /// Dual address cycle supported
        const DUAL_ADDRESS_CYCLE     = 0x8000;
    }
}

impl EdkiiIommuAttribute {
    /// Valid attributes for allocate_buffer
    pub const VALID_FOR_ALLOCATE_BUFFER: Self = Self::from_bits_truncate(
        Self::MEMORY_WRITE_COMBINE.bits()
            | Self::MEMORY_CACHED.bits()
            | Self::DUAL_ADDRESS_CYCLE.bits(),
    );

    /// Invalid attributes for allocate_buffer (all bits except valid)
    pub const INVALID_FOR_ALLOCATE_BUFFER: Self =
        Self::from_bits_truncate(!Self::VALID_FOR_ALLOCATE_BUFFER.bits());
}

bitflags! {
    /// EDKII IOMMU access flags for SetAttribute
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EdkiiIommuAccess: u64 {
        /// Read access
        const READ  = 0x1;
        /// Write access
        const WRITE = 0x2;
    }
}
