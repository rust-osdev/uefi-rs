// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Unicode Collation Protocol.
//!
//! This protocol is used in the boot services environment to perform
//! lexical comparison functions on Unicode strings for given languages.

use crate::data_types::{CStr8, CStr16};
use crate::proto::unsafe_protocol;
use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};
use core::{error, slice};
use uefi_raw::protocol::string::UnicodeCollationProtocol;

/// Unicode Collation [`Protocol`].
///
/// Used to perform case-insensitive comparisons of strings.
///
/// [`Protocol`]: uefi::proto::Protocol
#[derive(Debug)]
#[repr(transparent)]
#[unsafe_protocol(UnicodeCollationProtocol::GUID)]
pub struct UnicodeCollation(UnicodeCollationProtocol);

impl UnicodeCollation {
    /// Performs a case insensitive comparison of two
    /// null-terminated strings.
    #[must_use]
    pub fn stri_coll(&self, s1: &CStr16, s2: &CStr16) -> Ordering {
        // SAFETY: The memory is valid.
        let order = unsafe { (self.0.stri_coll)(&self.0, s1.as_ptr().cast(), s2.as_ptr().cast()) };
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
    /// |Pattern Character              |Meaning                                           |
    /// |-------------------------------|--------------------------------------------------|
    /// |*                              | Match 0 or more characters                       |
    /// |?                              | Match any one character                          |
    /// |``[`char1` `char2`...`charN`]``| Match any character in the set                   |
    /// |``[`char1`-`char2`]``          | Match any character between `char1` and `char2`|
    /// |`char`                         | Match the character `char`                      |
    ///
    /// For example, the pattern "*.Fw" will match all strings that end
    /// in ".FW", ".fw", ".Fw" or ".fW". The pattern "[a-z]" will match any
    /// letter in the alphabet. The pattern "z" will match the letter "z".
    /// The pattern "d?.*" will match the character "D" or "d" followed by
    /// any single character followed by a "." followed by any string.
    #[must_use]
    pub fn metai_match(&self, s: &CStr16, pattern: &CStr16) -> bool {
        // SAFETY: The memory is valid.
        unsafe { (self.0.metai_match)(&self.0, s.as_ptr().cast(), pattern.as_ptr().cast()) }.into()
    }

    /// Converts the characters in `s` to lower case characters.
    pub fn str_lwr<'a>(
        &self,
        s: &CStr16,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        let buf = copy_with_nul(s, buf)?;

        // SAFETY: The memory is valid.
        unsafe { (self.0.str_lwr)(&self.0, buf.as_mut_ptr()) };

        // SAFETY: `buf` holds exactly the NUL-terminated copy of `s`, which
        // the case conversion changes in place.
        Ok(unsafe { CStr16::from_u16_with_nul_unchecked(buf) })
    }

    /// Converts the characters in `s` to upper case characters.
    pub fn str_upr<'a>(
        &self,
        s: &CStr16,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        let buf = copy_with_nul(s, buf)?;

        // SAFETY: The memory is valid.
        unsafe { (self.0.str_upr)(&self.0, buf.as_mut_ptr()) };

        // SAFETY: `buf` holds exactly the NUL-terminated copy of `s`, which
        // the case conversion changes in place.
        Ok(unsafe { CStr16::from_u16_with_nul_unchecked(buf) })
    }

    /// Converts the 8.3 FAT file name `fat` to a null terminated string.
    pub fn fat_to_str<'a>(
        &self,
        fat: &CStr8,
        buf: &'a mut [u16],
    ) -> Result<&'a CStr16, StrConversionError> {
        // The conversion writes one character per FAT character plus the
        // NUL terminator, so the output is at most as long as `fat` with
        // its NUL.
        let fat_len = fat.num_bytes();
        let buf = buf
            .get_mut(..fat_len)
            .ok_or(StrConversionError::BufferTooSmall)?;
        // SAFETY: The memory is valid.
        unsafe { (self.0.fat_to_str)(&self.0, fat_len, fat.as_ptr().cast(), buf.as_mut_ptr()) };
        // The firmware wrote the output, so validate it instead of trusting
        // it blindly.
        CStr16::from_u16_until_nul(buf).map_err(|_| StrConversionError::ConversionFailed)
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
        // The protocol only writes the converted characters and no NUL
        // terminator; the remaining buffer keeps its previous content.
        // Pre-zero the buffer so the result is NUL-terminated.
        buf.fill(0);
        // SAFETY: The memory is valid.
        let failed = unsafe {
            (self.0.str_to_fat)(
                &self.0,
                s.as_ptr().cast(),
                s.as_slice_with_nul().len(),
                buf.as_mut_ptr(),
            )
        };
        if bool::from(failed) {
            Err(StrConversionError::ConversionFailed)
        } else {
            // The conversion writes at most one byte per input character
            // and never a NUL, so the first NUL terminates the result.
            let end = buf
                .iter()
                .position(|&b| b == 0)
                .expect("conversion should leave the pre-zeroed NUL of the input intact");
            // SAFETY: The pointer is valid for the requested slice length.
            let buf = unsafe { slice::from_raw_parts(buf.as_ptr(), end + 1) };
            // SAFETY: The slice ends with the first NUL, so there are no interior NULs.
            Ok(unsafe { CStr8::from_bytes_with_nul_unchecked(buf) })
        }
    }
}

