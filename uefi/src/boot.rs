//! TODO

// TODO
#![allow(clippy::missing_safety_doc)]

use crate::data_types::PhysicalAddress;
use crate::proto::device_path::DevicePath;
use crate::proto::loaded_image::LoadedImage;
use crate::proto::media::fs::SimpleFileSystem;
use crate::proto::{Protocol, ProtocolPointer};
use crate::table::boot::{
    AllocateType, BootServices, EventNotifyFn, EventType, LoadImageSource, MemoryMapOwned,
    MemoryType, OpenProtocolAttributes, OpenProtocolParams, SearchType, TimerTrigger, Tpl,
    IMAGE_HANDLE,
};
use crate::table::{Boot, SystemTable};
use crate::{table, Char16, Error, Event, Guid, Handle, Result, Status, StatusExt};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::slice;
use core::sync::atomic::Ordering;

// TODO
fn boot_services() -> NonNull<BootServices> {
    let st = table::system_table_boot().expect("boot services are not active");
    let ptr: *const _ = st.boot_services();
    NonNull::new(ptr.cast_mut()).unwrap()
}

fn boot_services_raw() -> NonNull<uefi_raw::table::boot::BootServices> {
    // OK to cast: `BootServices` is a `repr(transparent)` wrapper around
    // the raw type.
    boot_services().cast()
}

#[track_caller]
fn stboot() -> SystemTable<Boot> {
    table::system_table_boot().expect("boot services are not available")
}

