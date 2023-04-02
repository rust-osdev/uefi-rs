use super::chars::{Char16, Char8, NUL_16, NUL_8};
use super::UnalignedSlice;
use crate::polyfill::maybe_uninit_slice_assume_init_ref;
use core::ffi::CStr;
use core::iter::Iterator;
use core::mem::MaybeUninit;
use core::result::Result;
use core::{fmt, slice};

#[cfg(feature = "alloc")]
use super::CString16;

/// Errors which can occur during checked `[uN]` -> `CStrN` conversions
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FromSliceWithNulError {
    /// An invalid character was encountered before the end of the slice
    InvalidChar(usize),

    /// A null character was encountered before the end of the slice
    InteriorNul(usize),

    /// The slice was not null-terminated
    NotNulTerminated,
}

/// Error returned by [`CStr16::from_unaligned_slice`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnalignedCStr16Error {
    /// An invalid character was encountered.
    InvalidChar(usize),

    /// A null character was encountered before the end of the data.
    InteriorNul(usize),

    /// The data was not null-terminated.
    NotNulTerminated,

    /// The buffer is not big enough to hold the entire string and
    /// trailing null character.
    BufferTooSmall,
}

/// Error returned by [`CStr16::from_str_with_buf`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FromStrWithBufError {
    /// An invalid character was encountered before the end of the string
    InvalidChar(usize),

    /// A null character was encountered in the string
    InteriorNul(usize),

    /// The buffer is not big enough to hold the entire string and
    /// trailing null character
    BufferTooSmall,
}

/// A null-terminated Latin-1 string.
///
/// This type is largely inspired by [`core::ffi::CStr`] with the exception that all characters are
/// guaranteed to be 8 bit long.
///
/// A [`CStr8`] can be constructed from a [`core::ffi::CStr`] via a `try_from` call:
/// ```ignore
/// let cstr8: &CStr8 = TryFrom::try_from(cstr).unwrap();
/// ```
///
/// For convenience, a [`CStr8`] is comparable with [`core::str`] and
/// `alloc::string::String` from the standard library through the trait [`EqStrUntilNul`].
#[repr(transparent)]
#[derive(Eq, PartialEq)]
pub struct CStr8([Char8]);

impl CStr8 {
    /// Takes a raw pointer to a null-terminated Latin-1 string and wraps it in a CStr8 reference.
    ///
    /// # Safety
    ///
    /// The function will start accessing memory from `ptr` until the first
    /// null byte. It's the callers responsibility to ensure `ptr` points to
    /// a valid null-terminated string in accessible memory.
    #[must_use]
    pub unsafe fn from_ptr<'ptr>(ptr: *const Char8) -> &'ptr Self {
        let mut len = 0;
        while *ptr.add(len) != NUL_8 {
            len += 1
        }
        let ptr = ptr.cast::<u8>();
        Self::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr, len + 1))
    }

    /// Creates a CStr8 reference from bytes.
    pub fn from_bytes_with_nul(chars: &[u8]) -> Result<&Self, FromSliceWithNulError> {
        let nul_pos = chars.iter().position(|&c| c == 0);
        if let Some(nul_pos) = nul_pos {
            if nul_pos + 1 != chars.len() {
                return Err(FromSliceWithNulError::InteriorNul(nul_pos));
            }
            Ok(unsafe { Self::from_bytes_with_nul_unchecked(chars) })
        } else {
            Err(FromSliceWithNulError::NotNulTerminated)
        }
    }

    /// Unsafely creates a CStr8 reference from bytes.
    ///
    /// # Safety
    ///
    /// It's the callers responsibility to ensure chars is a valid Latin-1
    /// null-terminated string, with no interior null bytes.
    #[must_use]
    pub const unsafe fn from_bytes_with_nul_unchecked(chars: &[u8]) -> &Self {
        &*(chars as *const [u8] as *const Self)
    }

    /// Returns the inner pointer to this CStr8.
    #[must_use]
    pub const fn as_ptr(&self) -> *const Char8 {
        self.0.as_ptr()
    }

    /// Converts this CStr8 to a slice of bytes without the terminating null byte.
    #[must_use]
    pub fn to_bytes(&self) -> &[u8] {
        let chars = self.to_bytes_with_nul();
        &chars[..chars.len() - 1]
    }

    /// Converts this CStr8 to a slice of bytes containing the trailing null byte.
    #[must_use]
    pub const fn to_bytes_with_nul(&self) -> &[u8] {
        unsafe { &*(&self.0 as *const [Char8] as *const [u8]) }
    }
}

