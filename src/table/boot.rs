//! UEFI services available during boot.

use super::Header;
use crate::data_types::Align;
use crate::proto::{device_path::DevicePath, Protocol};
#[cfg(feature = "exts")]
use crate::proto::{loaded_image::LoadedImage, media::fs::SimpleFileSystem};
use crate::{Char16, Event, Guid, Handle, Result, Status};
#[cfg(feature = "exts")]
use alloc_api::vec::Vec;
use bitflags::bitflags;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::{self, MaybeUninit};
use core::{ptr, slice};

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,

    // Task Priority services
    raise_tpl: unsafe extern "efiapi" fn(new_tpl: Tpl) -> Tpl,
    restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl),

    // Memory allocation functions
    allocate_pages: extern "efiapi" fn(
        alloc_ty: u32,
        mem_ty: MemoryType,
        count: usize,
        addr: &mut u64,
    ) -> Status,
    free_pages: extern "efiapi" fn(addr: u64, pages: usize) -> Status,
    get_memory_map: unsafe extern "efiapi" fn(
        size: &mut usize,
        map: *mut MemoryDescriptor,
        key: &mut MemoryMapKey,
        desc_size: &mut usize,
        desc_version: &mut u32,
    ) -> Status,
    allocate_pool:
        extern "efiapi" fn(pool_type: MemoryType, size: usize, buffer: &mut *mut u8) -> Status,
    free_pool: extern "efiapi" fn(buffer: *mut u8) -> Status,

    // Event & timer functions
    create_event: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_func: Option<EventNotifyFn>,
        notify_ctx: *mut c_void,
        event: *mut Event,
    ) -> Status,
    set_timer: unsafe extern "efiapi" fn(event: Event, ty: u32, trigger_time: u64) -> Status,
    wait_for_event: unsafe extern "efiapi" fn(
        number_of_events: usize,
        events: *mut Event,
        out_index: *mut usize,
    ) -> Status,
    signal_event: usize,
    close_event: usize,
    check_event: usize,

    // Protocol handlers
    install_protocol_interface: usize,
    reinstall_protocol_interface: usize,
    uninstall_protocol_interface: usize,
    handle_protocol:
        extern "efiapi" fn(handle: Handle, proto: &Guid, out_proto: &mut *mut c_void) -> Status,
    _reserved: usize,
    register_protocol_notify: usize,
    locate_handle: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: *const Guid,
        key: *mut c_void,
        buf_sz: &mut usize,
        buf: *mut Handle,
    ) -> Status,
    locate_device_path: unsafe extern "efiapi" fn(
        proto: &Guid,
        device_path: &mut *mut DevicePath,
        out_handle: *mut Handle,
    ) -> Status,
    install_configuration_table: usize,

    // Image services
    load_image: unsafe extern "efiapi" fn(
        boot_policy: u8,
        parent_image_handle: Handle,
        device_path: *const DevicePath,
        source_buffer: *const u8,
        source_size: usize,
        *mut Handle,
    ) -> Status,
    start_image: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_data_size: *mut usize,
        exit_data: &mut *mut Char16,
    ) -> Status,
    exit: usize,
    unload_image: extern "efiapi" fn(image_handle: Handle) -> Status,
    exit_boot_services:
        unsafe extern "efiapi" fn(image_handle: Handle, map_key: MemoryMapKey) -> Status,

    // Misc services
    get_next_monotonic_count: usize,
    stall: extern "efiapi" fn(microseconds: usize) -> Status,
    set_watchdog_timer: unsafe extern "efiapi" fn(
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
    protocols_per_handle: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol_buffer: *mut *mut *const Guid,
        protocol_buffer_count: *mut usize,
    ) -> Status,
    locate_handle_buffer: usize,
    locate_protocol: extern "efiapi" fn(
        proto: &Guid,
        registration: *mut c_void,
        out_proto: &mut *mut c_void,
    ) -> Status,
    install_multiple_protocol_interfaces: usize,
    uninstall_multiple_protocol_interfaces: usize,

    // CRC services
    calculate_crc32: usize,

    // Misc services
    copy_mem: unsafe extern "efiapi" fn(dest: *mut u8, src: *const u8, len: usize),
    set_mem: unsafe extern "efiapi" fn(buffer: *mut u8, len: usize, value: u8),

    // New event functions (UEFI 2.0 or newer)
    create_event_ex: usize,
}

