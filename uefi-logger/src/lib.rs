//! This crate adds support for the `log` crate, providing
//! a custom logger implementation which writes to a UEFI text output protocol.
//!
//! The main export of this library is the `Logger` structure,
//! which implements the `log` crate's trait `Log`.
//!
//! # Implementation details
//!
//! The implementation is not the most efficient, since there is no buffering done,
//! and the messages have to be converted from UTF-8 to UEFI's UCS-2.
//!
//! The last part also means that some Unicode characters might not be
//! supported by the UEFI console. Don't expect emoji output support.

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy))]
#![no_std]

extern crate uefi;
use uefi::proto::console::text::Output;

extern crate log;

use core::cell::UnsafeCell;

mod writer;
use self::writer::OutputWriter;

/// Logging implementation which writes to a UEFI output stream.
pub struct Logger {
    writer: UnsafeCell<OutputWriter>,
}

impl Logger {
    /// Creates a new logger.
    pub fn new(output: &'static mut Output) -> Self {
        Logger {
            writer: UnsafeCell::new(OutputWriter::new(output)),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let args = record.args();

        let writer = unsafe { &mut *self.writer.get() };
        use core::fmt::Write;
        writeln!(writer, "{}", args).unwrap();
    }

    fn flush(&self) {
        // This simple logger does not buffer output.
    }
}

// The logger is not thread-safe, but the UEFI boot environment only uses one processor.
unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}
