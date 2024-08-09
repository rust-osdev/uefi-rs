//! UEFI boot services.
//!
//! These functions will panic if called after exiting boot services.

use crate::data_types::PhysicalAddress;
use crate::proto::device_path::DevicePath;
use crate::proto::{Protocol, ProtocolPointer};
use crate::util::opt_nonnull_to_ptr;
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, slice};
use uefi::{table, Char16, Event, Guid, Handle, Result, Status, StatusExt};
use uefi_raw::table::boot::InterfaceType;

#[cfg(doc)]
use {
    crate::proto::device_path::LoadedImageDevicePath, crate::proto::loaded_image::LoadedImage,
    crate::proto::media::fs::SimpleFileSystem,
};

pub use uefi::table::boot::{
    AllocateType, EventNotifyFn, LoadImageSource, OpenProtocolAttributes, OpenProtocolParams,
    SearchType, TimerTrigger,
};
pub use uefi_raw::table::boot::{EventType, MemoryType, Tpl};

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
/// If `event` was registered with `register_protocol_notify`, then the
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
        LoadImageSource::FromDevicePath {
            device_path: file_path,
            from_boot_manager,
        } => {
            boot_policy = u8::from(from_boot_manager);
            device_path = file_path.as_ffi_ptr();
            source_buffer = ptr::null();
            source_size = 0;
        }
    };

    let mut image_handle = ptr::null_mut();
    unsafe {
        (bt.load_image)(
            boot_policy,
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
