// SPDX-License-Identifier: MIT OR Apache-2.0

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

use crate::Result;
#[doc(hidden)]
pub use println::_print;

#[cfg(feature = "global_allocator")]
mod global_allocator;
#[cfg(feature = "logger")]
mod logger;
#[cfg(feature = "panic_handler")]
mod panic_handler;
mod println;

/// Initialize all helpers defined in [`uefi::helpers`] whose Cargo features
/// are activated.
///
/// This must be called as early as possible, before trying to use logging.
///
/// **PLEASE NOTE** that these helpers are meant for the pre exit boot service
/// epoch. Limited functionality might work after exiting them, such as logging
/// to the debugcon device.
///
/// # Panics
///
/// This function may panic if called more than once.
#[allow(clippy::missing_const_for_fn)]
pub fn init() -> Result<()> {
    // Set up logging.
    #[cfg(feature = "logger")]
    unsafe {
        logger::init();
    }

    Ok(())
}

#[allow(clippy::missing_const_for_fn)]
pub(crate) fn exit() {
    #[cfg(feature = "logger")]
    logger::disable();
}
