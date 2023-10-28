use core::{
    cell::UnsafeCell,
    ffi::c_void,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
};

use uefi_raw::{
    table::boot::{EventNotifyFn, EventType, MemoryType, Tpl},
    PhysicalAddress, Status,
};
use uguid::Guid;

use crate::{
    proto::Protocol,
    system::system_table,
    table::boot::{AllocateType, MemoryMap, MemoryMapSize, OpenProtocolParams, TimerTrigger},
    Event, Handle, Result,
};

use self::raw::{
    allocate_pages_raw, allocate_pool_raw, close_event_raw, create_event_ex_raw, create_event_raw,
    free_pages_raw, free_pool_raw, memory_map_raw, memory_map_size_raw, raise_tpl_raw,
    set_timer_raw, signal_event_raw, wait_for_event_raw,
};

mod raw;

static IMAGE_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// The reference counter for [`BootHandle`]s, which allows safe exiting from
/// boot services using a global design.
static BOOT_HANDLE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Set to true when [`exit_boot_services`] is called.
///
/// This causes all calls to [`acquire_boot_handle`] to fail,
/// but clone [`BootHandle`]s is still allowed.
static EXITING_BOOT: AtomicBool = AtomicBool::new(false);

pub(crate) fn boot_services_maybe_null() -> *mut uefi_raw::table::boot::BootServices {
    let system_table = unsafe { system_table().as_ref() };

    system_table.boot_services
}

pub(crate) fn boot_services() -> NonNull<uefi_raw::table::boot::BootServices> {
    NonNull::new(boot_services_maybe_null()).expect("boot services are not active")
}

/// Returns a boot handle.
pub fn acquire_boot_handle() -> BootHandle {
    BOOT_HANDLE_COUNT.fetch_add(1, Ordering::Relaxed);

    if EXITING_BOOT.load(Ordering::Relaxed) {
        BOOT_HANDLE_COUNT.fetch_sub(1, Ordering::Relaxed);
        panic!("boot services are not active or are exiting");
    }

    BootHandle(boot_services())
}

/// Updates the global image [`Handle`].
///
/// This is called automatically in the `main` entry point as part of
/// [`uefi_macros::entry`]. It should not be called at any other point in time,
/// unless the executable does not use [`uefi_macros::entry`], in which case it
/// should be called once before calling other `boot` functions.
///
/// # Safety
///
/// This function should be only called as described above, and the `image_handle`
/// must be a valid image [`Handle`]. Then the safety guarentees of
/// [`open_protocol_exclusive`] will be correct.
pub unsafe fn set_image_handle(image_handle: Handle) {
    IMAGE_HANDLE.store(image_handle.as_ptr(), Ordering::Relaxed)
}

/// Get the [`Handle`] of the currently-executing image.
pub fn image_handle() -> Handle {
    // SAFETY:
    // If this pointer is not null, then by the invariants of `set_image_handle`,
    // the value loaded from `IMAGE_HANDLE` is a valid handle.
    unsafe {
        Handle::from_ptr(IMAGE_HANDLE.load(Ordering::Relaxed))
            .expect("set_image_handle has not been called")
    }
}

/// A handle to all of the boot services.
///
/// # Accessing `BootServices`
///
/// A [`BootHandle`] can only be obtained by calling [`acquire_boot_handle`].
///
/// # Accessing Protocols
///
/// Protocols can be opened using several methods of
#[derive(Debug)]
#[repr(transparent)]
pub struct BootHandle(NonNull<uefi_raw::table::boot::BootServices>);

impl BootHandle {
    /// Raises a task's priority level and returns its previous level.
    ///
    /// The effect of calling [`raise_tpl`] with a [`Tpl`] that is below the current
    /// one (which, sadly, cannot be queried) is undefined by the UEFI spec,
    /// which also warns against remaining at high [`Tpl`]s for a long time.
    ///
    /// This function outputs an RAII guard that will automatically restore the
    /// original [`Tpl`] when dropped.
    ///
    /// # Safety
    ///
    /// Raising a task's priority level can affect other running tasks and
    /// critical processes run by UEFI. The highest priority level is the
    /// most dangerous, since it disables interrupts.
    pub unsafe fn raise_tpl(&self, tpl: Tpl) -> TplGuard {
        raise_tpl_raw(MaybeBootRef::Ref(self), tpl)
    }

