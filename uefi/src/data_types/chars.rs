//! UEFI character handling
//!
//! UEFI uses both Latin-1 and UCS-2 character encoding, this module implements
//! support for the associated character types.

use core::fmt::{self, Display, Formatter};

use alloc::vec::Vec;

/// Character conversion error
#[derive(Clone, Copy, Debug)]
pub struct CharConversionError;

impl Display for CharConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for CharConversionError {}

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

impl From<Char8> for char {
    fn from(char: Char8) -> char {
        char.0 as char
    }
}

impl From<u8> for Char8 {
    fn from(value: u8) -> Self {
        Char8(value)
    }
}

impl From<Char8> for u8 {
    fn from(char: Char8) -> u8 {
        char.0
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

/// Latin-1 version of the NUL character
pub const NUL_8: Char8 = Char8(0);

/// An UCS-2 code point
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Char16(u16);

impl Char16 {
    /// Creates a UCS-2 character from a Rust character without checks.
    ///
    /// # Safety
    /// The caller must be sure that the character is valid.
    #[must_use]
    pub const unsafe fn from_u16_unchecked(val: u16) -> Self {
        Self(val)
    }

    /// Checks if the value is within the ASCII range.
    #[must_use]
    pub const fn is_ascii(&self) -> bool {
        self.0 <= 127
    }
}

/// Provides various functions on slice-like container (e.g. Vec) of Char16.
pub trait SliceLikeChar16 {
    /// Checks if all char16 in this slice are within the ASCII range.
    fn is_ascii(&self) -> bool;
}

impl SliceLikeChar16 for [Char16] {
    fn is_ascii(&self) -> bool {
        self.iter().all(|c| c.is_ascii())
    }
}

impl SliceLikeChar16 for Vec<Char16> {
    fn is_ascii(&self) -> bool {
        self.iter().all(|c| c.is_ascii())
    }
}

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
        char.0
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
pub const NUL_16: Char16 = unsafe { Char16::from_u16_unchecked(0) };
