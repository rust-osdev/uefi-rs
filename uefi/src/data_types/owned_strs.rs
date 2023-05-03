use super::chars::{Char16, NUL_16};
use super::strs::{CStr16, FromSliceWithNulError};
use crate::data_types::strs::EqStrUntilNul;
use crate::data_types::UnalignedSlice;
use crate::polyfill::vec_into_raw_parts;
use alloc::borrow::{Borrow, ToOwned};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::{fmt, ops};

/// Error returned by [`CString16::try_from::<&str>`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FromStrError {
    /// Character conversion error.
    InvalidChar,
    /// Nul character found in the input.
    InteriorNul,
}

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UCS-2 Conversion Error: {}",
            match self {
                Self::InvalidChar => "Invalid character",
                Self::InteriorNul => "Interior null terminator",
            }
        )
    }
}

#[cfg(feature = "unstable")]
impl core::error::Error for FromStrError {}

/// An owned UCS-2 null-terminated string.
///
/// For convenience, a [`CString16`] is comparable with `&str` and `String` from
/// the standard library through the trait [`EqStrUntilNul`].
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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CString16(Vec<Char16>);

impl CString16 {
    /// Creates a new empty string with a terminating null character.
    #[must_use]
    pub fn new() -> Self {
        Self(vec![NUL_16])
    }

    /// Inserts a character at the end of the string, right before the null
    /// character.
    ///
    /// # Panics
    /// Panics if the char is a null character.
    pub fn push(&mut self, char: Char16) {
        assert_ne!(char, NUL_16, "Pushing a null-character is illegal");
        let last_elem = self
            .0
            .last_mut()
            .expect("There should be at least a null character");
        *last_elem = char;
        self.0.push(NUL_16);
    }

    /// Extends the string with the given [`CStr16`]. The null character is
    /// automatically kept at the end.
    pub fn push_str(&mut self, str: &CStr16) {
        str.as_slice()
            .iter()
            .copied()
            .for_each(|char| self.push(char));
    }

    /// Replaces all chars in the string with the replace value in-place.
    pub fn replace_char(&mut self, search: Char16, replace: Char16) {
        assert_ne!(search, NUL_16, "Replacing a null character is illegal");
        assert_ne!(
            replace, NUL_16,
            "Replacing with a null character is illegal"
        );
        self.0
            .as_mut_slice()
            .iter_mut()
            .filter(|char| **char == search)
            .for_each(|char| *char = replace);
    }

    /// Returns the number of characters without the trailing null character.
    #[must_use]
    pub fn num_chars(&self) -> usize {
        self.0.len() - 1
    }

    /// Returns if the string is empty. This ignores the null character.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.num_chars() == 0
    }
}

impl Default for CString16 {
    fn default() -> Self {
        CString16::new()
    }
}

impl TryFrom<&str> for CString16 {
    type Error = FromStrError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // Initially allocate one Char16 for each byte of the input, plus
        // one for the null character. This should be a good guess for ASCII-ish
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
        let (ptr, len, cap) = vec_into_raw_parts(input);
        let rebuilt = unsafe {
            let ptr = ptr.cast::<Char16>();
            Vec::from_raw_parts(ptr, len, cap)
        };

        Ok(Self(rebuilt))
    }
}

impl<'a> TryFrom<&UnalignedSlice<'a, u16>> for CString16 {
    type Error = FromSliceWithNulError;

    fn try_from(input: &UnalignedSlice<u16>) -> Result<Self, Self::Error> {
        let v = input.to_vec();
        CString16::try_from(v)
    }
}

impl From<&CStr16> for CString16 {
    fn from(value: &CStr16) -> Self {
        let vec = value.as_slice_with_nul().to_vec();
        Self(vec)
    }
}

impl From<&CString16> for String {
    fn from(value: &CString16) -> Self {
        let slice: &CStr16 = value.as_ref();
        String::from(slice)
    }
}

impl<'a> UnalignedSlice<'a, u16> {
    /// Copies `self` to a new [`CString16`].
    pub fn to_cstring16(&self) -> Result<CString16, FromSliceWithNulError> {
        CString16::try_from(self)
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

impl Borrow<CStr16> for CString16 {
    fn borrow(&self) -> &CStr16 {
        self
    }
}

impl ToOwned for CStr16 {
    type Owned = CString16;

    fn to_owned(&self) -> CString16 {
        CString16(self.as_slice_with_nul().to_vec())
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

impl<StrType: AsRef<str> + ?Sized> EqStrUntilNul<StrType> for CString16 {
    fn eq_str_until_nul(&self, other: &StrType) -> bool {
        let this = self.as_ref();
        this.eq_str_until_nul(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cstr16;
    use alloc::string::String;
    use alloc::vec;

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

    /// Tests the trait implementation of trait [`EqStrUntilNul]` for [`CString16`].
    ///
    /// This tests that `String` and `str` from the standard library can be
    /// checked for equality against a [`CString16`]. It checks both directions,
    /// i.e., the equality is reflexive.
    #[test]
    fn test_cstring16_eq_std_str() {
        let input = CString16::try_from("test").unwrap();

        assert!(input.eq_str_until_nul("test")); // requires ?Sized constraint
        assert!(input.eq_str_until_nul(&"test"));
        assert!(input.eq_str_until_nul(&String::from("test")));

        // now other direction
        assert!(String::from("test").eq_str_until_nul(&input));
        assert!("test".eq_str_until_nul(&input));
    }

    /// Test the `Borrow` and `ToOwned` impls.
    #[test]
    fn test_borrow_and_to_owned() {
        let s1: &CStr16 = cstr16!("ab");
        let owned: CString16 = s1.to_owned();
        let s2: &CStr16 = owned.borrow();
        assert_eq!(s1, s2);
        assert_eq!(
            owned.0,
            [
                Char16::try_from('a').unwrap(),
                Char16::try_from('b').unwrap(),
                NUL_16
            ]
        );
    }

    /// This tests the following UCS-2 string functions:
    /// - runtime constructor
    /// - len()
    /// - push() / push_str()
    /// - to rust string
    #[test]
    fn test_push_str() {
        let mut str1 = CString16::new();
        assert_eq!(str1.num_bytes(), 2, "Should have null character");
        assert_eq!(str1.num_chars(), 0);
        str1.push(Char16::try_from('h').unwrap());
        str1.push(Char16::try_from('i').unwrap());
        assert_eq!(str1.num_chars(), 2);

        let mut str2 = CString16::new();
        str2.push(Char16::try_from('!').unwrap());

        str2.push_str(str1.as_ref());
        assert_eq!(str2.num_chars(), 3);

        let rust_str = String::from(&str2);
        assert_eq!(rust_str, "!hi");
    }

    #[test]
    #[should_panic]
    fn test_push_str_panic() {
        CString16::new().push(NUL_16);
    }

    #[test]
    fn test_char_replace_all_in_place() {
        let mut input = CString16::try_from("foo/bar/foobar//").unwrap();
        let search = Char16::try_from('/').unwrap();
        let replace = Char16::try_from('\\').unwrap();
        input.replace_char(search, replace);

        let input = String::from(&input);
        assert_eq!(input, "foo\\bar\\foobar\\\\")
    }
}
