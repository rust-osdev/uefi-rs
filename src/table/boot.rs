//! UEFI services available during boot.

use super::{Header, Revision};
use crate::data_types::{Align, PhysicalAddress, VirtualAddress};
use crate::proto::device_path::{DevicePath, FfiDevicePath};
#[cfg(feature = "exts")]
use crate::proto::{loaded_image::LoadedImage, media::fs::SimpleFileSystem};
use crate::proto::{Protocol, ProtocolPointer};
use crate::{Char16, Event, Guid, Handle, Result, Status};
#[cfg(feature = "exts")]
use alloc_api::vec::Vec;
use bitflags::bitflags;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{ptr, slice};

// TODO: this similar to `SyncUnsafeCell`. Once that is stabilized we
// can use it instead.
struct GlobalImageHandle {
    handle: UnsafeCell<Option<Handle>>,
}

// Safety: reads and writes are managed via `set_image_handle` and
// `BootServices::image_handle`.
unsafe impl Sync for GlobalImageHandle {}

static IMAGE_HANDLE: GlobalImageHandle = GlobalImageHandle {
    handle: UnsafeCell::new(None),
};

/// Contains pointers to all of the boot services.
///
/// # Accessing `BootServices`
///
/// A reference to `BootServices` can only be accessed by calling [`SystemTable::boot_services`].
///
/// [`SystemTable::boot_services`]: crate::table::SystemTable::boot_services
///
/// # Accessing protocols
///
/// Protocols can be opened using several methods of `BootServices`. Most
/// commonly, [`open_protocol_exclusive`] should be used. This ensures that
/// nothing else can use the protocol until it is closed, and returns a
/// [`ScopedProtocol`] that takes care of closing the protocol when it is
/// dropped.
///
/// Other methods for opening protocols:
///
/// * [`open_protocol`]
/// * [`get_image_file_system`]
/// * [`handle_protocol`]
/// * [`locate_protocol`]
///
/// For protocol definitions, see the [`proto`] module.
///
/// [`proto`]: crate::proto
/// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
/// [`open_protocol`]: BootServices::open_protocol
/// [`get_image_file_system`]: BootServices::get_image_file_system
/// [`locate_protocol`]: BootServices::locate_protocol
/// [`handle_protocol`]: BootServices::handle_protocol
///
/// ## Use of [`UnsafeCell`] for protocol references
///
/// Some protocols require mutable access to themselves. For example,
/// most of the methods of the [`Output`] protocol take `&mut self`,
/// because the internal function pointers specified by UEFI for that
/// protocol take a mutable `*This` pointer. We don't want to directly
/// return a mutable reference to a protocol though because the lifetime
/// of the protocol is tied to `BootServices`. (That lifetime improves
/// safety by ensuring protocols aren't accessed after exiting boot
/// services.) If methods like [`open_protocol`] protocol took a mutable
/// reference to `BootServices` and returned a mutable reference to a
/// protocol it would prevent all other access to `BootServices` until
/// the protocol reference was dropped. To work around this, the
/// protocol reference is wrapped in an [`UnsafeCell`]. Callers can then
/// get a mutable reference to the protocol if needed.
///
/// [`Output`]: crate::proto::console::text::Output
/// [`open_protocol`]: BootServices::open_protocol
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
        addr: &mut PhysicalAddress,
    ) -> Status,
    free_pages: extern "efiapi" fn(addr: PhysicalAddress, pages: usize) -> Status,
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
        notify_ctx: Option<NonNull<c_void>>,
        out_event: *mut Event,
    ) -> Status,
    set_timer: unsafe extern "efiapi" fn(event: Event, ty: u32, trigger_time: u64) -> Status,
    wait_for_event: unsafe extern "efiapi" fn(
        number_of_events: usize,
        events: *mut Event,
        out_index: *mut usize,
    ) -> Status,
    signal_event: extern "efiapi" fn(event: Event) -> Status,
    close_event: unsafe extern "efiapi" fn(event: Event) -> Status,
    check_event: unsafe extern "efiapi" fn(event: Event) -> Status,

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
        buf: *mut MaybeUninit<Handle>,
    ) -> Status,
    locate_device_path: unsafe extern "efiapi" fn(
        proto: &Guid,
        device_path: &mut *const FfiDevicePath,
        out_handle: &mut MaybeUninit<Handle>,
    ) -> Status,
    install_configuration_table: usize,

    // Image services
    load_image: unsafe extern "efiapi" fn(
        boot_policy: u8,
        parent_image_handle: Handle,
        device_path: *const FfiDevicePath,
        source_buffer: *const u8,
        source_size: usize,
        image_handle: &mut MaybeUninit<Handle>,
    ) -> Status,
    start_image: unsafe extern "efiapi" fn(
        image_handle: Handle,
        exit_data_size: *mut usize,
        exit_data: &mut *mut Char16,
    ) -> Status,
    exit: extern "efiapi" fn(
        image_handle: Handle,
        exit_status: Status,
        exit_data_size: usize,
        exit_data: *mut Char16,
    ) -> !,
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
    connect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Option<Handle>,
        remaining_device_path: *const FfiDevicePath,
        recursive: bool,
    ) -> Status,
    disconnect_controller: unsafe extern "efiapi" fn(
        controller: Handle,
        driver_image: Option<Handle>,
        child: Option<Handle>,
    ) -> Status,

    // Protocol open / close services
    open_protocol: extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        interface: &mut *mut c_void,
        agent_handle: Handle,
        controller_handle: Option<Handle>,
        attributes: u32,
    ) -> Status,
    close_protocol: extern "efiapi" fn(
        handle: Handle,
        protocol: &Guid,
        agent_handle: Handle,
        controller_handle: Option<Handle>,
    ) -> Status,
    open_protocol_information: usize,

    // Library services
    protocols_per_handle: unsafe extern "efiapi" fn(
        handle: Handle,
        protocol_buffer: *mut *mut *const Guid,
        protocol_buffer_count: *mut usize,
    ) -> Status,
    locate_handle_buffer: unsafe extern "efiapi" fn(
        search_ty: i32,
        proto: *const Guid,
        key: *const c_void,
        no_handles: &mut usize,
        buf: &mut *mut Handle,
    ) -> Status,
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
    create_event_ex: unsafe extern "efiapi" fn(
        ty: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
        event_group: Option<NonNull<Guid>>,
        out_event: *mut Event,
    ) -> Status,
}

