//! The Unicode Collation Protocol.
//!
//! This protocol is used in the boot services environment to perform
//! lexical comparison functions on Unicode strings for given languages.

use crate::data_types::{CStr16, CStr8, Char16, Char8};
use crate::proto::unsafe_protocol;
use core::cmp::Ordering;

/// The Unicode Collation Protocol.
///
/// Used to perform case-insensitive comparisons of strings.
#[repr(C)]
#[unsafe_protocol("a4c751fc-23ae-4c3e-92e9-4964cf63f349")]
pub struct UnicodeCollation {
    stri_coll: extern "efiapi" fn(this: &Self, s1: *const Char16, s2: *const Char16) -> isize,
    metai_match:
        extern "efiapi" fn(this: &Self, string: *const Char16, pattern: *const Char16) -> bool,
    str_lwr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    str_upr: extern "efiapi" fn(this: &Self, s: *mut Char16),
    fat_to_str: extern "efiapi" fn(this: &Self, fat_size: usize, fat: *const Char8, s: *mut Char16),
    str_to_fat:
        extern "efiapi" fn(this: &Self, s: *const Char16, fat_size: usize, fat: *mut Char8) -> bool,
}

impl UnicodeCollation {
    /// Performs a case insensitive comparison of two
    /// null-terminated strings.
    #[must_use]
    pub fn stri_coll(&self, s1: &CStr16, s2: &CStr16) -> Ordering {
        let order = (self.stri_coll)(self, s1.as_ptr(), s2.as_ptr());
        order.cmp(&0)
    }

    /// Performs a case insensitive comparison between a null terminated
    /// pattern string and a null terminated string.
    ///
    /// This function checks if character pattern described in `pattern`
    /// is found in `string`. If the pattern match succeeds, true is returned.
    /// Otherwise, false is returned.
    ///
    /// The following syntax can be used to build the string `pattern`:
    ///
    /// |Pattern Character            |Meaning                                           |
    /// |-----------------------------|--------------------------------------------------|
    /// |*                            | Match 0 or more characters                       |
    /// |?                            | Match any one character                          |
    /// |[`char1` `char2`...`charN`]| Match any character in the set                   |
    /// |[`char1`-`char2`]          | Match any character between `char1` and `char2`|
    /// |`char`                      | Match the character `char`                      |
    ///
    /// For example, the pattern "*.Fw" will match all strings that end
    /// in ".FW", ".fw", ".Fw" or ".fW". The pattern "[a-z]" will match any
    /// letter in the alphabet. The pattern "z" will match the letter "z".
    /// The pattern "d?.*" will match the character "D" or "d" followed by
    /// any single character followed by a "." followed by any string.
    #[must_use]
    pub fn metai_match(&self, s: &CStr16, pattern: &CStr16) -> bool {
        (self.metai_match)(self, s.as_ptr(), pattern.as_ptr())
    }

    /// Converts the characters in `s` to lower case characters.
    pub fn str_lwr<'a>(
        &self,
        s: &CStr16,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        let mut last_index = 0;
        for (i, c) in s.iter().enumerate() {
            *buf.get_mut(i).ok_or(StrConversionError::BufferTooSmall)? = (*c).into();
            last_index = i;
        }
        *buf.get_mut(last_index + 1)
            .ok_or(StrConversionError::BufferTooSmall)? = 0;

        (self.str_lwr)(self, buf.as_ptr() as *mut _);

        Ok(unsafe { CStr16::from_u16_with_nul_unchecked(buf) })
    }

    /// Converts the characters in `s` to upper case characters.
    pub fn str_upr<'a>(
        &self,
        s: &CStr16,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        let mut last_index = 0;
        for (i, c) in s.iter().enumerate() {
            *buf.get_mut(i).ok_or(StrConversionError::BufferTooSmall)? = (*c).into();
            last_index = i;
        }
        *buf.get_mut(last_index + 1)
            .ok_or(StrConversionError::BufferTooSmall)? = 0;

        (self.str_upr)(self, buf.as_ptr() as *mut _);

        Ok(unsafe { CStr16::from_u16_with_nul_unchecked(buf) })
    }

    /// Converts the 8.3 FAT file name `fat` to a null terminated string.
    pub fn fat_to_str<'a>(
        &self,
        fat: &CStr8,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        if buf.len() < fat.to_bytes_with_nul().len() {
            return Err(StrConversionError::BufferTooSmall);
        }
        (self.fat_to_str)(
            self,
            fat.to_bytes_with_nul().len(),
            fat.as_ptr(),
            buf.as_ptr() as *mut _,
        );
        Ok(unsafe { CStr16::from_u16_with_nul_unchecked(buf) })
    }

    /// Converts the null terminated string `s` to legal characters in a FAT file name.
    pub fn str_to_fat<'a>(
        &self,
        s: &CStr16,
        buf: &'a mut [u8],
    ) -> Result<&'a CStr8, StrConversionError> {
        if s.as_slice_with_nul().len() > buf.len() {
            return Err(StrConversionError::BufferTooSmall);
        }
        let failed = (self.str_to_fat)(
            self,
            s.as_ptr(),
            s.as_slice_with_nul().len(),
            buf.as_ptr() as *mut _,
        );
        if failed {
            Err(StrConversionError::ConversionFailed)
        } else {
            // After the conversion, there is a possibility that the converted string
            // is smaller than the original `s` string.
            // When the converted string is smaller, there will be a bunch of trailing
            // nulls.
            // To remove all those trailing nulls:
            let mut last_null_index = buf.len() - 1;
            for i in (0..buf.len()).rev() {
                if buf[i] != 0 {
                    last_null_index = i + 1;
                    break;
                }
            }
            let buf = unsafe { core::slice::from_raw_parts(buf.as_ptr(), last_null_index + 1) };
            Ok(unsafe { CStr8::from_bytes_with_nul_unchecked(buf) })
        }
    }
}

/// Errors returned by [`UnicodeCollation::str_lwr`] and [`UnicodeCollation::str_upr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrConversionError {
    /// The conversion failed.
    ConversionFailed,
    /// The buffer given is too small to hold the string.
    BufferTooSmall,
}
