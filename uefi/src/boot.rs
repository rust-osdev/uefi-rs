//! UEFI boot services.
//!
//! These functions will panic if called after exiting boot services.

pub use uefi_raw::table::boot::{EventType, MemoryAttribute, MemoryDescriptor, MemoryType, Tpl};

use crate::data_types::PhysicalAddress;
use crate::mem::memory_map::{MemoryMapBackingMemory, MemoryMapKey, MemoryMapMeta, MemoryMapOwned};
use crate::polyfill::maybe_uninit_slice_assume_init_ref;
#[cfg(doc)]
use crate::proto::device_path::LoadedImageDevicePath;
use crate::proto::device_path::{DevicePath, FfiDevicePath};
use crate::proto::loaded_image::LoadedImage;
use crate::proto::media::fs::SimpleFileSystem;
use crate::proto::{BootPolicy, Protocol, ProtocolPointer};
use crate::runtime::{self, ResetType};
use crate::table::Revision;
use crate::util::opt_nonnull_to_ptr;
use crate::{table, Char16, Error, Event, Guid, Handle, Result, Status, StatusExt};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, slice};
use uefi_raw::table::boot::InterfaceType;
#[cfg(feature = "alloc")]
use {alloc::vec::Vec, uefi::ResultExt};

/// Global image handle. This is only set by [`set_image_handle`], and it is
/// only read by [`image_handle`].
static IMAGE_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Get the [`Handle`] of the currently-executing image.
#[must_use]
pub fn image_handle() -> Handle {
    let ptr = IMAGE_HANDLE.load(Ordering::Acquire);
    // Safety: the image handle must be valid. We know it is, because it was set
    // by `set_image_handle`, which has that same safety requirement.
    unsafe { Handle::from_ptr(ptr) }.expect("set_image_handle has not been called")
}

/// Update the global image [`Handle`].
///
/// This is called automatically in the `main` entry point as part of
/// [`uefi::entry`]. It should not be called at any other point in time, unless
/// the executable does not use [`uefi::entry`], in which case it should be
/// called once before calling other boot services functions.
///
/// # Safety
///
/// This function should only be called as described above, and the
/// `image_handle` must be a valid image [`Handle`]. The safety guarantees of
/// [`open_protocol_exclusive`] rely on the global image handle being correct.
pub unsafe fn set_image_handle(image_handle: Handle) {
    IMAGE_HANDLE.store(image_handle.as_ptr(), Ordering::Release);
}

/// Return true if boot services are active, false otherwise.
pub(crate) fn are_boot_services_active() -> bool {
    let Some(st) = table::system_table_raw() else {
        return false;
    };

    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };

    !st.boot_services.is_null()
}

fn boot_services_raw_panicking() -> NonNull<uefi_raw::table::boot::BootServices> {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    NonNull::new(st.boot_services).expect("boot services are not active")
}

/// Raises a task's priority level and returns a [`TplGuard`].
///
/// The effect of calling `raise_tpl` with a `Tpl` that is below the current
/// one (which, sadly, cannot be queried) is undefined by the UEFI spec,
/// which also warns against remaining at high `Tpl`s for a long time.
///
/// This function returns an RAII guard that will automatically restore the
/// original `Tpl` when dropped.
///
/// # Safety
///
/// Raising a task's priority level can affect other running tasks and
/// critical processes run by UEFI. The highest priority level is the
/// most dangerous, since it disables interrupts.
#[must_use]
pub unsafe fn raise_tpl(tpl: Tpl) -> TplGuard {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    TplGuard {
        old_tpl: (bt.raise_tpl)(tpl),
    }
}

/// Allocates memory pages from the system.
///
/// UEFI OS loaders should allocate memory of the type `LoaderData`.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: allocation failed.
/// * [`Status::INVALID_PARAMETER`]: `mem_ty` is [`MemoryType::PERSISTENT_MEMORY`],
///   [`MemoryType::UNACCEPTED`], or in the range [`MemoryType::MAX`]`..=0x6fff_ffff`.
/// * [`Status::NOT_FOUND`]: the requested pages could not be found.
pub fn allocate_pages(ty: AllocateType, mem_ty: MemoryType, count: usize) -> Result<NonNull<u8>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (ty, mut addr) = match ty {
        AllocateType::AnyPages => (0, 0),
        AllocateType::MaxAddress(addr) => (1, addr),
        AllocateType::Address(addr) => (2, addr),
    };
    let addr =
        unsafe { (bt.allocate_pages)(ty, mem_ty, count, &mut addr) }.to_result_with_val(|| addr)?;
    let ptr = addr as *mut u8;
    Ok(NonNull::new(ptr).expect("allocate_pages must not return a null pointer if successful"))
}

/// Frees memory pages allocated by [`allocate_pages`].
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: `ptr` was not allocated by [`allocate_pages`].
/// * [`Status::INVALID_PARAMETER`]: `ptr` is not page aligned or is otherwise invalid.
pub unsafe fn free_pages(ptr: NonNull<u8>, count: usize) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let addr = ptr.as_ptr() as PhysicalAddress;
    unsafe { (bt.free_pages)(addr, count) }.to_result()
}

/// Allocates from a memory pool. The pointer will be 8-byte aligned.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: allocation failed.
/// * [`Status::INVALID_PARAMETER`]: `mem_ty` is [`MemoryType::PERSISTENT_MEMORY`],
///   [`MemoryType::UNACCEPTED`], or in the range [`MemoryType::MAX`]`..=0x6fff_ffff`.
pub fn allocate_pool(mem_ty: MemoryType, size: usize) -> Result<NonNull<u8>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut buffer = ptr::null_mut();
    let ptr =
        unsafe { (bt.allocate_pool)(mem_ty, size, &mut buffer) }.to_result_with_val(|| buffer)?;

    Ok(NonNull::new(ptr).expect("allocate_pool must not return a null pointer if successful"))
}

/// Frees memory allocated by [`allocate_pool`].
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `ptr` is invalid.
pub unsafe fn free_pool(ptr: NonNull<u8>) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe { (bt.free_pool)(ptr.as_ptr()) }.to_result()
}