impl BootServices {
    /// Get the [`Handle`] of the currently-executing image.
    pub fn image_handle(&self) -> Handle {
        // Safety:
        //
        // `IMAGE_HANDLE` is only set by `set_image_handle`, see that
        // documentation for more details.
        //
        // Additionally, `image_handle` takes a `&self` which ensures it
        // can only be called while boot services are active. (After
        // exiting boot services, the image handle should not be
        // considered valid.)
        unsafe {
            IMAGE_HANDLE
                .handle
                .get()
                .read()
                .expect("set_image_handle has not been called")
        }
    }

    /// Update the global image [`Handle`].
    ///
    /// This is called automatically in the `main` entry point as part
    /// of [`uefi_macros::entry`]. It should not be called at any other
    /// point in time, unless the executable does not use
    /// [`uefi_macros::entry`], in which case it should be called once
    /// before calling other `BootServices` functions.
    ///
    /// # Safety
    ///
    /// This function should only be called as described above. The
    /// safety guarantees of [`BootServices::open_protocol_exclusive`]
    /// rely on the global image handle being correct.
    pub unsafe fn set_image_handle(&self, image_handle: Handle) {
        // As with `image_handle`, `&self` isn't actually used, but it
        // enforces that this function is only called while boot
        // services are active.
        IMAGE_HANDLE.handle.get().write(Some(image_handle));
    }

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
    ) -> Result<PhysicalAddress> {
        let (ty, mut addr) = match ty {
            AllocateType::AnyPages => (0, 0),
            AllocateType::MaxAddress(addr) => (1, addr),
            AllocateType::Address(addr) => (2, addr),
        };
        (self.allocate_pages)(ty, mem_ty, count, &mut addr).into_with_val(|| addr)
    }

    /// Frees memory pages allocated by UEFI.
    pub fn free_pages(&self, addr: PhysicalAddress, count: usize) -> Result {
        (self.free_pages)(addr, count).into()
    }

    /// Returns struct which contains the size of a single memory descriptor
    /// as well as the size of the current memory map.
    ///
    /// Note that the size of the memory map can increase any time an allocation happens,
    /// so when creating a buffer to put the memory map into, it's recommended to allocate a few extra
    /// elements worth of space above the size of the current memory map.
    pub fn memory_map_size(&self) -> MemoryMapSize {
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

        MemoryMapSize {
            entry_size,
            map_size,
        }
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
        let map_buffer = buffer.as_mut_ptr().cast::<MemoryDescriptor>();
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
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
    ) -> Result<Event> {
        // Prepare storage for the output Event
        let mut event = MaybeUninit::<Event>::uninit();

        // Now we're ready to call UEFI
        (self.create_event)(
            event_ty,
            notify_tpl,
            notify_fn,
            notify_ctx,
            event.as_mut_ptr(),
        )
        .into_with_val(|| event.assume_init())
    }

    /// Creates a new `Event` of type `event_type`. The event's notification function, context,
    /// and task priority are specified by `notify_fn`, `notify_ctx`, and `notify_tpl`, respectively.
    /// The `Event` will be added to the group of `Event`s identified by `event_group`.
    ///
    /// If no group is specified by `event_group`, this function behaves as if the same parameters
    /// had been passed to `create_event()`.
    ///
    /// Event groups are collections of events identified by a shared `Guid` where, when one member
    /// event is signaled, all other events are signaled and their individual notification actions
    /// are taken. All events are guaranteed to be signaled before the first notification action is
    /// taken. All notification functions will be executed in the order specified by their `Tpl`.
    ///
    /// A single event can only be part of a single event group. An event may be removed from an
    /// event group by using `close_event()`.
    ///
    /// The `EventType` of an event uses the same values as `create_event()`, except that
    /// `EventType::SIGNAL_EXIT_BOOT_SERVICES` and `EventType::SIGNAL_VIRTUAL_ADDRESS_CHANGE`
    /// are not valid.
    ///
    /// If `event_type` has `EventType::NOTIFY_SIGNAL` or `EventType::NOTIFY_WAIT`, then `notify_fn`
    /// mus be `Some` and `notify_tpl` must be a valid task priority level, otherwise these parameters
    /// are ignored.
    ///
    /// More than one event of type `EventType::TIMER` may be part of a single event group. However,
    /// there is no mechanism for determining which of the timers was signaled.
    ///
    /// This operation is only supported starting with UEFI 2.0; earlier
    /// versions will fail with [`Status::UNSUPPORTED`].
    ///
    /// # Safety
    ///
    /// The caller must ensure they are passing a valid `Guid` as `event_group`, if applicable.
    pub unsafe fn create_event_ex(
        &self,
        event_type: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
        event_group: Option<NonNull<Guid>>,
    ) -> Result<Event> {
        if self.header.revision < Revision::EFI_2_00 {
            return Err(Status::UNSUPPORTED.into());
        }

        let mut event = MaybeUninit::<Event>::uninit();

        (self.create_event_ex)(
            event_type,
            notify_tpl,
            notify_fn,
            notify_ctx,
            event_group,
            event.as_mut_ptr(),
        )
        .into_with_val(|| event.assume_init())
    }

    /// Sets the trigger for `EventType::TIMER` event.
    pub fn set_timer(&self, event: &Event, trigger_time: TimerTrigger) -> Result {
        let (ty, time) = match trigger_time {
            TimerTrigger::Cancel => (0, 0),
            TimerTrigger::Periodic(hundreds_ns) => (1, hundreds_ns),
            TimerTrigger::Relative(hundreds_ns) => (2, hundreds_ns),
        };
        unsafe { (self.set_timer)(event.unsafe_clone(), ty, time) }.into()
    }

    /// Stops execution until an event is signaled.
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

    /// Place 'event' in the signaled stated. If 'event' is already in the signaled state,
    /// then nothing further occurs and `Status::SUCCESS` is returned. If `event` is of type
    /// `EventType::NOTIFY_SIGNAL`, then the event's notification function is scheduled to
    /// be invoked at the event's notification task priority level.
    ///
    /// This function may be invoked from any task priority level.
    ///
    /// If `event` is part of an event group, then all of the events in the event group are
    /// also signaled and their notification functions are scheduled.
    ///
    /// When signaling an event group, it is possible to create an event in the group, signal
    /// it, and then close the event to remove it from the group.
    pub fn signal_event(&self, event: &Event) -> Result {
        // Safety: cloning this event should be safe, as we're directly passing it to firmware
        // and not keeping the clone around.
        unsafe { (self.signal_event)(event.unsafe_clone()).into() }
    }

    /// Removes `event` from any event group to which it belongs and closes it. If `event` was
    /// registered with `register_protocol_notify()`, then the corresponding registration will
    /// be removed. It is safe to call this function within the corresponding notify function.
    ///
    ///
    /// Note: The UEFI Specification v2.9 states that this may only return `EFI_SUCCESS`, but,
    /// at least for application based on EDK2 (such as OVMF), it may also return `EFI_INVALID_PARAMETER`.
    pub fn close_event(&self, event: Event) -> Result {
        unsafe { (self.close_event)(event).into() }
    }

    /// Checks to see if an event is signaled, without blocking execution to wait for it.
    ///
    /// The returned value will be `true` if the event is in the signaled state,
    /// otherwise `false` is returned.
    pub fn check_event(&self, event: Event) -> Result<bool> {
        let status = unsafe { (self.check_event)(event) };
        match status {
            Status::SUCCESS => Ok(true),
            Status::NOT_READY => Ok(false),
            _ => Err(status.into()),
        }
    }

    /// Query a handle for a certain protocol.
    ///
    /// This function attempts to get the protocol implementation of a handle,
    /// based on the protocol GUID.
    ///
    /// It is recommended that all new drivers and applications use
    /// [`open_protocol_exclusive`] or [`open_protocol`] instead of `handle_protocol`.
    ///
    /// UEFI protocols are neither thread-safe nor reentrant, but the firmware
    /// provides no mechanism to protect against concurrent usage. Such
    /// protections must be implemented by user-level code, for example via a
    /// global `HashSet`.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the handle database is not
    /// notified that the handle and protocol are in use; there is no
    /// guarantee that they will remain valid for the duration of their
    /// use. Use [`open_protocol_exclusive`] if possible, otherwise use
    /// [`open_protocol`].
    ///
    /// [`open_protocol`]: BootServices::open_protocol
    /// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
    #[deprecated(
        note = "it is recommended to use `open_protocol_exclusive` or `open_protocol` instead"
    )]
    pub unsafe fn handle_protocol<P: ProtocolPointer + ?Sized>(
        &self,
        handle: Handle,
    ) -> Result<&UnsafeCell<P>> {
        let mut ptr = ptr::null_mut();
        (self.handle_protocol)(handle, &P::GUID, &mut ptr).into_with_val(|| {
            let ptr = P::mut_ptr_from_ffi(ptr) as *const UnsafeCell<P>;
            &*ptr
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
        output: Option<&mut [MaybeUninit<Handle>]>,
    ) -> Result<usize> {
        let handle_size = mem::size_of::<Handle>();

        const NULL_BUFFER: *mut MaybeUninit<Handle> = ptr::null_mut();

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
            (NULL_BUFFER, Status::BUFFER_TOO_SMALL) => Ok(buffer_len),
            (_, other_status) => other_status.into_with_val(|| buffer_len),
        }
    }

    /// Locates the handle to a device on the device path that supports the specified protocol.
    ///
    /// The `device_path` is updated to point at the remaining part of the [`DevicePath`] after
    /// the part that matched the protocol. For example, it can be used with a device path
    /// that contains a file path to strip off the file system portion of the device path,
    /// leaving the file path and handle to the file system driver needed to access the file.
    ///
    /// If the first node of `device_path` matches the
    /// protocol, the `device_path` is advanced to the device path terminator node. If `device_path`
    /// is a multi-instance device path, the function will operate on the first instance.
    pub fn locate_device_path<P: Protocol>(&self, device_path: &mut &DevicePath) -> Result<Handle> {
        let mut handle = MaybeUninit::uninit();
        let mut device_path_ptr = device_path.as_ffi_ptr();
        unsafe {
            (self.locate_device_path)(&P::GUID, &mut device_path_ptr, &mut handle).into_with_val(
                || {
                    *device_path = DevicePath::from_ffi_ptr(device_path_ptr);
                    handle.assume_init()
                },
            )
        }
    }

    /// Find an arbitrary handle that supports a particular
    /// [`Protocol`]. Returns [`NOT_FOUND`] if no handles support the
    /// protocol.
    ///
    /// This method is a convenient wrapper around
    /// [`BootServices::locate_handle_buffer`] for getting just one
    /// handle. This is useful when you don't care which handle the
    /// protocol is opened on. For example, [`DevicePathToText`] isn't
    /// tied to a particular device, so only a single handle is expected
    /// to exist.
    ///
    /// [`NOT_FOUND`]: Status::NOT_FOUND
    /// [`DevicePathToText`]: uefi::proto::device_path::text::DevicePathToText
    ///
    /// # Example
    ///
    /// ```
    /// use uefi::proto::device_path::text::DevicePathToText;
    /// use uefi::table::boot::{BootServices, OpenProtocolAttributes, OpenProtocolParams};
    /// use uefi::Handle;
    /// # use uefi::Result;
    ///
    /// # fn get_fake_val<T>() -> T { todo!() }
    /// # fn test() -> Result {
    /// # let boot_services: &BootServices = get_fake_val();
    /// # let image_handle: Handle = get_fake_val();
    /// let handle = boot_services.get_handle_for_protocol::<DevicePathToText>()?;
    /// let device_path_to_text = boot_services.open_protocol_exclusive::<DevicePathToText>(handle)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_handle_for_protocol<P: Protocol>(&self) -> Result<Handle> {
        // Delegate to a non-generic function to potentially reduce code size.
        self.get_handle_for_protocol_impl(&P::GUID)
    }

    fn get_handle_for_protocol_impl(&self, guid: &Guid) -> Result<Handle> {
        self.locate_handle_buffer(SearchType::ByProtocol(guid))?
            .handles()
            .first()
            .cloned()
            .ok_or_else(|| Status::NOT_FOUND.into())
    }

    /// Load an EFI image into memory and return a [`Handle`] to the image.
    ///
    /// There are two ways to load the image: by copying raw image data
    /// from a source buffer, or by loading the image via the
    /// [`SimpleFileSystem`] protocol. See [`LoadImageSource`] for more
    /// details of the `source` parameter.
    ///
    /// The `parent_image_handle` is used to initialize the
    /// `parent_handle` field of the [`LoadedImage`] protocol for the
    /// image.
    ///
    /// If the image is successfully loaded, a [`Handle`] supporting the
    /// [`LoadedImage`] and `LoadedImageDevicePath` protocols is
    /// returned. The image can be started with [`start_image`] or
    /// unloaded with [`unload_image`].
    ///
    /// [`start_image`]: BootServices::start_image
    /// [`unload_image`]: BootServices::unload_image
    pub fn load_image(
        &self,
        parent_image_handle: Handle,
        source: LoadImageSource,
    ) -> uefi::Result<Handle> {
        let boot_policy;
        let device_path;
        let source_buffer;
        let source_size;
        match source {
            LoadImageSource::FromBuffer { buffer, file_path } => {
                // Boot policy is ignored when loading from source buffer.
                boot_policy = 0;

                device_path = file_path.map(|p| p.as_ffi_ptr()).unwrap_or(ptr::null());
                source_buffer = buffer.as_ptr();
                source_size = buffer.len();
            }
            LoadImageSource::FromFilePath {
                file_path,
                from_boot_manager,
            } => {
                boot_policy = u8::from(from_boot_manager);
                device_path = file_path.as_ffi_ptr();
                source_buffer = ptr::null();
                source_size = 0;
            }
        };

        let mut image_handle = MaybeUninit::uninit();
        unsafe {
            (self.load_image)(
                boot_policy,
                parent_image_handle,
                device_path,
                source_buffer,
                source_size,
                &mut image_handle,
            )
            .into_with_val(|| image_handle.assume_init())
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

    /// Exits the UEFI application and returns control to the UEFI component
    /// that started the UEFI application.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it is up to the caller to ensure that
    /// all resources allocated by the application is freed before invoking
    /// exit and returning control to the UEFI component that started the UEFI
    /// application.
    pub unsafe fn exit(
        &self,
        image_handle: Handle,
        exit_status: Status,
        exit_data_size: usize,
        exit_data: *mut Char16,
    ) -> ! {
        (self.exit)(image_handle, exit_status, exit_data_size, exit_data)
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

    /// Connect one or more drivers to a controller.
    ///
    /// Usually one disconnects and then reconnects certain drivers
    /// to make them rescan some state that changed, e.g. reconnecting
    /// a `BlockIO` handle after your app changed the partitions somehow.
    pub fn connect_controller(
        &self,
        controller: Handle,
        driver_image: Option<Handle>,
        remaining_device_path: Option<&DevicePath>,
        recursive: bool,
    ) -> Result {
        unsafe {
            (self.connect_controller)(
                controller,
                driver_image,
                remaining_device_path
                    .map(|dp| dp.as_ffi_ptr())
                    .unwrap_or(ptr::null()),
                recursive,
            )
        }
        .into_with_err(|_| ())
    }

    /// Disconnect one or more drivers from a controller.
    ///
    /// See [`connect_controller`][Self::connect_controller].
    pub fn disconnect_controller(
        &self,
        controller: Handle,
        driver_image: Option<Handle>,
        child: Option<Handle>,
    ) -> Result {
        unsafe { (self.disconnect_controller)(controller, driver_image, child) }
            .into_with_err(|_| ())
    }

    /// Open a protocol interface for a handle.
    ///
    /// See also [`open_protocol_exclusive`], which provides a safe
    /// subset of this functionality.
    ///
    /// This function attempts to get the protocol implementation of a
    /// handle, based on the protocol GUID. It is an extended version of
    /// [`handle_protocol`]. It is recommended that all
    /// new drivers and applications use `open_protocol_exclusive` or
    /// `open_protocol` instead of `handle_protocol`.
    ///
    /// See [`OpenProtocolParams`] and [`OpenProtocolAttributes`] for
    /// details of the input parameters.
    ///
    /// If successful, a [`ScopedProtocol`] is returned that will
    /// automatically close the protocol interface when dropped.
    ///
    /// UEFI protocols are neither thread-safe nor reentrant, but the firmware
    /// provides no mechanism to protect against concurrent usage. Such
    /// protections must be implemented by user-level code, for example via a
    /// global `HashSet`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it can be used to open a
    /// protocol in ways that don't get tracked by the UEFI
    /// implementation. This could allow the protocol to be removed from
    /// a handle, or for the handle to be deleted entirely, while a
    /// reference to the protocol is still active. The caller is
    /// responsible for ensuring that the handle and protocol remain
    /// valid until the `ScopedProtocol` is dropped.
    ///
    /// [`handle_protocol`]: BootServices::handle_protocol
    /// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
    pub unsafe fn open_protocol<P: ProtocolPointer + ?Sized>(
        &self,
        params: OpenProtocolParams,
        attributes: OpenProtocolAttributes,
    ) -> Result<ScopedProtocol<P>> {
        let mut interface = ptr::null_mut();
        (self.open_protocol)(
            params.handle,
            &P::GUID,
            &mut interface,
            params.agent,
            params.controller,
            attributes as u32,
        )
        .into_with_val(|| {
            let interface = P::mut_ptr_from_ffi(interface) as *const UnsafeCell<P>;

            #[allow(deprecated)]
            ScopedProtocol {
                interface: &*interface,
                open_params: params,
                boot_services: self,
            }
        })
    }

    /// Open a protocol interface for a handle in exclusive mode.
    ///
    /// If successful, a [`ScopedProtocol`] is returned that will
    /// automatically close the protocol interface when dropped.
    ///
    /// [`handle_protocol`]: BootServices::handle_protocol
    pub fn open_protocol_exclusive<P: ProtocolPointer + ?Sized>(
        &self,
        handle: Handle,
    ) -> Result<ScopedProtocol<P>> {
        // Safety: opening in exclusive mode with the correct agent
        // handle set ensures that the protocol cannot be modified or
        // removed while it is open, so this usage is safe.
        unsafe {
            self.open_protocol::<P>(
                OpenProtocolParams {
                    handle,
                    agent: self.image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::Exclusive,
            )
        }
    }

    /// Test whether a handle supports a protocol.
    pub fn test_protocol<P: Protocol>(&self, params: OpenProtocolParams) -> Result<()> {
        const TEST_PROTOCOL: u32 = 0x04;
        let mut interface = ptr::null_mut();
        (self.open_protocol)(
            params.handle,
            &P::GUID,
            &mut interface,
            params.agent,
            params.controller,
            TEST_PROTOCOL,
        )
        .into_with_val(|| ())
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

        status.into_with_val(|| ProtocolsPerHandle {
            boot_services: self,
            protocols: protocols.cast::<&Guid>(),
            count,
        })
    }

    /// Returns an array of handles that support the requested protocol in a buffer allocated from
    /// pool.
    pub fn locate_handle_buffer(&self, search_ty: SearchType) -> Result<HandleBuffer> {
        let mut num_handles: usize = 0;
        let mut buffer: *mut Handle = ptr::null_mut();

        // Obtain the needed data from the parameters.
        let (ty, guid, key) = match search_ty {
            SearchType::AllHandles => (0, ptr::null(), ptr::null_mut()),
            SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null_mut()),
        };

        unsafe { (self.locate_handle_buffer)(ty, guid, key, &mut num_handles, &mut buffer) }
            .into_with_val(|| HandleBuffer {
                boot_services: self,
                count: num_handles,
                buffer,
            })
    }

    /// Returns a protocol implementation, if present on the system.
    ///
    /// The caveats of `BootServices::handle_protocol()` also apply here.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the handle database is not
    /// notified that the handle and protocol are in use; there is no
    /// guarantee that they will remain valid for the duration of their
    /// use. Use [`get_handle_for_protocol`] and either
    /// [`open_protocol_exclusive`] or [`open_protocol`] instead.
    ///
    /// [`get_handle_for_protocol`]: BootServices::get_handle_for_protocol
    /// [`open_protocol`]: BootServices::open_protocol
    /// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
    #[deprecated(
        note = "it is recommended to use `open_protocol_exclusive` or `open_protocol` instead"
    )]
    pub unsafe fn locate_protocol<P: ProtocolPointer + ?Sized>(&self) -> Result<&UnsafeCell<P>> {
        let mut ptr = ptr::null_mut();
        (self.locate_protocol)(&P::GUID, ptr::null_mut(), &mut ptr).into_with_val(|| {
            let ptr = P::mut_ptr_from_ffi(ptr) as *const UnsafeCell<P>;
            &*ptr
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
        let buffer_size = self.locate_handle(search_type, None)?;

        // Allocate a large enough buffer without pointless initialization.
        let mut handles = Vec::with_capacity(buffer_size);
        let buffer = handles.spare_capacity_mut();

        // Perform the search.
        let buffer_size = self.locate_handle(search_type, Some(buffer))?;

        // Mark the returned number of elements as initialized.
        unsafe {
            handles.set_len(buffer_size);
        }

        // Emit output, with warnings
        Ok(handles)
    }

    /// Retrieves the `SimpleFileSystem` protocol associated with
    /// the device the given image was loaded from.
    ///
    /// You can retrieve the SFS protocol associated with the boot partition
    /// by passing the image handle received by the UEFI entry point to this function.
    pub fn get_image_file_system(
        &self,
        image_handle: Handle,
    ) -> Result<ScopedProtocol<SimpleFileSystem>> {
        let loaded_image = self.open_protocol_exclusive::<LoadedImage>(image_handle)?;

        let device_path = self.open_protocol_exclusive::<DevicePath>(loaded_image.device())?;

        let device_handle = self.locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

        self.open_protocol_exclusive(device_handle)
    }
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
}

impl Debug for BootServices {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootServices")
            .field("header", &self.header)
            .field("raise_tpl (fn ptr)", &(self.raise_tpl as *const usize))
            .field("restore_tpl (fn ptr)", &(self.restore_tpl as *const usize))
            .field(
                "allocate_pages (fn ptr)",
                &(self.allocate_pages as *const usize),
            )
            .field("free_pages (fn ptr)", &(self.free_pages as *const usize))
            .field(
                "get_memory_map (fn ptr)",
                &(self.get_memory_map as *const usize),
            )
            .field(
                "allocate_pool (fn ptr)",
                &(self.allocate_pool as *const usize),
            )
            .field("free_pool (fn ptr)", &(self.free_pool as *const usize))
            .field(
                "create_event (fn ptr)",
                &(self.create_event as *const usize),
            )
            .field("set_timer (fn ptr)", &(self.set_timer as *const usize))
            .field(
                "wait_for_event (fn ptr)",
                &(self.wait_for_event as *const usize),
            )
            .field("signal_event", &(self.signal_event as *const usize))
            .field("close_event", &(self.close_event as *const usize))
            .field("check_event", &(self.check_event as *const usize))
            .field(
                "install_protocol_interface",
                &(self.install_protocol_interface as *const usize),
            )
            .field(
                "reinstall_protocol_interface",
                &(self.reinstall_protocol_interface as *const usize),
            )
            .field(
                "uninstall_protocol_interface",
                &(self.uninstall_protocol_interface as *const usize),
            )
            .field(
                "handle_protocol (fn ptr)",
                &(self.handle_protocol as *const usize),
            )
            .field(
                "register_protocol_notify",
                &(self.register_protocol_notify as *const usize),
            )
            .field(
                "locate_handle (fn ptr)",
                &(self.locate_handle as *const usize),
            )
            .field(
                "locate_device_path (fn ptr)",
                &(self.locate_device_path as *const usize),
            )
            .field(
                "install_configuration_table",
                &(self.install_configuration_table as *const usize),
            )
            .field("load_image (fn ptr)", &(self.load_image as *const usize))
            .field("start_image (fn ptr)", &(self.start_image as *const usize))
            .field("exit", &(self.exit as *const usize))
            .field(
                "unload_image (fn ptr)",
                &(self.unload_image as *const usize),
            )
            .field(
                "exit_boot_services (fn ptr)",
                &(self.exit_boot_services as *const usize),
            )
            .field(
                "get_next_monotonic_count",
                &(self.get_next_monotonic_count as *const usize),
            )
            .field("stall (fn ptr)", &(self.stall as *const usize))
            .field(
                "set_watchdog_timer (fn ptr)",
                &(self.set_watchdog_timer as *const usize),
            )
            .field(
                "connect_controller",
                &(self.connect_controller as *const usize),
            )
            .field(
                "disconnect_controller",
                &(self.disconnect_controller as *const usize),
            )
            .field("open_protocol", &(self.open_protocol as *const usize))
            .field("close_protocol", &(self.close_protocol as *const usize))
            .field(
                "open_protocol_information",
                &(self.open_protocol_information as *const usize),
            )
            .field(
                "protocols_per_handle",
                &(self.protocols_per_handle as *const usize),
            )
            .field(
                "locate_handle_buffer",
                &(self.locate_handle_buffer as *const usize),
            )
            .field(
                "locate_protocol (fn ptr)",
                &(self.locate_protocol as *const usize),
            )
            .field(
                "install_multiple_protocol_interfaces",
                &(self.install_multiple_protocol_interfaces as *const usize),
            )
            .field(
                "uninstall_multiple_protocol_interfaces",
                &(self.uninstall_multiple_protocol_interfaces as *const usize),
            )
            .field("calculate_crc32", &(self.calculate_crc32 as *const usize))
            .field("copy_mem (fn ptr)", &(self.copy_mem as *const usize))
            .field("set_mem (fn ptr)", &(self.set_mem as *const usize))
            .field("create_event_ex", &(self.create_event_ex as *const usize))
            .finish()
    }
}

/// Used as a parameter of [`BootServices::load_image`] to provide the
/// image source.
pub enum LoadImageSource<'a> {
    /// Load an image from a buffer. The data will copied from the
    /// buffer, so the input reference doesn't need to remain valid
    /// after the image is loaded.
    FromBuffer {
        /// Raw image data.
        buffer: &'a [u8],

        /// If set, this path will be added as the file path of the
        /// loaded image. This is not required to load the image, but
        /// may be used by the image itself to load other resources
        /// relative to the image's path.
        file_path: Option<&'a DevicePath>,
    },

    /// Load an image via the [`SimpleFileSystem`] protocol. If there is
    /// no instance of that protocol associated with the path then the
    /// behavior depends on `from_boot_manager`. If `true`, attempt to
    /// load via the `LoadFile` protocol. If `false`, attempt to load
    /// via the `LoadFile2` protocol, then fall back to `LoadFile`.
    FromFilePath {
        /// Device path from which to load the image.
        file_path: &'a DevicePath,

        /// Whether the request originates from the boot manager.
        from_boot_manager: bool,
    },
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

// OpenProtocolAttributes is safe to model as a regular enum because it
// is only used as an input. The attributes are bitflags, but all valid
// combinations are listed in the spec and only ByDriver and Exclusive
// can actually be combined.
//
// Some values intentionally excluded:
//
// ByHandleProtocol (0x01) excluded because it is only intended to be
// used in an implementation of `HandleProtocol`.
//
// TestProtocol (0x04) excluded because it doesn't actually open the
// protocol, just tests if it's present on the handle. Since that
// changes the interface significantly, that's exposed as a separate
// method: `BootServices::test_protocol`.

/// Attributes for [`BootServices::open_protocol`].
#[repr(u32)]
pub enum OpenProtocolAttributes {
    /// Used by drivers to get a protocol interface for a handle. The
    /// driver will not be informed if the interface is uninstalled or
    /// reinstalled.
    GetProtocol = 0x02,

    /// Used by bus drivers to show that a protocol is being used by one
    /// of the child controllers of the bus.
    ByChildController = 0x08,

    /// Used by a driver to gain access to a protocol interface. When
    /// this mode is used, the driver's `Stop` function will be called
    /// if the protocol interface is reinstalled or uninstalled. Once a
    /// protocol interface is opened with this attribute, no other
    /// drivers will be allowed to open the same protocol interface with
    /// the `ByDriver` attribute.
    ByDriver = 0x10,

    /// Used by a driver to gain exclusive access to a protocol
    /// interface. If any other drivers have the protocol interface
    /// opened with an attribute of `ByDriver`, then an attempt will be
    /// made to remove them with `DisconnectController`.
    ByDriverExclusive = 0x30,

    /// Used by applications to gain exclusive access to a protocol
    /// interface. If any drivers have the protocol opened with an
    /// attribute of `ByDriver`, then an attempt will be made to remove
    /// them by calling the driver's `Stop` function.
    Exclusive = 0x20,
}

/// Parameters passed to [`BootServices::open_protocol`].
pub struct OpenProtocolParams {
    /// The handle for the protocol to open.
    pub handle: Handle,

    /// The handle of the calling agent. For drivers, this is the handle
    /// containing the `EFI_DRIVER_BINDING_PROTOCOL` instance. For
    /// applications, this is the image handle.
    pub agent: Handle,

    /// For drivers, this is the controller handle that requires the
    /// protocol interface. For applications this should be set to
    /// `None`.
    pub controller: Option<Handle>,
}

/// An open protocol interface. Automatically closes the protocol
/// interface on drop.
///
/// See also the [`BootServices`] documentation for details of how to open a
/// protocol and why [`UnsafeCell`] is used.
pub struct ScopedProtocol<'a, P: Protocol + ?Sized> {
    /// The protocol interface.
    #[deprecated(since = "0.17.0", note = "use Deref and DerefMut instead")]
    pub interface: &'a UnsafeCell<P>,

    open_params: OpenProtocolParams,
    boot_services: &'a BootServices,
}

