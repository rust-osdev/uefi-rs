//! UEFI System Table Interface
//!
//! The UEFI System Table is the single point from which all UEFI services can
//! be accessed. The table is provided to an UEFI application on startup, but not
//! all services will remain available forever.
//!
//! Some services, called "Boot Services", may only be called during the bootstrap
//! phase in which the UEFI firmware still has control of the hardware, and becomes
//! unavailable after the firmware hands over control of the hardware to an operating
//! system loader.
//!
//! Others, called "Runtime Services", may still be used after that point, but require
//! a specific CPU configuration which any operating system is unlikely to preserve.

use core::ptr::{self, NonNull};
use core::slice;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use uefi_raw::table::Revision;

use crate::proto::console::text;
use crate::table::cfg;
use crate::{CStr16, Char16};

static SYSTEM_TABLE: AtomicPtr<uefi_raw::table::system::SystemTable> =
    AtomicPtr::new(ptr::null_mut());

/// Sets the global system table pointer.
///
/// # Safety
/// `system_table` must point to the system table for the current image.
pub unsafe fn set_system_table(system_table: *mut uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.store(system_table, Ordering::Relaxed);
}

/// Returns `true` when we have access to a system table.
pub fn has_system_table() -> bool {
    !SYSTEM_TABLE.load(Ordering::Relaxed).is_null()
}

pub(crate) fn system_table_maybe_null() -> *mut uefi_raw::table::system::SystemTable {
    SYSTEM_TABLE.load(Ordering::Relaxed)
}

pub(crate) fn system_table() -> NonNull<uefi_raw::table::system::SystemTable> {
    NonNull::new(system_table_maybe_null()).expect("set_system_table has not been called")
}

/// Return the firmware vendor string.
#[must_use]
pub fn firmware_vendor() -> &'static CStr16 {
    unsafe { CStr16::from_ptr(system_table().as_ref().firmware_vendor.cast::<Char16>()) }
}

/// Return the firmware version.
#[must_use]
pub fn firmware_revision() -> u32 {
    unsafe { system_table().as_ref().firmware_revision }
}

/// Returns the revision of this table, which is defined to be
/// the revision of the UEFI specification implemented by the firmware.
#[must_use]
pub fn uefi_revision() -> Revision {
    unsafe { system_table().as_ref().header.revision }
}

/// Run the provided function on the configuration table array.
pub fn with_config_table<R>(
    f: impl for<'config> FnOnce(&'config [cfg::ConfigTableEntry]) -> R,
) -> R {
    let system_table = unsafe { system_table().as_ref() };

    let ptr = system_table
        .configuration_table
        .cast::<cfg::ConfigTableEntry>();

    let len = system_table.number_of_configuration_table_entries;

    let slice = if ptr.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(ptr, len) }
    };

    f(slice)
}

/// Run the provided function on stdin.
pub fn with_stdin<R>(f: impl for<'config> FnOnce(Option<&'config mut text::Input>) -> R) -> R {
    static LOCK: AtomicBool = AtomicBool::new(false);

    while LOCK
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {}

    let system_table = NonNull::new(system_table_maybe_null());

    let result = if let Some(system_table) = system_table {
        let system_table = unsafe { system_table.as_ref() };

        let output_ptr = NonNull::new(system_table.stdin.cast::<text::Input>());

        let output = output_ptr.map(|mut output_ptr| unsafe { output_ptr.as_mut() });

        f(output)
    } else {
        f(None)
    };

    LOCK.store(false, Ordering::Relaxed);

    result
}

/// Run the provided function on stdout.
pub fn with_stdout<R>(f: impl for<'config> FnOnce(Option<&'config mut text::Output>) -> R) -> R {
    static LOCK: AtomicBool = AtomicBool::new(false);

    while LOCK
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {}

    let system_table = NonNull::new(system_table_maybe_null());

    let result = if let Some(system_table) = system_table {
        let system_table = unsafe { system_table.as_ref() };

        let output_ptr = NonNull::new(system_table.stdout.cast::<text::Output>());

        let output = output_ptr.map(|mut output_ptr| unsafe { output_ptr.as_mut() });

        f(output)
    } else {
        f(None)
    };

    LOCK.store(false, Ordering::Relaxed);

    result
}

/// Run the provided function on stderr.
pub fn with_stderr<F, R>(f: impl for<'config> FnOnce(Option<&'config mut text::Output>) -> R) -> R {
    static LOCK: AtomicBool = AtomicBool::new(false);

    while LOCK
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {}

    let system_table = NonNull::new(system_table_maybe_null());

    let result = if let Some(system_table) = system_table {
        let system_table = unsafe { system_table.as_ref() };

        let output_ptr = NonNull::new(system_table.stderr.cast::<text::Output>());

        let output = output_ptr.map(|mut output_ptr| unsafe { output_ptr.as_mut() });

        f(output)
    } else {
        f(None)
    };

    LOCK.store(false, Ordering::Relaxed);

    result
}

/// Return the address of the SystemTable that resides in a UEFI runtime services
/// memory region.
#[must_use]
pub fn get_current_system_table_addr() -> u64 {
    SYSTEM_TABLE.load(Ordering::Relaxed) as u64
}
