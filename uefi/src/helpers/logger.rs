// SPDX-License-Identifier: MIT OR Apache-2.0

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
use crate::system;
use core::fmt::{self, Write};
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Global logger object
static LOGGER: Logger = Logger::new();

/// Set up logging
///
/// This is unsafe because you must arrange for the logger to be reset with
/// disable() on exit from UEFI boot services.
pub unsafe fn init() {
    // Connect the logger to stdout.
    system::with_stdout(|stdout| unsafe {
        LOGGER.set_output(stdout);
    });

    // Set the logger.
    log::set_logger(&LOGGER).unwrap(); // Can only fail if already initialized.

    // Set logger max level to level specified by log features
    log::set_max_level(log::STATIC_MAX_LEVEL);
}

pub fn disable() {
    LOGGER.disable();
}

/// Writer to the QEMU debugcon device and the debug-console of
/// cloud-hypervisor.
///
/// More info: <https://phip1611.de/blog/how-to-use-qemus-debugcon-feature/>
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "log-debugcon"
))]
#[derive(Copy, Clone, Debug)]
struct DebugconWriter;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "log-debugcon"
))]
impl DebugconWriter {
    const IO_PORT: u16 = 0xe9;
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    feature = "log-debugcon"
))]
impl core::fmt::Write for DebugconWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &byte in s.as_bytes() {
            unsafe {
                core::arch::asm!("outb %al, %dx", in("al") byte, in("dx") Self::IO_PORT, options(att_syntax))
            };
        }
        Ok(())
    }
}

/// Logging implementation which writes to a UEFI output stream.
///
/// If this logger is used as a global logger, you must disable it using the
/// `disable` method before exiting UEFI boot services in order to prevent
/// undefined behaviour from inadvertent logging.
#[derive(Debug)]
pub struct Logger {
    writer: AtomicPtr<Output>,
}

impl Logger {
    /// Creates a new logger.
    ///
    /// The logger is initially disabled. Call [`set_output`] to enable it.
    ///
    /// [`set_output`]: Self::set_output
    #[must_use]
    pub const fn new() -> Self {
        Self {
            writer: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Get the output pointer (may be null).
    #[must_use]
    fn output(&self) -> *mut Output {
        self.writer.load(Ordering::Acquire)
    }

    /// Set the [`Output`] to which the logger will write.
    ///
    /// If a null pointer is passed for `output`, this method is equivalent to
    /// calling [`disable`].
    ///
    /// # Safety
    ///
    /// The `output` pointer must either be null or point to a valid [`Output`]
    /// object. That object must remain valid until the logger is either
    /// disabled, or `set_output` is called with a different `output`.
    ///
    /// You must arrange for the [`disable`] method to be called or for this
    /// logger to be otherwise discarded before boot services are exited.
    ///
    /// [`disable`]: Self::disable
    pub unsafe fn set_output(&self, output: *mut Output) {
        self.writer.store(output, Ordering::Release);
    }

    /// Disable the logger.
    pub fn disable(&self) {
        unsafe { self.set_output(ptr::null_mut()) }
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        // We decide in `log` already if something is printed. We do not
        // need micro optimizations here.
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(writer) = unsafe { self.output().as_mut() } {
            // Ignore all errors. Since we're in the logger implementation we
            // can't log the error. We also don't want to panic, since logging
            // is generally not critical functionality.
            let _ = DecoratedLog::write(
                writer,
                record.level(),
                record.args(),
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
            );
        }

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            feature = "log-debugcon"
        ))]
        {
            // Ignore all errors. Since we're in the logger implementation we
            // can't log the error. We also don't want to panic, since logging
            // is generally not critical functionality.
            let _ = DecoratedLog::write(
                &mut DebugconWriter,
                record.level(),
                record.args(),
                record.file().unwrap_or("<unknown file>"),
                record.line().unwrap_or(0),
            );
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

impl<W: fmt::Write> fmt::Write for DecoratedLog<'_, '_, W> {
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
