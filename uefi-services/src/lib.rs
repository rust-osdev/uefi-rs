//! This crate simplifies the writing of higher-level code for UEFI.
//!
//! It initializes the memory allocation and logging crates,
//! allowing code to use Rust's data structures and to log errors.
//!
//! It also stores a global reference to the UEFI system table,
//! in order to reduce the redundant passing of references to it.
//!
//! Library code can simply use global UEFI functions
//! through the reference provided by `system_table`.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(panic_info_message)]

// These crates are required.
extern crate rlibc;

// Core types.
extern crate uefi;

// Logging support
extern crate uefi_logger;

// Allocator support.
extern crate uefi_alloc;

#[macro_use]
extern crate log;

use core::ptr::NonNull;

use uefi::{Event, Result};
use uefi::prelude::*;
use uefi::table::SystemTable;
use uefi::table::boot::{EventType, Tpl};

/// Reference to the system table.
///
/// This table is only fully safe to use until UEFI boot services have been exited.
/// After that, some fields and methods are unsafe to use, see the documentation of
/// UEFI's ExitBootServices entry point for more details.
static mut SYSTEM_TABLE: Option<NonNull<SystemTable>> = None;

/// Global logger object
static mut LOGGER: Option<uefi_logger::Logger> = None;

/// Obtains a pointer to the system table.
///
/// This is meant to be used by higher-level libraries,
/// which want a convenient way to access the system table singleton.
///
/// `init` must have been called first by the UEFI app.
pub fn system_table() -> NonNull<SystemTable> {
    unsafe { SYSTEM_TABLE.expect("The uefi-services library has not yet been initialized") }
}

/// Initialize the UEFI utility library.
///
/// This must be called as early as possible,
/// before trying to use logging or memory allocation capabilities.
pub fn init(st: &SystemTable) -> Result<()> {
    // Avoid double initialization.
    if unsafe { SYSTEM_TABLE.is_some() } {
        return Status::SUCCESS.into();
    }

    // Setup the system table singleton
    unsafe { SYSTEM_TABLE = NonNull::new(st as *const _ as *mut _) };

    // Setup logging and memory allocation
    let boot_services = st.boot_services();
    unsafe {
        init_logger(st);
        uefi_alloc::init(boot_services);
    }

    // Schedule these services to be shut down on exit from UEFI boot services
    boot_services.create_event(EventType::SIGNAL_EXIT_BOOT_SERVICES,
                               Tpl::NOTIFY,
                               Some(exit_boot_services)).map_inner(|_| ())
}

/// Set up logging
///
/// This is unsafe because you must arrange for the logger to be reset with
/// disable() on exit from UEFI boot services.
unsafe fn init_logger(st: &SystemTable) {
    let stdout = st.stdout();

    // Construct the logger.
    let logger = {
        LOGGER = Some(uefi_logger::Logger::new(stdout));
        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Log everything.
    log::set_max_level(log::LevelFilter::Info);
}

/// Notify the utility library that boot services are not safe to call anymore
fn exit_boot_services(_e: Event) {
    unsafe {
        SYSTEM_TABLE = None;
        if let Some(ref mut logger) = LOGGER {
            logger.disable();
        }
    }
    uefi_alloc::exit_boot_services();
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
        if let Some(message) = info.message() {
            error!("{}", message);
        }
    }

    // Give the user some time to read the message
    if let Some(st) = unsafe { SYSTEM_TABLE } {
        // This is safe if the user makes sure to call exit_boot_services before
        // exiting UEFI's boot services, as that will reset SYSTEM_TABLE.
        unsafe {
            st.as_ref().boot_services().stall(10_000_000);
        }
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            unsafe {
                core::ptr::write_volatile(&mut dummy, i);
            }
        }
    }

    // If running in QEMU, use the f4 exit port to signal the error and exit
    if cfg!(feature = "qemu") {
        use x86_64::instructions::port::Port;
        let mut port = Port::<u32>::new(0xf4);
        unsafe {
            port.write(42);
        }
    }

    // If the system table is available, use UEFI's standard shutdown mechanism
    if let Some(st) = unsafe { SYSTEM_TABLE } {
        use uefi::table::runtime::ResetType;
        unsafe {
            st.as_ref()
                .runtime_services()
                .reset(ResetType::Shutdown, uefi::Status::ABORTED, None)
        }
    }

    // If we don't have any shutdown mechanism handy, the best we can do is loop
    error!("Could not shut down, please power off the system manually...");

    loop {
        unsafe {
            // Try to at least keep CPU from running at 100%
            asm!("hlt" :::: "volatile");
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