impl BootServices {
    /// Raises a task's priority level and returns its previous level.
    ///
    /// The effect of calling `raise_tpl` with a `Tpl` that is below the current
    /// one (which, sadly, cannot be queried) is undefined by the UEFI spec,
    /// which also warns against remaining at high `Tpl`s for a long time.
    ///
    /// This function outputs an RAII guard that will automatically restore the
    /// original `Tpl` when dropped.
    ///
    /// # Safety
    ///
    /// Raising a task's priority level can affect other running tasks and
    /// critical processes run by UEFI. The highest priority level is the
    /// most dangerous, since it disables interrupts.
    pub unsafe fn raise_tpl(&self, tpl: Tpl) -> TplGuard<'_> {
        TplGuard {
            boot_services: self,
            old_tpl: (self.raise_tpl)(tpl),
        }
    }

    /// Allocates memory pages from the system.
    ///
    /// UEFI OS loaders should allocate memory of the type `LoaderData`. An `u64`
    /// is returned even on 32-bit platforms because some hardware configurations
    /// like Intel PAE enable 64-bit physical addressing on a 32-bit processor.
    pub fn allocate_pages(
        &self,
        ty: AllocateType,
        mem_ty: MemoryType,
        count: usize,
    ) -> Result<u64> {
        let (ty, mut addr) = match ty {
            AllocateType::AnyPages => (0, 0),
            AllocateType::MaxAddress(addr) => (1, addr as u64),
            AllocateType::Address(addr) => (2, addr as u64),
        };
        (self.allocate_pages)(ty, mem_ty, count, &mut addr).into_with_val(|| addr)
    }

    /// Frees memory pages allocated by UEFI.
    pub fn free_pages(&self, addr: u64, count: usize) -> Result {
        (self.free_pages)(addr, count).into()
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

        let status = unsafe {
            (self.get_memory_map)(
                &mut map_size,
                ptr::null_mut(),
                &mut map_key,
                &mut entry_size,
                &mut entry_version,
            )
        };
        assert_eq!(status, Status::BUFFER_TOO_SMALL);

        map_size
    }

    /// Retrieves the current memory map.
    ///
    /// The allocated buffer should be big enough to contain the memory map,
    /// and a way of estimating how big it should be is by calling `memory_map_size`.
    ///
    /// The buffer must be aligned like a `MemoryDescriptor`.
    ///
    /// The returned key is a unique identifier of the current configuration of memory.
    /// Any allocations or such will change the memory map's key.
    ///
    /// If you want to store the resulting memory map without having to keep
    /// the buffer around, you can use `.copied().collect()` on the iterator.
    pub fn memory_map<'buf>(
        &self,
        buffer: &'buf mut [u8],
    ) -> Result<(
        MemoryMapKey,
        impl ExactSizeIterator<Item = &'buf MemoryDescriptor> + Clone,
    )> {
        let mut map_size = buffer.len();
        MemoryDescriptor::assert_aligned(buffer);
        #[allow(clippy::cast_ptr_alignment)]
        let map_buffer = buffer.as_ptr() as *mut MemoryDescriptor;
        let mut map_key = MemoryMapKey(0);
        let mut entry_size = 0;
        let mut entry_version = 0;

        assert_eq!(
            (map_buffer as usize) % mem::align_of::<MemoryDescriptor>(),
            0,
            "Memory map buffers must be aligned like a MemoryDescriptor"
        );

        unsafe {
            (self.get_memory_map)(
                &mut map_size,
                map_buffer,
                &mut map_key,
                &mut entry_size,
                &mut entry_version,
            )
        }
        .into_with_val(move || {
            let len = map_size / entry_size;
            let iter = MemoryMapIter {
                buffer,
                entry_size,
                index: 0,
                len,
            };
            (map_key, iter)
        })
    }

    /// Allocates from a memory pool. The pointer will be 8-byte aligned.
    pub fn allocate_pool(&self, mem_ty: MemoryType, size: usize) -> Result<*mut u8> {
        let mut buffer = ptr::null_mut();
        (self.allocate_pool)(mem_ty, size, &mut buffer).into_with_val(|| buffer)
    }

    /// Frees memory allocated from a pool.
    pub fn free_pool(&self, addr: *mut u8) -> Result {
        (self.free_pool)(addr).into()
    }

    /// Creates an event
    ///
    /// This function creates a new event of the specified type and returns it.
    ///
    /// Events are created in a "waiting" state, and may switch to a "signaled"
    /// state. If the event type has flag `NotifySignal` set, this will result in
    /// a callback for the event being immediately enqueued at the `notify_tpl`
    /// priority level. If the event type has flag `NotifyWait`, the notification
    /// will be delivered next time `wait_for_event` or `check_event` is called.
    /// In both cases, a `notify_fn` callback must be specified.
    ///
    /// # Safety
    ///
    /// This function is unsafe because callbacks must handle exit from boot
    /// services correctly.
    pub unsafe fn create_event(
        &self,
        event_ty: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<fn(Event)>,
    ) -> Result<Event> {
        // Prepare storage for the output Event
        let mut event = MaybeUninit::<Event>::uninit();

        // Use a trampoline to handle the impedance mismatch between Rust & C
        unsafe extern "efiapi" fn notify_trampoline(e: Event, ctx: *mut c_void) {
            let notify_fn: fn(Event) = mem::transmute(ctx);
            notify_fn(e); // SAFETY: Aborting panics are assumed here
        }
        let (notify_func, notify_ctx) = notify_fn
            .map(|notify_fn| {
                (
                    Some(notify_trampoline as EventNotifyFn),
                    notify_fn as fn(Event) as *mut c_void,
                )
            })
            .unwrap_or((None, ptr::null_mut()));

        // Now we're ready to call UEFI
        (self.create_event)(
            event_ty,
            notify_tpl,
            notify_func,
            notify_ctx,
            event.as_mut_ptr(),
        )
        .into_with_val(|| event.assume_init())
    }

    /// Stops execution until an event is signaled
    ///
    /// This function must be called at priority level `Tpl::APPLICATION`. If an
    /// attempt is made to call it at any other priority level, an `Unsupported`
    /// error is returned.
    ///
    /// The input `Event` slice is repeatedly iterated from first to last until
    /// an event is signaled or an error is detected. The following checks are
    /// performed on each event:
    ///
    /// * If an event is of type `NotifySignal`, then an `InvalidParameter`
    ///   error is returned with the index of the eve,t that caused the failure.
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
    pub fn wait_for_event(&self, events: &mut [Event]) -> Result<usize, Option<usize>> {
        let (number_of_events, events) = (events.len(), events.as_mut_ptr());
        let mut index = MaybeUninit::<usize>::uninit();
        unsafe { (self.wait_for_event)(number_of_events, events, index.as_mut_ptr()) }.into_with(
            || unsafe { index.assume_init() },
            |s| {
                if s == Status::INVALID_PARAMETER {
                    unsafe { Some(index.assume_init()) }
                } else {
                    None
                }
            },
        )
    }

    /// Sets the trigger for `EventType::TIMER` event.
    pub fn set_timer(&self, event: Event, trigger_time: TimerTrigger) -> Result {
        let (ty, time) = match trigger_time {
            TimerTrigger::Cancel => (0, 0),
            TimerTrigger::Periodic(hundreds_ns) => (1, hundreds_ns),
            TimerTrigger::Relative(hundreds_ns) => (2, hundreds_ns),
        };
        unsafe { (self.set_timer)(event, ty, time) }.into()
    }

    /// Query a handle for a certain protocol.
    ///
    /// This function attempts to get the protocol implementation of a handle,
    /// based on the protocol GUID.
    ///
    /// UEFI protocols are neither thread-safe nor reentrant, but the firmware
    /// provides no mechanism to protect against concurrent usage. Such
    /// protections must be implemented by user-level code, for example via a
    /// global `HashSet`.
    pub fn handle_protocol<P: Protocol>(&self, handle: Handle) -> Result<&UnsafeCell<P>> {
        let mut ptr = ptr::null_mut();
        (self.handle_protocol)(handle, &P::GUID, &mut ptr).into_with_val(|| {
            let ptr = ptr as *mut P as *mut UnsafeCell<P>;
            unsafe { &*ptr }
        })
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

        const NULL_BUFFER: *mut Handle = ptr::null_mut();

        let (mut buffer_size, buffer) = match output {
            Some(buffer) => (buffer.len() * handle_size, buffer.as_mut_ptr()),
            None => (0, NULL_BUFFER),
        };

        // Obtain the needed data from the parameters.
        let (ty, guid, key) = match search_ty {
            SearchType::AllHandles => (0, ptr::null(), ptr::null_mut()),
            SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null_mut()),
        };

        let status = unsafe { (self.locate_handle)(ty, guid, key, &mut buffer_size, buffer) };

        // Must convert the returned size (in bytes) to length (number of elements).
        let buffer_len = buffer_size / handle_size;

        match (buffer, status) {
            (NULL_BUFFER, Status::BUFFER_TOO_SMALL) => Ok(buffer_len.into()),
            (_, other_status) => other_status.into_with_val(|| buffer_len),
        }
    }

    /// Locates the handle to a device on the device path that supports the specified protocol.
    pub fn locate_device_path<P: Protocol>(&self, device_path: &mut DevicePath) -> Result<Handle> {
        unsafe {
            let mut handle = Handle::uninitialized();
            let mut device_path_ptr = device_path as *mut DevicePath;
            (self.locate_device_path)(&P::GUID, &mut device_path_ptr, &mut handle)
                .into_with_val(|| handle)
        }
    }

    /// Load an EFI image from a buffer.
    pub fn load_image_from_buffer(
        &self,
        parent_image_handle: Handle,
        source_buffer: &[u8],
    ) -> Result<Handle> {
        unsafe {
            let boot_policy = 0;
            let device_path = ptr::null();
            let source_size = source_buffer.len();
            let mut image_handle = Handle::uninitialized();
            (self.load_image)(
                boot_policy,
                parent_image_handle,
                device_path,
                source_buffer.as_ptr(),
                source_size,
                &mut image_handle,
            )
            .into_with_val(|| image_handle)
        }
    }

    /// Unload an EFI image.
    pub fn unload_image(&self, image_handle: Handle) -> Result {
        (self.unload_image)(image_handle).into()
    }

    /// Transfer control to a loaded image's entry point.
    pub fn start_image(&self, image_handle: Handle) -> Result {
        unsafe {
            // TODO: implement returning exit data to the caller.
            let mut exit_data_size: usize = 0;
            let mut exit_data: *mut Char16 = ptr::null_mut();
            (self.start_image)(image_handle, &mut exit_data_size, &mut exit_data).into()
        }
    }

    /// Exits the UEFI boot services
    ///
    /// This unsafe method is meant to be an implementation detail of the safe
    /// `SystemTable<Boot>::exit_boot_services()` method, which is why it is not
    /// public.
    ///
    /// Everything that is explained in the documentation of the high-level
    /// `SystemTable<Boot>` method is also true here, except that this function
    /// is one-shot (no automatic retry) and does not prevent you from shooting
    /// yourself in the foot by calling invalid boot services after a failure.
    pub(super) unsafe fn exit_boot_services(
        &self,
        image: Handle,
        mmap_key: MemoryMapKey,
    ) -> Result {
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
    /// firmware use. Higher values can be used freely by applications.
    ///
    /// If provided, the watchdog data must be a null-terminated string
    /// optionally followed by other binary data.
    pub fn set_watchdog_timer(
        &self,
        timeout: usize,
        watchdog_code: u64,
        data: Option<&mut [u16]>,
    ) -> Result {
        assert!(
            watchdog_code > 0xffff,
            "Invalid use of a reserved firmware watchdog code"
        );

        let (data_len, data) = data
            .map(|d| {
                assert!(
                    d.contains(&0),
                    "Watchdog data must start with a null-terminated string"
                );
                (d.len(), d.as_mut_ptr())
            })
            .unwrap_or((0, ptr::null_mut()));

        unsafe { (self.set_watchdog_timer)(timeout, watchdog_code, data_len, data) }.into()
    }

    /// Get the list of protocol interface [`Guids`][Guid] that are installed
    /// on a [`Handle`].
    pub fn protocols_per_handle(&self, handle: Handle) -> Result<ProtocolsPerHandle> {
        let mut protocols = ptr::null_mut();
        let mut count = 0;

        let mut status = unsafe { (self.protocols_per_handle)(handle, &mut protocols, &mut count) };

        if !status.is_error() {
            // Ensure that protocols isn't null, and that none of the GUIDs
            // returned are null.
            if protocols.is_null() {
                status = Status::OUT_OF_RESOURCES;
            } else {
                let protocols: &[*const Guid] = unsafe { slice::from_raw_parts(protocols, count) };
                if protocols.iter().any(|ptr| ptr.is_null()) {
                    status = Status::OUT_OF_RESOURCES;
                }
            }
        }

        status.into_with_val(|| {
            let protocols = unsafe { slice::from_raw_parts_mut(protocols as *mut &Guid, count) };
            ProtocolsPerHandle {
                boot_services: self,
                protocols,
            }
        })
    }

    /// Returns a protocol implementation, if present on the system.
    ///
    /// The caveats of `BootServices::handle_protocol()` also apply here.
    pub fn locate_protocol<P: Protocol>(&self) -> Result<&UnsafeCell<P>> {
        let mut ptr = ptr::null_mut();
        (self.locate_protocol)(&P::GUID, ptr::null_mut(), &mut ptr).into_with_val(|| {
            let ptr = ptr as *mut P as *mut UnsafeCell<P>;
            unsafe { &*ptr }
        })
    }

    /// Copies memory from source to destination. The buffers can overlap.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it can be used to violate most safety
    /// invariants of the Rust type system.
    pub unsafe fn memmove(&self, dest: *mut u8, src: *const u8, size: usize) {
        (self.copy_mem)(dest, src, size);
    }

    /// Sets a buffer to a certain value.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it can be used to violate most safety
    /// invariants of the Rust type system.
    pub unsafe fn set_mem(&self, buffer: *mut u8, size: usize, value: u8) {
        (self.set_mem)(buffer, size, value);
    }
}

