use super::Output;
use uefi::{ucs2, Status};
use core::{fmt, str};

/// Struct which is used to implement the `fmt::Write` trait on a UEFI output protocol.
pub struct OutputWriter {
    output: &'static mut Output,
}

impl OutputWriter {
    pub fn new(output: &'static mut Output) -> Self {
        OutputWriter { output }
    }
}

impl fmt::Write for OutputWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Allocate a small buffer on the stack.
        const BUF_SIZE: usize = 128;
        // Add 1 extra character for the null terminator.
        let mut buf = [0u16; BUF_SIZE + 1];

        let mut i = 0;

        // This closure writes the local buffer to the output and resets the buffer.
        let mut flush_buffer = |buf: &mut [u16], i: &mut usize| {
            buf[*i] = 0;
            *i = 0;

            self.output.output_string(buf.as_ptr()).map_err(
                |_| fmt::Error,
            )
        };

        {
            // This closure converts a character to UCS-2 and adds it to the buffer,
            // flushing it as necessary.
            let mut add_char = |ch| {
                // UEFI only supports UCS-2 characters, not UTF-16,
                // so there are no multibyte characters.
                buf[i] = ch;
                i += 1;

                if i == BUF_SIZE {
                    flush_buffer(&mut buf, &mut i).map_err(|_| Status::ProtocolError)
                } else {
                    Ok(())
                }
            };
            let add_ch = |ch| {
                add_char(ch)?;

                if ch == '\n' as u16 {
                    add_char('\r' as u16)
                }
                else {
                    Ok(())
                }
            };

            ucs2::ucs2_encoder(s, add_ch).map_err(|_| fmt::Error)?;
        }

        // Flush whatever is left in the buffer.
        flush_buffer(&mut buf, &mut i)
    }
}