/// Queries the `get_memory_map` function of UEFI to retrieve the current
/// size of the map. Returns a [`MemoryMapMeta`].
///
/// It is recommended to add a few more bytes for a subsequent allocation
/// for the memory map, as the memory map itself also needs heap memory,
/// and other allocations might occur before that call.
#[must_use]
pub(crate) fn memory_map_size() -> MemoryMapMeta {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut map_size = 0;
    let mut map_key = MemoryMapKey(0);
    let mut desc_size = 0;
    let mut desc_version = 0;

    let status = unsafe {
        (bt.get_memory_map)(
            &mut map_size,
            ptr::null_mut(),
            &mut map_key.0,
            &mut desc_size,
            &mut desc_version,
        )
    };
    assert_eq!(status, Status::BUFFER_TOO_SMALL);

    assert_eq!(
        map_size % desc_size,
        0,
        "Memory map must be a multiple of the reported descriptor size."
    );

    let mmm = MemoryMapMeta {
        desc_size,
        map_size,
        map_key,
        desc_version,
    };

    mmm.assert_sanity_checks();

    mmm
}

/// Stores the current UEFI memory map in an UEFI-heap allocated buffer
/// and returns a [`MemoryMapOwned`].
///
/// # Parameters
///
/// - `mt`: The memory type for the backing memory on the UEFI heap.
///   Usually, this is [`MemoryType::LOADER_DATA`]. You can also use a
///   custom type.
///
/// # Errors
///
/// * [`Status::BUFFER_TOO_SMALL`]
/// * [`Status::INVALID_PARAMETER`]
pub fn memory_map(mt: MemoryType) -> Result<MemoryMapOwned> {
    let mut buffer = MemoryMapBackingMemory::new(mt)?;

    let meta = get_memory_map(buffer.as_mut_slice())?;

    Ok(MemoryMapOwned::from_initialized_mem(buffer, meta))
}

/// Calls the underlying `GetMemoryMap` function of UEFI. On success,
/// the buffer is mutated and contains the map. The map might be shorter
/// than the buffer, which is reflected by the return value.
pub(crate) fn get_memory_map(buf: &mut [u8]) -> Result<MemoryMapMeta> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut map_size = buf.len();
    let map_buffer = buf.as_mut_ptr().cast::<MemoryDescriptor>();
    let mut map_key = MemoryMapKey(0);
    let mut desc_size = 0;
    let mut desc_version = 0;

    assert_eq!(
        (map_buffer as usize) % mem::align_of::<MemoryDescriptor>(),
        0,
        "Memory map buffers must be aligned like a MemoryDescriptor"
    );

    unsafe {
        (bt.get_memory_map)(
            &mut map_size,
            map_buffer,
            &mut map_key.0,
            &mut desc_size,
            &mut desc_version,
        )
    }
    .to_result_with_val(|| MemoryMapMeta {
        map_size,
        desc_size,
        map_key,
        desc_version,
    })
}

/// Creates an event.
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
/// * [`Status::INVALID_PARAMETER`]: an invalid combination of parameters was provided.
/// * [`Status::OUT_OF_RESOURCES`]: the event could not be allocated.
pub unsafe fn create_event(
    event_ty: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
) -> Result<Event> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut event = ptr::null_mut();

    // Safety: the argument types of the function pointers are defined
    // differently, but are compatible and can be safely transmuted.
    let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

    let notify_ctx = opt_nonnull_to_ptr(notify_ctx);

    // Now we're ready to call UEFI
    (bt.create_event)(event_ty, notify_tpl, notify_fn, notify_ctx, &mut event).to_result_with_val(
        // OK to unwrap: event is non-null for Status::SUCCESS.
        || Event::from_ptr(event).unwrap(),
    )
}

/// Creates an event in an event group.
///
/// The event's notification function, context, and task priority are specified
/// by `notify_fn`, `notify_ctx`, and `notify_tpl`, respectively. The event will
/// be added to the group of events identified by `event_group`.
///
/// If no group is specified by `event_group`, this function behaves as if the
/// same parameters had been passed to `create_event()`.
///
/// Event groups are collections of events identified by a shared GUID where,
/// when one member event is signaled, all other events are signaled and their
/// individual notification actions are taken. All events are guaranteed to be
/// signaled before the first notification action is taken. All notification
/// functions will be executed in the order specified by their `Tpl`.
///
/// An event can only be part of a single event group. An event may be removed
/// from an event group by calling [`close_event`].
///
/// The [`EventType`] of an event uses the same values as `create_event()`, except that
/// `EventType::SIGNAL_EXIT_BOOT_SERVICES` and `EventType::SIGNAL_VIRTUAL_ADDRESS_CHANGE`
/// are not valid.
///
/// For events of type `NOTIFY_SIGNAL` or `NOTIFY_WAIT`, `notify_fn` must be
/// `Some` and `notify_tpl` must be a valid task priority level. For other event
/// types these parameters are ignored.
///
/// More than one event of type `EventType::TIMER` may be part of a single event
/// group. However, there is no mechanism for determining which of the timers
/// was signaled.
///
/// This operation is only supported starting with UEFI 2.0; earlier versions
/// will fail with [`Status::UNSUPPORTED`].
///
/// # Safety
///
/// The caller must ensure they are passing a valid `Guid` as `event_group`, if applicable.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: an invalid combination of parameters was provided.
/// * [`Status::OUT_OF_RESOURCES`]: the event could not be allocated.
pub unsafe fn create_event_ex(
    event_type: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
    event_group: Option<NonNull<Guid>>,
) -> Result<Event> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    if bt.header.revision < Revision::EFI_2_00 {
        return Err(Status::UNSUPPORTED.into());
    }

    let mut event = ptr::null_mut();

    // Safety: the argument types of the function pointers are defined
    // differently, but are compatible and can be safely transmuted.
    let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

    (bt.create_event_ex)(
        event_type,
        notify_tpl,
        notify_fn,
        opt_nonnull_to_ptr(notify_ctx),
        opt_nonnull_to_ptr(event_group),
        &mut event,
    )
    .to_result_with_val(
        // OK to unwrap: event is non-null for Status::SUCCESS.
        || Event::from_ptr(event).unwrap(),
    )
}