    /// Allocates memory pages from the system.
    ///
    /// UEFI OS loaders should allocate memory of the type `LoaderData`. An `u64`
    /// is returned even on 32-bit platforms because some hardware configurations
    /// like Intel PAE enable 64-bit physical addressing on a 32-bit processor.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.AllocatePages()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::NOT_FOUND`]
    pub fn allocate_pages(
        &self,
        ty: AllocateType,
        mem_ty: MemoryType,
        count: usize,
    ) -> Result<PhysicalAddress> {
        allocate_pages_raw(self, ty, mem_ty, count)
    }

    /// Frees memory pages allocated by UEFI.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no references into the allocation remain,
    /// and that the memory at the allocation is not used after it is freed.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.FreePages()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NOT_FOUND`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub unsafe fn free_pages(&self, addr: PhysicalAddress, count: usize) -> Result {
        free_pages_raw(self, addr, count)
    }

    /// Returns struct which contains the size of a single memory descriptor
    /// as well as the size of the current memory map.
    ///
    /// Note that the size of the memory map can increase any time an allocation happens,
    /// so when creating a buffer to put the memory map into, it's recommended to allocate a few extra
    /// elements worth of space above the size of the current memory map.
    #[must_use]
    pub fn memory_map_size(&self) -> MemoryMapSize {
        memory_map_size_raw(self)
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.GetMemoryMap()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::BUFFER_TOO_SMALL`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn memory_map<'buf>(&self, buffer: &'buf mut [u8]) -> Result<MemoryMap<'buf>> {
        memory_map_raw(self, buffer)
    }

    /// Allocates from a memory pool. The pointer will be 8-byte aligned.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.AllocatePool()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn allocate_pool(&self, mem_ty: MemoryType, size: usize) -> Result<*mut u8> {
        allocate_pool_raw(self, mem_ty, size)
    }

    /// Frees memory allocated from a pool.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no references into the allocation remain,
    /// and that the memory at the allocation is not used after it is freed.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.FreePool()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub unsafe fn free_pool(&self, addr: *mut u8) -> Result {
        free_pool_raw(self, addr)
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.CreateEvent()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    pub unsafe fn create_event(
        &self,
        event_ty: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
    ) -> Result<Event> {
        create_event_raw(self, event_ty, notify_tpl, notify_fn, notify_ctx)
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.CreateEventEx()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    pub unsafe fn create_event_ex(
        &self,
        event_type: EventType,
        notify_tpl: Tpl,
        notify_fn: Option<EventNotifyFn>,
        notify_ctx: Option<NonNull<c_void>>,
        event_group: Option<NonNull<Guid>>,
    ) -> Result<Event> {
        create_event_ex_raw(
            self,
            event_type,
            notify_tpl,
            notify_fn,
            notify_ctx,
            event_group,
        )
    }

    /// Sets the trigger for `EventType::TIMER` event.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.SetTimer()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn set_timer(&self, event: &Event, trigger_time: TimerTrigger) -> Result {
        set_timer_raw(self, event, trigger_time)
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
    ///   error is returned with the index of the event that caused the failure.
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.WaitForEvent()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::UNSUPPORTED`]
    pub fn wait_for_event(&self, events: &mut [Event]) -> Result<usize, Option<usize>> {
        wait_for_event_raw(self, events)
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.SignalEvent()` in the UEFI Specification for more details.
    ///
    /// Currently, (as of UEFI Spec v2.9) this only returns `EFI_SUCCESS`.
    pub fn signal_event(&self, event: &Event) -> Result {
        signal_event_raw(self, event)
    }

    /// Removes `event` from any event group to which it belongs and closes it. If `event` was
    /// registered with `register_protocol_notify()`, then the corresponding registration will
    /// be removed. It is safe to call this function within the corresponding notify function.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.CloseEvent()` in the UEFI Specification for more details.
    ///
    /// Note: The UEFI Specification v2.9 states that this may only return `EFI_SUCCESS`, but,
    /// at least for application based on EDK2 (such as OVMF), it may also return `EFI_INVALID_PARAMETER`.
    /// To be safe, ensure that error codes are handled properly.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn close_event(&self, event: Event) -> Result {
        close_event_raw(self, event)
    }
}

impl Clone for BootHandle {
    fn clone(&self) -> Self {
        let count = BOOT_HANDLE_COUNT.fetch_add(1, Ordering::Relaxed);

        if count > usize::MAX {
            BOOT_HANDLE_COUNT.fetch_sub(1, Ordering::Relaxed);
            panic!("boot handle reference counter grew too large");
        }

        BootHandle(self.0)
    }
}

