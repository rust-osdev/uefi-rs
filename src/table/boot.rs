//! UEFI services available during boot.

use super::Header;
use bitflags::bitflags;
use core::{mem, ptr, result};
use crate::proto::Protocol;
use crate::{Event, Guid, Handle, Result, Status};

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,

    // Task Priority services
    raise_tpl: extern "win64" fn(Tpl) -> Tpl,
    restore_tpl: extern "win64" fn(Tpl),

    // Memory allocation functions
    allocate_pages:
        extern "win64" fn(alloc_ty: u32, mem_ty: MemoryType, count: usize, addr: &mut u64)
            -> Status,
    free_pages: extern "win64" fn(u64, usize) -> Status,
    memory_map:
        extern "win64" fn(size: &mut usize, usize, key: &mut MemoryMapKey, &mut usize, &mut u32)
            -> Status,
    allocate_pool: extern "win64" fn(MemoryType, usize, addr: &mut usize) -> Status,
    free_pool: extern "win64" fn(buffer: usize) -> Status,

    // Event & timer functions
    create_event: usize,
    set_timer: usize,
    wait_for_event:
        extern "win64" fn(number_of_events: usize, events: *mut Event, out_index: &mut usize)
            -> Status,
    signal_event: usize,
    close_event: usize,
    check_event: usize,

    // Protocol handlers
    install_protocol_interface: usize,
    reinstall_protocol_interface: usize,
    uninstall_protocol_interface: usize,
    handle_protocol:
        extern "win64" fn(handle: Handle, proto: *const Guid, out_proto: &mut usize) -> Status,
    _reserved: usize,
    register_protocol_notify: usize,
    locate_handle: extern "win64" fn(
        search_ty: i32,
        proto: *const Guid,
        key: *mut (),
        buf_sz: &mut usize,
        buf: *mut Handle,
    ) -> Status,
    locate_device_path: usize,
    install_configuration_table: usize,

    // Image services
    load_image: usize,
    start_image: usize,
    exit: usize,
    unload_image: usize,
    exit_boot_services: extern "win64" fn(Handle, MemoryMapKey) -> Status,

    // Misc services
    get_next_monotonic_count: usize,
    stall: extern "win64" fn(usize) -> Status,
    set_watchdog_timer: extern "win64" fn(
        timeout: usize,
        watchdog_code: u64,
        data_size: usize,
        watchdog_data: *const u16,
    ) -> Status,

    // Driver support services
    connect_controller: usize,
    disconnect_controller: usize,

    // Protocol open / close services
    open_protocol: usize,
    close_protocol: usize,
    open_protocol_information: usize,

    // Library services
    protocols_per_handle: usize,
    locate_handle_buffer: usize,
    locate_protocol: usize,
    install_multiple_protocol_interfaces: usize,
    uninstall_multiple_protocol_interfaces: usize,

    // CRC services
    calculate_crc32: usize,

    // Misc services
    copy_mem: extern "win64" fn(dest: *mut u8, src: *const u8, len: usize),
    set_mem: extern "win64" fn(buffer: *mut u8, len: usize, value: u8),

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

    /// Retrieves the size, in bytes, of the current memory map.
    ///
    /// A buffer of this size will be capable of holding the whole current memory map,
    /// including padding. Note, however, that allocations will increase the size of the
    /// memory map, therefore it is better to allocate some extra space.
    pub fn memory_map_size(&self) -> usize {
        let mut map_size = 0;
        let mut map_key = MemoryMapKey(0);
        let mut entry_size = 0;
        let mut entry_version = 0;

        let status = (self.memory_map)(
            &mut map_size,
            0,
            &mut map_key,
            &mut entry_size,
            &mut entry_version,
        );
        assert_eq!(status, Status::BUFFER_TOO_SMALL);

        map_size * entry_size
    }

    /// Retrieves the current memory map.
    ///
    /// The allocated buffer should be big enough to contain the memory map,
    /// and a way of estimating how big it should be is by calling `memory_map_size`.
    ///
    /// The returned key is a unique identifier of the current configuration of memory.
    /// Any allocations or such will change the memory map's key.
    pub fn memory_map<'a>(
        &self,
        buffer: &'a mut [u8],
    ) -> Result<(
        MemoryMapKey,
        impl ExactSizeIterator<Item = &'a MemoryDescriptor>,
    )> {
        let mut map_size = buffer.len();
        let map_buffer = buffer.as_ptr() as usize;
        let mut map_key = MemoryMapKey(0);
        let mut entry_size = 0;
        let mut entry_version = 0;

        (self.memory_map)(
            &mut map_size,
            map_buffer,
            &mut map_key,
            &mut entry_size,
            &mut entry_version,
        )?;

        let len = map_size / entry_size;

        let iter = MemoryMapIter {
            buffer,
            entry_size,
            index: 0,
            len,
        };

        Ok((map_key, iter))
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

    /// Stops execution until an event is signaled
    ///
    /// This function must be called at priority level Tpl::Application. If an
    /// attempt is made to call it at any other priority level, an `Unsupported`
    /// error is returned.
    ///
    /// The input Event slice is repeatedly iterated from first to last until an
    /// event is signaled or an error is detected. The following checks are
    /// performed on each event:
    ///
    /// * If an event is of type NotifySignal, then an `InvalidParameter` error
    ///   is returned together with the index of the event that caused the failure.
    /// * If an event is in the signaled state, the signaled state is cleared
    ///   and the index of the event that was signaled is returned.
    /// * If an event is not in the signaled state but does have a notification
    ///   function, the notification function is queued at the event's
    ///   notification task priority level. If the execution of the event's
    ///   notification function causes the event to be signaled, then the
    ///   signaled state is cleared and the index of the event that was signaled
    ///   is returned.
    ///
    /// To wait for a specified time, a timer event must be included in the
    /// Event slice.
    ///
    /// To check if an event is signaled without waiting, an already signaled
    /// event can be used as the last event in the slice being checked, or the
    /// check_event() interface may be used.
    pub fn wait_for_event(&self, events: &mut [Event]) -> result::Result<usize, (Status, usize)> {
        let (number_of_events, events) = (events.len(), events.as_mut_ptr());
        let mut index = unsafe { mem::uninitialized() };
        match (self.wait_for_event)(number_of_events, events, &mut index) {
            Status::SUCCESS => Ok(index),
            s @ Status::INVALID_PARAMETER => Err((s, index)),
            error => Err((error, 0)),
        }
    }

    /// Query a handle for a certain protocol.
    ///
    /// This function attempts to get the protocol implementation of a handle,
    /// based on the protocol GUID.
    pub fn handle_protocol<P: Protocol>(&self, handle: Handle) -> Option<ptr::NonNull<P>> {
        let mut ptr = 0usize;
        match (self.handle_protocol)(handle, &P::GUID, &mut ptr) {
            Status::SUCCESS => ptr::NonNull::new(ptr as *mut P),
            _ => None,
        }
    }

    /// Enumerates all handles installed on the system which match a certain query.
    ///
    /// You should first call this function with `None` for the output buffer,
    /// in order to retrieve the length of the buffer you need to allocate.
    ///
    /// The next call will fill the buffer with the requested data.
    pub fn locate_handle(
        &self,
        search_ty: SearchType,
        output: Option<&mut [Handle]>,
    ) -> Result<usize> {
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
            Status::SUCCESS | Status::BUFFER_TOO_SMALL => Ok(buffer_len),
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
    pub unsafe fn exit_boot_services(&self, image: Handle, mmap_key: MemoryMapKey) -> Result<()> {
        (self.exit_boot_services)(image, mmap_key).into()
    }

    /// Stalls the processor for an amount of time.
    ///
    /// The time is in microseconds.
    pub fn stall(&self, time: usize) {
        assert_eq!((self.stall)(time), Status::SUCCESS);
    }

    /// Set the watchdog timer.
    ///
    /// UEFI will start a 5-minute countdown after an UEFI image is loaded.
    /// The image must either successfully load an OS and call `ExitBootServices`
    /// in that time, or disable the watchdog.
    ///
    /// Otherwise, the firmware will log the event using the provided numeric
    /// code and data, then reset the system.
    ///
    /// This function allows you to change the watchdog timer's timeout to a
    /// certain amount of seconds or to disable the watchdog entirely. It also
    /// allows you to change what will be logged when the timer expires.
    ///
    /// The watchdog codes from 0 to 0xffff (65535) are reserved for internal
    /// firmware use. You should therefore only use them if instructed to do so
    /// by firmware-specific documentation. Higher values can be used freely.
    ///
    /// If provided, the watchdog data must be a null-terminated string
    /// optionally followed by other binary data.
    pub fn set_watchdog_timer(
        &self,
        timeout: usize,
        watchdog_code: u64,
        data: Option<&mut [u16]>,
    ) -> Result<()> {
        let (data_len, data) = data
            .map(|d| {
                assert!(
                    d.contains(&0),
                    "Watchdog data must contain a null-terminated string"
                );
                (d.len(), d.as_mut_ptr())
            })
            .unwrap_or((0, ptr::null_mut()));

        (self.set_watchdog_timer)(timeout, watchdog_code, data_len, data).into()
    }

    /// Copies memory from source to destination. The buffers can overlap.
    ///
    /// This function is unsafe as it can be used to violate most safety
    /// invariants of the Rust type system.
    ///
    pub unsafe fn memmove(&self, dest: *mut u8, src: *const u8, size: usize) {
        (self.copy_mem)(dest, src, size);
    }

    /// Sets a buffer to a certain value.
    ///
    /// This function is unsafe as it can be used to violate most safety
    /// invariants of the Rust type system.
    ///
    pub unsafe fn memset(&self, buffer: *mut u8, size: usize, value: u8) {
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
    // SAFETY: The UEFI specification repeatedly states that only the these
    //         priority levels may be used, the rest being reserved for internal
    //         firmware use. So only these priority levels should be exposed to
    //         the application, and modeling them as a Rust enum seems safe.
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

newtype_enum! {
/// The type of a memory range.
///
/// UEFI allows firmwares and operating systems to introduce new memory types
/// in the 0x70000000..0xFFFFFFFF range. Therefore, we don't know the full set
/// of memory types at compile time, and it is _not_ safe to model this C enum
/// as a Rust enum.
pub enum MemoryType: u32 => {
    /// This enum variant is not used.
    RESERVED                =  0,
    /// The code portions of a loaded UEFI application.
    LOADER_CODE             =  1,
    /// The data portions of a loaded UEFI applications,
    /// as well as any memory allocated by it.
    LOADER_DATA             =  2,
    /// Code of the boot drivers.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_CODE      =  3,
    /// Memory used to store boot drivers' data.
    ///
    /// Can be reused after OS is loaded.
    BOOT_SERVICES_DATA      =  4,
    /// Runtime drivers' code.
    RUNTIME_SERVICES_CODE   =  5,
    /// Runtime services' code.
    RUNTIME_SERVICES_DATA   =  6,
    /// Free usable memory.
    CONVENTIONAL            =  7,
    /// Memory in which errors have been detected.
    UNUSABLE                =  8,
    /// Memory that holds ACPI tables.
    /// Can be reclaimed after they are parsed.
    ACPI_RECLAIM            =  9,
    /// Firmware-reserved addresses.
    ACPI_NON_VOLATILE       = 10,
    /// A region used for memory-mapped I/O.
    MMIO                    = 11,
    /// Address space used for memory-mapped port I/O.
    MMIO_PORT_SPACE         = 12,
    /// Address space which is part of the processor.
    PAL_CODE                = 13,
    /// Memory region which is usable and is also non-volatile.
    PERSISTENT_MEMORY       = 14,
}}

/// A structure describing a region of memory.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct MemoryDescriptor {
    /// Type of memory occupying this range.
    pub ty: MemoryType,
    /// Skip 4 bytes as UEFI declares items in structs should be naturally aligned
    padding: u32,
    /// Starting physical address.
    pub phys_start: u64,
    /// Starting virtual address.
    pub virt_start: u64,
    /// Number of 4 KiB pages contained in this range.
    pub page_count: u64,
    /// The capability attributes of this memory range.
    pub att: MemoryAttribute,
}

impl Default for MemoryDescriptor {
    fn default() -> MemoryDescriptor {
        MemoryDescriptor {
            ty: MemoryType::RESERVED,
            padding: 0,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        }
    }
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
        const RUNTIME = 0x8000_0000_0000_0000;
    }
}

/// A unique identifier of a memory map.
///
/// If the memory map changes, this value is no longer valid.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(usize);

#[derive(Debug)]
struct MemoryMapIter<'a> {
    buffer: &'a [u8],
    entry_size: usize,
    index: usize,
    len: usize,
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryDescriptor;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.len - self.index;

        (sz, Some(sz))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let ptr = self.buffer.as_ptr() as usize + self.entry_size * self.index;

            self.index += 1;

            let descriptor = unsafe { mem::transmute::<usize, &MemoryDescriptor>(ptr) };

            Some(descriptor)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for MemoryMapIter<'a> {}

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
