//! UEFI services available during boot.

use super::Revision;
use crate::data_types::{Align, PhysicalAddress};
use crate::proto::device_path::DevicePath;
use crate::proto::loaded_image::LoadedImage;
use crate::proto::media::fs::SimpleFileSystem;
use crate::proto::{Protocol, ProtocolPointer};
use crate::util::opt_nonnull_to_ptr;
use crate::{Char16, Error, Event, Guid, Handle, Result, Status, StatusExt};
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::fmt::{Debug, Formatter};
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::{ptr, slice};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use uefi_raw::table::boot::{
    EventType, InterfaceType, MemoryAttribute, MemoryDescriptor, MemoryType, Tpl,
};

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

/// Size in bytes of a UEFI page.
///
/// Note that this is not necessarily the processor's page size. The UEFI page
/// size is always 4 KiB.
pub const PAGE_SIZE: usize = 4096;

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
///
/// For protocol definitions, see the [`proto`] module.
///
/// [`proto`]: crate::proto
/// [`open_protocol_exclusive`]: BootServices::open_protocol_exclusive
/// [`open_protocol`]: BootServices::open_protocol
/// [`get_image_file_system`]: BootServices::get_image_file_system
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
#[repr(transparent)]
pub struct BootServices(uefi_raw::table::boot::BootServices);

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
    #[must_use]
    pub unsafe fn raise_tpl(&self, tpl: Tpl) -> TplGuard<'_> {
        TplGuard {
            boot_services: self,
            old_tpl: (self.0.raise_tpl)(tpl),
        }
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
        let (ty, mut addr) = match ty {
            AllocateType::AnyPages => (0, 0),
            AllocateType::MaxAddress(addr) => (1, addr),
            AllocateType::Address(addr) => (2, addr),
        };
        unsafe { (self.0.allocate_pages)(ty, mem_ty, count, &mut addr) }.to_result_with_val(|| addr)
    }

    /// Frees memory pages allocated by UEFI.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.FreePages()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NOT_FOUND`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn free_pages(&self, addr: PhysicalAddress, count: usize) -> Result {
        unsafe { (self.0.free_pages)(addr, count) }.to_result()
    }

    /// Returns struct which contains the size of a single memory descriptor
    /// as well as the size of the current memory map.
    ///
    /// Note that the size of the memory map can increase any time an allocation happens,
    /// so when creating a buffer to put the memory map into, it's recommended to allocate a few extra
    /// elements worth of space above the size of the current memory map.
    #[must_use]
    pub fn memory_map_size(&self) -> MemoryMapSize {
        let mut map_size = 0;
        let mut map_key = MemoryMapKey(0);
        let mut entry_size = 0;
        let mut entry_version = 0;

        let status = unsafe {
            (self.0.get_memory_map)(
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
            (self.0.get_memory_map)(
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

    /// Allocates from a memory pool. The pointer will be 8-byte aligned.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.AllocatePool()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn allocate_pool(&self, mem_ty: MemoryType, size: usize) -> Result<*mut u8> {
        let mut buffer = ptr::null_mut();
        unsafe { (self.0.allocate_pool)(mem_ty, size, &mut buffer) }.to_result_with_val(|| buffer)
    }

    /// Frees memory allocated from a pool.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.FreePool()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn free_pool(&self, addr: *mut u8) -> Result {
        unsafe { (self.0.free_pool)(addr) }.to_result()
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
        let mut event = ptr::null_mut();

        // Safety: the argument types of the function pointers are defined
        // differently, but are compatible and can be safely transmuted.
        let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

        let notify_ctx = opt_nonnull_to_ptr(notify_ctx);

        // Now we're ready to call UEFI
        (self.0.create_event)(event_ty, notify_tpl, notify_fn, notify_ctx, &mut event)
            .to_result_with_val(
                // OK to unwrap: event is non-null for Status::SUCCESS.
                || Event::from_ptr(event).unwrap(),
            )
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
        if self.0.header.revision < Revision::EFI_2_00 {
            return Err(Status::UNSUPPORTED.into());
        }

        let mut event = ptr::null_mut();

        // Safety: the argument types of the function pointers are defined
        // differently, but are compatible and can be safely transmuted.
        let notify_fn: Option<uefi_raw::table::boot::EventNotifyFn> = mem::transmute(notify_fn);

        (self.0.create_event_ex)(
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

    /// Sets the trigger for `EventType::TIMER` event.
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.SetTimer()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn set_timer(&self, event: &Event, trigger_time: TimerTrigger) -> Result {
        let (ty, time) = match trigger_time {
            TimerTrigger::Cancel => (0, 0),
            TimerTrigger::Periodic(hundreds_ns) => (1, hundreds_ns),
            TimerTrigger::Relative(hundreds_ns) => (2, hundreds_ns),
        };
        unsafe { (self.0.set_timer)(event.as_ptr(), ty, time) }.to_result()
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
        let number_of_events = events.len();
        let events: *mut uefi_raw::Event = events.as_mut_ptr().cast();

        let mut index = 0;
        unsafe { (self.0.wait_for_event)(number_of_events, events, &mut index) }.to_result_with(
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
        // Safety: cloning this event should be safe, as we're directly passing it to firmware
        // and not keeping the clone around.
        unsafe { (self.0.signal_event)(event.as_ptr()).to_result() }
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
        unsafe { (self.0.close_event)(event.as_ptr()).to_result() }
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
    pub fn check_event(&self, event: Event) -> Result<bool> {
        let status = unsafe { (self.0.check_event)(event.as_ptr()) };
        match status {
            Status::SUCCESS => Ok(true),
            Status::NOT_READY => Ok(false),
            _ => Err(status.into()),
        }
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
        &self,
        handle: Option<Handle>,
        protocol: &Guid,
        interface: *mut c_void,
    ) -> Result<Handle> {
        let mut handle = Handle::opt_to_ptr(handle);
        ((self.0.install_protocol_interface)(
            &mut handle,
            protocol,
            InterfaceType::NATIVE_INTERFACE,
            interface,
        ))
        .to_result_with_val(|| Handle::from_ptr(handle).unwrap())
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
        &self,
        handle: Handle,
        protocol: &Guid,
        old_interface: *mut c_void,
        new_interface: *mut c_void,
    ) -> Result<()> {
        (self.0.reinstall_protocol_interface)(
            handle.as_ptr(),
            protocol,
            old_interface,
            new_interface,
        )
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
    /// See section `EFI_BOOT_SERVICES.UninstallProtocolInterface()` in the UEFI Specification for
    /// more details.
    ///
    /// * [`uefi::Status::NOT_FOUND`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub unsafe fn uninstall_protocol_interface(
        &self,
        handle: Handle,
        protocol: &Guid,
        interface: *mut c_void,
    ) -> Result<()> {
        (self.0.uninstall_protocol_interface)(handle.as_ptr(), protocol, interface).to_result()
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
    pub fn register_protocol_notify(
        &self,
        protocol: &Guid,
        event: Event,
    ) -> Result<(Event, SearchType)> {
        let mut key = ptr::null();
        // Safety: we clone `event` a couple times, but there will be only one left once we return.
        unsafe { (self.0.register_protocol_notify)(protocol, event.as_ptr(), &mut key) }
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
            SearchType::AllHandles => (0, ptr::null(), ptr::null()),
            SearchType::ByRegisterNotify(registration) => {
                (1, ptr::null(), registration.0.as_ptr().cast_const())
            }
            SearchType::ByProtocol(guid) => (2, guid as *const Guid, ptr::null()),
        };

        let status =
            unsafe { (self.0.locate_handle)(ty, guid, key, &mut buffer_size, buffer.cast()) };

        // Must convert the returned size (in bytes) to length (number of elements).
        let buffer_len = buffer_size / handle_size;

        match (buffer, status) {
            (NULL_BUFFER, Status::BUFFER_TOO_SMALL) => Ok(buffer_len),
            (_, other_status) => other_status.to_result_with_val(|| buffer_len),
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.LocateDevicePath()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::NOT_FOUND`]
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub fn locate_device_path<P: ProtocolPointer + ?Sized>(
        &self,
        device_path: &mut &DevicePath,
    ) -> Result<Handle> {
        let mut handle = ptr::null_mut();
        let mut device_path_ptr: *const uefi_raw::protocol::device_path::DevicePathProtocol =
            device_path.as_ffi_ptr().cast();
        unsafe {
            (self.0.locate_device_path)(&P::GUID, &mut device_path_ptr, &mut handle)
                .to_result_with_val(|| {
                    *device_path = DevicePath::from_ffi_ptr(device_path_ptr.cast());
                    // OK to unwrap: handle is non-null for Status::SUCCESS.
                    Handle::from_ptr(handle).unwrap()
                })
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
    ///
    /// # Errors
    ///
    /// Returns [`NOT_FOUND`] if no handles support the requested protocol.
    pub fn get_handle_for_protocol<P: ProtocolPointer + ?Sized>(&self) -> Result<Handle> {
        // Delegate to a non-generic function to potentially reduce code size.
        self.get_handle_for_protocol_impl(&P::GUID)
    }

    fn get_handle_for_protocol_impl(&self, guid: &Guid) -> Result<Handle> {
        self.locate_handle_buffer(SearchType::ByProtocol(guid))?
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
            (self.0.load_image)(
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
    pub fn unload_image(&self, image_handle: Handle) -> Result {
        unsafe { (self.0.unload_image)(image_handle.as_ptr()) }.to_result()
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
    pub fn start_image(&self, image_handle: Handle) -> Result {
        unsafe {
            // TODO: implement returning exit data to the caller.
            let mut exit_data_size: usize = 0;
            let mut exit_data: *mut u16 = ptr::null_mut();
            (self.0.start_image)(image_handle.as_ptr(), &mut exit_data_size, &mut exit_data)
                .to_result()
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
        (self.0.exit)(
            image_handle.as_ptr(),
            exit_status,
            exit_data_size,
            exit_data.cast(),
        )
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.ExitBootServices()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    pub(super) unsafe fn exit_boot_services(
        &self,
        image: Handle,
        mmap_key: MemoryMapKey,
    ) -> Result {
        (self.0.exit_boot_services)(image.as_ptr(), mmap_key.0).to_result()
    }

    /// Stalls the processor for an amount of time.
    ///
    /// The time is in microseconds.
    pub fn stall(&self, time: usize) {
        assert_eq!(unsafe { (self.0.stall)(time) }, Status::SUCCESS);
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
    pub unsafe fn install_configuration_table(
        &self,
        guid_entry: &Guid,
        table_ptr: *const c_void,
    ) -> Result {
        (self.0.install_configuration_table)(guid_entry, table_ptr).to_result()
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

        unsafe { (self.0.set_watchdog_timer)(timeout, watchdog_code, data_len, data) }.to_result()
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
        &self,
        controller: Handle,
        driver_image: Option<Handle>,
        remaining_device_path: Option<&DevicePath>,
        recursive: bool,
    ) -> Result {
        unsafe {
            (self.0.connect_controller)(
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
    /// See [`connect_controller`][Self::connect_controller].
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.DisconnectController()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    /// * [`uefi::Status::DEVICE_ERROR`]
    pub fn disconnect_controller(
        &self,
        controller: Handle,
        driver_image: Option<Handle>,
        child: Option<Handle>,
    ) -> Result {
        unsafe {
            (self.0.disconnect_controller)(
                controller.as_ptr(),
                Handle::opt_to_ptr(driver_image),
                Handle::opt_to_ptr(child),
            )
        }
        .to_result_with_err(|_| ())
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
        &self,
        params: OpenProtocolParams,
        attributes: OpenProtocolAttributes,
    ) -> Result<ScopedProtocol<P>> {
        let mut interface = ptr::null_mut();
        (self.0.open_protocol)(
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
                boot_services: self,
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
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.OpenProtocol()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::UNSUPPORTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::ALREADY_STARTED`]
    pub fn test_protocol<P: ProtocolPointer + ?Sized>(
        &self,
        params: OpenProtocolParams,
    ) -> Result<()> {
        const TEST_PROTOCOL: u32 = 0x04;
        let mut interface = ptr::null_mut();
        unsafe {
            (self.0.open_protocol)(
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

    /// Get the list of protocol interface [`Guids`][Guid] that are installed
    /// on a [`Handle`].
    ///
    /// # Errors
    ///
    /// See section `EFI_BOOT_SERVICES.ProtocolsPerHandle()` in the UEFI Specification for more details.
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::OUT_OF_RESOURCES`]
    pub fn protocols_per_handle(&self, handle: Handle) -> Result<ProtocolsPerHandle> {
        let mut protocols = ptr::null_mut();
        let mut count = 0;

        let mut status =
            unsafe { (self.0.protocols_per_handle)(handle.as_ptr(), &mut protocols, &mut count) };

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
            boot_services: self,
            protocols: protocols.cast::<&Guid>(),
            count,
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
    pub fn locate_handle_buffer(&self, search_ty: SearchType) -> Result<HandleBuffer> {
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

        unsafe { (self.0.locate_handle_buffer)(ty, guid, key, &mut num_handles, &mut buffer) }
            .to_result_with_val(|| HandleBuffer {
                boot_services: self,
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
    /// [`open_protocol_exclusive`]: Self::open_protocol_exclusive
    /// [`locate_device_path`]: Self::locate_device_path
    ///
    /// * [`uefi::Status::INVALID_PARAMETER`]
    /// * [`uefi::Status::UNSUPPORTED`]
    /// * [`uefi::Status::ACCESS_DENIED`]
    /// * [`uefi::Status::ALREADY_STARTED`]
    /// * [`uefi::Status::NOT_FOUND`]
    pub fn get_image_file_system(
        &self,
        image_handle: Handle,
    ) -> Result<ScopedProtocol<SimpleFileSystem>> {
        let loaded_image = self.open_protocol_exclusive::<LoadedImage>(image_handle)?;

        let device_handle = loaded_image
            .device()
            .ok_or(Error::new(Status::UNSUPPORTED, ()))?;
        let device_path = self.open_protocol_exclusive::<DevicePath>(device_handle)?;

        let device_handle = self.locate_device_path::<SimpleFileSystem>(&mut &*device_path)?;

        self.open_protocol_exclusive(device_handle)
    }
}

#[cfg(feature = "alloc")]
impl BootServices {
    /// Returns all the handles implementing a certain protocol.
    ///
    /// # Errors
    ///
    /// All errors come from calls to [`locate_handle`].
    ///
    /// [`locate_handle`]: Self::locate_handle
    pub fn find_handles<P: ProtocolPointer + ?Sized>(&self) -> Result<Vec<Handle>> {
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
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
}

impl Debug for BootServices {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        #[allow(deprecated)]
        f.debug_struct("BootServices")
            .field("header", &self.0.header)
            .field("raise_tpl (fn ptr)", &(self.0.raise_tpl as *const usize))
            .field(
                "restore_tpl (fn ptr)",
                &(self.0.restore_tpl as *const usize),
            )
            .field(
                "allocate_pages (fn ptr)",
                &(self.0.allocate_pages as *const usize),
            )
            .field("free_pages (fn ptr)", &(self.0.free_pages as *const usize))
            .field(
                "get_memory_map (fn ptr)",
                &(self.0.get_memory_map as *const usize),
            )
            .field(
                "allocate_pool (fn ptr)",
                &(self.0.allocate_pool as *const usize),
            )
            .field("free_pool (fn ptr)", &(self.0.free_pool as *const usize))
            .field(
                "create_event (fn ptr)",
                &(self.0.create_event as *const usize),
            )
            .field("set_timer (fn ptr)", &(self.0.set_timer as *const usize))
            .field(
                "wait_for_event (fn ptr)",
                &(self.0.wait_for_event as *const usize),
            )
            .field("signal_event", &(self.0.signal_event as *const usize))
            .field("close_event", &(self.0.close_event as *const usize))
            .field("check_event", &(self.0.check_event as *const usize))
            .field(
                "install_protocol_interface",
                &(self.0.install_protocol_interface as *const usize),
            )
            .field(
                "reinstall_protocol_interface",
                &(self.0.reinstall_protocol_interface as *const usize),
            )
            .field(
                "uninstall_protocol_interface",
                &(self.0.uninstall_protocol_interface as *const usize),
            )
            .field(
                "handle_protocol (fn ptr)",
                &(self.0.handle_protocol as *const usize),
            )
            .field(
                "register_protocol_notify",
                &(self.0.register_protocol_notify as *const usize),
            )
            .field(
                "locate_handle (fn ptr)",
                &(self.0.locate_handle as *const usize),
            )
            .field(
                "locate_device_path (fn ptr)",
                &(self.0.locate_device_path as *const usize),
            )
            .field(
                "install_configuration_table (fn ptr)",
                &(self.0.install_configuration_table as *const usize),
            )
            .field("load_image (fn ptr)", &(self.0.load_image as *const usize))
            .field(
                "start_image (fn ptr)",
                &(self.0.start_image as *const usize),
            )
            .field("exit", &(self.0.exit as *const usize))
            .field(
                "unload_image (fn ptr)",
                &(self.0.unload_image as *const usize),
            )
            .field(
                "exit_boot_services (fn ptr)",
                &(self.0.exit_boot_services as *const usize),
            )
            .field(
                "get_next_monotonic_count",
                &(self.0.get_next_monotonic_count as *const usize),
            )
            .field("stall (fn ptr)", &(self.0.stall as *const usize))
            .field(
                "set_watchdog_timer (fn ptr)",
                &(self.0.set_watchdog_timer as *const usize),
            )
            .field(
                "connect_controller",
                &(self.0.connect_controller as *const usize),
            )
            .field(
                "disconnect_controller",
                &(self.0.disconnect_controller as *const usize),
            )
            .field("open_protocol", &(self.0.open_protocol as *const usize))
            .field("close_protocol", &(self.0.close_protocol as *const usize))
            .field(
                "open_protocol_information",
                &(self.0.open_protocol_information as *const usize),
            )
            .field(
                "protocols_per_handle",
                &(self.0.protocols_per_handle as *const usize),
            )
            .field(
                "locate_handle_buffer",
                &(self.0.locate_handle_buffer as *const usize),
            )
            .field(
                "locate_protocol (fn ptr)",
                &(self.0.locate_protocol as *const usize),
            )
            .field(
                "install_multiple_protocol_interfaces",
                &(self.0.install_multiple_protocol_interfaces as *const usize),
            )
            .field(
                "uninstall_multiple_protocol_interfaces",
                &(self.0.uninstall_multiple_protocol_interfaces as *const usize),
            )
            .field("calculate_crc32", &(self.0.calculate_crc32 as *const usize))
            .field("copy_mem (fn ptr)", &(self.0.copy_mem as *const usize))
            .field("set_mem (fn ptr)", &(self.0.set_mem as *const usize))
            .field("create_event_ex", &(self.0.create_event_ex as *const usize))
            .finish()
    }
}

/// Used as a parameter of [`BootServices::load_image`] to provide the
/// image source.
#[derive(Debug)]
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
    FromDevicePath {
        /// The full device path from which to load the image.
        ///
        /// The provided path should be a full device path and not just the
        /// file path portion of it. So for example, it must be (the binary
        /// representation)
        /// `PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)/HD(1,MBR,0xBE1AFDFA,0x3F,0xFBFC1)/\\EFI\\BOOT\\BOOTX64.EFI`
        /// and not just `\\EFI\\BOOT\\BOOTX64.EFI`.
        device_path: &'a DevicePath,

        /// If there is no instance of [`SimpleFileSystem`] protocol associated
        /// with the given device path, then this function will attempt to use
        /// `LoadFileProtocol` (`from_boot_manager` is `true`) or
        /// `LoadFile2Protocol`, and then `LoadFileProtocol`
        /// (`from_boot_manager` is `false`).
        from_boot_manager: bool,
    },
}

/// RAII guard for task priority level changes
///
/// Will automatically restore the former task priority level when dropped.
#[derive(Debug)]
pub struct TplGuard<'boot> {
    boot_services: &'boot BootServices,
    old_tpl: Tpl,
}

impl Drop for TplGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.boot_services.0.restore_tpl)(self.old_tpl);
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

/// Parameters passed to [`BootServices::open_protocol`].
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
pub struct ScopedProtocol<'a, P: Protocol + ?Sized> {
    /// The protocol interface.
    interface: Option<&'a UnsafeCell<P>>,

    open_params: OpenProtocolParams,
    boot_services: &'a BootServices,
}

impl<'a, P: Protocol + ?Sized> Drop for ScopedProtocol<'a, P> {
    fn drop(&mut self) {
        let status = unsafe {
            (self.boot_services.0.close_protocol)(
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

impl<'a, P: Protocol + ?Sized> ScopedProtocol<'a, P> {
    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get(&self) -> Option<&P> {
        self.interface.map(|p| unsafe { &*p.get() })
    }

    /// Get the protocol interface data, or `None` if the open protocol's
    /// interface is null.
    #[must_use]
    pub fn get_mut(&self) -> Option<&mut P> {
        self.interface.map(|p| unsafe { &mut *p.get() })
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

impl Align for MemoryDescriptor {
    fn alignment() -> usize {
        mem::align_of::<Self>()
    }
}

/// A unique identifier of a memory map.
///
/// If the memory map changes, this value is no longer valid.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryMapKey(usize);

/// A structure containing the size of a memory descriptor and the size of the
/// memory map.
#[derive(Debug)]
pub struct MemoryMapSize {
    /// Size of a single memory descriptor in bytes
    pub entry_size: usize,
    /// Size of the entire memory map in bytes
    pub map_size: usize,
}

/// An iterator of [`MemoryDescriptor`] that is always associated with the
/// unique [`MemoryMapKey`] contained in the struct.
///
/// To iterate over the entries, call [`MemoryMap::entries`]. To get a sorted
/// map, you manually have to call [`MemoryMap::sort`] first.
#[derive(Debug)]
pub struct MemoryMap<'buf> {
    key: MemoryMapKey,
    buf: &'buf mut [u8],
    entry_size: usize,
    len: usize,
}

impl<'buf> MemoryMap<'buf> {
    #[must_use]
    /// Returns the unique [`MemoryMapKey`] associated with the memory map.
    pub fn key(&self) -> MemoryMapKey {
        self.key
    }

    /// Sorts the memory map by physical address in place.
    /// This operation is optional and should be invoked only once.
    pub fn sort(&mut self) {
        unsafe {
            self.qsort(0, self.len - 1);
        }
    }

    /// Hoare partition scheme for quicksort.
    /// Must be called with `low` and `high` being indices within bounds.
    unsafe fn qsort(&mut self, low: usize, high: usize) {
        if low >= high {
            return;
        }

        let p = self.partition(low, high);
        self.qsort(low, p);
        self.qsort(p + 1, high);
    }

    unsafe fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.get_element_phys_addr(low + (high - low) / 2);

        let mut left_index = low.wrapping_sub(1);
        let mut right_index = high.wrapping_add(1);

        loop {
            while {
                left_index = left_index.wrapping_add(1);

                self.get_element_phys_addr(left_index) < pivot
            } {}

            while {
                right_index = right_index.wrapping_sub(1);

                self.get_element_phys_addr(right_index) > pivot
            } {}

            if left_index >= right_index {
                return right_index;
            }

            self.swap(left_index, right_index);
        }
    }

    /// Indices must be smaller than len.
    unsafe fn swap(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }

        let base = self.buf.as_mut_ptr();

        unsafe {
            ptr::swap_nonoverlapping(
                base.add(index1 * self.entry_size),
                base.add(index2 * self.entry_size),
                self.entry_size,
            );
        }
    }

    fn get_element_phys_addr(&self, index: usize) -> PhysicalAddress {
        let offset = index.checked_mul(self.entry_size).unwrap();
        let elem = unsafe { &*self.buf.as_ptr().add(offset).cast::<MemoryDescriptor>() };
        elem.phys_start
    }

    /// Returns an iterator over the contained memory map. To get a sorted map,
    /// call [`MemoryMap::sort`] first.
    #[must_use]
    pub fn entries(&self) -> MemoryMapIter {
        MemoryMapIter {
            buffer: self.buf,
            entry_size: self.entry_size,
            index: 0,
            len: self.len,
        }
    }
}

/// An iterator of [`MemoryDescriptor`]. The underlying memory map is always
/// associated with a unique [`MemoryMapKey`].
#[derive(Debug, Clone)]
pub struct MemoryMapIter<'buf> {
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
            let descriptor = unsafe {
                &*self
                    .buffer
                    .as_ptr()
                    .add(self.entry_size * self.index)
                    .cast::<MemoryDescriptor>()
            };

            self.index += 1;

            Some(descriptor)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for MemoryMapIter<'_> {
    fn len(&self) -> usize {
        self.len
    }
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

/// Raw event notification function
type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: Option<NonNull<c_void>>);

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

/// Protocol interface [`Guids`][Guid] that are installed on a [`Handle`] as
/// returned by [`BootServices::protocols_per_handle`].
#[derive(Debug)]
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
}

/// A buffer that contains an array of [`Handles`][Handle] that support the
/// requested protocol. Returned by [`BootServices::locate_handle_buffer`].
#[derive(Debug)]
pub struct HandleBuffer<'a> {
    // The pointer returned by `locate_handle_buffer` has to be freed with
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
}

/// Opaque pointer returned by [`BootServices::register_protocol_notify`] to be used
/// with [`BootServices::locate_handle`] via [`SearchType::ByRegisterNotify`].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ProtocolSearchKey(NonNull<c_void>);

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use crate::table::boot::{MemoryAttribute, MemoryMap, MemoryMapKey, MemoryType};

    use super::{MemoryDescriptor, MemoryMapIter};

    #[test]
    fn mem_map_sorting() {
        // Doesn't matter what type it is.
        const TY: MemoryType = MemoryType::RESERVED;

        const BASE: MemoryDescriptor = MemoryDescriptor {
            ty: TY,
            phys_start: 0,
            virt_start: 0,
            page_count: 0,
            att: MemoryAttribute::empty(),
        };

        let mut buffer = [
            MemoryDescriptor {
                phys_start: 2000,
                ..BASE
            },
            MemoryDescriptor {
                phys_start: 3000,
                ..BASE
            },
            BASE,
            MemoryDescriptor {
                phys_start: 1000,
                ..BASE
            },
        ];

        let desc_count = buffer.len();

        let byte_buffer = {
            let size = desc_count * size_of::<MemoryDescriptor>();
            unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, size) }
        };

        let mut mem_map = MemoryMap {
            // Key doesn't matter
            key: MemoryMapKey(0),
            len: desc_count,
            buf: byte_buffer,
            entry_size: size_of::<MemoryDescriptor>(),
        };

        mem_map.sort();

        if !is_sorted(&mem_map.entries()) {
            panic!("mem_map is not sorted: {}", mem_map);
        }
    }

    // Added for debug purposes on test failure
    impl core::fmt::Display for MemoryMap<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            writeln!(f)?;
            for desc in self.entries() {
                writeln!(f, "{:?}", desc)?;
            }
            Ok(())
        }
    }

    fn is_sorted(iter: &MemoryMapIter) -> bool {
        let mut iter = iter.clone();
        let mut curr_start;

        if let Some(val) = iter.next() {
            curr_start = val.phys_start;
        } else {
            return true;
        }

        for desc in iter {
            if desc.phys_start <= curr_start {
                return false;
            }
            curr_start = desc.phys_start
        }
        true
    }
}
