//! UEFI services available during boot.

use crate::protocol::device_path::DevicePathProtocol;
use crate::table::Header;
use crate::{Char16, Event, Guid, Handle, PhysicalAddress, Status, VirtualAddress};
use bitflags::bitflags;
use core::ffi::c_void;

/// Table of pointers to all the boot services.
#[repr(C)]
pub struct BootServices {
    pub header: Header,

    // Task Priority services
    pub raise_tpl: unsafe extern "efiapi" fn(new_tpl: Tpl) -> Tpl,
    pub restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl),

    // Memory allocation functions
    pub allocate_pages: unsafe extern "efiapi" fn(
        alloc_ty: u32,
        mem_ty: MemoryType,
        count: usize,
        addr: *mut PhysicalAddress,
    ) -> Status,
    pub free_pages: unsafe extern "efiapi" fn(addr: PhysicalAddress, pages: usize) -> Status,
    pub get_memory_map: unsafe extern "efiapi" fn(
        size: *mut usize,
        map: *mut MemoryDescriptor,
        key: *mut usize,
        desc_size: *mut usize,
        desc_version: *mut u32,
    ) -> Status,
    pub allocate_pool: unsafe extern "efiapi" fn(
        pool_type: MemoryType,
        size: usize,
        buffer: *mut *mut u8,
    ) -> Status,
    pub free_pool: unsafe extern "efiapi" fn(buffer: *mut u8) -> Status,

    // Event & timer functions
    pub create_event: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_func: Option<EventNotifyFn>,
        notify_ctx: *mut c_void,
        out_event: *mut Event,
    ) -> Status,
    pub set_timer: unsafe extern "efiapi" fn(event: Event, ty: u32, trigger_time: u64) -> Status,
    pub wait_for_event: unsafe extern "efiapi" fn(
        number_of_events: usize,
        events: *mut Event,
        out_index: *mut usize,
    ) -> Status,
    pub signal_event: unsafe extern "efiapi" fn(event: Event) -> Status,
    pub close_event: unsafe extern "efiapi" fn(event: Event) -> Status,
    pub check_event: unsafe extern "efiapi" fn(event: Event) -> Status,

    // Protocol handlers
    pub install_protocol_interface: unsafe extern "efiapi" fn(
        handle: *mut Handle,
        guid: *const Guid,
        interface_type: InterfaceType,
        interface: *mut c_void,
    ) -> Status,
    pub reinstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        old_interface: *mut c_void,
        new_interface: *mut c_void,
    ) -> Status,
    pub uninstall_protocol_interface: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        interface: *mut c_void,
    ) -> Status,
    pub handle_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        proto: *const Guid,
        out_proto: *mut *mut c_void,
    ) -> Status,
    pub reserved: *mut c_void,
    pub register_protocol_notify: unsafe extern "efiapi" fn(
        protocol: *const Guid,
        event: Event,
        registration: *mut *const c_void,
    ) -> Status,
    pub locate_handle: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: *const Guid,
        key: *const c_void,
        buf_sz: *mut usize,
        buf: *mut Handle,
    ) -> Status,
    pub locate_device_path: unsafe extern "efiapi" fn(
        proto: *const Guid,
        device_path: *mut *const DevicePathProtocol,
        out_handle: *mut Handle,
    ) -> Status,
    pub install_configuration_table:
        unsafe extern "efiapi" fn(guid_entry: *const Guid, table_ptr: *const c_void) -> Status,

    // Image services
    pub load_image: unsafe extern "efiapi" fn(
        boot_policy: u8,
        parent_image_handle: Handle,
        device_path: *const DevicePathProtocol,
        source_buffer: *const u8,
        source_size: usize,
        image_handle: *mut Handle,
    ) -> Status,
    pub start_image: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_data_size: *mut usize,
        exit_data: *mut *mut Char16,
    ) -> Status,
    pub exit: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_status: Status,
        exit_data_size: usize,
        exit_data: *mut Char16,
    ) -> !,
    pub unload_image: unsafe extern "efiapi" fn(image_handle: Handle) -> Status,
    pub exit_boot_services:
        unsafe extern "efiapi" fn(image_handle: Handle, map_key: usize) -> Status,

    // Misc services
    pub get_next_monotonic_count: unsafe extern "efiapi" fn(count: *mut u64) -> Status,
    pub stall: unsafe extern "efiapi" fn(microseconds: usize) -> Status,
    pub set_watchdog_timer: unsafe extern "efiapi" fn(
        timeout: usize,
        watchdog_code: u64,
        data_size: usize,
        watchdog_data: *const u16,
    ) -> Status,

    // Driver support services
    pub connect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Handle,
        remaining_device_path: *const DevicePathProtocol,
        recursive: bool,
    ) -> Status,
    pub disconnect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Handle,
        child: Handle,
    ) -> Status,

    // Protocol open / close services
    pub open_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        interface: *mut *mut c_void,
        agent_handle: Handle,
        controller_handle: Handle,
        attributes: u32,
    ) -> Status,
    pub close_protocol: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        agent_handle: Handle,
        controller_handle: Handle,
    ) -> Status,
    pub open_protocol_information: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol: *const Guid,
        entry_buffer: *mut *const OpenProtocolInformationEntry,
        entry_count: *mut usize,
    ) -> Status,

    // Library services
    pub protocols_per_handle: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol_buffer: *mut *mut *const Guid,
        protocol_buffer_count: *mut usize,
    ) -> Status,
    pub locate_handle_buffer: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: *const Guid,
        key: *const c_void,
        no_handles: *mut usize,
        buf: *mut *mut Handle,
    ) -> Status,
    pub locate_protocol: unsafe extern "efiapi" fn(
        proto: *const Guid,
        registration: *mut c_void,
        out_proto: *mut *mut c_void,
    ) -> Status,

    // These two function pointers require the `c_variadic` feature, which is
    // not yet available in stable Rust:
    // https://github.com/rust-lang/rust/issues/44930
    pub install_multiple_protocol_interfaces: usize,
    pub uninstall_multiple_protocol_interfaces: usize,

    // CRC services
    pub calculate_crc32:
        unsafe extern "efiapi" fn(data: *const c_void, data_size: usize, crc32: *mut u32) -> Status,

    // Misc services
    pub copy_mem: unsafe extern "efiapi" fn(dest: *mut u8, src: *const u8, len: usize),
    pub set_mem: unsafe extern "efiapi" fn(buffer: *mut u8, len: usize, value: u8),

    // New event functions (UEFI 2.0 or newer)
    pub create_event_ex: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: *mut c_void,
        event_group: *mut Guid,
        out_event: *mut Event,
    ) -> Status,
}