/// Get the [`Handle`] of the currently-executing image.
pub fn image_handle() -> Handle {
    let ptr = IMAGE_HANDLE.load(Ordering::Acquire);
    // Safety: the image handle must be valid. We know it is, because it was
    // set by `set_image_handle`, which has that same safety requirement.
    unsafe { Handle::from_ptr(ptr) }.expect("set_image_handle has not been called")
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
pub unsafe fn set_image_handle(image_handle: Handle) {
    IMAGE_HANDLE.store(image_handle.as_ptr(), Ordering::Release);
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
    stboot().boot_services().allocate_pages(ty, mem_ty, count)
}

/// Frees memory pages allocated by UEFI.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.FreePages()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn free_pages(addr: PhysicalAddress, count: usize) -> Result {
    stboot().boot_services().free_pages(addr, count)
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
#[must_use]
pub unsafe fn raise_tpl(tpl: Tpl) -> TplGuard {
    TplGuard {
        old_tpl: (boot_services_raw().as_mut().raise_tpl)(tpl),
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.GetMemoryMap()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::BUFFER_TOO_SMALL`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn memory_map(mt: MemoryType) -> Result<MemoryMapOwned> {
    stboot().boot_services().memory_map(mt)
}

/// Allocates from a memory pool. The pointer will be 8-byte aligned.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.AllocatePool()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn allocate_pool(mem_ty: MemoryType, size: usize) -> Result<NonNull<u8>> {
    stboot().boot_services().allocate_pool(mem_ty, size)
}

/// Frees memory allocated from a pool.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.FreePool()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub unsafe fn free_pool(addr: *mut u8) -> Result {
    stboot().boot_services().free_pool(addr)
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
    stboot()
        .boot_services()
        .create_event(event_ty, notify_tpl, notify_fn, notify_ctx)
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
    stboot().boot_services().create_event_ex(
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
    stboot().boot_services().set_timer(event, trigger_time)
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
    stboot().boot_services().wait_for_event(events)
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
    stboot().boot_services().signal_event(event)
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
    stboot().boot_services().close_event(event)
}

/// Checks to see if an event is signaled, without blocking execution to wait for it.
///
/// The returned value will be `true` if the event is in the signaled state,
/// otherwise `false` is returned.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.CheckEvent()` in the UEFI Specification for more details.
///
/// Note: Instead of returning the `EFI_NOT_READY` error, as listed in the UEFI
/// Specification, this function will return `false`.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn check_event(event: Event) -> Result<bool> {
    stboot().boot_services().check_event(event)
}

/// Installs a protocol interface on a device handle. If the inner `Option` in `handle` is `None`,
/// one will be created and added to the list of handles in the system and then returned.
///
/// When a protocol interface is installed, firmware will call all functions that have registered
/// to wait for that interface to be installed.
///
/// # Safety
///
/// The caller is responsible for ensuring that they pass a valid `Guid` for `protocol`.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.InstallProtocolInterface()` in the UEFI Specification for
/// more details.
///
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn install_protocol_interface(
    handle: Option<Handle>,
    protocol: &Guid,
    interface: *mut c_void,
) -> Result<Handle> {
    stboot()
        .boot_services()
        .install_protocol_interface(handle, protocol, interface)
}

/// Reinstalls a protocol interface on a device handle. `old_interface` is replaced with `new_interface`.
/// These interfaces may be the same, in which case the registered protocol notifies occur for the handle
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
/// See section `EFI_BOOT_SERVICES.ReinstallProtocolInterface()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn reinstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    old_interface: *mut c_void,
    new_interface: *mut c_void,
) -> Result<()> {
    stboot().boot_services().reinstall_protocol_interface(
        handle,
        protocol,
        old_interface,
        new_interface,
    )
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
/// See section `EFI_BOOT_SERVICES.UninstallProtocolInterface()` in the UEFI Specification for
/// more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub unsafe fn uninstall_protocol_interface(
    handle: Handle,
    protocol: &Guid,
    interface: *mut c_void,
) -> Result<()> {
    stboot()
        .boot_services()
        .uninstall_protocol_interface(handle, protocol, interface)
}

/// Registers `event` to be signalled whenever a protocol interface is registered for
/// `protocol` by `install_protocol_interface()` or `reinstall_protocol_interface()`.
///
/// Once `event` has been signalled, `BootServices::locate_handle()` can be used to identify
/// the newly (re)installed handles that support `protocol`. The returned `SearchKey` on success
/// corresponds to the `search_key` parameter in `locate_handle()`.
///
/// Events can be unregistered from protocol interface notification by calling `close_event()`.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.RegisterProtocolNotify()` in the UEFI Specification for
/// more details.
///
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn register_protocol_notify(protocol: &Guid, event: Event) -> Result<(Event, SearchType)> {
    stboot()
        .boot_services()
        .register_protocol_notify(protocol, event)
}

/// Enumerates all handles installed on the system which match a certain query.
///
/// You should first call this function with `None` for the output buffer,
/// in order to retrieve the length of the buffer you need to allocate.
///
/// The next call will fill the buffer with the requested data.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.LocateHandle()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::BUFFER_TOO_SMALL`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn locate_handle(
    search_ty: SearchType,
    output: Option<&mut [MaybeUninit<Handle>]>,
) -> Result<usize> {
    stboot().boot_services().locate_handle(search_ty, output)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.LocateDevicePath()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn locate_device_path<P: ProtocolPointer + ?Sized>(
    device_path: &mut &DevicePath,
) -> Result<Handle> {
    stboot()
        .boot_services()
        .locate_device_path::<P>(device_path)
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
///
/// # Errors
///
/// Returns [`NOT_FOUND`] if no handles support the requested protocol.
pub fn get_handle_for_protocol<P: ProtocolPointer + ?Sized>() -> Result<Handle> {
    stboot().boot_services().get_handle_for_protocol::<P>()
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
/// [`LoadedImage`] and [`LoadedImageDevicePath`] protocols is
/// returned. The image can be started with [`start_image`] or
/// unloaded with [`unload_image`].
///
/// [`LoadedImageDevicePath`]: crate::proto::device_path::LoadedImageDevicePath
/// [`start_image`]: BootServices::start_image
/// [`unload_image`]: BootServices::unload_image
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.LoadImage()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::LOAD_ERROR`]
/// * [`uefi::Status::DEVICE_ERROR`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::SECURITY_VIOLATION`]
pub fn load_image(parent_image_handle: Handle, source: LoadImageSource) -> uefi::Result<Handle> {
    stboot()
        .boot_services()
        .load_image(parent_image_handle, source)
}

/// Unload an EFI image.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.UnloadImage()` in the UEFI Specification for more details.
///
/// As this function can return an error code from the unloaded image, any error type
/// can be returned by this function.
///
/// The following error codes can also be returned while unloading an image:
///
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::INVALID_PARAMETER`]
pub fn unload_image(image_handle: Handle) -> Result {
    stboot().boot_services().unload_image(image_handle)
}

/// Transfer control to a loaded image's entry point.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.StartImage()` in the UEFI Specification for more details.
///
/// As this function can return an error code from the started image, any error type
/// can be returned by this function.
///
/// The following error code can also be returned while starting an image:
///
/// * [`uefi::Status::UNSUPPORTED`]
pub fn start_image(image_handle: Handle) -> Result {
    stboot().boot_services().start_image(image_handle)
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
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut Char16,
) -> ! {
    stboot()
        .boot_services()
        .exit(image_handle, exit_status, exit_data_size, exit_data.cast())
}

/// Exit the UEFI boot services.
///
/// After this function completes, UEFI hands over control of the hardware
/// to the executing OS loader, which implies that the UEFI boot services
/// are shut down and cannot be used anymore. Only UEFI configuration tables
/// and run-time services can be used.
///
/// The memory map at the time of exiting boot services is returned. The map is
/// backed by a allocation with given `memory_type`.  Since the boot services
/// function to free that memory is no longer available after calling
/// `exit_boot_services`, the allocation is live until the program ends. The
/// lifetime of the memory map is therefore `'static`.
///
/// Note that once the boot services are exited, associated loggers and
/// allocators can't use the boot services anymore. For the corresponding
/// abstractions provided by this crate, invoking this function will
/// automatically disable them.
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
#[must_use]
pub unsafe fn exit_boot_services(memory_type: MemoryType) -> MemoryMapOwned {
    table::system_table_boot()
        .expect("boot services are not active")
        .exit_boot_services(memory_type)
        .1
}

/// Stalls the processor for an amount of time.
///
/// The time is in microseconds.
pub fn stall(time: usize) {
    stboot().boot_services().stall(time)
}

/// Adds, updates, or removes a configuration table entry
/// from the EFI System Table.
///
/// # Safety
///
/// This relies on `table_ptr` being allocated in the
/// pool of type [`uefi::table::boot::MemoryType::RUNTIME_SERVICES_DATA`]
/// according to the specification.
/// Other memory types such as
/// [`uefi::table::boot::MemoryType::ACPI_RECLAIM`]
/// can be considered.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.InstallConfigurationTable()` in the UEFI
/// Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
pub unsafe fn install_configuration_table(guid_entry: &Guid, table_ptr: *const c_void) -> Result {
    stboot()
        .boot_services()
        .install_configuration_table(guid_entry, table_ptr)
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
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.SetWatchdogTimer()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::DEVICE_ERROR`]
pub fn set_watchdog_timer(timeout: usize, watchdog_code: u64, data: Option<&mut [u16]>) -> Result {
    stboot()
        .boot_services()
        .set_watchdog_timer(timeout, watchdog_code, data)
}

/// Connect one or more drivers to a controller.
///
/// Usually one disconnects and then reconnects certain drivers
/// to make them rescan some state that changed, e.g. reconnecting
/// a `BlockIO` handle after your app changed the partitions somehow.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.ConnectController()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::SECURITY_VIOLATION`]
pub fn connect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    remaining_device_path: Option<&DevicePath>,
    recursive: bool,
) -> Result {
    stboot().boot_services().connect_controller(
        controller,
        driver_image,
        remaining_device_path,
        recursive,
    )
}

/// Disconnect one or more drivers from a controller.
///
/// See [`connect_controller`].
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.DisconnectController()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
/// * [`uefi::Status::DEVICE_ERROR`]
pub fn disconnect_controller(
    controller: Handle,
    driver_image: Option<Handle>,
    child: Option<Handle>,
) -> Result {
    stboot()
        .boot_services()
        .disconnect_controller(controller, driver_image, child)
}

/// Open a protocol interface for a handle.
///
/// See also [`open_protocol_exclusive`], which provides a safe
/// subset of this functionality.
///
/// This function attempts to get the protocol implementation of a
/// handle, based on the protocol GUID. It is recommended that all
/// new drivers and applications use [`open_protocol_exclusive`] or
/// [`open_protocol`].
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
/// [`open_protocol`]: BootServices::open_protocol
/// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.OpenProtocol()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::ALREADY_STARTED`]
pub unsafe fn open_protocol<P: ProtocolPointer + ?Sized>(
    params: OpenProtocolParams,
    attributes: OpenProtocolAttributes,
) -> Result<ScopedProtocol<P>> {
    let mut interface = ptr::null_mut();
    (boot_services_raw().as_mut().open_protocol)(
        params.handle.as_ptr(),
        &P::GUID,
        &mut interface,
        params.agent.as_ptr(),
        Handle::opt_to_ptr(params.controller),
        attributes as u32,
    )
    .to_result_with_val(|| {
        let interface = if interface.is_null() {
            None
        } else {
            Some(P::mut_ptr_from_ffi(interface))
        };

        ScopedProtocol {
            interface,
            open_params: params,
        }
    })
}

/// Open a protocol interface for a handle in exclusive mode.
///
/// If successful, a [`ScopedProtocol`] is returned that will
/// automatically close the protocol interface when dropped.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.OpenProtocol()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::ALREADY_STARTED`]
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

/// Test whether a handle supports a protocol.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.OpenProtocol()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::ALREADY_STARTED`]
pub fn test_protocol<P: ProtocolPointer + ?Sized>(params: OpenProtocolParams) -> Result<()> {
    stboot().boot_services().test_protocol::<P>(params)
}

/// Get the list of protocol interface [`Guids`][Guid] that are installed
/// on a [`Handle`].
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.ProtocolsPerHandle()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
pub fn protocols_per_handle(handle: Handle) -> Result<ProtocolsPerHandle> {
    let mut protocols = ptr::null_mut();
    let mut count = 0;

    let mut status = unsafe {
        (boot_services_raw().as_mut().protocols_per_handle)(
            handle.as_ptr(),
            &mut protocols,
            &mut count,
        )
    };

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

    status.to_result_with_val(|| ProtocolsPerHandle {
        protocols,
        count,
        index: 0,
    })
}

/// Returns an array of handles that support the requested protocol in a buffer allocated from
/// pool.
///
/// # Errors
///
/// See section `EFI_BOOT_SERVICES.LocateHandleBuffer()` in the UEFI Specification for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::NOT_FOUND`]
/// * [`uefi::Status::OUT_OF_RESOURCES`]
pub fn locate_handle_buffer(search_ty: SearchType) -> Result<HandleBuffer> {
    let mut num_handles: usize = 0;
    let mut buffer: *mut uefi_raw::Handle = ptr::null_mut();

    // Obtain the needed data from the parameters.
    let (ty, guid, key) = match search_ty {
        SearchType::AllHandles => (0, ptr::null(), ptr::null()),
        SearchType::ByRegisterNotify(registration) => {
            (1, ptr::null(), registration.0.as_ptr().cast_const())
        }
        SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null()),
    };

    unsafe {
        (boot_services_raw().as_mut().locate_handle_buffer)(
            ty,
            guid,
            key,
            &mut num_handles,
            &mut buffer,
        )
    }
    .to_result_with_val(|| HandleBuffer {
        count: num_handles,
        buffer: buffer.cast(),
    })
}

