#![no_std]

extern crate uefi;
use uefi::proto::console::text::Output;

extern crate log;

use core::cell::UnsafeCell;

mod writer;
use self::writer::OutputWriter;

/// Logging implementation which writes to a UEFI output stream.
pub struct UefiLogger {
    writer: UnsafeCell<OutputWriter>,
}

impl UefiLogger {
    /// Creates a new logger.
    pub fn new(output: &'static mut Output) -> Self {
        UefiLogger {
            writer: UnsafeCell::new(OutputWriter::new(output))
        }
    }
}

impl log::Log for UefiLogger {
    fn enabled(&self, _metadata: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        let args = record.args();

        let writer = unsafe { &mut *self.writer.get() };
        use core::fmt::Write;
        writeln!(writer, "{}", args);
    }
}

// The logger is not thread-safe, but the UEFI boot environment only uses one processor.
unsafe impl Sync for UefiLogger {}
unsafe impl Send for UefiLogger {}