impl fmt::Debug for CStr8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CStr8({:?})", &self.0)
    }
}

impl fmt::Display for CStr8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.0.iter() {
            <Char8 as fmt::Display>::fmt(c, f)?;
        }
        Ok(())
    }
}

impl<StrType: AsRef<str> + ?Sized> EqStrUntilNul<StrType> for CStr8 {
    fn eq_str_until_nul(&self, other: &StrType) -> bool {
        let other = other.as_ref();

        // TODO: CStr16 has .iter() implemented, CStr8 not yet
        let any_not_equal = self
            .0
            .iter()
            .copied()
            .map(char::from)
            .zip(other.chars())
            // This only works as CStr8 is guaranteed to have a fixed character length
            // (unlike UTF-8).
            .take_while(|(l, r)| *l != '\0' && *r != '\0')
            .any(|(l, r)| l != r);

        !any_not_equal
    }
}

impl<'a> TryFrom<&'a CStr> for &'a CStr8 {
    type Error = FromSliceWithNulError;

    fn try_from(cstr: &'a CStr) -> Result<Self, Self::Error> {
        CStr8::from_bytes_with_nul(cstr.to_bytes_with_nul())
    }
}

/// An UCS-2 null-terminated string.
///
/// This type is largely inspired by [`core::ffi::CStr`] with the exception that all characters are
/// guaranteed to be 16 bit long.
///
/// For convenience, a [`CStr16`] is comparable with [`core::str`] and
/// `alloc::string::String` from the standard library through the trait [`EqStrUntilNul`].
#[derive(Eq, PartialEq)]
#[repr(transparent)]
pub struct CStr16([Char16]);

impl CStr16 {
    /// Wraps a raw UEFI string with a safe C string wrapper
    ///
    /// # Safety
    ///
    /// The function will start accessing memory from `ptr` until the first
    /// null character. It's the callers responsibility to ensure `ptr` points to
    /// a valid string, in accessible memory.
    #[must_use]
    pub unsafe fn from_ptr<'ptr>(ptr: *const Char16) -> &'ptr Self {
        let mut len = 0;
        while *ptr.add(len) != NUL_16 {
            len += 1
        }
        let ptr = ptr.cast::<u16>();
        Self::from_u16_with_nul_unchecked(slice::from_raw_parts(ptr, len + 1))
    }

    /// Creates a C string wrapper from a u16 slice
    ///
    /// Since not every u16 value is a valid UCS-2 code point, this function
    /// must do a bit more validity checking than CStr::from_bytes_with_nul
    pub fn from_u16_with_nul(codes: &[u16]) -> Result<&Self, FromSliceWithNulError> {
        for (pos, &code) in codes.iter().enumerate() {
            match code.try_into() {
                Ok(NUL_16) => {
                    if pos != codes.len() - 1 {
                        return Err(FromSliceWithNulError::InteriorNul(pos));
                    } else {
                        return Ok(unsafe { Self::from_u16_with_nul_unchecked(codes) });
                    }
                }
                Err(_) => {
                    return Err(FromSliceWithNulError::InvalidChar(pos));
                }
                _ => {}
            }
        }
        Err(FromSliceWithNulError::NotNulTerminated)
    }

    /// Unsafely creates a C string wrapper from a u16 slice.
    ///
    /// # Safety
    ///
    /// It's the callers responsibility to ensure chars is a valid UCS-2
    /// null-terminated string, with no interior null characters.
    #[must_use]
    pub const unsafe fn from_u16_with_nul_unchecked(codes: &[u16]) -> &Self {
        &*(codes as *const [u16] as *const Self)
    }