/// Retrieves a [`SimpleFileSystem`] protocol associated with the device the given
/// image was loaded from.
///
/// # Errors
///
/// This function can return errors from [`open_protocol_exclusive`] and
/// [`locate_device_path`]. See those functions for more details.
///
/// * [`uefi::Status::INVALID_PARAMETER`]
/// * [`uefi::Status::UNSUPPORTED`]
/// * [`uefi::Status::ACCESS_DENIED`]
/// * [`uefi::Status::ALREADY_STARTED`]
/// * [`uefi::Status::NOT_FOUND`]
pub fn get_image_file_system(image_handle: Handle) -> Result<ScopedProtocol<SimpleFileSystem>> {
    let loaded_image = open_protocol_exclusive::<LoadedImage>(image_handle)?;

    let device_handle = loaded_image
        .device()
        .ok_or(Error::new(Status::UNSUPPORTED, ()))?;
    let device_path = open_protocol_exclusive::<DevicePath>(device_handle)?;

    let device_handle = locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

    open_protocol_exclusive(device_handle)
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
/// [`LoadedImageDevicePath`]: crate::proto::device_path::LoadedImageDevicePath
/// [`get`]: ScopedProtocol::get
/// [`get_mut`]: ScopedProtocol::get_mut
#[derive(Debug)]
pub struct ScopedProtocol<P: Protocol + ?Sized> {
    interface: Option<*mut P>,
    open_params: OpenProtocolParams,
}

impl<P: Protocol + ?Sized> Drop for ScopedProtocol<P> {
    fn drop(&mut self) {
        let status = unsafe {
            (boot_services_raw().as_mut().close_protocol)(
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
        unsafe { &*self.interface.unwrap() }
    }
}

impl<P: Protocol + ?Sized> DerefMut for ScopedProtocol<P> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.interface.unwrap() }
    }
}

