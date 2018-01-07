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
        Logger { writer: UnsafeCell::new(OutputWriter::new(output)) }
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
