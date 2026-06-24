// SPDX-License-Identifier: MIT OR Apache-2.0

use core::ptr::{self, NonNull};

/// Copy the bytes of `val` to `ptr`, then advance pointer to just after the
/// newly-copied bytes.
pub const unsafe fn ptr_write_unaligned_and_add<T>(ptr: &mut *mut u8, val: T) {
    // SAFETY: The memory is valid.
    unsafe {
        ptr.cast::<T>().write_unaligned(val);
        *ptr = ptr.add(size_of::<T>());
    }
}

/// Converts a `u32` to `usize`.
///
/// Panics if `val` does not fit in `usize`.
///
/// On targets where `usize` is at least 32 bits wide, this never panics.
///
/// Unlike `as`, this does not silently truncate on narrow `usize` targets.
/// Unlike `usize::try_from(...).unwrap()`, this works in `const` contexts.
pub const fn usize_from_u32(val: u32) -> usize {
    if size_of::<usize>() < size_of::<u32>() && val > usize::MAX as u32 {
        panic!("value does not fit in usize");
    }

    val as usize
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