impl Drop for BootHandle {
    fn drop(&mut self) {
        assert!(
            BOOT_HANDLE_COUNT.fetch_sub(1, Ordering::Relaxed) > 0,
            "corrupted boot handle counter"
        );
    }
}

/// Raises a task's priority level and returns its previous level.
///
/// The effect of calling [`raise_tpl`] with a [`Tpl`] that is below the current
/// one (which, sadly, cannot be queried) is undefined by the UEFI spec,
/// which also warns against remaining at high [`Tpl`]s for a long time.
///
/// This function outputs an RAII guard that will automatically restore the
/// original [`Tpl`] when dropped.
///
/// # Safety
///
/// Raising a task's priority level can affect other running tasks and
/// critical processes run by UEFI. The highest priority level is the
/// most dangerous, since it disables interrupts.
pub unsafe fn raise_tpl(tpl: Tpl) -> TplGuard<'static> {
    let boot_handle = acquire_boot_handle();

    raise_tpl_raw(MaybeBootRef::Value(boot_handle), tpl)
}

/// Allocates memory pages from the system.
///
/// UEFI OS loaders should allocate memory of the type `LoaderData`. An `u64`
/// is returned even on 32-bit platforms because some hardware configurations
/// like Intel PAE enable 64-bit physical addressing on a 32-bit processor.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.AllocatePages()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::NOT_FOUND`]
pub fn allocate_pages(
    ty: AllocateType,
    mem_ty: MemoryType,
    count: usize,
) -> Result<PhysicalAddress> {
    let boot_handle = acquire_boot_handle();

    allocate_pages_raw(&boot_handle, ty, mem_ty, count)
}

/// Frees memory pages allocated by UEFI.
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.FreePages()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn free_pages(addr: PhysicalAddress, count: usize) -> Result {
    let boot_handle = acquire_boot_handle();

    free_pages_raw(&boot_handle, addr, count)
}

/// Returns struct which contains the size of a single memory descriptor
/// as well as the size of the current memory map.
///
/// Note that the size of the memory map can increase any time an allocation happens,
/// so when creating a buffer to put the memory map into, it's recommended to allocate a few extra
/// elements worth of space above the size of the current memory map.
#[must_use]
pub fn memory_map_size() -> MemoryMapSize {
    let boot_handle = acquire_boot_handle();

    memory_map_size_raw(&boot_handle)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.GetMemoryMap()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::BUFFER_TOO_SMALL`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn memory_map<'buf>(buffer: &'buf mut [u8]) -> Result<MemoryMap<'buf>> {
    let boot_handle = acquire_boot_handle();

    memory_map_raw(&boot_handle, buffer)
}

/// Allocates from a memory pool. The pointer will be 8-byte aligned.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.AllocatePool()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn allocate_pool(mem_ty: MemoryType, size: usize) -> Result<*mut u8> {
    let boot_handle = acquire_boot_handle();

    allocate_pool_raw(&boot_handle, mem_ty, size)
}

/// Frees memory allocated from a pool.
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.FreePool()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn free_pool(addr: *mut u8) -> Result {
    let boot_handle = acquire_boot_handle();

    free_pool_raw(&boot_handle, addr)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.CreateEvent()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
pub unsafe fn create_event(
    event_ty: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
) -> Result<Event> {
    let boot_handle = acquire_boot_handle();

    create_event_raw(&boot_handle, event_ty, notify_tpl, notify_fn, notify_ctx)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.CreateEventEx()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
pub unsafe fn create_event_ex(
    event_type: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
    event_group: Option<NonNull<Guid>>,
) -> Result<Event> {
    let boot_handle = acquire_boot_handle();

    create_event_ex_raw(
        &boot_handle,
        event_type,
        notify_tpl,
        notify_fn,
        notify_ctx,
        event_group,
    )
}

/// Sets the trigger for `EventType::TIMER` event.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.SetTimer()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn set_timer(event: &Event, trigger_time: TimerTrigger) -> Result {
    let boot_handle = acquire_boot_handle();

    set_timer_raw(&boot_handle, event, trigger_time)
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
///   error is returned with the index of the event that caused the failure.
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.WaitForEvent()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
pub fn wait_for_event(events: &mut [Event]) -> Result<usize, Option<usize>> {
    let boot_handle = acquire_boot_handle();

    wait_for_event_raw(&boot_handle, events)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.SignalEvent()` in the UEFI Specification for more details.
///
/// Currently, (as of UEFI Spec v2.9) this only returns `EFI_SUCCESS`.
pub fn signal_event(event: &Event) -> Result {
    let boot_handle = acquire_boot_handle();

    signal_event_raw(&boot_handle, event)
}

/// Removes `event` from any event group to which it belongs and closes it. If `event` was
/// registered with `register_protocol_notify()`, then the corresponding registration will
/// be removed. It is safe to call this function within the corresponding notify function.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.CloseEvent()` in the UEFI Specification for more details.
///
/// Note: The UEFI Specification v2.9 states that this may only return `EFI_SUCCESS`, but,
/// at least for application based on EDK2 (such as OVMF), it may also return `EFI_INVALID_PARAMETER`.
/// To be safe, ensure that error codes are handled properly.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn close_event(event: Event) -> Result {
    let boot_handle = acquire_boot_handle();

    close_event_raw(&boot_handle, event)
}

