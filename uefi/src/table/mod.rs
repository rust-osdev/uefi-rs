//! Standard UEFI tables.

pub mod boot;
pub mod cfg;
pub mod runtime;

mod header;
mod system;

pub use header::Header;
pub use system::{Boot, Runtime, SystemTable};
pub use uefi_raw::table::Revision;

use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Global system table pointer. This is only modified by [`set_system_table`].
static SYSTEM_TABLE: AtomicPtr<uefi_raw::table::system::SystemTable> =
    AtomicPtr::new(ptr::null_mut());

/// Update the global system table pointer.
///
/// This is called automatically in the `main` entry point as part of
/// [`uefi::entry`]. It should not be called at any other point in time, unless
/// the executable does not use [`uefi::entry`], in which case it should be
/// called once before calling any other API in this crate.
///
/// # Safety
///
/// This function should only be called as described above, and the
/// `ptr` must be a valid [`SystemTable`].
pub unsafe fn set_system_table(ptr: *const uefi_raw::table::system::SystemTable) {
    SYSTEM_TABLE.store(ptr.cast_mut(), Ordering::Release);
}

/// Get the system table while boot services are active.
///
/// # Panics
///
/// Panics if the system table has not been set with `set_system_table`, or if
/// boot services are not available (e.g. if [`exit_boot_services`] has been
/// called).
///
/// [`exit_boot_services`]: SystemTable::exit_boot_services
pub fn system_table_boot() -> SystemTable<Boot> {
    let st = SYSTEM_TABLE.load(Ordering::Acquire);
    assert!(!st.is_null());

    // SAFETY: the system table is valid per the requirements of `set_system_table`.
    unsafe {
        if (*st).boot_services.is_null() {
            panic!("boot services are not active");
        }

        SystemTable::<Boot>::from_ptr(st.cast()).unwrap()
    }
}

/// Get the system table while runtime services are active.
///
/// # Panics
///
/// Panics if the system table has not been set with `set_system_table`, or if
/// runtime services are not available.
pub fn system_table_runtime() -> SystemTable<Runtime> {
    let st = SYSTEM_TABLE.load(Ordering::Acquire);
    assert!(!st.is_null());

    // SAFETY: the system table is valid per the requirements of `set_system_table`.
    unsafe {
        if (*st).runtime_services.is_null() {
            panic!("runtime services are not active");
        }

        SystemTable::<Runtime>::from_ptr(st.cast()).unwrap()
    }
}

/// Common trait implemented by all standard UEFI tables.
pub trait Table {
    /// A unique number assigned by the UEFI specification
    /// to the standard tables.
    const SIGNATURE: u64;
}