bitflags! {
    /// Flags describing the type of an UEFI event and its attributes.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct EventType: u32 {
        /// The event is a timer event and may be passed to `BootServices::set_timer()`
        /// Note that timers only function during boot services time.
        const TIMER = 0x8000_0000;

        /// The event is allocated from runtime memory.
        /// This must be done if the event is to be signaled after ExitBootServices.
        const RUNTIME = 0x4000_0000;

        /// Calling wait_for_event or check_event will enqueue the notification
        /// function if the event is not already in the signaled state.
        /// Mutually exclusive with `NOTIFY_SIGNAL`.
        const NOTIFY_WAIT = 0x0000_0100;

        /// The notification function will be enqueued when the event is signaled
        /// Mutually exclusive with `NOTIFY_WAIT`.
        const NOTIFY_SIGNAL = 0x0000_0200;

        /// The event will be signaled at ExitBootServices time.
        /// This event type should not be combined with any other.
        /// Its notification function must follow some special rules:
        /// - Cannot use memory allocation services, directly or indirectly
        /// - Cannot depend on timer events, since those will be deactivated
        const SIGNAL_EXIT_BOOT_SERVICES = 0x0000_0201;

        /// The event will be notified when SetVirtualAddressMap is performed.
        /// This event type should not be combined with any other.
        const SIGNAL_VIRTUAL_ADDRESS_CHANGE = 0x6000_0202;
    }
}

