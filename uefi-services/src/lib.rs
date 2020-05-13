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
#![feature(llvm_asm)]
#![feature(lang_items)]
#![feature(panic_info_message)]

// Core types.
extern crate uefi;

#[macro_use]
extern crate log;

use core::ptr::NonNull;

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
pub fn init(st: &SystemTable<Boot>) -> Result {
    unsafe {
        // Avoid double initialization.
        if SYSTEM_TABLE.is_some() {
            return Status::SUCCESS.into();
        }

        // Setup the system table singleton
        SYSTEM_TABLE = Some(st.unsafe_clone());

        // Setup logging and memory allocation
        let boot_services = st.boot_services();
        init_logger(st);
        uefi::alloc::init(boot_services);

        // Schedule these tools to be disabled on exit from UEFI boot services
        boot_services
            .create_event(
                EventType::SIGNAL_EXIT_BOOT_SERVICES,
                Tpl::NOTIFY,
                Some(exit_boot_services),
            )
            .map_inner(|_| ())
    }
}

/// Set up logging
///
/// This is unsafe because you must arrange for the logger to be reset with
/// disable() on exit from UEFI boot services.
unsafe fn init_logger(st: &SystemTable<Boot>) {
    let stdout = st.stdout();

    // Construct the logger.
    let logger = {
        LOGGER = Some(uefi::logger::Logger::new(stdout));
        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Log everything.
    log::set_max_level(log::LevelFilter::Info);
}

/// Notify the utility library that boot services are not safe to call anymore
fn exit_boot_services(_e: Event) {
    // DEBUG: The UEFI spec does not guarantee that this printout will work, as
    //        the services used by logging might already have been shut down.
    //        But it works on current OVMF, and can be used as a handy way to
    //        check that the callback does get called.
    //
    // info!("Shutting down the UEFI utility library");
    unsafe {
        SYSTEM_TABLE = None;
        if let Some(ref mut logger) = LOGGER {
            logger.disable();
        }
    }
    uefi::alloc::exit_boot_services();
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

    // If running in QEMU, use the f4 exit port to signal the error and exit
    if cfg!(feature = "qemu") {
        use x86_64::instructions::port::Port;
        let mut port = Port::<u32>::new(0xf4);
        unsafe {
            port.write(42);
        }
    }

    // If the system table is available, use UEFI's standard shutdown mechanism
    if let Some(st) = unsafe { SYSTEM_TABLE.as_ref() } {
        use uefi::table::runtime::ResetType;
        st.runtime_services()
            .reset(ResetType::Shutdown, uefi::Status::ABORTED, None);
    }

    // If we don't have any shutdown mechanism handy, the best we can do is loop
    error!("Could not shut down, please power off the system manually...");

    loop {
        unsafe {
            // Try to at least keep CPU from running at 100%
            llvm_asm!("hlt" :::: "volatile");
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