#[cfg(feature = "exts")]
impl BootServices {
    /// Returns all the handles implementing a certain protocol.
    pub fn find_handles<P: Protocol>(&self) -> Result<Vec<Handle>> {
        // Search by protocol.
        let search_type = SearchType::from_proto::<P>();

        // Determine how much we need to allocate.
        let (status1, buffer_size) = self.locate_handle(search_type, None)?.split();

        // Allocate a large enough buffer.
        let mut buffer = Vec::with_capacity(buffer_size);

        unsafe {
            buffer.set_len(buffer_size);
        }

        // Perform the search.
        let (status2, buffer_size) = self.locate_handle(search_type, Some(&mut buffer))?.split();

        // Once the vector has been filled, update its size.
        unsafe {
            buffer.set_len(buffer_size);
        }

        // Emit output, with warnings
        status1
            .into_with_val(|| buffer)
            .map(|completion| completion.with_status(status2))
    }

    /// Retrieves the `SimpleFileSystem` protocol associated with
    /// the device the given image was loaded from.
    ///
    /// You can retrieve the SFS protocol associated with the boot partition
    /// by passing the image handle received by the UEFI entry point to this function.
    pub fn get_image_file_system(
        &self,
        image_handle: Handle,
    ) -> Result<&UnsafeCell<SimpleFileSystem>> {
        let loaded_image = self
            .handle_protocol::<LoadedImage>(image_handle)?
            .expect("Failed to retrieve `LoadedImage` protocol from handle");
        let loaded_image = unsafe { &*loaded_image.get() };

        let device_handle = loaded_image.device();

        let device_path = self
            .handle_protocol::<DevicePath>(device_handle)?
            .expect("Failed to retrieve `DevicePath` protocol from image's device handle");
        let device_path = unsafe { &mut *device_path.get() };

        let device_handle = self
            .locate_device_path::<SimpleFileSystem>(device_path)?
            .expect("Failed to locate `SimpleFileSystem` protocol on device path");

        self.handle_protocol::<SimpleFileSystem>(device_handle)
    }
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
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

/// RAII guard for task priority level changes
///
/// Will automatically restore the former task priority level when dropped.
pub struct TplGuard<'boot> {
    boot_services: &'boot BootServices,
    old_tpl: Tpl,
}