/// RAII guard for task priority level changes
///
/// Will automatically restore the former task priority level when dropped.
#[derive(Debug)]
pub struct TplGuard<'boot> {
    old_tpl: Tpl,
    boot_handle: MaybeBootRef<'boot>,
}

impl<'boot> TplGuard<'boot> {
    /// Converts potentially scoped [`TplGuard`] into a `'static` [`TplGuard`].
    pub fn make_static(self) -> TplGuard<'static> {
        let old_tpl = self.old_tpl;

        let boot_handle = match self.boot_handle {
            MaybeBootRef::Ref(boot_ref) => boot_ref.clone(),
            MaybeBootRef::Value(ref boot_handle) => BootHandle(boot_handle.0),
        };

        mem::forget(self);

        TplGuard {
            old_tpl,
            boot_handle: MaybeBootRef::Value(boot_handle),
        }
    }
}

impl Drop for TplGuard<'_> {
    fn drop(&mut self) {
        let boot_services = self.boot_handle.0;

        unsafe {
            (boot_services.as_ref().restore_tpl)(self.old_tpl);
        }
    }
}

/// An open protocol interface. Automatically closes the protocol
/// interface on drop.
///
/// Most protocols have interface data associated with them. `ScopedProtocol`
/// implements [`Deref`] and [`DerefMut`] to access this data. A few protocols
/// (such as [`DevicePath`] and [`LoadedImageDevicePath`]) may be installed with
/// null interface data, in which case [`Deref`] and [`DerefMut`] will
/// panic. The [`get`] and [`get_mut`] methods may be used to access the
/// optional interface data without panicking.
///
/// See also the [`BootServices`] documentation for details of how to open a
/// protocol and why [`UnsafeCell`] is used.
///
/// [`LoadedImageDevicePath`]: crate::proto::device_path::LoadedImageDevicePath
/// [`get`]: ScopedProtocol::get
/// [`get_mut`]: ScopedProtocol::get_mut
#[derive(Debug)]
pub struct ScopedProtocol<'a, P: Protocol + ?Sized + 'static> {
    /// The protocol interface.
    interface: Option<&'static UnsafeCell<P>>,

    open_params: OpenProtocolParams,
    boot_handle: MaybeBootRef<'a>,
}

impl<'a, P: Protocol + ?Sized> ScopedProtocol<'a, P> {
    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get(&self) -> Option<&'a P> {
        self.interface.map(|p| unsafe { &*p.get() })
    }

    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get_mut(&self) -> Option<&'a mut P> {
        self.interface.map(|p| unsafe { &mut *p.get() })
    }

    /// Converts potentially scoped [`ScopedProtocol`] into a `'static` [`ScopedProtocol`].
    pub fn make_static(self) -> ScopedProtocol<'static, P> {
        let interface = self.interface;
        let open_params = self.open_params;

        let boot_handle = match self.boot_handle {
            MaybeBootRef::Ref(boot_ref) => boot_ref.clone(),
            MaybeBootRef::Value(ref boot_handle) => BootHandle(boot_handle.0),
        };

        mem::forget(self);

        ScopedProtocol {
            interface,
            open_params,
            boot_handle: MaybeBootRef::Value(boot_handle),
        }
    }
}

impl<'a, P: Protocol + ?Sized> Drop for ScopedProtocol<'a, P> {
    fn drop(&mut self) {
        let status = unsafe {
            (self.boot_handle.0.as_ref().close_protocol)(
                self.open_params.handle.as_ptr(),
                &P::GUID,
                self.open_params.agent.as_ptr(),
                Handle::opt_to_ptr(self.open_params.controller),
            )
        };
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

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.interface.unwrap().get() }
    }
}

