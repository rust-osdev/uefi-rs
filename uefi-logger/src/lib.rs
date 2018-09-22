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
use core::fmt::{self, Write};

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
        let writer = unsafe { &mut *self.writer.get() };
        DecoratedLog::write(writer,
                            record.level(),
                            record.args()).unwrap();
    }

    fn flush(&self) {
        // This simple logger does not buffer output.
    }
}

// The logger is not thread-safe, but the UEFI boot environment only uses one processor.
unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}


/// Writer wrapper which prints a log level in front of every line of text
///
/// This is less easy than it sounds because...
///
/// 1. The fmt::Arguments is a rather opaque type, the ~only thing you can do
///    with it is to hand it to an fmt::Write implementation.
/// 2. Without using memory allocation, the easy cop-out of writing everything
///    to a String then post-processing is not available.
///
/// Therefore, we need to inject ourselves in the middle of the fmt::Write
/// machinery and intercept the strings that it sends to the Writer.
///.
struct DecoratedLog<'a, W: fmt::Write> {
    backend: &'a mut W,
    log_level: log::Level,
    at_line_start: bool,
}
//
impl<'a, W: fmt::Write> DecoratedLog<'a, W> {
    // Call this method to print a level-annotated log
    fn write(writer: &'a mut W,
             level: log::Level,
             args: &fmt::Arguments) -> fmt::Result {
        let mut decorated_writer = Self {
            backend: writer,
            log_level: level,
            at_line_start: true,
        };
        writeln!(decorated_writer, "{}", *args)
    }
}
//
impl<'a, W: fmt::Write> fmt::Write for DecoratedLog<'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Split the input string into lines
        let mut lines = s.lines();

        // The beginning of the input string may actually fall in the middle of
        // a line of output. We only print the log level if it truly is at the
        // beginning of a line of output.
        let first = lines.next().unwrap_or("");
        if self.at_line_start {
            write!(self.backend, "{}: ", self.log_level);
            self.at_line_start = false;
        }
        write!(self.backend, "{}", first)?;

        // For the remainder of the line iterator (if any), we know that we are
        // truly at the beginning of lines of output.
        for line in lines {
            write!(self.backend, "\n{}: {}", self.log_level, line);
        }

        // If the string ends with a newline character, we must 1/propagate it
        // to the output (it was swallowed by the iteration) and 2/prepare to
        // write the log level of the beginning of the next line (if any).
        if let Some('\n') = s.chars().next_back() {
            writeln!(self.backend);
            self.at_line_start = true;
        }
        Ok(())
    }
}