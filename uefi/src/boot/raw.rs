#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::{self, MaybeUninit};
use core::ops::Deref;
use core::ptr::{self, NonNull};
use core::slice;

use uefi_raw::table::boot::{EventType, InterfaceType, MemoryDescriptor, MemoryType, Tpl};
use uefi_raw::table::Revision;
use uefi_raw::{PhysicalAddress, Status};
use uguid::Guid;

use crate::data_types::Align;
use crate::proto::device_path::DevicePath;
use crate::proto::loaded_image::LoadedImage;
use crate::proto::media::fs::SimpleFileSystem;
use crate::proto::ProtocolPointer;
use crate::table::boot::{
    AllocateType, EventNotifyFn, LoadImageSource, MemoryMap, MemoryMapKey, MemoryMapSize,
    OpenProtocolAttributes, OpenProtocolParams, ProtocolSearchKey, SearchType, TimerTrigger,
};
use crate::util::opt_nonnull_to_ptr;
use crate::{Char16, Error, Event, Handle, Result, StatusExt};

use super::{
    image_handle, BootHandle, HandleBuffer, MaybeBootRef, ProtocolsPerHandle, ScopedProtocol,
    TplGuard,
};

pub(super) unsafe fn raise_tpl_raw(boot_handle: MaybeBootRef, tpl: Tpl) -> TplGuard {
    let boot_services = unsafe { boot_handle.0.as_ref() };

    let old_tpl = unsafe { (boot_services.raise_tpl)(tpl) };

    TplGuard {
        old_tpl,
        boot_handle,
    }
}

pub(super) fn allocate_pages_raw(
    boot_handle: &BootHandle,
    ty: AllocateType,
    mem_ty: MemoryType,
    count: usize,
) -> Result<PhysicalAddress> {
    let (ty, mut addr) = match ty {
        AllocateType::AnyPages => (0, 0),
        AllocateType::MaxAddress(addr) => (1, addr),
        AllocateType::Address(addr) => (2, addr),
    };

    unsafe { (boot_handle.0.as_ref().allocate_pages)(ty, mem_ty, count, &mut addr) }
        .to_result_with_val(|| addr)
}

pub(super) unsafe fn free_pages_raw(
    boot_handle: &BootHandle,
    addr: PhysicalAddress,
    count: usize,
) -> Result {
    unsafe { ((*boot_handle.0.as_ptr()).free_pages)(addr, count) }.to_result()
}

pub(super) fn memory_map_size_raw(boot_handle: &BootHandle) -> MemoryMapSize {
    let mut map_size = 0;
    let mut map_key = MemoryMapKey(0);
    let mut entry_size = 0;
    let mut entry_version = 0;

    let status = unsafe {
        ((boot_handle.0.as_ref()).get_memory_map)(
            &mut map_size,
            ptr::null_mut(),
            &mut map_key.0,
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

pub(super) fn memory_map_raw<'buf>(
    boot_handle: &BootHandle,
    buffer: &'buf mut [u8],
) -> Result<MemoryMap<'buf>> {
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
        ((boot_handle.0.as_ref()).get_memory_map)(
            &mut map_size,
            map_buffer,
            &mut map_key.0,
            &mut entry_size,
            &mut entry_version,
        )
    }
    .to_result_with_val(move || {
        let len = map_size / entry_size;

        MemoryMap {
            key: map_key,
            buf: buffer,
            entry_size,
            len,
        }
    })
}

pub(super) fn allocate_pool_raw(
    boot_handle: &BootHandle,
    mem_ty: MemoryType,
    size: usize,
) -> Result<*mut u8> {
    let mut buffer = ptr::null_mut();
    unsafe { ((boot_handle.0.as_ref()).allocate_pool)(mem_ty, size, &mut buffer) }
        .to_result_with_val(|| buffer)
}

pub(super) unsafe fn free_pool_raw(boot_handle: &BootHandle, addr: *mut u8) -> Result {
    unsafe { (boot_handle.0.as_ref().free_pool)(addr) }.to_result()
}

pub(super) unsafe fn create_event_raw(
    boot_handle: &BootHandle,
    event_ty: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
) -> Result<Event> {
    let mut event = ptr::null_mut();

    let notify_ctx = opt_nonnull_to_ptr(notify_ctx);

    // Safety: the argument types of the function pointers are defined
    // differently, but are compatible and can be safely transmuted.
    let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

    // Now we're ready to call UEFI
    (boot_handle.0.as_ref().create_event)(event_ty, notify_tpl, notify_fn, notify_ctx, &mut event)
        .to_result_with_val(
            // OK to unwrap: event is non-null for Status::SUCCESS.
            || Event::from_ptr(event).unwrap(),
        )
}

