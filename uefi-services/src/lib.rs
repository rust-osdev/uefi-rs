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
//! ## Optional crate features
//!
//! - `logger` (enabled by default): Initialize a global logger.
//! - `panic_handler` (enabled by default): Register a panic handler. A
//!   panic handler must be provided for your program to compile, but
//!   you can choose to provide your own if you don't want to use this
//!   one.
//! - `qemu`: On x86_64, make qemu exit with code 3 if a panic
//!   occurs. This feature assumes the program is running under QEMU.
//!
//! [`exit_boot_services`]: uefi::table::SystemTable::exit_boot_services

#![no_std]
#![deny(clippy::must_use_candidate)]
#![deny(missing_debug_implementations)]

extern crate log;
// Core types.
extern crate uefi;

use core::ffi::c_void;
use core::fmt::Write;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(feature = "panic_handler")]
use cfg_if::cfg_if;

use uefi::table::boot::{EventType, Tpl};
use uefi::table::{Boot, SystemTable};
use uefi::{Event, Result, Status, StatusExt};

/// Reference to the system table.
///
/// This table is only fully safe to use until UEFI boot services have been exited.
/// After that, some fields and methods are unsafe to use, see the documentation of
/// UEFI's ExitBootServices entry point for more details.
static SYSTEM_TABLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Global logger object
#[cfg(feature = "logger")]
static LOGGER: uefi::logger::Logger = uefi::logger::Logger::new();

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
pub fn system_table() -> SystemTable<Boot> {
    system_table_opt().expect("The system table handle is not available")
}

/// Initialize the UEFI utility library.
///
/// This must be called as early as possible,
/// before trying to use logging or memory allocation capabilities.
pub fn init(st: &mut SystemTable<Boot>) -> Result<Option<Event>> {
    if system_table_opt().is_some() {
        // Avoid double initialization.
        return Status::SUCCESS.to_result_with_val(|| None);
    }

    // Setup the system table singleton
    SYSTEM_TABLE.store(st.as_ptr().cast_mut(), Ordering::Release);

    unsafe {
        // Setup logging and memory allocation

        #[cfg(feature = "logger")]
        init_logger(st);

        uefi::allocator::init(st);

        // Schedule these tools to be disabled on exit from UEFI boot services
        let boot_services = st.boot_services();
        boot_services
            .create_event(
                EventType::SIGNAL_EXIT_BOOT_SERVICES,
                Tpl::NOTIFY,
                Some(exit_boot_services),
                None,
            )
            .map(Some)
    }
}

// Internal function for print macros.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    system_table()
        .stdout()
        .write_fmt(args)
        .expect("Failed to write to stdout");
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
    // Connect the logger to stdout.
    LOGGER.set_output(st.stdout());

    // Set the logger.
    log::set_logger(&LOGGER).unwrap(); // Can only fail if already initialized.

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
    SYSTEM_TABLE.store(ptr::null_mut(), Ordering::Release);

    #[cfg(feature = "logger")]
    LOGGER.disable();

    uefi::allocator::exit_boot_services();
}

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("[PANIC]: {}", info);

    // Give the user some time to read the message
    if let Some(st) = system_table_opt() {
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
            if let Some(st) = system_table_opt() {
                use uefi::table::runtime::ResetType;
                st.runtime_services()
                    .reset(ResetType::SHUTDOWN, uefi::Status::ABORTED, None);
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
