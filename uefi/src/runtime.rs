//! UEFI runtime services.
//!
//! These services are available both before and after exiting boot
//! services. Note that various restrictions apply when calling runtime services
//! functions after exiting boot services; see the "Calling Convention" section
//! of the UEFI specification for details.

use crate::{table, CStr16, Error, Result, Status, StatusExt};
use core::ptr::{self, NonNull};

#[cfg(feature = "alloc")]
use {crate::mem::make_boxed, alloc::boxed::Box};

#[cfg(all(feature = "unstable", feature = "alloc"))]
use alloc::alloc::Global;

pub use crate::table::runtime::{Daylight, Time, TimeCapabilities, TimeError, TimeParams};
pub use uefi_raw::capsule::{CapsuleBlockDescriptor, CapsuleFlags, CapsuleHeader};
pub use uefi_raw::table::runtime::{ResetType, VariableAttributes, VariableVendor};

fn runtime_services_raw_panicking() -> NonNull<uefi_raw::table::runtime::RuntimeServices> {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    NonNull::new(st.runtime_services).expect("runtime services are not active")
}

/// Query the current time and date information.
pub fn get_time() -> Result<Time> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    unsafe { (rt.get_time)(time_ptr.cast(), ptr::null_mut()) }.to_result_with_val(|| time)
}

/// Query the current time and date information and the RTC capabilities.
pub fn get_time_and_caps() -> Result<(Time, TimeCapabilities)> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut time = Time::invalid();
    let time_ptr: *mut Time = &mut time;
    let mut caps = TimeCapabilities::default();
    unsafe { (rt.get_time)(time_ptr.cast(), &mut caps) }.to_result_with_val(|| (time, caps))
}

/// Sets the current local time and date information
///
/// During runtime, if a PC-AT CMOS device is present in the platform, the
/// caller must synchronize access to the device before calling `set_time`.
///
/// # Safety
///
/// Undefined behavior could happen if multiple tasks try to
/// use this function at the same time without synchronisation.
pub unsafe fn set_time(time: &Time) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let time: *const Time = time;
    (rt.set_time)(time.cast()).to_result()
}

/// Gets the contents and attributes of a variable. The size of `buf` must be at
/// least as big as the variable's size, although it can be larger.
///
/// On success, returns a tuple containing the variable's value (a slice of
/// `buf`) and the variable's attributes.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::BUFFER_TOO_SMALL`]: `buf` is not large enough. The required size
///   will be returned in the error data.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn get_variable<'buf>(
    name: &CStr16,
    vendor: &VariableVendor,
    buf: &'buf mut [u8],
) -> Result<(&'buf mut [u8], VariableAttributes), Option<usize>> {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    let mut attributes = VariableAttributes::empty();
    let mut data_size = buf.len();
    let status = unsafe {
        (rt.get_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            &mut attributes,
            &mut data_size,
            buf.as_mut_ptr(),
        )
    };

    match status {
        Status::SUCCESS => Ok((&mut buf[..data_size], attributes)),
        Status::BUFFER_TOO_SMALL => Err(Error::new(status, Some(data_size))),
        _ => Err(Error::new(status, None)),
    }
}

/// Gets the contents and attributes of a variable.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: variable was not found.
/// * [`Status::DEVICE_ERROR`]: variable could not be read due to a hardware error.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be read due to an
///   authentication error.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
#[cfg(feature = "alloc")]
pub fn get_variable_boxed(
    name: &CStr16,
    vendor: &VariableVendor,
) -> Result<(Box<[u8]>, VariableAttributes)> {
    let mut out_attr = VariableAttributes::empty();
    let get_var = |buf| {
        get_variable(name, vendor, buf).map(|(val, attr)| {
            // `make_boxed` expects only a DST value to be returned (`val` in
            // this case), so smuggle the `attr` value out via a separate
            // variable.
            out_attr = attr;
            val
        })
    };
    #[cfg(not(feature = "unstable"))]
    {
        make_boxed(get_var).map(|val| (val, out_attr))
    }
    #[cfg(feature = "unstable")]
    {
        make_boxed(get_var, Global).map(|val| (val, out_attr))
    }
}

/// Sets the value of a variable. This can be used to create a new variable,
/// update an existing variable, or (when the size of `data` is zero)
/// delete a variable.
///
/// # Warnings
///
/// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
/// this function to transition the Secure Boot mode to setup mode or audit
/// mode if the firmware requires a reboot for that operation.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid attributes, name, or vendor.
/// * [`Status::OUT_OF_RESOURCES`]: not enough storage is available to hold
///   the variable.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be written due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to update a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn set_variable(
    name: &CStr16,
    vendor: &VariableVendor,
    attributes: VariableAttributes,
    data: &[u8],
) -> Result {
    let rt = runtime_services_raw_panicking();
    let rt = unsafe { rt.as_ref() };

    unsafe {
        (rt.set_variable)(
            name.as_ptr().cast(),
            &vendor.0,
            attributes,
            data.len(),
            data.as_ptr(),
        )
        .to_result()
    }
}

/// Deletes a UEFI variable.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: invalid name or vendor.
/// * [`Status::WRITE_PROTECTED`]: variable is read-only.
/// * [`Status::SECURITY_VIOLATION`]: variable could not be deleted due to an
///   authentication error.
/// * [`Status::NOT_FOUND`]: attempted to delete a non-existent variable.
/// * [`Status::UNSUPPORTED`]: this platform does not support variable storage
///   after exiting boot services.
pub fn delete_variable(name: &CStr16, vendor: &VariableVendor) -> Result {
    set_variable(name, vendor, VariableAttributes::empty(), &[])
}