impl<'a, P: Protocol + ?Sized> DerefMut for ScopedProtocol<'a, P> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.interface.unwrap().get() }
    }
}

/// Protocol interface [`Guids`][Guid] that are installed on a [`Handle`] as
/// returned by [`BootServices::protocols_per_handle`].
#[derive(Debug)]
pub struct ProtocolsPerHandle<'a> {
    protocols: *mut &'static Guid,
    count: usize,

    // The pointer returned by `protocols_per_handle` has to be free'd with
    // `free_pool`, so keep a reference to boot services for that purpose.
    boot_handle: MaybeBootRef<'a>,
}

impl<'a> Drop for ProtocolsPerHandle<'a> {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = unsafe { self.boot_handle.free_pool(self.protocols.cast::<u8>()) };
    }
}

impl<'a> Deref for ProtocolsPerHandle<'a> {
    type Target = [&'a Guid];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.protocols, self.count) }
    }
}

impl<'a> ProtocolsPerHandle<'a> {
    /// Get the protocol interface [`Guids`][Guid] that are installed on the
    /// [`Handle`].
    #[allow(clippy::missing_const_for_fn)] // Required until we bump the MSRV.
    #[deprecated = "use Deref instead"]
    #[must_use]
    pub fn protocols<'b>(&'b self) -> &'b [&'a Guid] {
        // convert raw pointer to slice here so that we can get
        // appropriate lifetime of the slice.
        unsafe { slice::from_raw_parts(self.protocols, self.count) }
    }

    /// Converts potentially scoped [`ProtocolsPerHandle`] into a `'static` [`ProtocolsPerHandle`].
    pub fn make_static(self) -> ProtocolsPerHandle<'static> {
        let protocols = self.protocols;
        let count = self.count;

        let boot_handle = match self.boot_handle {
            MaybeBootRef::Ref(boot_ref) => boot_ref.clone(),
            MaybeBootRef::Value(ref boot_handle) => BootHandle(boot_handle.0),
        };

        mem::forget(self);

        ProtocolsPerHandle {
            protocols,
            count,
            boot_handle: MaybeBootRef::Value(boot_handle),
        }
    }
}

/// A buffer that contains an array of [`Handles`][Handle] that support the
/// requested protocol. Returned by [`BootServices::locate_handle_buffer`].
#[derive(Debug)]
pub struct HandleBuffer<'a> {
    count: usize,
    buffer: *mut Handle,

    // The pointer returned by `locate_handle_buffer` has to be freed with
    // `free_pool`, so keep a reference to boot services for that purpose.
    boot_handle: MaybeBootRef<'a>,
}

impl<'a> Drop for HandleBuffer<'a> {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = unsafe { self.boot_handle.free_pool(self.buffer.cast::<u8>()) };
    }
}

impl<'a> Deref for HandleBuffer<'a> {
    type Target = [Handle];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buffer, self.count) }
    }
}

impl<'a> HandleBuffer<'a> {
    /// Get an array of [`Handles`][Handle] that support the requested protocol.
    #[allow(clippy::missing_const_for_fn)] // Required until we bump the MSRV.
    #[deprecated = "use Deref instead"]
    #[must_use]
    pub fn handles(&self) -> &[Handle] {
        // convert raw pointer to slice here so that we can get
        // appropriate lifetime of the slice.
        unsafe { slice::from_raw_parts(self.buffer, self.count) }
    }

    /// Converts potentially scoped [`HandleBuffer`] into a `'static` [`HandleBuffer`].
    pub fn make_static(self) -> HandleBuffer<'static> {
        let count = self.count;
        let buffer = self.buffer;

        let boot_handle = match self.boot_handle {
            MaybeBootRef::Ref(boot_ref) => boot_ref.clone(),
            MaybeBootRef::Value(ref boot_handle) => BootHandle(boot_handle.0),
        };

        mem::forget(self);

        HandleBuffer {
            buffer,
            count,
            boot_handle: MaybeBootRef::Value(boot_handle),
        }
    }
}

#[derive(Debug)]
enum MaybeBootRef<'boot> {
    Ref(&'boot BootHandle),
    Value(BootHandle),
}

impl Deref for MaybeBootRef<'_> {
    type Target = BootHandle;

    fn deref(&self) -> &Self::Target {
        match *self {
            MaybeBootRef::Ref(reference) => reference,
            MaybeBootRef::Value(ref value) => value,
        }
    }
}
