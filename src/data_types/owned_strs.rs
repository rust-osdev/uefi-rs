use super::chars::{Char16, NUL_16};
use super::strs::{CStr16, FromSliceWithNulError};
use crate::alloc_api::vec::Vec;
use crate::data_types::strs::EqStrUntilNul;
use core::fmt;
use core::ops;

/// Error returned by [`CString16::try_from::<&str>`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FromStrError {
    /// Character conversion error.
    InvalidChar,
    /// Nul character found in the input.
    InteriorNul,
}

/// An owned UCS-2 null-terminated string.
///
/// For convenience, a [CString16] is comparable with `&str` and `String` from the standard library
/// through the trait [EqStrUntilNul].
///
/// # Examples
///
/// Round-trip conversion from a [`&str`] to a `CString16` and back:
///
/// ```
/// use uefi::CString16;
///
/// let s = CString16::try_from("abc").unwrap();
/// assert_eq!(s.to_string(), "abc");
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct CString16(Vec<Char16>);

impl TryFrom<&str> for CString16 {
    type Error = FromStrError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // Initially allocate one Char16 for each byte of the input, plus
        // one for the null byte. This should be a good guess for ASCII-ish
        // input.
        let mut output = Vec::with_capacity(input.len() + 1);

        // Convert to UTF-16, then convert to UCS-2.
        for c in input.encode_utf16() {
            let c = Char16::try_from(c).map_err(|_| FromStrError::InvalidChar)?;

            // Check for interior nul chars.
            if c == NUL_16 {
                return Err(FromStrError::InteriorNul);
            }

            output.push(c);
        }

        // Add trailing nul.
        output.push(NUL_16);

        Ok(CString16(output))
    }
}

impl TryFrom<Vec<u16>> for CString16 {
    type Error = FromSliceWithNulError;

    fn try_from(input: Vec<u16>) -> Result<Self, Self::Error> {
        // Try creating a CStr16 from the input. We throw away the
        // result if successful, but it takes care of all the necessary
        // validity checks (valid UCS-2, ends in null, contains no
        // interior nulls).
        CStr16::from_u16_with_nul(&input)?;

        // Convert the input vector from `u16` to `Char16`.
        //
        // Safety: `Char16` is a transparent struct wrapping `u16`, so
        // the types are compatible. The pattern used here matches the
        // example in the docs for `into_raw_parts`.
        let (ptr, len, cap) = input.into_raw_parts();
        let rebuilt = unsafe {
            let ptr = ptr.cast::<Char16>();
            Vec::from_raw_parts(ptr, len, cap)
        };

        Ok(Self(rebuilt))
    }
}

impl ops::Deref for CString16 {
    type Target = CStr16;

    fn deref(&self) -> &CStr16 {
        unsafe { &*(self.0.as_slice() as *const [Char16] as *const CStr16) }
    }
}

impl AsRef<CStr16> for CString16 {
    fn as_ref(&self) -> &CStr16 {
        self
    }
}

impl fmt::Display for CString16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl PartialEq<&CStr16> for CString16 {
    fn eq(&self, other: &&CStr16) -> bool {
        PartialEq::eq(self.as_ref(), other)
    }
}

impl<StrType: AsRef<str>> EqStrUntilNul<StrType> for CString16 {
    fn eq_str_until_nul(&self, other: &StrType) -> bool {
        let this = self.as_ref();
        this.eq_str_until_nul(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc_api::string::String;
    use crate::alloc_api::vec;

    #[test]
    fn test_cstring16_from_str() {
        assert_eq!(
            CString16::try_from("x").unwrap(),
            CString16(vec![Char16::try_from('x').unwrap(), NUL_16])
        );

        assert_eq!(CString16::try_from("😀"), Err(FromStrError::InvalidChar));

        assert_eq!(CString16::try_from("x\0"), Err(FromStrError::InteriorNul));
    }

    #[test]
    fn test_cstring16_from_u16_vec() {
        // Test that invalid inputs are caught.
        assert_eq!(
            CString16::try_from(vec![]),
            Err(FromSliceWithNulError::NotNulTerminated)
        );
        assert_eq!(
            CString16::try_from(vec![b'a'.into(), 0, b'b'.into(), 0]),
            Err(FromSliceWithNulError::InteriorNul(1))
        );
        assert_eq!(
            CString16::try_from(vec![0xd800, 0]),
            Err(FromSliceWithNulError::InvalidChar(0))
        );

        // Test valid input.
        assert_eq!(
            CString16::try_from(vec![b'x'.into(), 0]).unwrap(),
            CString16::try_from("x").unwrap()
        );
    }

    /// Test `CString16 == &CStr16` and `&CStr16 == CString16`.
    #[test]
    fn test_cstring16_cstr16_eq() {
        assert_eq!(
            crate::prelude::cstr16!("abc"),
            CString16::try_from("abc").unwrap()
        );

        assert_eq!(
            CString16::try_from("abc").unwrap(),
            crate::prelude::cstr16!("abc")
        );
    }

    /// Tests the trait implementation of trait [EqStrUntilNul].
    #[test]
    fn test_cstring16_eq_std_str() {
        let input = CString16::try_from("test").unwrap();

        // test various comparisons with different order (left, right)
        assert!(input.eq_str_until_nul(&"test"));
        assert!(input.eq_str_until_nul(&String::from("test")));

        // now other direction
        assert!(String::from("test").eq_str_until_nul(&input));
        assert!("test".eq_str_until_nul(&input));
    }
}