newtype_enum! {
/// Interface type of a protocol interface.
pub enum InterfaceType: u32 => {
    /// Native interface
    NATIVE_INTERFACE = 0,
}}

/// Raw event notification function.
pub type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: *mut c_void);

bitflags! {
    /// Flags describing the capabilities of a memory range.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        /// This memory is earmarked for specific purposes such as for specific
        /// device drivers or applications. This serves as a hint to the OS to
        /// avoid this memory for core OS data or code that cannot be relocated.
        const SPECIAL_PURPOSE = 0x4_0000;
        /// This memory region is capable of being protected with the CPU's memory
        /// cryptography capabilities.
        const CPU_CRYPTO = 0x8_0000;
        /// This memory must be mapped by the OS when a runtime service is called.
        const RUNTIME = 0x8000_0000_0000_0000;
        /// This memory region is described with additional ISA-specific memory
        /// attributes as specified in `MemoryAttribute::ISA_MASK`.
        const ISA_VALID = 0x4000_0000_0000_0000;
        /// These bits are reserved for describing optional ISA-specific cache-
        /// ability attributes that are not covered by the standard UEFI Memory
        /// Attribute cacheability bits such as `UNCACHEABLE`, `WRITE_COMBINE`,
        /// `WRITE_THROUGH`, `WRITE_BACK`, and `UNCACHEABLE_EXPORTED`.
        ///
        /// See Section 2.3 "Calling Conventions" in the UEFI Specification
        /// for further information on each ISA that takes advantage of this.
        const ISA_MASK = 0x0FFF_F000_0000_0000;
    }
}

/// A structure describing a region of memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of memory occupying this range.
    pub ty: MemoryType,
    /// Starting physical address.
    pub phys_start: PhysicalAddress,
    /// Starting virtual address.
    pub virt_start: VirtualAddress,
    /// Number of 4 KiB pages contained in this range.
    pub page_count: u64,
    /// The capability attributes of this memory range.
    pub att: MemoryAttribute,
}

impl MemoryDescriptor {
    /// Memory descriptor version number.
    pub const VERSION: u32 = 1;
}

impl Default for MemoryDescriptor {
    fn default() -> MemoryDescriptor {
        MemoryDescriptor {
            ty: MemoryType::RESERVED,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        }
    }
}

newtype_enum! {
/// The type of a memory range.
///
/// UEFI allows firmwares and operating systems to introduce new memory types
/// in the 0x70000000..0xFFFFFFFF range. Therefore, we don't know the full set
/// of memory types at compile time, and it is _not_ safe to model this C enum
/// as a Rust enum.
#[derive(PartialOrd, Ord, Hash)]
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

impl MemoryType {
    /// Construct a custom `MemoryType`. Values in the range `0x80000000..=0xffffffff` are free for use if you are
    /// an OS loader.
    #[must_use]
    pub const fn custom(value: u32) -> MemoryType {
        assert!(value >= 0x80000000);
        MemoryType(value)
    }
}

#[repr(C)]
pub struct OpenProtocolInformationEntry {
    pub agent_handle: Handle,
    pub controller_handle: Handle,
    pub attributes: u32,
    pub open_count: u32,
}

newtype_enum! {
/// Task priority level.
///
/// Although the UEFI specification repeatedly states that only the variants
/// specified below should be used in application-provided input, as the other
/// are reserved for internal firmware use, it might still happen that the
/// firmware accidentally discloses one of these internal TPLs to us.
///
/// Since feeding an unexpected variant to a Rust enum is UB, this means that
/// this C enum must be interfaced via the newtype pattern.
pub enum Tpl: usize => {
    /// Normal task execution level.
    APPLICATION = 4,
    /// Async interrupt-style callbacks run at this TPL.
    CALLBACK    = 8,
    /// Notifications are masked at this level.
    ///
    /// This is used in critical sections of code.
    NOTIFY      = 16,
    /// Highest priority level.
    ///
    /// Even processor interrupts are disable at this level.
    HIGH_LEVEL  = 31,
}}
