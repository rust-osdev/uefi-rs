/// Encode a string literal as a [`&CStr8`].
///
/// The encoding is done at compile time, so the result can be used in a
/// `const` item.
///
/// An empty string containing just a null character can be created with either
/// `cstr8!()` or `cstr8!("")`.
///
/// # Example
///
/// ```
/// use uefi::{CStr8, cstr8};
///
/// const S: &CStr8 = cstr8!("abÿ");
/// assert_eq!(S.as_bytes(), [97, 98, 255, 0]);
///
/// const EMPTY: &CStr8 = cstr8!();
/// assert_eq!(EMPTY.as_bytes(), [0]);
/// assert_eq!(cstr8!(""), EMPTY);
/// ```
///
/// [`&CStr8`]: crate::CStr8
#[macro_export]
macro_rules! cstr8 {
    () => {{
        const S: &[u8] = &[0];
        // SAFETY: `S` is a trivially correct Latin-1 C string.
        unsafe { $crate::CStr8::from_bytes_with_nul_unchecked(S) }
    }};
    ($s:literal) => {{
        // Use `const` values here to force errors to happen at compile
        // time.

        // Add one for the null char.
        const NUM_CHARS: usize = $crate::data_types::str_num_latin1_chars($s) + 1;

        const VAL: [u8; NUM_CHARS] = $crate::data_types::str_to_latin1($s);

        // SAFETY: the `str_to_latin1` function always produces a valid Latin-1
        // string with a trailing null character.
        unsafe { $crate::CStr8::from_bytes_with_nul_unchecked(&VAL) }
    }};
}

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

/// Set the entry point for a UEFI executable.
///
/// This macro takes one argument, the function to run when the executable is
/// launched. That function must take no arguments and return a [`uefi::Result`].
///
/// # Example
///
/// ```
/// uefi::set_main!(main);
///
/// fn main() -> uefi::Result {
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! set_main {
    ($main:ident) => {
        #[export_name = "efi_main"]
        fn efi_main(
            image: ::uefi::Handle,
            system_table: *mut ::core::ffi::c_void,
        ) -> ::uefi::Status {
            unsafe { ::uefi::boot::set_image_handle(image) };
            unsafe { ::uefi::system::set_system_table(system_table.cast()) };
            let result = $main();
            match result {
                Ok(()) => ::uefi::Status::SUCCESS,
                Err(err) => err.status(),
            }
        }
    };
}
