//! TODO

// TODO
#![allow(clippy::missing_safety_doc)]

use crate::table::boot::MemoryDescriptor;
use crate::table::runtime::{
    ResetType, RuntimeServices, Time, TimeCapabilities, VariableAttributes, VariableStorageInfo,
    VariableVendor,
};
use crate::{system, CStr16, Result, Status};
use core::ptr::NonNull;

#[cfg(feature = "alloc")]
use {crate::table::runtime::VariableKey, alloc::boxed::Box, alloc::vec::Vec};

fn runtime_services() -> NonNull<RuntimeServices> {
    let st = unsafe { system::system_table().as_mut() };
    NonNull::new(st.runtime_services.cast()).expect("runtime services are not active")
}

/// Query the current time and date information
pub fn get_time() -> Result<Time> {
    unsafe { runtime_services().as_mut() }.get_time()
}

/// Query the current time and date information and the RTC capabilities
pub fn get_time_and_caps() -> Result<(Time, TimeCapabilities)> {
    unsafe { runtime_services().as_mut() }.get_time_and_caps()
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
    unsafe { runtime_services().as_mut() }.set_time(time)
}

/// Get the size (in bytes) of a variable. This can be used to find out how
/// big of a buffer should be passed in to `get_variable`.
pub fn get_variable_size(name: &CStr16, vendor: &VariableVendor) -> Result<usize> {
    unsafe { runtime_services().as_mut() }.get_variable_size(name, vendor)
}

/// Get the contents and attributes of a variable. The size of `buf` must
/// be at least as big as the variable's size, although it can be
/// larger. If it is too small, `BUFFER_TOO_SMALL` is returned.
///
/// On success, a tuple containing the variable's value (a slice of `buf`)
/// and the variable's attributes is returned.
pub fn get_variable<'a>(
    name: &CStr16,
    vendor: &VariableVendor,
    buf: &'a mut [u8],
) -> Result<(&'a [u8], VariableAttributes)> {
    unsafe { runtime_services().as_mut() }.get_variable(name, vendor, buf)
}

/// Get the contents and attributes of a variable.
#[cfg(feature = "alloc")]
pub fn get_variable_boxed(
    name: &CStr16,
    vendor: &VariableVendor,
) -> Result<(Box<[u8]>, VariableAttributes)> {
    unsafe { runtime_services().as_mut() }.get_variable_boxed(name, vendor)
}

/// Get the names and vendor GUIDs of all currently-set variables.
#[cfg(feature = "alloc")]
pub fn variable_keys() -> Result<Vec<VariableKey>> {
    unsafe { runtime_services().as_mut() }.variable_keys()
}

/// Set the value of a variable. This can be used to create a new variable,
/// update an existing variable, or (when the size of `data` is zero)
/// delete a variable.
///
/// # Warnings
///
/// The [`Status::WARN_RESET_REQUIRED`] warning will be returned when using
/// this function to transition the Secure Boot mode to setup mode or audit
/// mode if the firmware requires a reboot for that operation.
pub fn set_variable(
    name: &CStr16,
    vendor: &VariableVendor,
    attributes: VariableAttributes,
    data: &[u8],
) -> Result {
    unsafe { runtime_services().as_mut() }.set_variable(name, vendor, attributes, data)
}

/// Deletes a UEFI variable.
pub fn delete_variable(name: &CStr16, vendor: &VariableVendor) -> Result {
    unsafe { runtime_services().as_mut() }.delete_variable(name, vendor)
}

/// Get information about UEFI variable storage space for the type
/// of variable specified in `attributes`.
///
/// This operation is only supported starting with UEFI 2.0; earlier
/// versions will fail with [`Status::UNSUPPORTED`].
///
/// See [`VariableStorageInfo`] for details of the information returned.
pub fn query_variable_info(attributes: VariableAttributes) -> Result<VariableStorageInfo> {
    unsafe { runtime_services().as_mut() }.query_variable_info(attributes)
}

/// Resets the computer.
pub fn reset(rt: ResetType, status: Status, data: Option<&[u8]>) -> ! {
    unsafe { runtime_services().as_mut() }.reset(rt, status, data)
}

/// TODO
pub unsafe fn set_virtual_address_map(
    map_size: usize,
    desc_size: usize,
    desc_version: u32,
    virtual_map: *mut MemoryDescriptor,
) -> Status {
    unsafe { runtime_services().as_mut() }.set_virtual_address_map(
        map_size,
        desc_size,
        desc_version,
        virtual_map,
    )
}
