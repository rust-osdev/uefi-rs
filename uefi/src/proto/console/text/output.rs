use crate::proto::unsafe_protocol;
use crate::{CStr16, Result, ResultExt, Status, StatusExt};
use core::fmt;
use core::fmt::{Debug, Formatter};
use uefi_raw::protocol::console::{SimpleTextOutputMode, SimpleTextOutputProtocol};

/// Interface for text-based output devices.
///
/// It implements the fmt::Write trait, so you can use it to print text with
/// standard Rust constructs like the `write!()` and `writeln!()` macros.
///
/// # Accessing `Output` protocol
///
/// The standard output and standard error output protocols can be accessed
/// using [`SystemTable::stdout`] and [`SystemTable::stderr`], respectively.
///
/// An `Output` protocol can also be accessed like any other UEFI protocol.
/// See the [`BootServices`] documentation for more details of how to open a
/// protocol.
///
/// [`SystemTable::stdout`]: crate::table::SystemTable::stdout
/// [`SystemTable::stderr`]: crate::table::SystemTable::stderr
/// [`BootServices`]: crate::table::boot::BootServices#accessing-protocols
#[repr(transparent)]
#[unsafe_protocol(SimpleTextOutputProtocol::GUID)]
pub struct Output(SimpleTextOutputProtocol);

impl Output {
    /// Resets and clears the text output device hardware.
    pub fn reset(&mut self, extended: bool) -> Result {
        unsafe { (self.0.reset)(&mut self.0, extended) }.to_result()
    }

    /// Clears the output screen.
    ///
    /// The background is set to the current background color.
    /// The cursor is moved to (0, 0).
    pub fn clear(&mut self) -> Result {
        unsafe { (self.0.clear_screen)(&mut self.0) }.to_result()
    }

    /// Writes a string to the output device.
    pub fn output_string(&mut self, string: &CStr16) -> Result {
        unsafe { (self.0.output_string)(&mut self.0, string.as_ptr().cast()) }.to_result()
    }

    /// Writes a string to the output device. If the string contains
    /// unknown characters that cannot be rendered they will be silently
    /// skipped.
    pub fn output_string_lossy(&mut self, string: &CStr16) -> Result {
        self.output_string(string).handle_warning(|err| {
            if err.status() == Status::WARN_UNKNOWN_GLYPH {
                Ok(())
            } else {
                Err(err)
            }
        })
    }

    /// Checks if a string contains only supported characters.
    ///
    /// UEFI applications are encouraged to try to print a string even if it contains
    /// some unsupported characters.
    pub fn test_string(&mut self, string: &CStr16) -> Result<bool> {
        match unsafe { (self.0.test_string)(&mut self.0, string.as_ptr().cast()) } {
            Status::UNSUPPORTED => Ok(false),
            other => other.to_result_with_val(|| true),
        }
    }