/// Copies `s` including its NUL terminator to the start of `buf` and returns
/// the written part.
fn copy_with_nul<'a>(s: &CStr16, buf: &'a mut [u16]) -> Result<&'a mut [u16], StrConversionError> {
    let src = s.to_u16_slice_with_nul();
    let dst = buf
        .get_mut(..src.len())
        .ok_or(StrConversionError::BufferTooSmall)?;
    dst.copy_from_slice(src);
    Ok(dst)
}

/// Errors returned by [`UnicodeCollation::str_lwr`] and [`UnicodeCollation::str_upr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrConversionError {
    /// The conversion failed.
    ConversionFailed,
    /// The buffer given is too small to hold the string.
    BufferTooSmall,
}

impl Display for StrConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ConversionFailed => "conversion failed",
                Self::BufferTooSmall => "buffer too small",
            }
        )
    }
}

impl error::Error for StrConversionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cstr8, cstr16};
    use core::ptr;
    use uefi_raw::{Boolean, Char8, Char16};

    /// Mock of `UnicodeCollationProtocol::str_lwr` for ASCII strings.
    ///
    /// # Safety
    /// `s` must point to a NUL-terminated string.
    unsafe extern "efiapi" fn mock_str_lwr(_: *const UnicodeCollationProtocol, s: *mut Char16) {
        let mut p = s.cast::<u16>();
        // SAFETY: Guaranteed by the caller.
        unsafe {
            while *p != 0 {
                *p = u16::from((*p as u8).to_ascii_lowercase());
                p = p.add(1);
            }
        }
    }

    /// Mock of `UnicodeCollationProtocol::fat_to_str` with EDK2 semantics:
    /// copy until NUL or `fat_size` is exhausted, then write a NUL.
    ///
    /// # Safety
    /// `fat` must be valid for reading and `s` for writing `fat_size` items
    /// plus one.
    unsafe extern "efiapi" fn mock_fat_to_str(
        _: *const UnicodeCollationProtocol,
        fat_size: usize,
        fat: *const Char8,
        s: *mut Char16,
    ) {
        // SAFETY: Guaranteed by the caller.
        let fat = unsafe { core::slice::from_raw_parts(fat.cast::<u8>(), fat_size) };
        let len = fat.iter().position(|&b| b == 0).unwrap_or(fat_size);
        for (i, &b) in fat[..len].iter().enumerate() {
            // SAFETY: Guaranteed by the caller.
            unsafe { s.cast::<u16>().add(i).write(u16::from(b)) };
        }
        // SAFETY: Guaranteed by the caller.
        unsafe { s.cast::<u16>().add(len).write(0) };
    }

    // Stubs for the operations the tests do not exercise.

    extern "efiapi" fn stub_stri_coll(
        _: *const UnicodeCollationProtocol,
        _: *const Char16,
        _: *const Char16,
    ) -> isize {
        unimplemented!()
    }

    extern "efiapi" fn stub_metai_match(
        _: *const UnicodeCollationProtocol,
        _: *const Char16,
        _: *const Char16,
    ) -> Boolean {
        unimplemented!()
    }

    extern "efiapi" fn stub_str_upr(_: *const UnicodeCollationProtocol, _: *mut Char16) {
        unimplemented!()
    }

    extern "efiapi" fn stub_str_to_fat(
        _: *const UnicodeCollationProtocol,
        _: *const Char16,
        _: usize,
        _: *mut Char8,
    ) -> Boolean {
        unimplemented!()
    }

    const MOCK: UnicodeCollationProtocol = UnicodeCollationProtocol {
        stri_coll: stub_stri_coll,
        metai_match: stub_metai_match,
        str_lwr: mock_str_lwr,
        str_upr: stub_str_upr,
        fat_to_str: mock_fat_to_str,
        str_to_fat: stub_str_to_fat,
        supported_languages: ptr::null(),
    };

    fn mock() -> &'static UnicodeCollation {
        // SAFETY: `UnicodeCollation` is a transparent wrapper.
        unsafe { &*ptr::from_ref(&MOCK).cast::<UnicodeCollation>() }
    }

    #[test]
    fn test_str_lwr_oversized_buffer() {
        let mut buf = [0x41; 8];
        let s = mock().str_lwr(cstr16!("AbC"), &mut buf).unwrap();
        assert_eq!(s.num_chars(), 3);
        assert_eq!(s, cstr16!("abc"));

        let mut buf = [0x41; 2];
        let s = mock().str_lwr(cstr16!(""), &mut buf).unwrap();
        assert_eq!(s.num_chars(), 0);

        let mut buf = [0x41; 3];
        assert_eq!(
            mock().str_lwr(cstr16!("AbC"), &mut buf).unwrap_err(),
            StrConversionError::BufferTooSmall
        );
    }

    #[test]
    fn test_fat_to_str_oversized_buffer() {
        let mut buf = [0x41; 16];
        let s = mock().fat_to_str(cstr8!("AB"), &mut buf).unwrap();
        assert_eq!(s.num_chars(), 2);
        assert_eq!(s, cstr16!("AB"));

        let mut buf = [0x41; 2];
        assert_eq!(
            mock().fat_to_str(cstr8!("AB"), &mut buf).unwrap_err(),
            StrConversionError::BufferTooSmall
        );
    }
}