    /// Convert a [`&str`] to a `&CStr16`, backed by a buffer.
    ///
    /// The input string must contain only characters representable with
    /// UCS-2, and must not contain any null characters (even at the end of
    /// the input).
    ///
    /// The backing buffer must be big enough to hold the converted string as
    /// well as a trailing null character.
    ///
    /// # Examples
    ///
    /// Convert the UTF-8 string "ABC" to a `&CStr16`:
    ///
    /// ```
    /// use uefi::CStr16;
    ///
    /// let mut buf = [0; 4];
    /// CStr16::from_str_with_buf("ABC", &mut buf).unwrap();
    /// ```
    pub fn from_str_with_buf<'a>(
        input: &str,
        buf: &'a mut [u16],
    ) -> Result<&'a Self, FromStrWithBufError> {
        let mut index = 0;

        // Convert to UTF-16.
        for c in input.encode_utf16() {
            *buf.get_mut(index)
                .ok_or(FromStrWithBufError::BufferTooSmall)? = c;
            index += 1;
        }

        // Add trailing null character.
        *buf.get_mut(index)
            .ok_or(FromStrWithBufError::BufferTooSmall)? = 0;

        // Convert from u16 to Char16. This checks for invalid UCS-2 chars and
        // interior nulls. The NotNulTerminated case is unreachable because we
        // just added a trailing null character.
        Self::from_u16_with_nul(&buf[..index + 1]).map_err(|err| match err {
            FromSliceWithNulError::InvalidChar(p) => FromStrWithBufError::InvalidChar(p),
            FromSliceWithNulError::InteriorNul(p) => FromStrWithBufError::InteriorNul(p),
            FromSliceWithNulError::NotNulTerminated => unreachable!(),
        })
    }

    /// Create a [`CStr16`] from an [`UnalignedSlice`] using an aligned
    /// buffer for storage. The lifetime of the output is tied to `buf`,
    /// not `src`.
    pub fn from_unaligned_slice<'buf>(
        src: &UnalignedSlice<'_, u16>,
        buf: &'buf mut [MaybeUninit<u16>],
    ) -> Result<&'buf CStr16, UnalignedCStr16Error> {
        // The input `buf` might be longer than needed, so get a
        // subslice of the required length.
        let buf = buf
            .get_mut(..src.len())
            .ok_or(UnalignedCStr16Error::BufferTooSmall)?;

        src.copy_to_maybe_uninit(buf);
        let buf = unsafe {
            // Safety: `copy_buf` fully initializes the slice.
            maybe_uninit_slice_assume_init_ref(buf)
        };
        CStr16::from_u16_with_nul(buf).map_err(|e| match e {
            FromSliceWithNulError::InvalidChar(v) => UnalignedCStr16Error::InvalidChar(v),
            FromSliceWithNulError::InteriorNul(v) => UnalignedCStr16Error::InteriorNul(v),
            FromSliceWithNulError::NotNulTerminated => UnalignedCStr16Error::NotNulTerminated,
        })
    }

    /// Returns the inner pointer to this C16 string.
    #[must_use]
    pub const fn as_ptr(&self) -> *const Char16 {
        self.0.as_ptr()
    }

    /// Get the underlying [`Char16`]s as slice without the trailing null.
    #[must_use]
    pub fn as_slice(&self) -> &[Char16] {
        &self.0[..self.num_chars()]
    }

    /// Get the underlying [`Char16`]s as slice including the trailing null.
    #[must_use]
    pub const fn as_slice_with_nul(&self) -> &[Char16] {
        &self.0
    }

    /// Converts this C string to a u16 slice without the trailing null.
    #[must_use]
    pub fn to_u16_slice(&self) -> &[u16] {
        let chars = self.to_u16_slice_with_nul();
        &chars[..chars.len() - 1]
    }

    /// Converts this C string to a u16 slice containing the trailing null.
    #[must_use]
    pub const fn to_u16_slice_with_nul(&self) -> &[u16] {
        unsafe { &*(&self.0 as *const [Char16] as *const [u16]) }
    }

    /// Returns an iterator over this C string
    #[must_use]
    pub const fn iter(&self) -> CStr16Iter {
        CStr16Iter {
            inner: self,
            pos: 0,
        }
    }

    /// Returns the number of characters without the trailing null. character
    #[must_use]
    pub const fn num_chars(&self) -> usize {
        self.0.len() - 1
    }

    /// Returns if the string is empty. This ignores the null character.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.num_chars() == 0
    }

    /// Get the number of bytes in the string (including the trailing null).
    #[must_use]
    pub const fn num_bytes(&self) -> usize {
        self.0.len() * 2
    }

    /// Writes each [`Char16`] as a [`char`] (4 bytes long in Rust language) into the buffer.
    /// It is up to the implementer of [`core::fmt::Write`] to convert the char to a string
    /// with proper encoding/charset. For example, in the case of [`alloc::string::String`]
    /// all Rust chars (UTF-32) get converted to UTF-8.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let firmware_vendor_c16_str: CStr16 = ...;
    /// // crate "arrayvec" uses stack-allocated arrays for Strings => no heap allocations
    /// let mut buf = arrayvec::ArrayString::<128>::new();
    /// firmware_vendor_c16_str.as_str_in_buf(&mut buf);
    /// log::info!("as rust str: {}", buf.as_str());
    /// ```
    ///
    /// [`alloc::string::String`]: https://doc.rust-lang.org/nightly/alloc/string/struct.String.html
    pub fn as_str_in_buf(&self, buf: &mut dyn core::fmt::Write) -> core::fmt::Result {
        for c16 in self.iter() {
            buf.write_char(char::from(*c16))?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl From<&CStr16> for alloc::string::String {
    fn from(value: &CStr16) -> Self {
        value
            .as_slice()
            .iter()
            .copied()
            .map(u16::from)
            .map(|int| int as u32)
            .map(|int| char::from_u32(int).expect("Should be encodable as UTF-8"))
            .collect::<alloc::string::String>()
    }
}

impl<StrType: AsRef<str> + ?Sized> EqStrUntilNul<StrType> for CStr16 {
    fn eq_str_until_nul(&self, other: &StrType) -> bool {
        let other = other.as_ref();

        let any_not_equal = self
            .iter()
            .copied()
            .map(char::from)
            .zip(other.chars())
            // This only works as CStr16 is guaranteed to have a fixed character length
            // (unlike UTF-8 or UTF-16).
            .take_while(|(l, r)| *l != '\0' && *r != '\0')
            .any(|(l, r)| l != r);

        !any_not_equal
    }
}

/// An iterator over the [`Char16`]s in a [`CStr16`].
#[derive(Debug)]
pub struct CStr16Iter<'a> {
    inner: &'a CStr16,
    pos: usize,
}

impl<'a> Iterator for CStr16Iter<'a> {
    type Item = &'a Char16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.0.len() - 1 {
            None
        } else {
            self.pos += 1;
            self.inner.0.get(self.pos - 1)
        }
    }
}

