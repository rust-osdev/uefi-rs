use core::convert::TryFrom;

/// Character conversion error
pub struct CharConversionError;

/// A Latin-1 character
pub struct Char8(u8);
//
impl TryFrom<char> for Char8 {
    type Error = CharConversionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let code_point = value as u32;
        if code_point <= 0xff {
            Ok(Char8(code_point as u8))
        } else {
            Err(CharConversionError)
        }
    }
}
//
impl Into<u8> for Char8 {
    fn into(self) -> u8 {
        self.0 as u8
    }
}

/// An UCS-2 code point
pub struct Char16(u16);
//
impl TryFrom<char> for Char16 {
    type Error = CharConversionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let code_point = value as u32;
        if code_point <= 0xffff {
            Ok(Char16(code_point as u16))
        } else {
            Err(CharConversionError)
        }
    }
}
//
impl Into<u16> for Char16 {
    fn into(self) -> u16 {
        self.0 as u16
    }
}