//! TODO

use crate::proto::console::text;
use crate::table::{cfg, Boot, Revision, SystemTable};
use crate::CStr16;
use core::ptr::{self, NonNull};
use core::slice;
use core::sync::atomic::{AtomicPtr, Ordering};

static SYSTEM_TABLE: AtomicPtr<uefi_raw::table::system::SystemTable> =
    AtomicPtr::new(ptr::null_mut());

/// Update the global system table pointer.
///
/// This is usually called automatically in the `main` entry point as part of
/// [`set_main`]. It should not be called at any other point in time, unless the
/// executable does not use [`set_main`], in which case it should be called once
/// before calling any other functions in this crate.
///
/// # Safety
///
/// This function should only be called as described above. The pointer must
/// point to a valid system table.
///
/// [`set_main`]: uefi::set_main
pub unsafe fn set_system_table(system_table: *mut uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.store(system_table, Ordering::Release);
}

/// Get a pointer to the system table.
///
/// # Panics
///
/// Panics if [`set_system_table`] has not been called with a non-null pointer.
pub(crate) fn system_table() -> NonNull<uefi_raw::table::system::SystemTable> {
    let st = SYSTEM_TABLE.load(Ordering::Acquire);
    NonNull::new(st).expect("set_system_table has not been called")
}

/// TODO
#[must_use]
pub fn system_table_boot() -> SystemTable<Boot> {
    unsafe { SystemTable::<Boot>::from_ptr(system_table().as_ptr().cast()) }.unwrap()
}

// TODO: is static lifetime OK for these returned references?

/// Get the firmware vendor string.
#[must_use]
pub fn firmware_vendor() -> &'static CStr16 {
    // SAFETY: the system table is valid as required by `set_system_table`.
    let st = unsafe { system_table().as_ref() };

    // SAFETY: relies on two assumptions:
    // * The firmware vendor pointer is valid.
    // * The firmware vender string is never mutated.
    unsafe { CStr16::from_ptr(st.firmware_vendor.cast()) }
}

/// Get the firmware revision.
#[must_use]
pub fn firmware_revision() -> u32 {
    // SAFETY: the system table is valid as required by `set_system_table`.
    let st = unsafe { system_table().as_ref() };
    st.firmware_revision
}

/// Get the revision of the system table, which is defined to be the revision of
/// the UEFI specification implemented by the firmware.
#[must_use]
pub fn uefi_revision() -> Revision {
    // SAFETY: the system table is valid as required by `set_system_table`.
    let st = unsafe { system_table().as_ref() };
    st.header.revision
}

/// Get the config table entries, a linear array of structures pointing to other
/// system-specific tables.
pub fn with_config_table<F, R>(f: F) -> R
where
    F: Fn(&[cfg::ConfigTableEntry]) -> R,
{
    let st = unsafe { system_table().as_mut() };

    let ptr: *const cfg::ConfigTableEntry = st.configuration_table.cast();
    let len = st.number_of_configuration_table_entries;
    let slice = if ptr.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
    };
    f(slice)
}

/// Call `f` with the [`Output`] protocol attached to stdout.
///
/// [`Output`]: uefi::proto::console::text::Output
pub fn with_stdout<F, R>(f: F) -> R
where
    F: Fn(&mut text::Output) -> R,
{
    // SAFETY: the system table is valid as required by `set_system_table`.
    let st = unsafe { system_table().as_ref() };

    let output_ptr: *mut text::Output = st.stdout.cast();
    assert!(!output_ptr.is_null());

    // SAFETY: assume the output pointer is valid.
    let output = unsafe { &mut *output_ptr };
    f(output)
}
