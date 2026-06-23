// SPDX-License-Identifier: MIT OR Apache-2.0

//! High-level support for writing UEFI drivers.
//!
//! # Driver How-To
//!
//! Start by defining a new Cargo binary target, which will be a (mostly)
//! regular `uefi-rs` application.
//!
//! The binary image must be explicitly designated as a UEFI driver by setting
//! the PE32+ subsystem to `EFI_BOOT_SERVICE_DRIVER` (see UEFI spec, section
//! 2.1). This can be done by adding the following line to `build.rs`:
//!
//! ```ignore
//! println!("cargo::rustc-link-arg=/SUBSYSTEM:EFI_BOOT_SERVICE_DRIVER");
//! ```
//!
//! Setting the subsystem like this instructs the firmware to leave the driver's
//! code and data segments loaded in memory after its entrypoint returns. This
//! is necessary to ensure any objects such as protocol interfaces installed by
//! the driver remain functional after the driver is finished loading.
//!
//! In the driver's entrypoint, create and install one or more implementations
//! of the [`Driver`] trait using the [`install`] function. Later, the firmware
//! will invoke the `Driver` methods to identify supported controllers and begin
//! managing them.
//!
//! One method to load the driver is to use the UEFI shell's `load` command.
//! This will load the driver image, invoke its entrypoint, *and* subsequently
//! perform a [`ConnectController`][cnct-ctrl] sequence to bind the driver to
//! any compatible controller handles present on the system. Following a
//! successful `load`, any other application run from the UEFI shell should be
//! able to open and use the protocols installed by the driver.
//!
//! [cnct-ctrl]: crate::boot::connect_controller
//!
//! # Device vs. Bus Drivers
//!
//! The UEFI driver model distinguishes between "device" and "bus" drivers.
//! Device drivers manage a single device controller, while bus drivers manage a
//! bus which may have multiple child controllers. Both types of drivers use the
//! same underlying protocols and firmware infrastructure, but with different
//! expected behaviors and semantics.
//!
//! The helpers provided in this module are currently tailored towards (and
//! validated for use with) device drivers. Additional functionality may be
//! added in the future to better support bus drivers.
//!
//! # References:
//!
//! * [UEFI Specification](https://uefi.org/specifications)
//! * [EDKII Driver Writer's Guide](https://tianocore-docs.github.io/edk2-UefiDriverWritersGuide/draft/)
//! * [/SUBSYSTEM (MSVC Linker)](https://learn.microsoft.com/en-us/cpp/build/reference/subsystem-specify-subsystem)

use core::mem;

use alloc::boxed::Box;

use log::{debug, error, trace, warn};

use uefi_raw::protocol::device_path::DevicePathProtocol;
use uefi_raw::protocol::driver::DriverBindingProtocol;

use crate::mem::memory_map::MemoryType;
use crate::proto::device_path::DevicePath;
use crate::proto::loaded_image::LoadedImage;
use crate::{Handle, Result, ResultExt, Status, boot};

/// Trait defining the behavior of a UEFI driver
///
/// The methods of this trait correspond to the functions of the UEFI Driver
/// Binding Protocol. For complete technical information about the expected
/// behavior, refer to section 11.1 of the UEFI specification.
///
/// In each of this trait's methods, the `agent` parameter is the handle of the
/// driver itself (which may not always be the same as the handle of it
/// containing image). This may be used when calling other UEFI services such as
/// `OpenProtocol`.
pub trait Driver {
    /// Determines whether this driver supports device specified by
    /// `controller`.
    fn supported(
        &mut self,
        agent: Handle,
        controller: Handle,
        remaining: Option<&DevicePath>,
    ) -> Result;

    /// Activates this driver for the device specified by `controller`.
    fn start(
        &mut self,
        agent: Handle,
        controller: Handle,
        remaining: Option<&DevicePath>,
    ) -> Result;

    /// Deactivates this driver for the device specified by `controller`.
    fn stop(&mut self, agent: Handle, controller: Handle) -> Result;
}

