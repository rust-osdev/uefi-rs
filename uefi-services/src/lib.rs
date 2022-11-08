//! This crate simplifies the writing of higher-level code for UEFI.
//!
//! It initializes the memory allocation and logging crates,
//! allowing code to use Rust's data structures and to log errors.
//!
//! Logging and allocation are only allowed while boot services are
//! active. Once runtime services are activated by calling
//! [`exit_boot_services`], the logger will be disabled and the
//! allocator will always return null.
//!
//! It also stores a global reference to the UEFI system table,
//! in order to reduce the redundant passing of references to it.
//!
//! Library code can simply use global UEFI functions
//! through the reference provided by `system_table`.
//!
//! [`exit_boot_services`]: uefi::table::SystemTable::exit_boot_services

#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_efiapi)]

extern crate log;
// Core types.
extern crate uefi;

use core::ffi::c_void;
use core::fmt::Write;
use core::ptr::NonNull;

use cfg_if::cfg_if;

use uefi::prelude::*;
use uefi::table::boot::{EventType, Tpl};
use uefi::table::{Boot, SystemTable};
use uefi::{Event, Result};

/// Reference to the system table.
///
/// This table is only fully safe to use until UEFI boot services have been exited.
/// After that, some fields and methods are unsafe to use, see the documentation of
/// UEFI's ExitBootServices entry point for more details.
static mut SYSTEM_TABLE: Option<SystemTable<Boot>> = None;

/// Global logger object
#[cfg(feature = "logger")]
static mut LOGGER: Option<uefi::logger::Logger> = None;

/// Obtains a pointer to the system table.
///
/// This is meant to be used by higher-level libraries,
/// which want a convenient way to access the system table singleton.
///
/// `init` must have been called first by the UEFI app.
///
/// The returned pointer is only valid until boot services are exited.
pub fn system_table() -> NonNull<SystemTable<Boot>> {
    unsafe {
        let table_ref = SYSTEM_TABLE
            .as_ref()
            .expect("The system table handle is not available");
        NonNull::new(table_ref as *const _ as *mut _).unwrap()
    }
}

/// Initialize the UEFI utility library.
///
/// This must be called as early as possible,
/// before trying to use logging or memory allocation capabilities.
pub fn init(st: &mut SystemTable<Boot>) -> Result {
    unsafe {
        // Avoid double initialization.
        if SYSTEM_TABLE.is_some() {
            return Status::SUCCESS.into();
        }

        // Setup the system table singleton
        SYSTEM_TABLE = Some(st.unsafe_clone());

        // Setup logging and memory allocation

        #[cfg(feature = "logger")]
        init_logger(st);

        let boot_services = st.boot_services();
        uefi::alloc::init(boot_services);

        // Schedule these tools to be disabled on exit from UEFI boot services
        boot_services
            .create_event(
                EventType::SIGNAL_EXIT_BOOT_SERVICES,
                Tpl::NOTIFY,
                Some(exit_boot_services),
                None,
            )
            .map(|_| ())
    }
}

// Internal function for print macros.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    unsafe {
        let st = SYSTEM_TABLE
            .as_mut()
            .expect("The system table handle is not available");

        st.stdout()
            .write_fmt(args)
            .expect("Failed to write to stdout");
    }
}

/// Prints to the standard output.
///
/// # Panics
/// Will panic if `SYSTEM_TABLE` is `None` (Before [init()] and after [uefi::prelude::SystemTable::exit_boot_services()]).
///
/// # Examples
/// ```
/// print!("");
/// print!("Hello World\n");
/// print!("Hello {}", "World");
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(core::format_args!($($arg)*)));
}

/// Prints to the standard output, with a newline.
///
/// # Panics
/// Will panic if `SYSTEM_TABLE` is `None` (Before [init()] and after [uefi::prelude::SystemTable::exit_boot_services()]).
///
/// # Examples
/// ```
/// println!();
/// println!("Hello World");
/// println!("Hello {}", "World");
/// ```
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::_print(core::format_args!("{}{}", core::format_args!($($arg)*), "\n")));
}

/// Set up logging
///
/// This is unsafe because you must arrange for the logger to be reset with
/// disable() on exit from UEFI boot services.
#[cfg(feature = "logger")]
unsafe fn init_logger(st: &mut SystemTable<Boot>) {
    let stdout = st.stdout();

    // Construct the logger.
    let logger = {
        LOGGER = Some(uefi::logger::Logger::new(stdout));
        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Set logger max level to level specified by log features
    log::set_max_level(log::STATIC_MAX_LEVEL);
}

/// Notify the utility library that boot services are not safe to call anymore
/// As this is a callback, it must be `extern "efiapi"`.
unsafe extern "efiapi" fn exit_boot_services(_e: Event, _ctx: Option<NonNull<c_void>>) {
    // DEBUG: The UEFI spec does not guarantee that this printout will work, as
    //        the services used by logging might already have been shut down.
    //        But it works on current OVMF, and can be used as a handy way to
    //        check that the callback does get called.
    //
    // info!("Shutting down the UEFI utility library");
    SYSTEM_TABLE = None;

    #[cfg(feature = "logger")]
    if let Some(ref mut logger) = LOGGER {
        logger.disable();
    }

    uefi::alloc::exit_boot_services();
}

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC]: {}", info);

    // Give the user some time to read the message
    if let Some(st) = unsafe { SYSTEM_TABLE.as_ref() } {
        st.boot_services().stall(10_000_000);
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            unsafe {
                core::ptr::write_volatile(&mut dummy, i);
            }
        }
    }

    cfg_if! {
        if #[cfg(all(target_arch = "x86_64", feature = "qemu"))] {
            // If running in QEMU, use the f4 exit port to signal the error and exit
            use qemu_exit::QEMUExit;
            let custom_exit_success = 3;
            let qemu_exit_handle = qemu_exit::X86::new(0xF4, custom_exit_success);
            qemu_exit_handle.exit_failure();
        } else {
            // If the system table is available, use UEFI's standard shutdown mechanism
            if let Some(st) = unsafe { SYSTEM_TABLE.as_ref() } {
                use uefi::table::runtime::ResetType;
                st.runtime_services()
                    .reset(ResetType::Shutdown, uefi::Status::ABORTED, None);
            }

            // If we don't have any shutdown mechanism handy, the best we can do is loop
            log::error!("Could not shut down, please power off the system manually...");

            cfg_if! {
                if #[cfg(target_arch = "x86_64")] {
                    loop {
                        unsafe {
                            // Try to at least keep CPU from running at 100%
                            core::arch::asm!("hlt", options(nomem, nostack));
                        }
                    }
                } else if #[cfg(target_arch = "aarch64")] {
                    loop {
                        unsafe {
                            // Try to at least keep CPU from running at 100%
                            core::arch::asm!("hlt 420", options(nomem, nostack));
                        }
                    }
                } else {
                    loop {
                        // just run forever dammit how do you return never anyway
                    }
                }
            }
        }
    }
}

#[alloc_error_handler]
fn out_of_memory(layout: ::core::alloc::Layout) -> ! {
    panic!(
        "Ran out of free memory while trying to allocate {:#?}",
        layout
    );
}
