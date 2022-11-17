//! This optional feature adds support for the `log` crate, providing
//! a custom logger implementation which writes to a UEFI text output protocol.
//!
//! The main export of this module is the `Logger` structure,
//! which implements the `log` crate's trait `Log`.
//!
//! # Implementation details
//!
//! The implementation is not the most efficient, since there is no buffering done,
//! and the messages have to be converted from UTF-8 to UEFI's UCS-2.
//!
//! The last part also means that some Unicode characters might not be
//! supported by the UEFI console. Don't expect emoji output support.

use crate::proto::console::text::Output;

use core::fmt::{self, Write};
use core::ptr::NonNull;

/// Logging implementation which writes to a UEFI output stream.
///
/// If this logger is used as a global logger, you must disable it using the
/// `disable` method before exiting UEFI boot services in order to prevent
/// undefined behaviour from inadvertent logging.
pub struct Logger {
    writer: Option<NonNull<Output<'static>>>,
}

impl Logger {
    /// Creates a new logger.
    ///
    /// You must arrange for the `disable` method to be called or for this logger
    /// to be otherwise discarded before boot services are exited.
    ///
    /// # Safety
    ///
    /// Undefined behaviour may occur if this logger is still active after the
    /// application has exited the boot services stage.
    pub unsafe fn new(output: &mut Output) -> Self {
        Logger {
            writer: NonNull::new(output as *const _ as *mut _),
        }
    }

    /// Disable the logger
    pub fn disable(&mut self) {
        self.writer = None;
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        self.writer.is_some()
    }

    fn log(&self, record: &log::Record) {
        if let Some(mut ptr) = self.writer {
            let writer = unsafe { ptr.as_mut() };
            let result = DecoratedLog::write(
                writer,
                record.level(),
                record.args(),
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
            );

            // Some UEFI implementations, such as the one used by VirtualBox,
            // may intermittently drop out some text from SimpleTextOutput and
            // report an EFI_DEVICE_ERROR. This will be reported here as an
            // `fmt::Error`, and given how the `log` crate is designed, our main
            // choices when that happens are to ignore the error or panic.
            //
            // Ignoring errors is bad, especially when they represent loss of
            // precious early-boot system diagnosis data, so we panic by
            // default. But if you experience this problem and want your UEFI
            // application to keep running when it happens, you can disable the
            // `panic-on-logger-errors` cargo feature. If you do so, logging errors
            // will be ignored by `uefi-rs` instead.
            //
            if cfg!(feature = "panic-on-logger-errors") {
                result.unwrap()
            }
        }
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
struct DecoratedLog<'writer, 'a, W: fmt::Write> {
    writer: &'writer mut W,
    log_level: log::Level,
    at_line_start: bool,
    file: &'a str,
    line: u32,
}

impl<'writer, 'a, W: fmt::Write> DecoratedLog<'writer, 'a, W> {
    // Call this method to print a level-annotated log
    fn write(
        writer: &'writer mut W,
        log_level: log::Level,
        args: &fmt::Arguments,
        file: &'a str,
        line: u32,
    ) -> fmt::Result {
        let mut decorated_writer = Self {
            writer,
            log_level,
            at_line_start: true,
            file,
            line,
        };
        writeln!(decorated_writer, "{}", *args)
    }
}

impl<'writer, 'a, W: fmt::Write> fmt::Write for DecoratedLog<'writer, 'a, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Split the input string into lines
        let mut lines = s.lines();

        // The beginning of the input string may actually fall in the middle of
        // a line of output. We only print the log level if it truly is at the
        // beginning of a line of output.
        let first = lines.next().unwrap_or("");
        if self.at_line_start {
            write!(
                self.writer,
                "[{:>5}]: {:>12}@{:03}: ",
                self.log_level, self.file, self.line
            )?;
            self.at_line_start = false;
        }
        write!(self.writer, "{first}")?;

        // For the remainder of the line iterator (if any), we know that we are
        // truly at the beginning of lines of output.
        for line in lines {
            let level = self.log_level;
            write!(self.writer, "\n{level}: {line}")?;
        }

        // If the string ends with a newline character, we must 1/propagate it
        // to the output (it was swallowed by the iteration) and 2/prepare to
        // write the log level of the beginning of the next line (if any).
        if let Some('\n') = s.chars().next_back() {
            writeln!(self.writer)?;
            self.at_line_start = true;
        }
        Ok(())
    }
}