/// Protocol interface structure for [`DriverBindingProtocol`].
struct DriverBindingInterface<T> {
    protocol: DriverBindingProtocol,
    driver: T,
}

impl<T> DriverBindingInterface<T> {
    /// Recovers a reference to the interface structure from a pointer to its
    /// protocol function table.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer to a [`DriverBindingProtocol`] instance
    /// embedded within a `DriverBindingInterface` structure.
    ///
    /// The lifetime `'a` specified by the caller must not outlive the actual
    /// lifetime of the installed protocol interface.
    ///
    /// The caller must also ensure that it has exclusive access to the
    /// interface for the lifetime `'a`.
    unsafe fn from_proto_ptr_mut<'a>(ptr: *mut DriverBindingProtocol) -> &'a mut Self {
        // Compute base offset from protocol pointer
        let ptr = ptr
            .cast::<u8>()
            .wrapping_sub(mem::offset_of!(Self, protocol))
            .cast::<Self>();

        // SAFETY: The caller guarantees that `ptr` points to the `protocol`
        // field of a live `DriverBindingInterface` and that the returned
        // reference has exclusive access for its lifetime.
        unsafe { &mut *ptr }
    }
}

unsafe extern "efiapi" fn driver_supported<T: Driver>(
    this: *const DriverBindingProtocol,
    controller: uefi_raw::Handle,
    remaining: *const DevicePathProtocol,
) -> Status {
    // N.B. No trace logging since this is a very noisy function

    if this.is_null() || controller.is_null() {
        return Status::INVALID_PARAMETER;
    }

    // SAFETY: UEFI calls this function with the driver binding protocol
    // pointer installed by `install`, whose allocation is intentionally leaked.
    let this = unsafe { DriverBindingInterface::<T>::from_proto_ptr_mut(this.cast_mut()) };
    // SAFETY: `driver_binding_handle` was initialized from a valid `Handle`
    // when the protocol was installed.
    let agent = unsafe { Handle::from_ptr(this.protocol.driver_binding_handle).unwrap() };
    // SAFETY: `controller` was checked for null above and is a UEFI handle
    // supplied by the firmware for this callback.
    let controller = unsafe { Handle::from_ptr(controller).unwrap() };
    let remaining = if remaining.is_null() {
        None
    } else {
        // SAFETY: A non-null remaining device path is supplied by the firmware
        // for this callback and remains valid for the duration of the call.
        Some(unsafe { DevicePath::from_ffi_ptr(remaining.cast()) })
    };

    this.driver.supported(agent, controller, remaining).status()
}

unsafe extern "efiapi" fn driver_start<T: Driver>(
    this: *const DriverBindingProtocol,
    controller: uefi_raw::Handle,
    remaining: *const DevicePathProtocol,
) -> Status {
    trace!("this: {this:p}, controller: {controller:p}, remaining: {remaining:p}");

    if this.is_null() || controller.is_null() {
        return Status::INVALID_PARAMETER;
    }

    // SAFETY: UEFI calls this function with the driver binding protocol
    // pointer installed by `install`, whose allocation is intentionally leaked.
    let this = unsafe { DriverBindingInterface::<T>::from_proto_ptr_mut(this.cast_mut()) };
    // SAFETY: `driver_binding_handle` was initialized from a valid `Handle`
    // when the protocol was installed.
    let agent = unsafe { Handle::from_ptr(this.protocol.driver_binding_handle).unwrap() };
    // SAFETY: `controller` was checked for null above and is a UEFI handle
    // supplied by the firmware for this callback.
    let controller = unsafe { Handle::from_ptr(controller).unwrap() };
    let remaining = if remaining.is_null() {
        None
    } else {
        // SAFETY: A non-null remaining device path is supplied by the firmware
        // for this callback and remains valid for the duration of the call.
        Some(unsafe { DevicePath::from_ffi_ptr(remaining.cast()) })
    };

    this.driver.start(agent, controller, remaining).status()
}