/// Checks to see if an event is signaled, without blocking execution to wait for it.
///
/// Returns `Ok(true)` if the event is in the signaled state or `Ok(false)`
/// if the event is not in the signaled state.
///
/// # Errors
///
/// Note: Instead of returning [`Status::NOT_READY`] as listed in the UEFI
/// Specification, this function will return `Ok(false)`.
///
/// * [`Status::INVALID_PARAMETER`]: `event` is of type [`NOTIFY_SIGNAL`].
///
/// [`NOTIFY_SIGNAL`]: EventType::NOTIFY_SIGNAL
pub fn check_event(event: Event) -> Result<bool> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let status = unsafe { (bt.check_event)(event.as_ptr()) };
    match status {
        Status::SUCCESS => Ok(true),
        Status::NOT_READY => Ok(false),
        _ => Err(status.into()),
    }
}

/// Removes `event` from any event group to which it belongs and closes it.
///
/// If `event` was registered with [`register_protocol_notify`], then the
/// corresponding registration will be removed. Calling this function within the
/// corresponding notify function is allowed.
///
/// # Errors
///
/// The specification does not list any errors, however implementations are
/// allowed to return an error if needed.
pub fn close_event(event: Event) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe { (bt.close_event)(event.as_ptr()) }.to_result()
}

/// Sets the trigger for an event of type [`TIMER`].
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `event` is not valid.
///
/// [`TIMER`]: EventType::TIMER
pub fn set_timer(event: &Event, trigger_time: TimerTrigger) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (ty, time) = match trigger_time {
        TimerTrigger::Cancel => (0, 0),
        TimerTrigger::Periodic(hundreds_ns) => (1, hundreds_ns),
        TimerTrigger::Relative(hundreds_ns) => (2, hundreds_ns),
    };
    unsafe { (bt.set_timer)(event.as_ptr(), ty, time) }.to_result()
}

/// Stops execution until an event is signaled.
///
/// This function must be called at priority level [`Tpl::APPLICATION`].
///
/// The input [`Event`] slice is repeatedly iterated from first to last until
/// an event is signaled or an error is detected. The following checks are
/// performed on each event:
///
/// * If an event is of type [`NOTIFY_SIGNAL`], then a
///   [`Status::INVALID_PARAMETER`] error is returned with the index of the
///   event that caused the failure.
/// * If an event is in the signaled state, the signaled state is cleared
///   and the index of the event that was signaled is returned.
/// * If an event is not in the signaled state but does have a notification
///   function, the notification function is queued at the event's
///   notification task priority level. If the execution of the event's
///   notification function causes the event to be signaled, then the
///   signaled state is cleared and the index of the event that was signaled
///   is returned.
///
/// To wait for a specified time, a timer event must be included in `events`.
///
/// To check if an event is signaled without waiting, an already signaled
/// event can be used as the last event in the slice being checked, or the
/// [`check_event`] interface may be used.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `events` is empty, or one of the events of
///   of type [`NOTIFY_SIGNAL`].
/// * [`Status::UNSUPPORTED`]: the current TPL is not [`Tpl::APPLICATION`].
///
/// [`NOTIFY_SIGNAL`]: EventType::NOTIFY_SIGNAL
pub fn wait_for_event(events: &mut [Event]) -> Result<usize, Option<usize>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let number_of_events = events.len();
    let events: *mut uefi_raw::Event = events.as_mut_ptr().cast();

    let mut index = 0;
    unsafe { (bt.wait_for_event)(number_of_events, events, &mut index) }.to_result_with(
        || index,
        |s| {
            if s == Status::INVALID_PARAMETER {
                Some(index)
            } else {
                None
            }
        },
    )
}

/// Connect one or more drivers to a controller.
///
/// Usually one disconnects and then reconnects certain drivers
/// to make them rescan some state that changed, e.g. reconnecting
/// a block handle after your app modified disk partitions.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: there are no driver-binding protocol instances
///   present in the system, or no drivers are connected to `controller`.
/// * [`Status::SECURITY_VIOLATION`]: the caller does not have permission to
///   start drivers associated with `controller`.
pub fn connect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    remaining_device_path: Option<&DevicePath>,
    recursive: bool,
) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe {
        (bt.connect_controller)(
            controller.as_ptr(),
            Handle::opt_to_ptr(driver_image),
            remaining_device_path
                .map(|dp| dp.as_ffi_ptr())
                .unwrap_or(ptr::null())
                .cast(),
            recursive,
        )
    }
    .to_result_with_err(|_| ())
}

/// Disconnect one or more drivers from a controller.
///
/// See also [`connect_controller`].
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `driver_image` is set but does not manage
///   `controller`, or does not support the driver binding protocol, or one of
///   the handles is invalid.
/// * [`Status::OUT_OF_RESOURCES`]: not enough resources available to disconnect
///   drivers.
/// * [`Status::DEVICE_ERROR`]: the controller could not be disconnected due to
///   a device error.
pub fn disconnect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    child: Option<Handle>,
) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe {
        (bt.disconnect_controller)(
            controller.as_ptr(),
            Handle::opt_to_ptr(driver_image),
            Handle::opt_to_ptr(child),
        )
    }
    .to_result_with_err(|_| ())
}

/// Installs a protocol interface on a device handle.
///
/// When a protocol interface is installed, firmware will call all functions
/// that have registered to wait for that interface to be installed.
///
/// If `handle` is `None`, a new handle will be created and returned.
///
/// # Safety
///
/// The caller is responsible for ensuring that they pass a valid `Guid` for `protocol`.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: failed to allocate a new handle.
/// * [`Status::INVALID_PARAMETER`]: this protocol is already installed on the handle.
pub unsafe fn install_protocol_interface(
    handle: Option<Handle>,
    protocol: &Guid,
    interface: *const c_void,
) -> Result<Handle> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut handle = Handle::opt_to_ptr(handle);
    ((bt.install_protocol_interface)(
        &mut handle,
        protocol,
        InterfaceType::NATIVE_INTERFACE,
        interface,
    ))
    .to_result_with_val(|| Handle::from_ptr(handle).unwrap())
}