impl<'a, P: Protocol + ?Sized> Drop for ScopedProtocol<'a, P> {
    fn drop(&mut self) {
        let status = (self.boot_services.close_protocol)(
            self.open_params.handle,
            &P::GUID,
            self.open_params.agent,
            self.open_params.controller,
        );
        // All of the error cases for close_protocol boil down to
        // calling it with a different set of parameters than what was
        // passed to open_protocol. The public API prevents such errors,
        // and the error can't be propagated out of drop anyway, so just
        // assert success.
        assert_eq!(status, Status::SUCCESS);
    }
}

impl<'a, P: Protocol + ?Sized> Deref for ScopedProtocol<'a, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        #[allow(deprecated)]
        unsafe {
            &*self.interface.get()
        }
    }
}

impl<'a, P: Protocol + ?Sized> DerefMut for ScopedProtocol<'a, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(deprecated)]
        unsafe {
            &mut *self.interface.get()
        }
    }
}

/// Type of allocation to perform.
#[derive(Debug, Copy, Clone)]
pub enum AllocateType {
    /// Allocate any possible pages.
    AnyPages,
    /// Allocate pages at any address below the given address.
    MaxAddress(PhysicalAddress),
    /// Allocate pages at the specified address.
    Address(PhysicalAddress),
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
    pub phys_start: PhysicalAddress,
    /// Starting virtual address.
    pub virt_start: VirtualAddress,
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

/// A structure containing the size of a memory descriptor and the size of the memory map
pub struct MemoryMapSize {
    /// Size of a single memory descriptor in bytes
    pub entry_size: usize,
    /// Size of the entire memory map in bytes
    pub map_size: usize,
}

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
type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: Option<NonNull<c_void>>);

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

    protocols: *mut &'a Guid,
    count: usize,
}