impl<P: Protocol + ?Sized> ScopedProtocol<P> {
    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get(&self) -> Option<&P> {
        self.interface.map(|p| unsafe { &*p })
    }

    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get_mut(&self) -> Option<&mut P> {
        self.interface.map(|p| unsafe { &mut *p })
    }
}

/// A buffer that contains an array of [`Handles`][Handle] that support the
/// requested protocol. Returned by [`BootServices::locate_handle_buffer`].
#[derive(Debug)]
pub struct HandleBuffer {
    count: usize,
    buffer: *mut Handle,
}

impl Drop for HandleBuffer {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = unsafe { free_pool(self.buffer.cast::<u8>()) };
    }
}

impl Deref for HandleBuffer {
    type Target = [Handle];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buffer, self.count) }
    }
}

/// Protocol interface [`Guids`][Guid] that are installed on a [`Handle`] as
/// returned by [`BootServices::protocols_per_handle`].
#[derive(Debug)]
pub struct ProtocolsPerHandle {
    protocols: *mut *const Guid,
    count: usize,
    index: usize,
}

impl Drop for ProtocolsPerHandle {
    fn drop(&mut self) {
        // Ignore the result, we can't do anything about an error here.
        let _ = unsafe { free_pool(self.protocols.cast::<u8>()) };
    }
}

// TODO: switched this to an iterator instead of Deref, since there's no way for
// Deref to return a reference to self.
impl Iterator for ProtocolsPerHandle {
    type Item = Guid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let protocols = unsafe { slice::from_raw_parts(self.protocols, self.count) };
            let guid = protocols[self.index];
            self.index += 1;
            Some(unsafe { *guid })
        } else {
            None
        }
    }
}

/// RAII guard for task priority level changes
///
/// Will automatically restore the former task priority level when dropped.
#[derive(Debug)]
pub struct TplGuard {
    old_tpl: Tpl,
}

impl Drop for TplGuard {
    fn drop(&mut self) {
        unsafe {
            (boot_services_raw().as_mut().restore_tpl)(self.old_tpl);
        }
    }
}