unsafe extern "efiapi" fn driver_stop<T: Driver>(
    this: *const DriverBindingProtocol,
    controller: uefi_raw::Handle,
    number_of_children: usize,
    child_handle_buffer: *const uefi_raw::Handle,
) -> Status {
    trace!(
        "this: {this:p}, controller: {controller:p}, number_of_children: {number_of_children}, child_handle_buffer: {child_handle_buffer:p}"
    );

    if this.is_null() || controller.is_null() {
        return Status::INVALID_PARAMETER;
    }

    // SAFETY: UEFI calls this function with the driver binding protocol
    // pointer installed by `install`, whose allocation is intentionally leaked.
    let this = unsafe { DriverBindingInterface::<T>::from_proto_ptr_mut(this.cast_mut()) };
    // SAFETY: `driver_binding_handle` was initialized from a valid `Handle`
    // when the protocol was installed.
    let agent = unsafe { Handle::from_ptr(this.protocol.driver_binding_handle).unwrap() };
    // SAFETY: `controller` was checked for null above and is a UEFI handle
    // supplied by the firmware for this callback.
    let controller = unsafe { Handle::from_ptr(controller).unwrap() };

    if number_of_children == 0 {
        this.driver.stop(agent, controller).status()
    } else {
        warn!("stop with children not currently supported");
        Status::UNSUPPORTED
    }
}

/// Installs `driver` onto the specified `handle` (or the current image handle
/// if `None`).
///
/// The current image must have been loaded as an EFI boot service driver image.
/// This function verifies that condition using the current image's
/// [`LoadedImage`] protocol before installing the driver binding protocol. This
/// is required because the installed protocol contains function pointers into
/// the current image, and those pointers must remain valid after the image's
/// entrypoint returns.
///
/// # Errors
///
/// This function returns errors from [`boot::open_protocol_exclusive`] and
/// [`boot::install_protocol_interface`]. It returns [`Status::UNSUPPORTED`] if
/// the current image was not loaded as an EFI boot service driver image.
pub fn install<T: Driver>(driver: T, handle: Option<Handle>) -> Result {
    trace!("handle: {handle:?}");

    let image_handle = boot::image_handle();
    let target_handle = handle.unwrap_or(image_handle);

    {
        let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(image_handle)?;
        if loaded_image.code_type() != MemoryType::BOOT_SERVICES_CODE
            || loaded_image.data_type() != MemoryType::BOOT_SERVICES_DATA
        {
            error!("current image was not loaded as an EFI boot service driver");
            return Err(Status::UNSUPPORTED.into());
        }
    }

    let mut ctx = Box::new(DriverBindingInterface {
        protocol: DriverBindingProtocol {
            supported: driver_supported::<T>,
            start: driver_start::<T>,
            stop: driver_stop::<T>,
            version: 1,
            image_handle: image_handle.as_ptr(),
            driver_binding_handle: target_handle.as_ptr(),
        },
        driver,
    });

    let proto_ptr = &raw mut ctx.protocol;
    trace!("proto_ptr: {proto_ptr:p}");

    debug!("installing driver binding protocol");

    // SAFETY: `DriverBindingProtocol::GUID` matches the interface being
    // installed, the loaded-image check above verifies that this image's code
    // and data remain valid after its entrypoint returns, and `proto_ptr` points
    // into `ctx`, which is leaked below so the installed protocol interface
    // remains valid as long as the image remains loaded.
    unsafe {
        boot::install_protocol_interface(
            Some(target_handle),
            &DriverBindingProtocol::GUID,
            proto_ptr.cast(),
        )?;
    }

    // A pointer to the protocol interface is now installed onto the target
    // handle and may be dereferenced at a later time by any user of the
    // protocol. We intentionally leak the allocation here to ensure the memory
    // is not cleaned up or repurposed while the interface remains installed.
    Box::leak(ctx);

    Ok(())
}
