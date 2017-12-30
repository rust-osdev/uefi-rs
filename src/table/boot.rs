//! UEFI services available during boot.

use {Status, Result};
use super::Header;

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,
    raise_tpl: extern "C" fn(Tpl) -> Tpl,
    restore_tpl: extern "C" fn(Tpl),
    allocate_pages: extern "C" fn(alloc_ty: u32, mem_ty: MemoryType, count: usize, addr: &mut u64) -> Status,
    free_pages: extern "C" fn(u64, usize) -> Status,
    memory_map: extern "C" fn(size: &mut usize, usize, key: &mut MemoryMapKey, &mut usize, &mut u32) -> Status,
    allocate_pool: extern "C" fn(MemoryType, usize, addr: &mut usize) -> Status,
    free_pool: extern "C" fn(buffer: usize) -> Status,
    _pad: [usize; 21],
    stall: extern "C" fn(usize) -> Status,
}

impl BootServices {
    /// Raises a task's priority level and returns its previous level.
    pub fn raise_tpl(&self, tpl: Tpl) -> Tpl {
        (self.raise_tpl)(tpl)
    }

    /// Restores a task’s priority level to its previous value.
    pub fn restore_tpl(&self, old_tpl: Tpl) {
        (self.restore_tpl)(old_tpl)
    }

    /// Allocates memory pages from the system.
    ///
    /// UEFI OS loaders should allocate memory of the type `LoaderData`.
    pub fn allocate_pages(
        &self,
        ty: AllocateType,
        mem_ty: MemoryType,
        count: usize,
    ) -> Result<usize> {
        let (ty, mut addr) = match ty {
            AllocateType::AnyPages => (0, 0),
            AllocateType::MaxAddress(addr) => (1, addr as u64),
            AllocateType::Address(addr) => (2, addr as u64),
        };
        (self.allocate_pages)(ty, mem_ty, count, &mut addr).into_with(|| addr as usize)
    }

    /// Frees memory pages allocated by UEFI.
    pub fn free_pages(&self, addr: usize, count: usize) -> Result<()> {
        (self.free_pages)(addr as u64, count).into()
    }

    /// Allocates a memory pool.
    pub fn allocate_pool(&self, mem_ty: MemoryType, size: usize) -> Result<usize> {
        let mut buffer = 0;
        (self.allocate_pool)(mem_ty, size, &mut buffer).into_with(|| buffer)
    }

    /// Frees a memory pool allocated by UEFI.
    pub fn free_pool(&self, addr: usize) -> Result<()> {
        (self.free_pool)(addr).into()
    }

    /// Stalls the processor for an amount of time.
    ///
    /// The time is in microseconds.
    pub fn stall(&self, time: usize) {
        // The spec says this cannot fail.
        (self.stall)(time);
    }
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
}

/// Task priority level.
#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Tpl {
    /// Normal task execution level.
    Application = 4,
    /// Async interrupt-style callbacks run at this TPL.
    Callback = 8,
    /// Notifications are masked at this level.
    ///
    /// This is used in critical sections of code.
    Notify = 16,
    /// Highest priority level.
    ///
    /// Even processor interrupts are disable at this level.
    HighLevel = 31,
}

/// Type of allocation to perform.
#[derive(Debug, Copy, Clone)]
pub enum AllocateType {
    /// Allocate any possible pages.
    AnyPages,
    /// Allocate pages at any address below the given address.
    MaxAddress(usize),
    /// Allocate pages at the specified address.
    Address(usize),
}

/// The type of a memory range.
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum MemoryType {
    /// This enum variant is not used.
    Reserved,
    /// The code portions of a loaded UEFI application.
    LoaderCode,
    /// The data portions of a loaded UEFI applications,
    /// as well as any memory allocated by it.
    LoaderData,
    /// Code of the boot drivers.
    ///
    /// Can be reused after OS is loaded.
    BootServicesCode,
    /// Memory used to store boot drivers' data.
    ///
    /// Can be reused after OS is loaded.
    BootServicesData,
    /// Runtime drivers' code.
    RuntimeServicesCode,
    /// Runtime services' code.
    RuntimeServicesData,
    /// Free usable memory.
    Conventional,
    /// Memory in which errors have been detected.
    Unusable,
    /// Memory that holds ACPI tables.
    /// Can be reclaimed after they are parsed.
    AcpiReclaim,
    /// Firmware-reserved addresses.
    AcpiNonVolatile,
    /// A region used for memory-mapped I/O.
    Mmio,
    /// Address space used for memory-mapped port I/O.
    MmioPortSpace,
    /// Address space which is part of the processor.
    PalCode,
    /// Memory region which is usable and is also non-volatile.
    PersistentMemory,
}

/// A structure describing a region of memory.
#[repr(C, packed)]
pub struct MemoryDescriptor {
    /// Type of memory occupying this range.
    pub ty: MemoryType,
    /// Starting physical address.
    pub phys_start: u64,
    /// Starting virtual address.
    pub virt_start: u64,
    /// Number of 4 KiB pages contained in this range.
    pub page_count: u64,
    /// The capability attributes of this memory range.
    pub att: MemoryAttribute,
}

bitflags! {
    /// Flags describing the capabilities of a memory range.
    pub struct MemoryAttribute: u64 {
        /// Supports marking as uncacheable.
        const UNCACHEABLE = 0x1;
        /// Supports write-combining.
        const WRITE_COMBINE = 0x2;
        /// Supports write-through.
        const WRITE_THROUGH = 0x4;
        /// Support write-back.
        const WRITE_BACK = 0x8;
        /// Supports marking as uncacheable, exported and
        /// supports the "fetch and add" semaphore mechanism.
        const UNCACHABLE_EXPORTED = 0x10;
        /// Supports write-protection.
        const WRITE_PROTECT = 0x1000;
        /// Supports read-protection.
        const READ_PROTECT = 0x2000;
        /// Supports disabling code execution.
        const EXECUTE_PROTECT = 0x4000;
        /// Persistent memory.
        const NON_VOLATILE = 0x8000;
        /// This memory region is more reliable than other memory.
        const MORE_RELIABLE = 0x10000;
        /// This memory range can be set as read-only.
        const READ_ONLY = 0x20000;
        /// This memory must be mapped by the OS when a runtime service is called.
        const RUNTIME = 0x8000000000000000;
    }
}

/// A unique identifier of a memory map.
///
/// If the memory map changes, this value is no longer valid.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(usize);
