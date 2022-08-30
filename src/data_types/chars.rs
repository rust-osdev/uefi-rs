//! UEFI character handling
//!
//! UEFI uses both Latin-1 and UCS-2 character encoding, this module implements
//! support for the associated character types.

use core::fmt;

/// Character conversion error
#[derive(Clone, Copy, Debug)]
pub struct CharConversionError;

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

impl From<Char16> for char {
    fn from(char: Char16) -> char {
        u32::from(char.0).try_into().unwrap()
    }
}

impl TryFrom<u16> for Char16 {
    type Error = CharConversionError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // We leverage char's TryFrom<u32> impl for Unicode validity checking
        let res: Result<char, _> = u32::from(value).try_into();
        if let Ok(ch) = res {
            ch.try_into()
        } else {
            Err(CharConversionError)
        }
    }
}

impl From<Char16> for u16 {
    fn from(char: Char16) -> u16 {
        char.0 as u16
    }
}

impl fmt::Debug for Char16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(c) = u32::from(self.0).try_into() {
            <char as fmt::Debug>::fmt(&c, f)
        } else {
            write!(f, "Char16({:?})", self.0)
        }
    }
}

impl fmt::Display for Char16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Ok(c) = u32::from(self.0).try_into() {
            <char as fmt::Display>::fmt(&c, f)
        } else {
            write!(f, "{}", core::char::REPLACEMENT_CHARACTER)
        }
    }
}

/// UCS-2 version of the NUL character
pub const NUL_16: Char16 = Char16(0);