/// Reinstalls a protocol interface on a device handle. `old_interface` is replaced with `new_interface`.
/// These interfaces may be the same, in which case the registered protocol notifications occur for the handle
/// without replacing the interface.
///
/// As with `install_protocol_interface`, any process that has registered to wait for the installation of
/// the interface is notified.
///
/// # Safety
///
/// The caller is responsible for ensuring that there are no references to the `old_interface` that is being
/// removed.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: the old interface was not found on the handle.
/// * [`Status::ACCESS_DENIED`]: the old interface is still in use and cannot be uninstalled.
pub unsafe fn reinstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    old_interface: *const c_void,
    new_interface: *const c_void,
) -> Result<()> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    (bt.reinstall_protocol_interface)(handle.as_ptr(), protocol, old_interface, new_interface)
        .to_result()
}

/// Removes a protocol interface from a device handle.
///
/// # Safety
///
/// The caller is responsible for ensuring that there are no references to a protocol interface
/// that has been removed. Some protocols may not be able to be removed as there is no information
/// available regarding the references. This includes Console I/O, Block I/O, Disk I/o, and handles
/// to device protocols.
///
/// The caller is responsible for ensuring that they pass a valid `Guid` for `protocol`.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: the interface was not found on the handle.
/// * [`Status::ACCESS_DENIED`]: the interface is still in use and cannot be uninstalled.
pub unsafe fn uninstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    interface: *const c_void,
) -> Result<()> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    (bt.uninstall_protocol_interface)(handle.as_ptr(), protocol, interface).to_result()
}

/// Registers `event` to be signaled whenever a protocol interface is registered for
/// `protocol` by [`install_protocol_interface`] or [`reinstall_protocol_interface`].
///
/// If successful, a [`SearchType::ByRegisterNotify`] is returned. This can be
/// used with [`locate_handle`] or [`locate_handle_buffer`] to identify the
/// newly (re)installed handles that support `protocol`.
///
/// Events can be unregistered from protocol interface notification by calling [`close_event`].
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: the event could not be allocated.
pub fn register_protocol_notify(
    protocol: &'static Guid,
    event: &Event,
) -> Result<SearchType<'static>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut key = ptr::null();
    unsafe { (bt.register_protocol_notify)(protocol, event.as_ptr(), &mut key) }.to_result_with_val(
        || {
            // OK to unwrap: key is non-null for Status::SUCCESS.
            SearchType::ByRegisterNotify(ProtocolSearchKey(NonNull::new(key.cast_mut()).unwrap()))
        },
    )
}

/// Get the list of protocol interface [`Guids`][Guid] that are installed
/// on a [`Handle`].
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `handle` is invalid.
/// * [`Status::OUT_OF_RESOURCES`]: out of memory.
pub fn protocols_per_handle(handle: Handle) -> Result<ProtocolsPerHandle> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut protocols = ptr::null_mut();
    let mut count = 0;

    unsafe { (bt.protocols_per_handle)(handle.as_ptr(), &mut protocols, &mut count) }
        .to_result_with_val(|| ProtocolsPerHandle {
            count,
            protocols: NonNull::new(protocols)
                .expect("protocols_per_handle must not return a null pointer"),
        })
}

/// Locates the handle of a device on the device path that supports the specified protocol.
///
/// The `device_path` is updated to point at the remaining part of the [`DevicePath`] after
/// the part that matched the protocol. For example, it can be used with a device path
/// that contains a file path to strip off the file system portion of the device path,
/// leaving the file path and handle to the file system driver needed to access the file.
///
/// If the first node of `device_path` matches the protocol, the `device_path`
/// is advanced to the device path terminator node. If `device_path` is a
/// multi-instance device path, the function will operate on the first instance.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handles.
pub fn locate_device_path<P: ProtocolPointer + ?Sized>(
    device_path: &mut &DevicePath,
) -> Result<Handle> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut handle = ptr::null_mut();
    let mut device_path_ptr: *const uefi_raw::protocol::device_path::DevicePathProtocol =
        device_path.as_ffi_ptr().cast();
    unsafe {
        (bt.locate_device_path)(&P::GUID, &mut device_path_ptr, &mut handle).to_result_with_val(
            || {
                *device_path = DevicePath::from_ffi_ptr(device_path_ptr.cast());
                // OK to unwrap: handle is non-null for Status::SUCCESS.
                Handle::from_ptr(handle).unwrap()
            },
        )
    }
}

/// Enumerates all handles installed on the system which match a certain query.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handles found.
/// * [`Status::BUFFER_TOO_SMALL`]: the buffer is not large enough. The required
///   size (in number of handles, not bytes) will be returned in the error data.
pub fn locate_handle<'buf>(
    search_ty: SearchType,
    buffer: &'buf mut [MaybeUninit<Handle>],
) -> Result<&'buf [Handle], Option<usize>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    // Obtain the needed data from the parameters.
    let (ty, guid, key) = match search_ty {
        SearchType::AllHandles => (0, ptr::null(), ptr::null()),
        SearchType::ByRegisterNotify(registration) => {
            (1, ptr::null(), registration.0.as_ptr().cast_const())
        }
        SearchType::ByProtocol(guid) => (2, guid as *const Guid, ptr::null()),
    };

    let mut buffer_size = buffer.len() * mem::size_of::<Handle>();
    let status =
        unsafe { (bt.locate_handle)(ty, guid, key, &mut buffer_size, buffer.as_mut_ptr().cast()) };

    let num_handles = buffer_size / mem::size_of::<Handle>();

    match status {
        Status::SUCCESS => {
            let buffer = &buffer[..num_handles];
            // SAFETY: the entries up to `num_handles` have been initialized.
            let handles = unsafe { maybe_uninit_slice_assume_init_ref(buffer) };
            Ok(handles)
        }
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(num_handles))),
        _ => Err(Error::new(status, None)),
    }
}

