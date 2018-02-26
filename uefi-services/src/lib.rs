#![no_std]

#![feature(compiler_builtins_lib)]
#![feature(lang_items)]

// These crates are required.
extern crate rlibc;
extern crate compiler_builtins;

// Core types.
extern crate uefi;

// Logging support
extern crate uefi_logger;

// Allocator support.
extern crate uefi_alloc;

#[macro_use]
extern crate log;

use uefi::table::SystemTable;

/// Initialize the UEFI utility library.
///
/// This must be called as early as possible,
/// before trying to use logging or memory allocation capabilities.
pub fn init(st: &'static SystemTable) {
    init_logger(st);
    init_alloc(st);
}

fn init_logger(st: &'static SystemTable) {
    static mut LOGGER: Option<uefi_logger::Logger> = None;

    let stdout = st.stdout();

    // Construct the logger.
    let logger = unsafe {
        LOGGER = Some(uefi_logger::Logger::new(stdout));

        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Log everything.
    log::set_max_level(log::LevelFilter::Info);
}

fn init_alloc(st: &'static SystemTable) {
    uefi_alloc::init(st.boot);
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub fn panic_fmt(_fmt: core::fmt::Arguments, file_line_col: &(&'static str, u32, u32)) {
    let &(file, line, column) = file_line_col;

    error!("Panic in {} at ({}, {})", file, line, column);

    loop {
        // TODO: add a timeout then shutdown.
    }
}