impl Drop for TplGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.boot_services.restore_tpl)(self.old_tpl);
        }
    }
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

impl MemoryType {
    /// Construct a custom `MemoryType`. Values in the range `0x80000000..=0xffffffff` are free for use if you are
    /// an OS loader.
    pub const fn custom(value: u32) -> MemoryType {
        assert!(value >= 0x80000000);
        MemoryType(value)
    }
}

/// Memory descriptor version number
pub const MEMORY_DESCRIPTOR_VERSION: u32 = 1;

/// A structure describing a region of memory.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
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

impl Align for MemoryDescriptor {
    fn alignment() -> usize {
        mem::align_of::<Self>()
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

/// An iterator of memory descriptors
#[derive(Debug, Clone)]
struct MemoryMapIter<'buf> {
    buffer: &'buf [u8],
    entry_size: usize,
    index: usize,
    len: usize,
}

impl<'buf> Iterator for MemoryMapIter<'buf> {
    type Item = &'buf MemoryDescriptor;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.len - self.index;

        (sz, Some(sz))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let ptr = self.buffer.as_ptr() as usize + self.entry_size * self.index;

            self.index += 1;

            let descriptor = unsafe { &*(ptr as *const MemoryDescriptor) };

            Some(descriptor)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {}

/// The type of handle search to perform.
#[derive(Debug, Copy, Clone)]
pub enum SearchType<'guid> {
    /// Return all handles present on the system.
    AllHandles,
    /// Returns all handles supporting a certain protocol, specified by its GUID.
    ///
    /// If the protocol implements the `Protocol` interface,
    /// you can use the `from_proto` function to construct a new `SearchType`.
    ByProtocol(&'guid Guid),
    // TODO: add ByRegisterNotify once the corresponding function is implemented.
}

impl<'guid> SearchType<'guid> {
    /// Constructs a new search type for a specified protocol.
    pub fn from_proto<P: Protocol>() -> Self {
        SearchType::ByProtocol(&P::GUID)
    }
}

bitflags! {
    /// Flags describing the type of an UEFI event and its attributes.
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

/// Raw event notification function
type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: *mut c_void);

/// Timer events manipulation
pub enum TimerTrigger {
    /// Cancel event's timer
    Cancel,
    /// The event is to be signaled periodically.
    /// Parameter is the period in 100ns units.
    /// Delay of 0 will be signalled on every timer tick.
    Periodic(u64),
    /// The event is to be signaled once in 100ns units.
    /// Parameter is the delay in 100ns units.
    /// Delay of 0 will be signalled on next timer tick.
    Relative(u64),
}

/// Protocol interface [`Guids`][Guid] that are installed on a [`Handle`] as
/// returned by [`BootServices::protocols_per_handle`].
pub struct ProtocolsPerHandle<'a> {
    // The pointer returned by `protocols_per_handle` has to be free'd with
    // `free_pool`, so keep a reference to boot services for that purpose.
    boot_services: &'a BootServices,

    // This is mutable so that it can later be free'd with `free_pool`. Users
    // should only get an immutable reference though, so the field is not
    // public.
    protocols: &'a mut [&'a Guid],
}

impl<'a> Drop for ProtocolsPerHandle<'a> {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = self
            .boot_services
            .free_pool(self.protocols.as_mut_ptr() as *mut u8);
    }
}

impl<'a> ProtocolsPerHandle<'a> {
    /// Get the protocol interface [`Guids`][Guid] that are installed on the
    /// [`Handle`].
    pub fn protocols(&self) -> &[&Guid] {
        self.protocols
    }
}