/// Returns an array of handles that support the requested protocol in a
/// pool-allocated buffer.
///
/// See [`SearchType`] for details of the available search operations.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handles.
/// * [`Status::OUT_OF_RESOURCES`]: out of memory.
pub fn locate_handle_buffer(search_ty: SearchType) -> Result<HandleBuffer> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (ty, guid, key) = match search_ty {
        SearchType::AllHandles => (0, ptr::null(), ptr::null()),
        SearchType::ByRegisterNotify(registration) => {
            (1, ptr::null(), registration.0.as_ptr().cast_const())
        }
        SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null()),
    };

    let mut num_handles: usize = 0;
    let mut buffer: *mut uefi_raw::Handle = ptr::null_mut();
    unsafe { (bt.locate_handle_buffer)(ty, guid, key, &mut num_handles, &mut buffer) }
        .to_result_with_val(|| HandleBuffer {
            count: num_handles,
            buffer: NonNull::new(buffer.cast())
                .expect("locate_handle_buffer must not return a null pointer"),
        })
}

/// Returns all the handles implementing a certain protocol.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handles.
#[cfg(feature = "alloc")]
pub fn find_handles<P: ProtocolPointer + ?Sized>() -> Result<Vec<Handle>> {
    // Search by protocol.
    let search_type = SearchType::from_proto::<P>();

    // Determine how much we need to allocate.
    let num_handles = match locate_handle(search_type, &mut []) {
        Err(err) => {
            if err.status() == Status::BUFFER_TOO_SMALL {
                err.data().expect("error data is missing")
            } else {
                return Err(err.to_err_without_payload());
            }
        }
        // This should never happen: if no handles match the search then a
        // `NOT_FOUND` error should be returned.
        Ok(_) => panic!("locate_handle should not return success with empty buffer"),
    };

    // Allocate a large enough buffer without pointless initialization.
    let mut handles = Vec::with_capacity(num_handles);

    // Perform the search.
    let num_handles = locate_handle(search_type, handles.spare_capacity_mut())
        .discard_errdata()?
        .len();

    // Mark the returned number of elements as initialized.
    unsafe {
        handles.set_len(num_handles);
    }

    // Emit output, with warnings
    Ok(handles)
}

/// Find an arbitrary handle that supports a particular [`Protocol`]. Returns
/// [`NOT_FOUND`] if no handles support the protocol.
///
/// This method is a convenient wrapper around [`locate_handle_buffer`] for
/// getting just one handle. This is useful when you don't care which handle the
/// protocol is opened on. For example, [`DevicePathToText`] isn't tied to a
/// particular device, so only a single handle is expected to exist.
///
/// [`NOT_FOUND`]: Status::NOT_FOUND
/// [`DevicePathToText`]: uefi::proto::device_path::text::DevicePathToText
///
/// # Example
///
/// ```
/// use uefi::proto::device_path::text::DevicePathToText;
/// use uefi::{boot, Handle};
/// # use uefi::Result;
///
/// # fn get_fake_val<T>() -> T { todo!() }
/// # fn test() -> Result {
/// # let image_handle: Handle = get_fake_val();
/// let handle = boot::get_handle_for_protocol::<DevicePathToText>()?;
/// let device_path_to_text = boot::open_protocol_exclusive::<DevicePathToText>(handle)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handle.
/// * [`Status::OUT_OF_RESOURCES`]: out of memory.
pub fn get_handle_for_protocol<P: ProtocolPointer + ?Sized>() -> Result<Handle> {
    locate_handle_buffer(SearchType::ByProtocol(&P::GUID))?
        .first()
        .cloned()
        .ok_or_else(|| Status::NOT_FOUND.into())
}

/// Opens a protocol interface for a handle.
///
/// See also [`open_protocol_exclusive`], which provides a safe subset of this
/// functionality.
///
/// This function attempts to get the protocol implementation of a handle, based
/// on the [protocol GUID].
///
/// See [`OpenProtocolParams`] and [`OpenProtocolAttributes`] for details of the
/// input parameters.
///
/// If successful, a [`ScopedProtocol`] is returned that will automatically
/// close the protocol interface when dropped.
///
/// [protocol GUID]: uefi::data_types::Identify::GUID
///
/// # Safety
///
/// This function is unsafe because it can be used to open a protocol in ways
/// that don't get tracked by the UEFI implementation. This could allow the
/// protocol to be removed from a handle, or for the handle to be deleted
/// entirely, while a reference to the protocol is still active. The caller is
/// responsible for ensuring that the handle and protocol remain valid until the
/// `ScopedProtocol` is dropped.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: an invalid combination of `params` and
///   `attributes` was provided.
/// * [`Status::UNSUPPORTED`]: the handle does not support the protocol.
/// * [`Status::ACCESS_DENIED`] or [`Status::ALREADY_STARTED`]: the protocol is
///   already open in a way that is incompatible with the new request.
pub unsafe fn open_protocol<P: ProtocolPointer + ?Sized>(
    params: OpenProtocolParams,
    attributes: OpenProtocolAttributes,
) -> Result<ScopedProtocol<P>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut interface = ptr::null_mut();
    (bt.open_protocol)(
        params.handle.as_ptr(),
        &P::GUID,
        &mut interface,
        params.agent.as_ptr(),
        Handle::opt_to_ptr(params.controller),
        attributes as u32,
    )
    .to_result_with_val(|| ScopedProtocol {
        interface: NonNull::new(P::mut_ptr_from_ffi(interface)),
        open_params: params,
    })
}