impl fmt::Debug for CStr16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CStr16({:?})", &self.0)
    }
}

impl fmt::Display for CStr16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.iter() {
            <Char16 as fmt::Display>::fmt(c, f)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl PartialEq<CString16> for &CStr16 {
    fn eq(&self, other: &CString16) -> bool {
        PartialEq::eq(*self, other.as_ref())
    }
}

impl<'a> UnalignedSlice<'a, u16> {
    /// Create a [`CStr16`] from an [`UnalignedSlice`] using an aligned
    /// buffer for storage. The lifetime of the output is tied to `buf`,
    /// not `self`.
    pub fn to_cstr16<'buf>(
        &self,
        buf: &'buf mut [MaybeUninit<u16>],
    ) -> Result<&'buf CStr16, UnalignedCStr16Error> {
        CStr16::from_unaligned_slice(self, buf)
    }
}

/// The EqStrUntilNul trait helps to compare Rust strings against UEFI string types (UCS-2 strings).
/// The given generic implementation of this trait enables us that we only have to
/// implement one direction (`left.eq_str_until_nul(&right)`) for each UEFI string type and we
/// get the other direction (`right.eq_str_until_nul(&left)`) for free. Hence, the relation is
/// reflexive.
pub trait EqStrUntilNul<StrType: ?Sized> {
    /// Checks if the provided Rust string `StrType` is equal to [Self] until the first null character
    /// is found. An exception is the terminating null character of [Self] which is ignored.
    ///
    /// As soon as the first null character in either `&self` or `other` is found, this method returns.
    /// Note that Rust strings are allowed to contain null bytes that do not terminate the string.
    /// Although this is rather unusual, you can compare `"foo\0bar"` with an instance of [Self].
    /// In that case, only `foo"` is compared against [Self] (if [Self] is long enough).
    fn eq_str_until_nul(&self, other: &StrType) -> bool;
}

