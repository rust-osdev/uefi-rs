/// Encode a string literal as a [`&CStr16`].
///
/// The encoding is done at compile time, so the result can be used in a
/// `const` item.
///
/// An empty string containing just a null character can be created with either
/// `cstr16!()` or `cstr16!("")`.
///
/// # Example
///
/// ```
/// use uefi::{CStr16, cstr16};
///
/// const S: &CStr16 = cstr16!("abc");
/// assert_eq!(S.to_u16_slice_with_nul(), [97, 98, 99, 0]);
///
/// const EMPTY: &CStr16 = cstr16!();
/// assert_eq!(EMPTY.to_u16_slice_with_nul(), [0]);
/// assert_eq!(cstr16!(""), EMPTY);
/// ```
///
/// [`&CStr16`]: crate::CStr16
#[macro_export]
macro_rules! cstr16 {
    () => {{
        const S: &[u16] = &[0];
        // SAFETY: `S` is a trivially correct UCS-2 C string.
        unsafe { $crate::CStr16::from_u16_with_nul_unchecked(S) }
    }};
    ($s:literal) => {{
        const S: &[u16] = &$crate::ucs2_cstr!($s);
        // SAFETY: the ucs2_cstr macro always produces a valid UCS-2 string with
        // a trailing null character.
        unsafe { $crate::CStr16::from_u16_with_nul_unchecked(S) }
    }};
}
