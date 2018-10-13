use core::convert::TryFrom;
use core::fmt;

/// Character conversion error
pub struct CharConversionError;

/// A Latin-1 character
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Char8(u8);

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

impl Into<u8> for Char8 {
    fn into(self) -> u8 {
        self.0 as u8
    }
}

impl fmt::Debug for Char8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <char as fmt::Debug>::fmt(&From::from(self.0), f)
    }
}

impl fmt::Display for Char8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <char as fmt::Display>::fmt(&From::from(self.0), f)
    }
}

/// An UCS-2 code point
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Char16(u16);

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

impl Into<u16> for Char16 {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl fmt::Debug for Char16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(c) = TryFrom::try_from(self.0 as u32) {
            <char as fmt::Debug>::fmt(&c, f)
        } else {
            write!(f, "Char16({:?})", self.0)
        }
    }
}

impl fmt::Display for Char16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(c) = TryFrom::try_from(self.0 as u32) {
            <char as fmt::Display>::fmt(&c, f)
        } else {
            write!(f, "{}", core::char::REPLACEMENT_CHARACTER)
        }
    }
}