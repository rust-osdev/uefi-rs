use {Status, Result};

/// Encode UTF-8 string to UCS-2
pub fn ucs2_encoder<F>(input: &str, mut output: F) -> Result<()>
        where F: FnMut(u16) -> Result<()> {   
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        let mut ch = 0u16;

        if bytes[i] & 0b1000_0000 == 0b0000_0000 {
            ch = bytes[i] as u16;
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
            let a = (bytes[i] & 0b0001_1111) as u16;
            let b = (bytes[i+1] & 0b0011_1111) as u16;
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
            let a = (bytes[i] & 0b0000_1111) as u16;
            let b = (bytes[i+1] & 0b0011_1111) as u16;
            let c = (bytes[i+2] & 0b0011_1111) as u16;
            ch = a << 12 | b << 6 | c;
            i += 3;
        }
        else if bytes[i] & 0b1111_0000 == 0b1111_0000 {
            return Err(Status::Unsupported); // UTF-16
        }
        else {
            return Err(Status::CompromisedData);
        }
        match output(ch) {
            Ok(()) => {},
            Err(err) => { return Err(err); },
        }
    }
    Ok(())
}

pub fn encode_ucs2(input: &str, buffer: &mut [u16]) -> Result<usize> {
    let mut result = Ok(());
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
    result = ucs2_encoder(input, add_ch);
    }
    match result {
        Ok(()) => { Ok(i) },
        Err(err) => { Err(err) },
    }
}