    /// Returns an iterator of all supported text modes.
    // TODO: Bring back impl Trait once the story around bounds improves
    pub fn modes(&mut self) -> OutputModeIter<'_> {
        let max = self.data().max_mode as usize;
        OutputModeIter {
            output: self,
            current: 0,
            max,
        }
    }

    /// Returns the width (column count) and height (row count) of a text mode.
    ///
    /// Devices are required to support at least an 80x25 text mode and to
    /// assign index 0 to it. If 80x50 is supported, then it will be mode 1,
    /// otherwise querying for mode 1 will return the `Unsupported` error.
    /// Modes 2+ will describe other text modes supported by the device.
    ///
    /// If you want to iterate over all text modes supported by the device,
    /// consider using the iterator produced by `modes()` as a more ergonomic
    /// alternative to this method.
    fn query_mode(&self, index: usize) -> Result<(usize, usize)> {
        let (mut columns, mut rows) = (0, 0);
        let this: *const _ = &self.0;
        unsafe { (self.0.query_mode)(this.cast_mut(), index, &mut columns, &mut rows) }
            .to_result_with_val(|| (columns, rows))
    }

    /// Returns the current text mode.
    pub fn current_mode(&self) -> Result<Option<OutputMode>> {
        match self.data().mode {
            -1 => Ok(None),
            n if n >= 0 => {
                let index = n as usize;
                self.query_mode(index)
                    .map(|dims| Some(OutputMode { index, dims }))
            }
            _ => unreachable!(),
        }
    }

    /// Sets a mode as current.
    pub fn set_mode(&mut self, mode: OutputMode) -> Result {
        unsafe { (self.0.set_mode)(&mut self.0, mode.index) }.to_result()
    }

    /// Returns whether the cursor is currently shown or not.
    #[must_use]
    pub const fn cursor_visible(&self) -> bool {
        self.data().cursor_visible
    }

    /// Make the cursor visible or invisible.
    ///
    /// The output device may not support this operation, in which case an
    /// `Unsupported` error will be returned.
    pub fn enable_cursor(&mut self, visible: bool) -> Result {
        unsafe { (self.0.enable_cursor)(&mut self.0, visible) }.to_result()
    }

    /// Returns the column and row of the cursor.
    #[must_use]
    pub const fn cursor_position(&self) -> (usize, usize) {
        let column = self.data().cursor_column;
        let row = self.data().cursor_row;
        (column as usize, row as usize)
    }

    /// Sets the cursor's position, relative to the top-left corner, which is (0, 0).
    ///
    /// This function will fail if the cursor's new position would exceed the screen's bounds.
    pub fn set_cursor_position(&mut self, column: usize, row: usize) -> Result {
        unsafe { (self.0.set_cursor_position)(&mut self.0, column, row) }.to_result()
    }

    /// Sets the text and background colors for the console.
    ///
    /// Note that for the foreground color you can choose any color.
    /// The background must be one of the first 8 colors.
    pub fn set_color(&mut self, foreground: Color, background: Color) -> Result {
        let fgc = foreground as usize;
        let bgc = background as usize;

        assert!(bgc < 8, "An invalid background color was requested");

        let attr = ((bgc & 0x7) << 4) | (fgc & 0xF);
        unsafe { (self.0.set_attribute)(&mut self.0, attr) }.to_result()
    }

    /// Get a reference to `OutputData`. The lifetime of the reference is tied
    /// to `self`.
    const fn data(&self) -> &SimpleTextOutputMode {
        // Can't dereference mut pointers in a const function, so cast to const.
        let mode = self.0.mode.cast_const();
        unsafe { &*mode }
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Allocate a small buffer on the stack.
        const BUF_SIZE: usize = 128;
        // Add 1 extra character for the null terminator.
        let mut buf = [0u16; BUF_SIZE + 1];

        let mut i = 0;

        // This closure writes the local buffer to the output and resets the buffer.
        let mut flush_buffer = |buf: &mut [u16], i: &mut usize| {
            buf[*i] = 0;
            let codes = &buf[..=*i];
            *i = 0;

            let text = CStr16::from_u16_with_nul(codes).map_err(|_| fmt::Error)?;

            self.output_string(text).map_err(|_| fmt::Error)
        };

        // This closure converts a character to UCS-2 and adds it to the buffer,
        // flushing it as necessary.
        let mut add_char = |ch| {
            // UEFI only supports UCS-2 characters, not UTF-16,
            // so there are no multibyte characters.
            buf[i] = ch;
            i += 1;

            if i == BUF_SIZE {
                flush_buffer(&mut buf, &mut i).map_err(|_| ucs2::Error::BufferOverflow)
            } else {
                Ok(())
            }
        };

        // This one converts Rust line feeds to UEFI line feeds beforehand
        let add_ch = |ch| {
            if ch == '\n' as u16 {
                add_char('\r' as u16)?;
            }
            add_char(ch)
        };

        // Translate and write the input string, flushing the buffer when needed
        ucs2::encode_with(s, add_ch).map_err(|_| fmt::Error)?;

        // Flush the remainder of the buffer
        flush_buffer(&mut buf, &mut i)
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("reset (fn ptr)", &(self.0.reset as *const u64))
            .field(
                "output_string (fn ptr)",
                &(self.0.output_string as *const u64),
            )
            .field("test_string (fn ptr)", &(self.0.test_string as *const u64))
            .field("query_mode (fn ptr)", &(self.0.query_mode as *const u64))
            .field("set_mode (fn ptr)", &(self.0.set_mode as *const u64))
            .field(
                "set_attribute (fn ptr)",
                &(self.0.set_attribute as *const u64),
            )
            .field(
                "clear_screen (fn ptr)",
                &(self.0.clear_screen as *const u64),
            )
            .field(
                "set_cursor_position (fn ptr)",
                &(self.0.set_cursor_position as *const u64),
            )
            .field(
                "enable_cursor (fn ptr)",
                &(self.0.enable_cursor as *const u64),
            )
            .field("data", &self.0.mode)
            .finish()
    }
}

/// The text mode (resolution) of the output device.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OutputMode {
    index: usize,
    dims: (usize, usize),
}

impl OutputMode {
    /// Returns the index of this mode.
    #[inline]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the width in columns.
    #[inline]
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.dims.0
    }

    /// Returns the height in rows.
    #[inline]
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.dims.1
    }
}

/// An iterator of the text modes (possibly) supported by a device.
#[derive(Debug)]
pub struct OutputModeIter<'out> {
    output: &'out mut Output,
    current: usize,
    max: usize,
}

impl<'out> Iterator for OutputModeIter<'out> {
    type Item = OutputMode;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current;
        if index < self.max {
            self.current += 1;

            if let Ok(dims) = self.output.query_mode(index) {
                Some(OutputMode { index, dims })
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

/// Colors for the UEFI console.
///
/// All colors can be used as foreground colors.
/// The first 8 colors can also be used as background colors.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White,
}
