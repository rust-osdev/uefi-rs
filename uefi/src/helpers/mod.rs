//! This module provides miscellaneous opinionated but optional helpers to
//! better integrate your application with the Rust runtime and the Rust
//! ecosystem.
//!
//! For now, this includes:
//! - using [`uefi::allocator::Allocator`] as global allocator (feature `global_allocator`)
//! - an implementation of  [`log::Log`] (feature `logger`) which logs to
//!   the stdout text protocol of UEFI (as long as boot services were not
//!   excited) and to the [debugcon device](https://phip1611.de/blog/how-to-use-qemus-debugcon-feature/)
//!   (only on x86)  (feature `log-debugcon`).
//! - [`print!`][print_macro] and [`println!`][println_macro] macros defaulting
//!   to the uefi boot service stdout stream
//! - default panic handler (feature `panic_handler`)
//!
//! **PLEASE NOTE** that these helpers are meant for the pre exit boot service
//! epoch.
//!
//! [print_macro]: uefi::print!
//! [println_macro]: uefi::println!

use crate::prelude::{Boot, SystemTable};
use crate::{Result, StatusExt};
use core::ffi::c_void;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
#[doc(hidden)]
pub use println::_print;
use uefi_raw::Status;

#[cfg(feature = "global_allocator")]
mod global_allocator;
#[cfg(feature = "logger")]
mod logger;
#[cfg(feature = "panic_handler")]
mod panic_handler;
mod println;

/// Reference to the system table.
///
/// This table is only fully safe to use until UEFI boot services have been exited.
/// After that, some fields and methods are unsafe to use, see the documentation of
/// UEFI's ExitBootServices entry point for more details.
static SYSTEM_TABLE: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

#[must_use]
fn system_table_opt() -> Option<SystemTable<Boot>> {
    let ptr = SYSTEM_TABLE.load(Ordering::Acquire);
    // Safety: the `SYSTEM_TABLE` pointer either be null or a valid system
    // table.
    //
    // Null is the initial value, as well as the value set when exiting boot
    // services. Otherwise, the value is set by the call to `init`, which
    // requires a valid system table reference as input.
    unsafe { SystemTable::from_ptr(ptr) }
}

/// Obtains a pointer to the system table.
///
/// This is meant to be used by higher-level libraries,
/// which want a convenient way to access the system table singleton.
///
/// `init` must have been called first by the UEFI app.
///
/// The returned pointer is only valid until boot services are exited.
#[must_use]
// TODO do we want to keep this public?
pub fn system_table() -> SystemTable<Boot> {
    system_table_opt().expect("The system table handle is not available")
}

/// Initialize all helpers defined in [`uefi::helpers`] whose Cargo features
/// are activated.
///
/// This must be called as early as possible, before trying to use logging or
/// memory allocation capabilities.
///
/// **PLEASE NOTE** that these helpers are meant for the pre exit boot service
/// epoch. Limited functionality might work after exiting them, such as logging
/// to the debugcon device.
pub fn init(st: &mut SystemTable<Boot>) -> Result<()> {
    if system_table_opt().is_some() {
        // Avoid double initialization.
        return Status::SUCCESS.to_result_with_val(|| ());
    }

    // Setup the system table singleton
    SYSTEM_TABLE.store(st.as_ptr().cast_mut(), Ordering::Release);

    // Setup logging and memory allocation

    #[cfg(feature = "logger")]
    unsafe {
        logger::init(st);
    }

    #[cfg(feature = "global_allocator")]
    unsafe {
        crate::allocator::init(st);
    }

    Ok(())
}

pub(crate) fn exit() {
    // DEBUG: The UEFI spec does not guarantee that this printout will work, as
    //        the services used by logging might already have been shut down.
    //        But it works on current OVMF, and can be used as a handy way to
    //        check that the callback does get called.
    //
    // info!("Shutting down the UEFI utility library");
    SYSTEM_TABLE.store(ptr::null_mut(), Ordering::Release);

    #[cfg(feature = "logger")]
    logger::disable();

    #[cfg(feature = "global_allocator")]
    crate::allocator::exit_boot_services();
}
