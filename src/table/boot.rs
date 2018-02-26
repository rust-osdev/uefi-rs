//! UEFI services available during boot.

use {Status, Result, Handle, Guid};
use super::Header;
use proto::Protocol;
use core::{ptr, mem};

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,

    // Task Priority services
    raise_tpl: extern "C" fn(Tpl) -> Tpl,
    restore_tpl: extern "C" fn(Tpl),

    // Memory allocation functions
    allocate_pages: extern "C" fn(alloc_ty: u32, mem_ty: MemoryType, count: usize, addr: &mut u64) -> Status,
    free_pages: extern "C" fn(u64, usize) -> Status,
    memory_map: extern "C" fn(size: &mut usize, usize, key: &mut MemoryMapKey, &mut usize, &mut u32) -> Status,
    allocate_pool: extern "C" fn(MemoryType, usize, addr: &mut usize) -> Status,
    free_pool: extern "C" fn(buffer: usize) -> Status,

    // Event & timer functions
    create_event: usize,
    set_timer: usize,
    wait_for_event: usize,
    signal_event: usize,
    close_event: usize,
    check_event: usize,

    // Protocol handlers
    install_protocol_interface: usize,
    reinstall_protocol_interface: usize,
    uninstall_protocol_interface: usize,
    handle_protocol: usize,
    _reserved: usize,
    register_protocol_notify: usize,
    locate_handle: extern "C" fn(search_ty: i32, proto: *const Guid, key: *mut (), buf_sz: &mut usize, buf: *mut Handle) -> Status,
    locate_device_path: usize,
    install_configuration_table: usize,

    // Image services
    load_image: usize,
    start_image: usize,
    exit: usize,
    unload_image: usize,
    exit_boot_services: extern "C" fn(Handle, MemoryMapKey) -> Status,

    // Misc functions
    get_next_monotonic_count: usize,
    stall: extern "C" fn(usize) -> Status,
    copy_mem: extern "C" fn(dest: usize, src: usize, len: usize),
    set_mem: extern "C" fn(buffer: usize, len: usize, value: u8),

    // New event functions (UEFI 2.0 or newer)
    create_event_ex: usize,
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

    /// Allocates from a memory pool. The address is 8-byte aligned.
    pub fn allocate_pool(&self, mem_ty: MemoryType, size: usize) -> Result<usize> {
        let mut buffer = 0;
        (self.allocate_pool)(mem_ty, size, &mut buffer).into_with(|| buffer)
    }

    /// Frees memory allocated from a pool.
    pub fn free_pool(&self, addr: usize) -> Result<()> {
        (self.free_pool)(addr).into()
    }

    /// Enumerates all handles installed on the system which match a certain query.
    ///
    /// You should first call this function with `None` for the output buffer,
    /// in order to retrieve the length of the buffer you need to allocate.
    ///
    /// The next call will fill the buffer with the requested data.
    pub fn locate_handle(&self, search_ty: SearchType, output: Option<&mut [Handle]>) -> Result<usize> {
        let handle_size = mem::size_of::<Handle>();

        let (mut buffer_size, buffer) = match output {
            Some(buffer) => (buffer.len() * handle_size, buffer.as_mut_ptr()),
            None => (0, ptr::null_mut()),
        };

        // Obtain the needed data from the parameters.
        let (ty, guid, key) = match search_ty {
            SearchType::AllHandles => (0, ptr::null(), ptr::null_mut()),
            SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null_mut()),
        };

        let status = (self.locate_handle)(ty, guid, key, &mut buffer_size, buffer);

        // Must convert the returned size (in bytes) to length (number of elements).
        let buffer_len = buffer_size / handle_size;

        match status {
            Status::Success | Status::BufferTooSmall => Ok(buffer_len),
            err => Err(err),
        }
    }

    /// Exits the early boot stage.
    ///
    /// After calling this function, the boot services functions become invalid.
    /// Only runtime services may be used.
    ///
    /// The handle passed must be the one of the currently executing image.
    ///
    /// The application **must** retrieve the current memory map, and pass in a key to ensure it is the latest.
    /// If the memory map was changed, you must obtain the new memory map,
    /// and then immediately call this function again.
    ///
    /// After you first call this function, the firmware may perform a partial shutdown of boot services.
    /// You should only call the mmap-related functions in order to update the memory map.
    pub fn exit_boot_services(&self, image: Handle, mmap_key: MemoryMapKey) -> Result<()> {
        (self.exit_boot_services)(image, mmap_key).into()
    }

    /// Stalls the processor for an amount of time.
    ///
    /// The time is in microseconds.
    pub fn stall(&self, time: usize) {
        // The spec says this cannot fail.
        (self.stall)(time);
    }

    /// Copies memory from source to destination. The buffers can overlap.
    pub fn memmove(&self, dest: usize, src: usize, size: usize) {
        (self.copy_mem)(dest, src, size);
    }

    /// Sets a buffer to a certain value.
    pub fn memset(&self, buffer: usize, size: usize, value: u8) {
        (self.set_mem)(buffer, size, value);
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

/// The type of handle search to perform.
#[derive(Debug, Copy, Clone)]
pub enum SearchType<'a> {
    /// Return all handles present on the system.
    AllHandles,
    /// Returns all handles supporting a certain protocol, specified by its GUID.
    ///
    /// If the protocol implements the `Protocol` interface,
    /// you can use the `from_proto` function to construct a new `SearchType`.
    ByProtocol(&'a Guid),
    // TODO: add ByRegisterNotify once the corresponding function is implemented.
}

impl<'a> SearchType<'a> {
    /// Constructs a new search type for a specified protocol.
    pub fn from_proto<P: Protocol>() -> Self {
        SearchType::ByProtocol(&P::GUID)
    }
}