pub(super) unsafe fn create_event_ex_raw(
    boot_handle: &BootHandle,
    event_type: EventType,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: Option<NonNull<c_void>>,
    event_group: Option<NonNull<Guid>>,
) -> Result<Event> {
    if boot_handle.0.as_ref().header.revision < Revision::EFI_2_00 {
        return Err(Status::UNSUPPORTED.into());
    }

    let mut event = ptr::null_mut();

    // Safety: the argument types of the function pointers are defined
    // differently, but are compatible and can be safely transmuted.
    let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

    (boot_handle.0.as_ref().create_event_ex)(
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

pub(super) fn set_timer_raw(
    boot_handle: &BootHandle,
    event: &Event,
    trigger_time: TimerTrigger,
) -> Result {
    let (ty, time) = match trigger_time {
        TimerTrigger::Cancel => (0, 0),
        TimerTrigger::Periodic(hundreds_ns) => (1, hundreds_ns),
        TimerTrigger::Relative(hundreds_ns) => (2, hundreds_ns),
    };
    unsafe { (boot_handle.0.as_ref().set_timer)(event.as_ptr(), ty, time) }.to_result()
}

pub(super) fn wait_for_event_raw(
    boot_handle: &BootHandle,
    events: &mut [Event],
) -> Result<usize, Option<usize>> {
    let number_of_events = events.len();
    let events: *mut uefi_raw::Event = events.as_mut_ptr().cast();

    let mut index = 0;
    unsafe { (boot_handle.0.as_ref().wait_for_event)(number_of_events, events, &mut index) }
        .to_result_with(
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

pub(super) fn signal_event_raw(boot_handle: &BootHandle, event: &Event) -> Result {
    // Safety: cloning this event should be safe, as we're directly passing it to firmware
    // and not keeping the clone around.
    unsafe { (boot_handle.0.as_ref().signal_event)(event.as_ptr()).to_result() }
}

pub(super) fn close_event_raw(boot_handle: &BootHandle, event: Event) -> Result {
    unsafe { (boot_handle.0.as_ref().close_event)(event.as_ptr()).to_result() }
}

pub(super) fn check_event_raw(boot_handle: &BootHandle, event: Event) -> Result<bool> {
    let status = unsafe { (boot_handle.0.as_ref().check_event)(event.as_ptr()) };
    match status {
        Status::SUCCESS => Ok(true),
        Status::NOT_READY => Ok(false),
        _ => Err(status.into()),
    }
}

pub(super) unsafe fn install_protocol_interface_raw(
    boot_handle: &BootHandle,
    handle: Option<Handle>,
    protocol: &Guid,
    interface: *mut c_void,
) -> Result<Handle> {
    let mut handle = Handle::opt_to_ptr(handle);
    ((boot_handle.0.as_ref().install_protocol_interface)(
        &mut handle,
        protocol,
        InterfaceType::NATIVE_INTERFACE,
        interface,
    ))
    .to_result_with_val(|| Handle::from_ptr(handle).unwrap())
}

pub(super) unsafe fn reinstall_protocol_interface_raw(
    boot_handle: &BootHandle,
    handle: Handle,
    protocol: &Guid,
    old_interface: *mut c_void,
    new_interface: *mut c_void,
) -> Result<()> {
    (boot_handle.0.as_ref().reinstall_protocol_interface)(
        handle.as_ptr(),
        protocol,
        old_interface,
        new_interface,
    )
    .to_result()
}

pub(super) unsafe fn uninstall_protocol_interface_raw(
    boot_handle: &BootHandle,
    handle: Handle,
    protocol: &Guid,
    interface: *mut c_void,
) -> Result<()> {
    (boot_handle.0.as_ref().uninstall_protocol_interface)(handle.as_ptr(), protocol, interface)
        .to_result()
}

pub fn register_protocol_notify_raw<'guid>(
    boot_handle: &BootHandle,
    protocol: &'guid Guid,
    event: Event,
) -> Result<(Event, SearchType<'guid>)> {
    let mut key = ptr::null();
    // Safety: we clone `event` a couple times, but there will be only one left once we return.
    unsafe { (boot_handle.0.as_ref().register_protocol_notify)(protocol, event.as_ptr(), &mut key) }
        // Safety: as long as this call is successful, `key` will be valid.
        .to_result_with_val(|| unsafe {
            (
                event.unsafe_clone(),
                // OK to unwrap: key is non-null for Status::SUCCESS.
                SearchType::ByRegisterNotify(ProtocolSearchKey(
                    NonNull::new(key.cast_mut()).unwrap(),
                )),
            )
        })
}

pub(super) fn locate_handle_raw(
    boot_handle: &BootHandle,
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
        SearchType::AllHandles => (0, ptr::null(), ptr::null()),
        SearchType::ByRegisterNotify(registration) => {
            (1, ptr::null(), registration.0.as_ptr().cast_const())
        }
        SearchType::ByProtocol(guid) => (2, guid as *const Guid, ptr::null()),
    };

    let status = unsafe {
        (boot_handle.0.as_ref().locate_handle)(ty, guid, key, &mut buffer_size, buffer.cast())
    };

    // Must convert the returned size (in bytes) to length (number of elements).
    let buffer_len = buffer_size / handle_size;

    match (buffer, status) {
        (NULL_BUFFER, Status::BUFFER_TOO_SMALL) => Ok(buffer_len),
        (_, other_status) => other_status.to_result_with_val(|| buffer_len),
    }
}

pub(super) fn locate_device_path_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: &BootHandle,
    device_path: &mut &DevicePath,
) -> Result<Handle> {
    let mut handle = ptr::null_mut();
    let mut device_path_ptr: *const uefi_raw::protocol::device_path::DevicePathProtocol =
        device_path.as_ffi_ptr().cast();
    unsafe {
        (boot_handle.0.as_ref().locate_device_path)(&P::GUID, &mut device_path_ptr, &mut handle)
            .to_result_with_val(|| {
                *device_path = DevicePath::from_ffi_ptr(device_path_ptr.cast());
                // OK to unwrap: handle is non-null for Status::SUCCESS.
                Handle::from_ptr(handle).unwrap()
            })
    }
}

pub(super) fn get_handle_for_protocol_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: &BootHandle,
) -> Result<Handle> {
    // Delegate to a non-generic function to potentially reduce code size.
    get_handle_for_protocol_impl(boot_handle, &P::GUID)
}