/// Opens a protocol interface for a handle in exclusive mode.
///
/// If successful, a [`ScopedProtocol`] is returned that will automatically
/// close the protocol interface when dropped.
///
/// # Errors
///
/// * [`Status::UNSUPPORTED`]: the handle does not support the protocol.
/// * [`Status::ACCESS_DENIED`]: the protocol is already open in a way that is
///   incompatible with the new request.
pub fn open_protocol_exclusive<P: ProtocolPointer + ?Sized>(
    handle: Handle,
) -> Result<ScopedProtocol<P>> {
    // Safety: opening in exclusive mode with the correct agent
    // handle set ensures that the protocol cannot be modified or
    // removed while it is open, so this usage is safe.
    unsafe {
        open_protocol::<P>(
            OpenProtocolParams {
                handle,
                agent: image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
    }
}

/// Tests whether a handle supports a protocol.
///
/// Returns `Ok(true)` if the handle supports the protocol, `Ok(false)` if not.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: one of the handles in `params` is invalid.
pub fn test_protocol<P: ProtocolPointer + ?Sized>(params: OpenProtocolParams) -> Result<bool> {
    const TEST_PROTOCOL: u32 = 0x04;

    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut interface = ptr::null_mut();
    let status = unsafe {
        (bt.open_protocol)(
            params.handle.as_ptr(),
            &P::GUID,
            &mut interface,
            params.agent.as_ptr(),
            Handle::opt_to_ptr(params.controller),
            TEST_PROTOCOL,
        )
    };

    match status {
        Status::SUCCESS => Ok(true),
        Status::UNSUPPORTED => Ok(false),
        _ => Err(Error::from(status)),
    }
}

/// Loads a UEFI image into memory and return a [`Handle`] to the image.
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
/// [`LoadedImage`] and [`LoadedImageDevicePath`] protocols is returned. The
/// image can be started with [`start_image`] and unloaded with
/// [`unload_image`].
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `source` contains an invalid value.
/// * [`Status::UNSUPPORTED`]: the image type is not supported.
/// * [`Status::OUT_OF_RESOURCES`]: insufficient resources to load the image.
/// * [`Status::LOAD_ERROR`]: the image is invalid.
/// * [`Status::DEVICE_ERROR`]: failed to load image due to a read error.
/// * [`Status::ACCESS_DENIED`]: failed to load image due to a security policy.
/// * [`Status::SECURITY_VIOLATION`]: a security policy specifies that the image
///   should not be started.
pub fn load_image(parent_image_handle: Handle, source: LoadImageSource) -> Result<Handle> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (boot_policy, device_path, source_buffer, source_size) = source.to_ffi_params();

    let mut image_handle = ptr::null_mut();
    unsafe {
        (bt.load_image)(
            boot_policy.into(),
            parent_image_handle.as_ptr(),
            device_path.cast(),
            source_buffer,
            source_size,
            &mut image_handle,
        )
        .to_result_with_val(
            // OK to unwrap: image handle is non-null for Status::SUCCESS.
            || Handle::from_ptr(image_handle).unwrap(),
        )
    }
}

/// Unloads a UEFI image.
///
/// # Errors
///
/// * [`Status::UNSUPPORTED`]: the image has been started, and does not support unload.
/// * [`Status::INVALID_PARAMETER`]: `image_handle` is not valid.
pub fn unload_image(image_handle: Handle) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe { (bt.unload_image)(image_handle.as_ptr()) }.to_result()
}

/// Transfers control to a loaded image's entry point.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `image_handle` is not valid, or the image
///   has already been initialized with `start_image`.
/// * [`Status::SECURITY_VIOLATION`]: a security policy specifies that the image
///   should not be started.
pub fn start_image(image_handle: Handle) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    // TODO: implement returning exit data to the caller.
    let mut exit_data_size: usize = 0;
    let mut exit_data: *mut u16 = ptr::null_mut();

    unsafe {
        (bt.start_image)(image_handle.as_ptr(), &mut exit_data_size, &mut exit_data).to_result()
    }
}

/// Exits the UEFI application and returns control to the UEFI component
/// that started the UEFI application.
///
/// # Safety
///
/// The caller must ensure that resources owned by the application are properly
/// cleaned up.
///
/// Note that event callbacks installed by the application are not automatically
/// uninstalled. If such a callback is invoked after exiting the application,
/// the function's code may no longer be loaded in memory, leading to a crash or
/// other unexpected behavior.
pub unsafe fn exit(
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut Char16,
) -> ! {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    (bt.exit)(
        image_handle.as_ptr(),
        exit_status,
        exit_data_size,
        exit_data.cast(),
    )
}

/// Get the current memory map and exit boot services.
unsafe fn get_memory_map_and_exit_boot_services(buf: &mut [u8]) -> Result<MemoryMapMeta> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    // Get the memory map.
    let memory_map = get_memory_map(buf)?;

    // Try to exit boot services using the memory map key. Note that after
    // the first call to `exit_boot_services`, there are restrictions on
    // what boot services functions can be called. In UEFI 2.8 and earlier,
    // only `get_memory_map` and `exit_boot_services` are allowed. Starting
    // in UEFI 2.9 other memory allocation functions may also be called.
    (bt.exit_boot_services)(image_handle().as_ptr(), memory_map.map_key.0)
        .to_result_with_val(|| memory_map)
}

