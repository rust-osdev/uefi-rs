//! Utility functions for the UCS-2 encoding.
//!
//! UEFI primarily uses the UCS-2 encoding, a precursor of UTF-16.
//! Every character is encoded using 2 bytes, but this is a fixed-length,
//! not a multibyte encoding such as UTF-8 / UTF-16.
//! This means UCS-2 does *not* cover the whole Unicode range.
//!
//! UEFI implementations are allowed to not support all of the possible UCS-2
//! characters for printing.

use crate::{Status, Result};

/// Encode UTF-8 string to UCS-2
pub fn ucs2_encoder<F>(input: &str, mut output: F) -> Result<()>
        where F: FnMut(u16) -> Result<()> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        let ch;

        if bytes[i] & 0b1000_0000 == 0b0000_0000 {
            ch = u16::from(bytes[i]);
            i += 1;
        }
        else if bytes[i] & 0b1110_0000 == 0b1100_0000 {
            // 2 byte codepoint
            if i + 1 == len {
                // Buffer underflow
                return Err(Status::BadBufferSize);
            }
            if bytes[i+1] & 0b1100_0000 != 0b1000_0000 {
                // Invalid data
                return Err(Status::CompromisedData);
            }
            let a = u16::from(bytes[i] & 0b0001_1111);
            let b = u16::from(bytes[i+1] & 0b0011_1111);
            ch = a << 6 | b;
            i += 2;
        }
        else if bytes[i] & 0b1111_0000 == 0b1110_0000 {
            // 3 byte codepoint
            if i + 2 >= len {
                // Buffer underflow
                return Err(Status::BadBufferSize);
            }
            if bytes[i+1] & 0b1100_0000 != 0b1000_0000 ||
                bytes[i+2] & 0b1100_0000 != 0b1000_0000 {
                // Invalid data
                return Err(Status::CompromisedData);
            }
            let a = u16::from(bytes[i] & 0b0000_1111);
            let b = u16::from(bytes[i+1] & 0b0011_1111);
            let c = u16::from(bytes[i+2] & 0b0011_1111);
            ch = a << 12 | b << 6 | c;
            i += 3;
        }
        else if bytes[i] & 0b1111_0000 == 0b1111_0000 {
            return Err(Status::Unsupported); // UTF-16
        }
        else {
            return Err(Status::CompromisedData);
        }
        output(ch)?;
    }
    Ok(())
}

/// Encodes an input UTF-8 string into a UCS-2 string.
///
/// The returned `usize` represents the length of the returned buffer,
/// measured in 2-byte characters.
pub fn encode_ucs2(input: &str, buffer: &mut [u16]) -> Result<usize> {
    let buffer_size = buffer.len();
    let mut i = 0;

    {
        let add_ch = |ch| {
            if i >= buffer_size {
                Err(Status::BufferTooSmall)
            }
            else {
                buffer[i] = ch;
                i += 1;
                if ch == '\n' as u16 {
                    if i == buffer_size {
                        Err(Status::BufferTooSmall)
                    }
                    else {
                        buffer[i] = '\r' as u16;
                        i += 1;
                        Ok(())
                    }
                }
                else {
                    Ok(())
                }
            }
        };
        ucs2_encoder(input, add_ch)?;
    }

    Ok(i)
}
