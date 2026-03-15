// SPDX-License-Identifier: MIT OR Apache-2.0

use core::{
    ptr::{self, NonNull},
    slice,
};

/// Copy the bytes of `val` to `ptr`, then advance pointer to just after the
/// newly-copied bytes.
pub const unsafe fn ptr_write_unaligned_and_add<T>(ptr: &mut *mut u8, val: T) {
    unsafe {
        ptr.cast::<T>().write_unaligned(val);
        *ptr = ptr.add(size_of::<T>());
    }
}

/// Convert from a `u32` to a `usize`. Panic if the input does fit. On typical
/// targets `usize` is at least as big as `u32`, so this should never panic
/// except on unusual targets.
///
/// Comparison to alternatives:
/// * `val as usize` doesn't check that `val` actually fits in a `usize`.
/// * `usize::try_from(val).unwrap()` doesn't work in a const context.
pub const fn usize_from_u32(val: u32) -> usize {
    // This is essentially the same as `usize::try_from(val).unwrap()`, but
    // works in a `const` context on stable.
    if size_of::<usize>() < size_of::<u32>() && val < (usize::MAX as u32) {
        panic!("value does not fit in a usize");
    } else {
        val as usize
    }
}

/// Get the raw pointer from `opt`, defaulting to `null_mut`.
pub fn opt_nonnull_to_ptr<T>(opt: Option<NonNull<T>>) -> *mut T {
    opt.map(NonNull::as_ptr).unwrap_or(ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usize_from_u32() {
        assert_eq!(usize_from_u32(0), 0usize);
        assert_eq!(usize_from_u32(u32::MAX), 4294967295usize);
    }
}

/// Error returned when converting a `&[u8]` to `&[u16]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SliceConversionError {
    /// There were odd number of bytes
    InvalidLength,

    /// The byte slice pointer aligntment was incorrect
    InvalidAlignment,
}

// Converts a byte slice to a u16 slice.
//
/// Checks for:
/// * `bytes` has an even length (so that it can be completely converte to slice of `u16`).
/// * starting byte of `bytes` is not properly aligned for `u16`.
pub(crate) fn try_cast_u8_to_u16(bytes: &[u8]) -> Result<&[u16], SliceConversionError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(SliceConversionError::InvalidLength);
    }

    if bytes.as_ptr().align_offset(align_of::<u16>()) != 0 {
        return Err(SliceConversionError::InvalidAlignment);
    }

    let u16_slice = unsafe { slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len() / 2) };
    Ok(u16_slice)
}

#[test]
fn test_try_cast_u8_to_u16() {
    use crate::cstr16;

    // 1. good case
    let s = cstr16!("hello");
    let input = s.as_bytes();
    let expected = s.to_u16_slice_with_nul();
    assert_eq!(try_cast_u8_to_u16(input), Ok(expected));

    // 2. bad case (odd length)
    let input = b"123";
    assert_eq!(
        try_cast_u8_to_u16(input),
        Err(SliceConversionError::InvalidLength)
    );
}