/// Exit UEFI boot services.
///
/// After this function completes, UEFI hands over control of the hardware
/// to the executing OS loader, which implies that the UEFI boot services
/// are shut down and cannot be used anymore. Only UEFI configuration tables
/// and run-time services can be used.
///
/// The memory map at the time of exiting boot services returned. The map is
/// backed by a pool allocation of the given `memory_type`. Since the boot
/// services function to free that memory is no longer available after calling
/// `exit_boot_services`, the allocation will not be freed on drop.
///
/// Note that once the boot services are exited, associated loggers and
/// allocators can't use the boot services anymore. For the corresponding
/// abstractions provided by this crate (see the [`helpers`] module),
/// invoking this function will automatically disable them. If the
/// `global_allocator` feature is enabled, attempting to use the allocator
/// after exiting boot services will panic.
///
/// # Safety
///
/// The caller is responsible for ensuring that no references to
/// boot-services data remain. A non-exhaustive list of resources to check:
///
/// * All protocols will be invalid after exiting boot services. This
///   includes the [`Output`] protocols attached to stdout/stderr. The
///   caller must ensure that no protocol references remain.
/// * The pool allocator is not usable after exiting boot services. Types
///   such as [`PoolString`] which call [`free_pool`] on drop
///   must be cleaned up before calling `exit_boot_services`, or leaked to
///   avoid drop ever being called.
/// * All data in the memory map marked as
///   [`MemoryType::BOOT_SERVICES_CODE`] and
///   [`MemoryType::BOOT_SERVICES_DATA`] will become free memory.
///
/// # Errors
///
/// This function will fail if it is unable to allocate memory for
/// the memory map, if it fails to retrieve the memory map, or if
/// exiting boot services fails (with up to one retry).
///
/// All errors are treated as unrecoverable because the system is
/// now in an undefined state. Rather than returning control to the
/// caller, the system will be reset.
///
/// [`helpers`]: crate::helpers
/// [`Output`]: crate::proto::console::text::Output
/// [`PoolString`]: crate::proto::device_path::text::PoolString
#[must_use]
pub unsafe fn exit_boot_services(memory_type: MemoryType) -> MemoryMapOwned {
    crate::helpers::exit();

    let mut buf = MemoryMapBackingMemory::new(memory_type).expect("Failed to allocate memory");

    // Calling `exit_boot_services` can fail if the memory map key is not
    // current. Retry a second time if that occurs. This matches the
    // behavior of the Linux kernel:
    // https://github.com/torvalds/linux/blob/e544a0743/drivers/firmware/efi/libstub/efi-stub-helper.c#L375
    let mut status = Status::ABORTED;
    for _ in 0..2 {
        match unsafe { get_memory_map_and_exit_boot_services(buf.as_mut_slice()) } {
            Ok(memory_map) => {
                return MemoryMapOwned::from_initialized_mem(buf, memory_map);
            }
            Err(err) => {
                log::error!("Error retrieving the memory map for exiting the boot services");
                status = err.status()
            }
        }
    }

    // Failed to exit boot services.
    log::warn!("Resetting the machine");
    runtime::reset(ResetType::COLD, status, None);
}

/// Adds, updates, or removes a configuration table entry
/// from the EFI System Table.
///
/// # Safety
///
/// When installing or updating a configuration table, the data pointed to by
/// `table_ptr` must be a pool allocation of type
/// [`RUNTIME_SERVICES_DATA`]. Once this table has been installed, the caller
/// should not modify or free the data.
///
/// [`RUNTIME_SERVICES_DATA`]: MemoryType::RUNTIME_SERVICES_DATA
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: tried to delete a nonexistent entry.
/// * [`Status::OUT_OF_RESOURCES`]: out of memory.
pub unsafe fn install_configuration_table(
    guid_entry: &'static Guid,
    table_ptr: *const c_void,
) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    (bt.install_configuration_table)(guid_entry, table_ptr).to_result()
}

/// Sets the watchdog timer.
///
/// UEFI will start a 5-minute countdown after an UEFI image is loaded.  The
/// image must either successfully load an OS and exit boot services in that
/// time, or disable the watchdog.
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
/// If provided, the watchdog data must be a null-terminated string optionally
/// followed by other binary data.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `watchdog_code` is invalid.
/// * [`Status::UNSUPPORTED`]: the system does not have a watchdog timer.
/// * [`Status::DEVICE_ERROR`]: the watchdog timer could not be set due to a
///   hardware error.
pub fn set_watchdog_timer(
    timeout_in_seconds: usize,
    watchdog_code: u64,
    data: Option<&mut [u16]>,
) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (data_len, data) = data
        .map(|d| {
            assert!(
                d.contains(&0),
                "Watchdog data must start with a null-terminated string"
            );
            (d.len(), d.as_mut_ptr())
        })
        .unwrap_or((0, ptr::null_mut()));

    unsafe { (bt.set_watchdog_timer)(timeout_in_seconds, watchdog_code, data_len, data) }
        .to_result()
}

/// Stalls execution for the given number of microseconds.
pub fn stall(microseconds: usize) {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe {
        // No error conditions are defined in the spec for this function, so
        // ignore the status.
        let _ = (bt.stall)(microseconds);
    }
}

/// Retrieves a [`SimpleFileSystem`] protocol associated with the device the given
/// image was loaded from.
///
/// # Errors
///
/// This function can return errors from [`open_protocol_exclusive`] and
/// [`locate_device_path`]. See those functions for more details.
///
/// * [`Status::INVALID_PARAMETER`]
/// * [`Status::UNSUPPORTED`]
/// * [`Status::ACCESS_DENIED`]
/// * [`Status::ALREADY_STARTED`]
/// * [`Status::NOT_FOUND`]
pub fn get_image_file_system(image_handle: Handle) -> Result<ScopedProtocol<SimpleFileSystem>> {
    let loaded_image = open_protocol_exclusive::<LoadedImage>(image_handle)?;

    let device_handle = loaded_image
        .device()
        .ok_or(Error::new(Status::UNSUPPORTED, ()))?;
    let device_path = open_protocol_exclusive::<DevicePath>(device_handle)?;

    let device_handle = locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

    open_protocol_exclusive(device_handle)
}

/// Protocol interface [`Guids`][Guid] that are installed on a [`Handle`] as
/// returned by [`protocols_per_handle`].
#[derive(Debug)]
pub struct ProtocolsPerHandle {
    protocols: NonNull<*const Guid>,
    count: usize,
}

impl Drop for ProtocolsPerHandle {
    fn drop(&mut self) {
        let _ = unsafe { free_pool(self.protocols.cast::<u8>()) };
    }
}

