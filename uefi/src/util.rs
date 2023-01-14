use core::mem;

/// Copy the bytes of `val` to `ptr`, then advance pointer to just after the
/// newly-copied bytes.
pub unsafe fn ptr_write_unaligned_and_add<T>(ptr: &mut *mut u8, val: T) {
    ptr.cast::<T>().write_unaligned(val);
    *ptr = ptr.add(mem::size_of::<T>());
}
