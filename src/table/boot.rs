//! UEFI services available during boot.

use {Status, Result};
use super::Header;

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,
    raise_tpl: extern "C" fn(Tpl) -> Tpl,
    restore_tpl: extern "C" fn(Tpl),
    allocate_pages: extern "C" fn(alloc_ty: u32, mem_ty: u32, count: usize, addr: &mut u64) -> Status,
    free_pages: extern "C" fn(u64, usize) -> Status,
    _pad: [usize; 24],
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
        (self.allocate_pages)(ty, mem_ty as u32, count, &mut addr).into_with(|| addr as usize)
    }

    /// Frees memory pages allocated by UEFI.
    pub fn free_pages(&self, addr: usize, count: usize) -> Result<()> {
        (self.free_pages)(addr as u64, count).into()
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
