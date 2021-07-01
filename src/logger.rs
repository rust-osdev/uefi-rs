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

use core::fmt::Write;
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

impl<'boot> log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        self.writer.is_some()
    }

    fn log(&self, record: &log::Record) {
        if let Some(mut ptr) = self.writer {
            // Assumption is that 4096 byte is big enough for every possible
            // log message. Stack is cheap and according to UEFI spec, we have
            // at least 128KiB available.
            let mut buf = arrayvec::ArrayString::<4096>::new();
            let result = writeln!(
                buf,
                "[{:>5}]: {:>12}@{:03}: {}",
                record.level(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                record.args(),
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
            // application to keep running when it happens, you can enable the
            // `ignore-logger-error` cargo feature. If you do so, logging errors
            // will be ignored by `uefi-rs` instead.
            //
            if !cfg!(feature = "ignore-logger-errors") {
                result.unwrap()
            }

            // Actually write the data to UEFI stdout.
            let result = unsafe { ptr.as_mut() }.write_str(buf.as_str());
            if !cfg!(feature = "ignore-logger-errors") {
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