// magic implementation which transforms an existing `left.eq_str_until_nul(&right)` implementation
// into an additional working `right.eq_str_until_nul(&left)` implementation.
impl<StrType, C16StrType> EqStrUntilNul<C16StrType> for StrType
where
    StrType: AsRef<str>,
    C16StrType: EqStrUntilNul<StrType> + ?Sized,
{
    fn eq_str_until_nul(&self, other: &C16StrType) -> bool {
        // reuse the existing implementation
        other.eq_str_until_nul(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use uefi_macros::{cstr16, cstr8};

    // Tests if our CStr8 type can be constructed from a valid core::ffi::CStr
    #[test]
    fn test_cstr8_from_cstr() {
        let msg = "hello world\0";
        let cstr = unsafe { CStr::from_ptr(msg.as_ptr().cast()) };
        let cstr8: &CStr8 = TryFrom::try_from(cstr).unwrap();
        assert!(cstr8.eq_str_until_nul(msg));
        assert!(msg.eq_str_until_nul(cstr8));
    }

    #[test]
    fn test_cstr16_num_bytes() {
        let s = CStr16::from_u16_with_nul(&[65, 66, 67, 0]).unwrap();
        assert_eq!(s.num_bytes(), 8);
    }

    #[test]
    fn test_cstr16_from_str_with_buf() {
        let mut buf = [0; 4];

        // OK: buf is exactly the right size.
        let s = CStr16::from_str_with_buf("ABC", &mut buf).unwrap();
        assert_eq!(s.to_u16_slice_with_nul(), [65, 66, 67, 0]);

        // OK: buf is bigger than needed.
        let s = CStr16::from_str_with_buf("A", &mut buf).unwrap();
        assert_eq!(s.to_u16_slice_with_nul(), [65, 0]);

        // Error: buf is too small.
        assert_eq!(
            CStr16::from_str_with_buf("ABCD", &mut buf).unwrap_err(),
            FromStrWithBufError::BufferTooSmall
        );

        // Error: invalid character.
        assert_eq!(
            CStr16::from_str_with_buf("a😀", &mut buf).unwrap_err(),
            FromStrWithBufError::InvalidChar(1),
        );

        // Error: interior null.
        assert_eq!(
            CStr16::from_str_with_buf("a\0b", &mut buf).unwrap_err(),
            FromStrWithBufError::InteriorNul(1),
        );
    }

    #[test]
    fn test_cstr16_macro() {
        // Just a sanity check to make sure it's spitting out the right characters
        assert_eq!(
            crate::prelude::cstr16!("ABC").to_u16_slice_with_nul(),
            [65, 66, 67, 0]
        )
    }

    #[test]
    fn test_unaligned_cstr16() {
        let mut buf = [0u16; 6];
        let us = unsafe {
            let ptr = buf.as_mut_ptr() as *mut u8;
            // Intentionally create an unaligned u16 pointer. This
            // leaves room for five u16 characters.
            let ptr = ptr.add(1) as *mut u16;
            // Write out the "test" string.
            ptr.add(0).write_unaligned(b't'.into());
            ptr.add(1).write_unaligned(b'e'.into());
            ptr.add(2).write_unaligned(b's'.into());
            ptr.add(3).write_unaligned(b't'.into());
            ptr.add(4).write_unaligned(b'\0'.into());

            // Create the `UnalignedSlice`.
            UnalignedSlice::new(ptr, 5)
        };

        // Test `to_cstr16()` with too small of a buffer.
        let mut buf = [MaybeUninit::new(0); 4];
        assert_eq!(
            us.to_cstr16(&mut buf).unwrap_err(),
            UnalignedCStr16Error::BufferTooSmall
        );
        // Test with a big enough buffer.
        let mut buf = [MaybeUninit::new(0); 5];
        assert_eq!(
            us.to_cstr16(&mut buf).unwrap(),
            CString16::try_from("test").unwrap()
        );

        // Test `to_cstring16()`.
        assert_eq!(
            us.to_cstring16().unwrap(),
            CString16::try_from("test").unwrap()
        );
    }

    #[test]
    fn test_cstr16_as_slice() {
        let string: &CStr16 = cstr16!("a");
        assert_eq!(string.as_slice(), &[Char16::try_from('a').unwrap()]);
        assert_eq!(
            string.as_slice_with_nul(),
            &[Char16::try_from('a').unwrap(), NUL_16]
        );
    }

    // Code generation helper for the compare tests of our CStrX types against "str" and "String"
    // from the standard library.
    #[allow(non_snake_case)]
    macro_rules! test_compare_cstrX {
        ($input:ident) => {
            assert!($input.eq_str_until_nul(&"test"));
            assert!($input.eq_str_until_nul(&String::from("test")));

            // now other direction
            assert!(String::from("test").eq_str_until_nul($input));
            assert!("test".eq_str_until_nul($input));

            // some more tests
            // this is fine: compare until the first null
            assert!($input.eq_str_until_nul(&"te\0st"));
            // this is fine
            assert!($input.eq_str_until_nul(&"test\0"));
            assert!(!$input.eq_str_until_nul(&"hello"));
        };
    }

    #[test]
    fn test_compare_cstr8() {
        // test various comparisons with different order (left, right)
        let input: &CStr8 = cstr8!("test");
        test_compare_cstrX!(input);
    }

    #[test]
    fn test_compare_cstr16() {
        let input: &CStr16 = cstr16!("test");
        test_compare_cstrX!(input);
    }

    /// Test that the `cstr16!` macro can be used in a `const` context.
    #[test]
    fn test_cstr16_macro_const() {
        const S: &CStr16 = cstr16!("ABC");
        assert_eq!(S.to_u16_slice_with_nul(), [65, 66, 67, 0]);
    }

    /// Tests the trait implementation of trait [`EqStrUntilNul]` for [`CStr8`].
    ///
    /// This tests that `String` and `str` from the standard library can be
    /// checked for equality against a [`CStr8`]. It checks both directions,
    /// i.e., the equality is reflexive.
    #[test]
    fn test_cstr8_eq_std_str() {
        let input: &CStr8 = cstr8!("test");

        // test various comparisons with different order (left, right)
        assert!(input.eq_str_until_nul("test")); // requires ?Sized constraint
        assert!(input.eq_str_until_nul(&"test"));
        assert!(input.eq_str_until_nul(&String::from("test")));

        // now other direction
        assert!(String::from("test").eq_str_until_nul(input));
        assert!("test".eq_str_until_nul(input));
    }

    /// Tests the trait implementation of trait [`EqStrUntilNul]` for [`CStr16`].
    ///
    /// This tests that `String` and `str` from the standard library can be
    /// checked for equality against a [`CStr16`]. It checks both directions,
    /// i.e., the equality is reflexive.
    #[test]
    fn test_cstr16_eq_std_str() {
        let input: &CStr16 = cstr16!("test");

        assert!(input.eq_str_until_nul("test")); // requires ?Sized constraint
        assert!(input.eq_str_until_nul(&"test"));
        assert!(input.eq_str_until_nul(&String::from("test")));

        // now other direction
        assert!(String::from("test").eq_str_until_nul(input));
        assert!("test".eq_str_until_nul(input));
    }
}
