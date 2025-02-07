// SPDX-License-Identifier: MIT OR Apache-2.0

//! Standard UEFI tables.

pub mod cfg;

mod header;

pub use header::Header;
pub use uefi_raw::table::Revision;

use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, Ordering};

/// Global system table pointer. This is only modified by [`set_system_table`].
static SYSTEM_TABLE: AtomicPtr<uefi_raw::table::system::SystemTable> =
    AtomicPtr::new(ptr::null_mut());

/// Get the raw system table pointer.
///
/// If called before `set_system_table` has been called, this will return `None`.
pub fn system_table_raw() -> Option<NonNull<uefi_raw::table::system::SystemTable>> {
    let ptr = SYSTEM_TABLE.load(Ordering::Acquire);
    NonNull::new(ptr)
}

/// Get the raw system table pointer. This may only be called after
/// `set_system_table` has been used to set the global pointer.
///
/// # Panics
///
/// Panics if the global system table pointer is null.
#[track_caller]
pub(crate) fn system_table_raw_panicking() -> NonNull<uefi_raw::table::system::SystemTable> {
    system_table_raw().expect("global system table pointer is not set")
}

/// Update the global system table pointer.
///
/// This is called automatically in the `main` entry point as part of
/// [`uefi::entry`].
///
/// It is also called by [`set_virtual_address_map`] to transition from a
/// physical address to a virtual address.
///
/// This function should not be called at any other point in time, unless the
/// executable does not use [`uefi::entry`], in which case it should be called
/// once before calling any other API in this crate.
///
/// # Safety
///
/// This function should only be called as described above, and the
/// `ptr` must be a valid [`SystemTable`].
///
/// [`SystemTable`]: uefi_raw::table::system::SystemTable
/// [`set_virtual_address_map`]: uefi::runtime::set_virtual_address_map
pub unsafe fn set_system_table(ptr: *const uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.store(ptr.cast_mut(), Ordering::Release);
}

/// Common trait implemented by all standard UEFI tables.
pub trait Table {
    /// A unique number assigned by the UEFI specification
    /// to the standard tables.
    const SIGNATURE: u64;
}
