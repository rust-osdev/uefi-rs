use core::mem;
use core::ptr::{self, NonNull};

/// Copy the bytes of `val` to `ptr`, then advance pointer to just after the
/// newly-copied bytes.
pub unsafe fn ptr_write_unaligned_and_add<T>(ptr: &mut *mut u8, val: T) {
    ptr.cast::<T>().write_unaligned(val);
    *ptr = ptr.add(mem::size_of::<T>());
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
    if mem::size_of::<usize>() < mem::size_of::<u32>() && val < (usize::MAX as u32) {
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
