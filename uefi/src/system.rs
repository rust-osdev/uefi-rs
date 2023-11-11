//! TODO

// TODO
#![allow(clippy::missing_safety_doc)]

use crate::proto::console::text;
use crate::table::{cfg, Boot, Revision, SystemTable};
use crate::CStr16;
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use core::slice;

// TODO: this similar to `SyncUnsafeCell`. Once that is stabilized we
// can use it instead.
struct GlobalSystemTable {
    ptr: UnsafeCell<*mut uefi_raw::table::system::SystemTable>,
}

unsafe impl Sync for GlobalSystemTable {}

static SYSTEM_TABLE: GlobalSystemTable = GlobalSystemTable {
    ptr: UnsafeCell::new(ptr::null_mut()),
};

/// TODO
pub unsafe fn set_system_table(system_table: *mut uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.ptr.get().write(system_table);
}

/// TODO
pub(crate) fn system_table_maybe_null() -> *mut uefi_raw::table::system::SystemTable {
    unsafe { SYSTEM_TABLE.ptr.get().read() }
}

/// TODO
pub(crate) fn system_table() -> NonNull<uefi_raw::table::system::SystemTable> {
    let st = system_table_maybe_null();
    NonNull::new(st).expect("set_system_table has not been called")
}

/// TODO
#[must_use]
pub fn system_table_boot() -> SystemTable<Boot> {
    unsafe { SystemTable::<Boot>::from_ptr(system_table().as_ptr().cast()) }.unwrap()
}

// TODO: is static lifetime OK for these returned references?

/// Return the firmware vendor string
#[must_use]
pub fn firmware_vendor() -> &'static CStr16 {
    unsafe { CStr16::from_ptr(system_table().as_mut().firmware_vendor.cast()) }
}

/// Return the firmware revision
#[must_use]
pub fn firmware_revision() -> u32 {
    unsafe { system_table().as_mut().firmware_revision }
}

/// Returns the revision of this table, which is defined to be
/// the revision of the UEFI specification implemented by the firmware.
#[must_use]
pub fn uefi_revision() -> Revision {
    unsafe { system_table().as_mut().header.revision }
}

/// TODO
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

/// TODO
pub fn with_stdout<F, R>(f: F) -> R
where
    F: Fn(Option<&mut text::Output>) -> R,
{
    let st = system_table_maybe_null();
    if st.is_null() {
        f(None)
    } else {
        let st = unsafe { &*st };
        let output_ptr: *mut text::Output = st.stdout.cast();
        if output_ptr.is_null() {
            f(None)
        } else {
            let output = unsafe { &mut *output_ptr };
            f(Some(output))
        }
    }
}