fn get_handle_for_protocol_impl(boot_handle: &BootHandle, guid: &Guid) -> Result<Handle> {
    locate_handle_buffer_raw(MaybeBootRef::Ref(boot_handle), SearchType::ByProtocol(guid))?
        .first()
        .cloned()
        .ok_or_else(|| Status::NOT_FOUND.into())
}

pub(super) fn load_image_raw(
    boot_handle: &BootHandle,
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
        (boot_handle.0.as_ref().load_image)(
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

pub(super) fn unload_image_raw(boot_handle: &BootHandle, image_handle: Handle) -> Result {
    unsafe { (boot_handle.0.as_ref().unload_image)(image_handle.as_ptr()) }.to_result()
}

pub(super) fn start_image_raw(boot_handle: &BootHandle, image_handle: Handle) -> Result {
    unsafe {
        // TODO: implement returning exit data to the caller.
        let mut exit_data_size: usize = 0;
        let mut exit_data: *mut u16 = ptr::null_mut();
        (boot_handle.0.as_ref().start_image)(
            image_handle.as_ptr(),
            &mut exit_data_size,
            &mut exit_data,
        )
        .to_result()
    }
}

pub(super) unsafe fn exit_raw(
    boot_handle: &BootHandle,
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut Char16,
) -> ! {
    (boot_handle.0.as_ref().exit)(
        image_handle.as_ptr(),
        exit_status,
        exit_data_size,
        exit_data.cast(),
    )
}

pub(super) unsafe fn exit_boot_services_raw(
    boot_handle: &BootHandle,
    image: Handle,
    mmap_key: MemoryMapKey,
) -> Result {
    (boot_handle.0.as_ref().exit_boot_services)(image.as_ptr(), mmap_key.0).to_result()
}

pub(super) fn stall_raw(boot_handle: &BootHandle, time: usize) {
    assert_eq!(
        unsafe { (boot_handle.0.as_ref().stall)(time) },
        Status::SUCCESS
    );
}

pub(super) unsafe fn install_configuration_table_raw(
    boot_handle: &BootHandle,
    guid_entry: &Guid,
    table_ptr: *const c_void,
) -> Result {
    (boot_handle.0.as_ref().install_configuration_table)(guid_entry, table_ptr).to_result()
}

pub(super) fn set_watchdog_timer_raw(
    boot_handle: &BootHandle,
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

    unsafe { (boot_handle.0.as_ref().set_watchdog_timer)(timeout, watchdog_code, data_len, data) }
        .to_result()
}

pub(super) fn connect_controller_raw(
    boot_handle: &BootHandle,
    controller: Handle,
    driver_image: Option<Handle>,
    remaining_device_path: Option<&DevicePath>,
    recursive: bool,
) -> Result {
    unsafe {
        (boot_handle.0.as_ref().connect_controller)(
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

pub(super) fn disconnect_controller_raw(
    boot_handle: &BootHandle,
    controller: Handle,
    driver_image: Option<Handle>,
    child: Option<Handle>,
) -> Result {
    unsafe {
        (boot_handle.0.as_ref().disconnect_controller)(
            controller.as_ptr(),
            Handle::opt_to_ptr(driver_image),
            Handle::opt_to_ptr(child),
        )
    }
    .to_result_with_err(|_| ())
}

pub(super) unsafe fn open_protocol_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: MaybeBootRef,
    params: OpenProtocolParams,
    attributes: OpenProtocolAttributes,
) -> Result<ScopedProtocol<P>> {
    let mut interface = ptr::null_mut();
    (boot_handle.0.as_ref().open_protocol)(
        params.handle.as_ptr(),
        &P::GUID,
        &mut interface,
        params.agent.as_ptr(),
        Handle::opt_to_ptr(params.controller),
        attributes as u32,
    )
    .to_result_with_val(|| {
        let interface = (!interface.is_null()).then(|| {
            let interface = P::mut_ptr_from_ffi(interface) as *const UnsafeCell<P>;
            &*interface
        });

        ScopedProtocol {
            interface,
            open_params: params,
            boot_handle,
        }
    })
}

pub(super) fn open_protocol_exclusive_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: MaybeBootRef,
    handle: Handle,
) -> Result<ScopedProtocol<P>> {
    // Safety: opening in exclusive mode with the correct agent
    // handle set ensures that the protocol cannot be modified or
    // removed while it is open, so this usage is safe.
    unsafe {
        open_protocol_raw::<P>(
            boot_handle,
            OpenProtocolParams {
                handle,
                agent: image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::Exclusive,
        )
    }
}

pub(super) fn test_protocol_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: &BootHandle,
    params: OpenProtocolParams,
) -> Result<()> {
    const TEST_PROTOCOL: u32 = 0x04;
    let mut interface = ptr::null_mut();
    unsafe {
        (boot_handle.0.as_ref().open_protocol)(
            params.handle.as_ptr(),
            &P::GUID,
            &mut interface,
            params.agent.as_ptr(),
            Handle::opt_to_ptr(params.controller),
            TEST_PROTOCOL,
        )
    }
    .to_result_with_val(|| ())
}

pub(super) fn protocols_per_handle_raw(
    boot_handle: MaybeBootRef,
    handle: Handle,
) -> Result<ProtocolsPerHandle> {
    let mut protocols = ptr::null_mut();
    let mut count = 0;

    let mut status = unsafe {
        (boot_handle.0.as_ref().protocols_per_handle)(handle.as_ptr(), &mut protocols, &mut count)
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
        protocols: protocols.cast::<&Guid>(),
        count,
        boot_handle,
    })
}

pub(super) fn locate_handle_buffer_raw<'boof>(
    boot_handle: MaybeBootRef<'boof>,
    search_ty: SearchType,
) -> Result<HandleBuffer<'boof>> {
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
        (boot_handle.0.as_ref().locate_handle_buffer)(ty, guid, key, &mut num_handles, &mut buffer)
    }
    .to_result_with_val(|| HandleBuffer {
        boot_handle,
        count: num_handles,
        buffer: buffer.cast(),
    })
}