impl<'a> Drop for ProtocolsPerHandle<'a> {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = self.boot_services.free_pool(self.protocols.cast::<u8>());
    }
}

impl<'a> ProtocolsPerHandle<'a> {
    /// Get the protocol interface [`Guids`][Guid] that are installed on the
    /// [`Handle`].
    pub fn protocols<'b>(&'b self) -> &'b [&'a Guid] {
        // convert raw pointer to slice here so that we can get
        // appropriate lifetime of the slice.
        unsafe { slice::from_raw_parts(self.protocols, self.count) }
    }
}

/// A buffer that contains an array of [`Handles`][Handle] that support the requested protocol.
/// Returned by [`BootServices::locate_handle_buffer`].
pub struct HandleBuffer<'a> {
    // The pointer returned by `locate_handle_buffer` has to be free'd with
    // `free_pool`, so keep a reference to boot services for that purpose.
    boot_services: &'a BootServices,
    count: usize,
    buffer: *mut Handle,
}

impl<'a> Drop for HandleBuffer<'a> {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = self.boot_services.free_pool(self.buffer.cast::<u8>());
    }
}

impl<'a> HandleBuffer<'a> {
    /// Get an array of [`Handles`][Handle] that support the requested protocol.
    pub fn handles(&self) -> &[Handle] {
        // convert raw pointer to slice here so that we can get
        // appropriate lifetime of the slice.
        unsafe { slice::from_raw_parts(self.buffer, self.count) }
    }
}