impl Deref for ProtocolsPerHandle {
    type Target = [&'static Guid];

    fn deref(&self) -> &Self::Target {
        let ptr: *const &'static Guid = self.protocols.as_ptr().cast();

        // SAFETY:
        //
        // * The firmware is assumed to provide a correctly-aligned pointer and
        //   array length.
        // * The firmware is assumed to provide valid GUID pointers.
        // * Protocol GUIDs should be constants or statics, so a 'static
        //   lifetime (of the individual pointers, not the overall slice) can be
        //   assumed.
        unsafe { slice::from_raw_parts(ptr, self.count) }
    }
}

/// A buffer returned by [`locate_handle_buffer`] that contains an array of
/// [`Handle`]s that support the requested protocol.
#[derive(Debug, Eq, PartialEq)]
pub struct HandleBuffer {
    count: usize,
    buffer: NonNull<Handle>,
}

impl Drop for HandleBuffer {
    fn drop(&mut self) {
        let _ = unsafe { free_pool(self.buffer.cast::<u8>()) };
    }
}

impl Deref for HandleBuffer {
    type Target = [Handle];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buffer.as_ptr(), self.count) }
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
/// [`DevicePath`]: crate::proto::device_path::DevicePath
/// [`LoadedImageDevicePath`]: crate::proto::device_path::LoadedImageDevicePath
/// [`get`]: ScopedProtocol::get
/// [`get_mut`]: ScopedProtocol::get_mut
#[derive(Debug)]
pub struct ScopedProtocol<P: Protocol + ?Sized> {
    /// The protocol interface.
    interface: Option<NonNull<P>>,
    open_params: OpenProtocolParams,
}

impl<P: Protocol + ?Sized> Drop for ScopedProtocol<P> {
    fn drop(&mut self) {
        let bt = boot_services_raw_panicking();
        let bt = unsafe { bt.as_ref() };

        let status = unsafe {
            (bt.close_protocol)(
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

impl<P: Protocol + ?Sized> Deref for ScopedProtocol<P> {
    type Target = P;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        unsafe { self.interface.unwrap().as_ref() }
    }
}

impl<P: Protocol + ?Sized> DerefMut for ScopedProtocol<P> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.interface.unwrap().as_mut() }
    }
}

impl<P: Protocol + ?Sized> ScopedProtocol<P> {
    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get(&self) -> Option<&P> {
        self.interface.map(|p| unsafe { p.as_ref() })
    }

    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get_mut(&mut self) -> Option<&mut P> {
        self.interface.map(|mut p| unsafe { p.as_mut() })
    }
}

/// RAII guard for task priority level changes.
///
/// Will automatically restore the former task priority level when dropped.
#[derive(Debug)]
pub struct TplGuard {
    old_tpl: Tpl,
}

impl Drop for TplGuard {
    fn drop(&mut self) {
        let bt = boot_services_raw_panicking();
        let bt = unsafe { bt.as_ref() };

        unsafe {
            (bt.restore_tpl)(self.old_tpl);
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

/// Attributes for [`open_protocol`].
#[repr(u32)]
#[derive(Debug)]
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

/// Parameters passed to [`open_protocol`].
#[derive(Debug)]
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

/// Used as a parameter of [`load_image`] to provide the image source.
#[derive(Debug)]
pub enum LoadImageSource<'a> {
    /// Load an image from a buffer. The data will be copied from the
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
    /// behavior depends on [`BootPolicy`]. If [`BootPolicy::BootSelection`],
    /// attempt to load via the [`LoadFile`] protocol. If
    /// [`BootPolicy::ExactMatch`], attempt to load via the [`LoadFile2`]
    /// protocol, then fall back to [`LoadFile`].
    ///
    /// [`LoadFile`]: crate::proto::media::load_file::LoadFile
    /// [`LoadFile2`]: crate::proto::media::load_file::LoadFile2
    FromDevicePath {
        /// The full device path from which to load the image.
        ///
        /// The provided path should be a full device path and not just the
        /// file path portion of it. So for example, it must be (the binary
        /// representation)
        /// `PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)/HD(1,MBR,0xBE1AFDFA,0x3F,0xFBFC1)/\\EFI\\BOOT\\BOOTX64.EFI`
        /// and not just `\\EFI\\BOOT\\BOOTX64.EFI`.
        device_path: &'a DevicePath,

        /// The [`BootPolicy`] to use.
        boot_policy: BootPolicy,
    },
}

impl<'a> LoadImageSource<'a> {
    /// Returns the raw FFI parameters for `load_image`.
    #[must_use]
    pub(crate) fn to_ffi_params(
        &self,
    ) -> (
        BootPolicy,
        *const FfiDevicePath,
        *const u8, /* buffer */
        usize,     /* buffer length */
    ) {
        let boot_policy;
        let device_path;
        let source_buffer;
        let source_size;
        match self {
            LoadImageSource::FromBuffer { buffer, file_path } => {
                // Boot policy is ignored when loading from source buffer.
                boot_policy = BootPolicy::default();

                device_path = file_path.map(|p| p.as_ffi_ptr()).unwrap_or(ptr::null());
                source_buffer = buffer.as_ptr();
                source_size = buffer.len();
            }
            LoadImageSource::FromDevicePath {
                device_path: d_path,
                boot_policy: b_policy,
            } => {
                boot_policy = *b_policy;
                device_path = d_path.as_ffi_ptr();
                source_buffer = ptr::null();
                source_size = 0;
            }
        };
        (boot_policy, device_path, source_buffer, source_size)
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
    /// Return all handles that implement a protocol when an interface for that protocol
    /// is (re)installed.
    ByRegisterNotify(ProtocolSearchKey),
}

impl<'guid> SearchType<'guid> {
    /// Constructs a new search type for a specified protocol.
    #[must_use]
    pub const fn from_proto<P: ProtocolPointer + ?Sized>() -> Self {
        SearchType::ByProtocol(&P::GUID)
    }
}

/// Event notification callback type.
pub type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: Option<NonNull<c_void>>);

/// Timer events manipulation.
#[derive(Debug)]
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

/// Opaque pointer returned by [`register_protocol_notify`] to be used
/// with [`locate_handle`] via [`SearchType::ByRegisterNotify`].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProtocolSearchKey(pub(crate) NonNull<c_void>);