pub(super) fn get_image_file_system_raw(
    boot_handle: MaybeBootRef,
    image_handle: Handle,
) -> Result<ScopedProtocol<SimpleFileSystem>> {
    let raw_boot_handle: &BootHandle = boot_handle.deref();

    let loaded_image = open_protocol_exclusive_raw::<LoadedImage>(
        MaybeBootRef::Ref(raw_boot_handle),
        image_handle,
    )?;

    let device_handle = loaded_image
        .device()
        .ok_or(Error::new(Status::UNSUPPORTED, ()))?;

    let device_path = open_protocol_exclusive_raw::<DevicePath>(
        MaybeBootRef::Ref(raw_boot_handle),
        device_handle,
    )?;

    let device_handle =
        locate_device_path_raw::<SimpleFileSystem>(raw_boot_handle, &mut &*device_path)?;

    drop(loaded_image);
    drop(device_path);

    open_protocol_exclusive_raw(boot_handle, device_handle)
}

#[cfg(feature = "alloc")]
pub(super) fn find_handles_raw<P: ProtocolPointer + ?Sized>(
    boot_handle: &BootHandle,
) -> Result<Vec<Handle>> {
    // Search by protocol.
    let search_type = SearchType::from_proto::<P>();

    // Determine how much we need to allocate.
    let buffer_size = locate_handle_raw(boot_handle, search_type, None)?;

    // Allocate a large enough buffer without pointless initialization.
    let mut handles = Vec::with_capacity(buffer_size);
    let buffer = handles.spare_capacity_mut();

    // Perform the search.
    let buffer_size = locate_handle_raw(boot_handle, search_type, Some(buffer))?;

    // Mark the returned number of elements as initialized.
    unsafe {
        handles.set_len(buffer_size);
    }

    // Emit output, with warnings
    Ok(handles)
}
