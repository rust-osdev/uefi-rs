//! UEFI runtime services.
//!
//! These services are available both before and after exiting boot
//! services. Note that various restrictions apply when calling runtime services
//! functions after exiting boot services; see the "Calling Convention" section
//! of the UEFI specification for details.

use crate::table::{self};
use crate::{Result, StatusExt};
use core::ptr::{self, NonNull};

pub use crate::table::runtime::{Daylight, Time, TimeCapabilities, TimeError, TimeParams};

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
