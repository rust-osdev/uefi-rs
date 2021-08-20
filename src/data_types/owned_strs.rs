use super::chars::{Char16, NUL_16};
use super::strs::CStr16;
use crate::alloc_api::vec::Vec;
use core::convert::TryFrom;
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
/// # Examples
///
/// Round-trip conversion from a [`&str`] to a `CString16` and back:
///
/// ```
/// use core::convert::TryFrom;
/// use uefi::CString16;
///
/// let s = CString16::try_from("abc").unwrap();
/// assert_eq!(s.as_string(), "abc");
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
