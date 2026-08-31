// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::table::boot::{AllocateType, MemoryType};
use crate::{Guid, PhysicalAddress, Status, guid, newtype_enum};
use bitflags::bitflags;
use core::ffi::c_void;

newtype_enum! {
    /// Corresponds to the `EFI_PCI_IO_PROTOCOL_WIDTH` enum.
    pub enum PciIoProtocolWidth: u32 => {
        UINT8 = 0,
        UINT16 = 1,
        UINT32 = 2,
        UINT64 = 3,
        FIFO_UINT8 = 4,
        FIFO_UINT16 = 5,
        FIFO_UINT32 = 6,
        FIFO_UINT64 = 7,
        FILL_UINT8 = 8,
        FILL_UINT16 = 9,
        FILL_UINT32 = 10,
        FILL_UINT64 = 11,
        MAXIMUM = 12,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_PCI_IO_PROTOCOL_OPERATION` enum.
    pub enum PciIoProtocolOperation: u32 => {
        BUS_MASTER_READ = 0,
        BUS_MASTER_WRITE = 1,
        BUS_MASTER_COMMON_BUFFER = 2,
        MAXIMUM = 3,
    }
}

newtype_enum! {
    /// Corresponds to the `EFI_PCI_IO_PROTOCOL_ATTRIBUTE_OPERATION` enum.
    pub enum PciIoProtocolAttributeOperation: u32 => {
        GET = 0,
        SET = 1,
        ENABLE = 2,
        DISABLE = 3,
        SUPPORTED = 4,
        MAXIMUM = 5,
    }
}

/// Special BAR that passes a memory or I/O cycle through unchanged (`EFI_PCI_IO_PASS_THROUGH_BAR`).
pub const PCI_IO_PASS_THROUGH_BAR: u8 = 0xff;

bitflags! {
    /// Corresponds to the `EFI_PCI_IO_PROTOCOL` attribute bitflags.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct PciIoProtocolAttributes: u64 {
        const EFI_PCI_IO_ATTRIBUTE_ISA_MOTHERBOARD_IO = 0x0001;
        const EFI_PCI_IO_ATTRIBUTE_ISA_IO = 0x0002;
        const EFI_PCI_IO_ATTRIBUTE_VGA_PALETTE_IO = 0x0004;
        const EFI_PCI_IO_ATTRIBUTE_VGA_MEMORY = 0x0008;
        const EFI_PCI_IO_ATTRIBUTE_VGA_IO = 0x0010;
        const EFI_PCI_IO_ATTRIBUTE_IDE_PRIMARY_IO = 0x0020;
        const EFI_PCI_IO_ATTRIBUTE_IDE_SECONDARY_IO = 0x0040;
        const EFI_PCI_IO_ATTRIBUTE_MEMORY_WRITE_COMBINE = 0x0080;
        const EFI_PCI_IO_ATTRIBUTE_IO = 0x0100;
        const EFI_PCI_IO_ATTRIBUTE_MEMORY = 0x0200;
        const EFI_PCI_IO_ATTRIBUTE_BUS_MASTER = 0x0400;
        const EFI_PCI_IO_ATTRIBUTE_MEMORY_CACHED = 0x0800;
        const EFI_PCI_IO_ATTRIBUTE_MEMORY_DISABLE = 0x1000;
        const EFI_PCI_IO_ATTRIBUTE_EMBEDDED_DEVICE = 0x2000;
        const EFI_PCI_IO_ATTRIBUTE_EMBEDDED_ROM = 0x4000;
        const EFI_PCI_IO_ATTRIBUTE_DUAL_ADDRESS_CYCLE = 0x8000;
        const EFI_PCI_IO_ATTRIBUTE_ISA_IO_16 = 0x10000;
        const EFI_PCI_IO_ATTRIBUTE_VGA_PALETTE_IO_16 = 0x20000;
        const EFI_PCI_IO_ATTRIBUTE_VGA_IO_16 = 0x40000;

        const EFI_PCI_IO_ATTRIBUTE_MASK = 0x077f;
        const EFI_PCI_DEVICE_ENABLE = Self::EFI_PCI_IO_ATTRIBUTE_IO.bits()
            | Self::EFI_PCI_IO_ATTRIBUTE_MEMORY.bits()
            | Self::EFI_PCI_IO_ATTRIBUTE_BUS_MASTER.bits();
        const EFI_VGA_DEVICE_ENABLE = Self::EFI_PCI_IO_ATTRIBUTE_VGA_PALETTE_IO.bits()
            | Self::EFI_PCI_IO_ATTRIBUTE_VGA_MEMORY.bits()
            | Self::EFI_PCI_IO_ATTRIBUTE_VGA_IO.bits()
            | Self::EFI_PCI_IO_ATTRIBUTE_IO.bits();
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PciIoProtocolAccess {
    pub read: unsafe extern "efiapi" fn(
        this: *mut PciIoProtocol,
        width: PciIoProtocolWidth,
        bar_index: u8,
        offset: u64,
        count: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: *mut PciIoProtocol,
        width: PciIoProtocolWidth,
        bar_index: u8,
        offset: u64,
        count: usize,
        buffer: *const c_void,
    ) -> Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct PciIoProtocolConfigAccess {
    pub read: unsafe extern "efiapi" fn(
        this: *mut PciIoProtocol,
        width: PciIoProtocolWidth,
        offset: u32,
        count: usize,
        buffer: *mut c_void,
    ) -> Status,
    pub write: unsafe extern "efiapi" fn(
        this: *mut PciIoProtocol,
        width: PciIoProtocolWidth,
        offset: u32,
        count: usize,
        buffer: *const c_void,
    ) -> Status,
}

#[derive(Debug)]
#[repr(C)]
pub struct PciIoProtocol {
    pub poll_mem: unsafe extern "efiapi" fn(
        this: *mut Self,
        width: PciIoProtocolWidth,
        bar_index: u8,
        offset: u64,
        mask: u64,
        value: u64,
        delay: u64,
        result: *mut u64,
    ) -> Status,
    pub poll_io: unsafe extern "efiapi" fn(
        this: *mut Self,
        width: PciIoProtocolWidth,
        bar_index: u8,
        offset: u64,
        mask: u64,
        value: u64,
        delay: u64,
        result: *mut u64,
    ) -> Status,
    pub mem: PciIoProtocolAccess,
    pub io: PciIoProtocolAccess,
    pub pci: PciIoProtocolConfigAccess,
    pub copy_mem: unsafe extern "efiapi" fn(
        this: *mut Self,
        width: PciIoProtocolWidth,
        dest_bar_index: u8,
        dest_offset: u64,
        src_bar_index: u8,
        src_offset: u64,
        count: usize,
    ) -> Status,
    pub map: unsafe extern "efiapi" fn(
        this: *const Self,
        operation: PciIoProtocolOperation,
        host_address: *const c_void,
        number_of_bytes: *mut usize,
        device_address: *mut PhysicalAddress,
        mapping: *mut *mut c_void,
    ) -> Status,
    pub unmap: unsafe extern "efiapi" fn(this: *const Self, mapping: *const c_void) -> Status,
    pub allocate_buffer: unsafe extern "efiapi" fn(
        this: *const Self,
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
        host_address: *mut *const c_void,
        attributes: u64,
    ) -> Status,
    pub free_buffer: unsafe extern "efiapi" fn(
        this: *const Self,
        pages: usize,
        host_address: *const c_void,
    ) -> Status,
    pub flush: unsafe extern "efiapi" fn(this: *mut Self) -> Status,
    pub get_location: unsafe extern "efiapi" fn(
        this: *const Self,
        segment_number: *mut usize,
        bus_number: *mut usize,
        device_number: *mut usize,
        function_number: *mut usize,
    ) -> Status,
    pub attributes: unsafe extern "efiapi" fn(
        this: *mut Self,
        operation: PciIoProtocolAttributeOperation,
        attributes: u64,
        result: *mut u64,
    ) -> Status,
    pub get_bar_attributes: unsafe extern "efiapi" fn(
        this: *const Self,
        bar_index: u8,
        supports: *mut u64,
        resources: *mut *const c_void,
    ) -> Status,
    pub set_bar_attributes: unsafe extern "efiapi" fn(
        this: *mut Self,
        attributes: u64,
        bar_index: u8,
        offset: *mut u64,
        length: *mut u64,
    ) -> Status,
    pub rom_size: u64,
    pub rom_image: *const c_void,
}

impl PciIoProtocol {
    pub const GUID: Guid = guid!("4cf5b200-68b8-4ca5-9eec-b23e3f50029a");
}
