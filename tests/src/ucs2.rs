use uefi::{Status, Result};
use uefi::ucs2::encode_ucs2;
use alloc::Vec;

pub fn ucs2_encoding_test() -> Result<()> {
    let utf8_string = "őэ╋";
    let mut ucs2_buffer : Vec<u16> = Vec::with_capacity(5);
    unsafe {
        ucs2_buffer.set_len(5);
    }
    match encode_ucs2(utf8_string, &mut ucs2_buffer) {
        Ok(3) => {
            unsafe { ucs2_buffer.set_len(3); }
            match &ucs2_buffer[..] {
                &[0x0151, 0x044D, 0x254B] => {
                    Ok(())
                }
                _ => {
                    Err(Status::CrcError)
                }
            }
        }
        Ok(_) => { Err(Status::CrcError) },
        Err(err) => { Err(err) },
    }
}
