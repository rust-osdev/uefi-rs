//! Polyfills for functions in the standard library that are currently gated
//! behind unstable features.

use core::mem::MaybeUninit;
#[cfg(feature = "alloc")]
use {alloc::vec::Vec, core::mem::ManuallyDrop};

/// Polyfill for the unstable `MaybeUninit::slice_assume_init_ref` function.
///
/// See <https://github.com/rust-lang/rust/issues/63569>.
pub const unsafe fn maybe_uninit_slice_assume_init_ref<T>(s: &[MaybeUninit<T>]) -> &[T] {
    unsafe { &*(s as *const [MaybeUninit<T>] as *const [T]) }
}

/// Polyfill for the unstable `MaybeUninit::slice_as_mut_ptr` function.
///
/// See <https://github.com/rust-lang/rust/issues/63569>.
pub fn maybe_uninit_slice_as_mut_ptr<T>(s: &mut [MaybeUninit<T>]) -> *mut T {
    s.as_mut_ptr().cast::<T>()
}

/// Polyfill for the unstable `Vec::into_raw_parts` function.
///
/// See <https://github.com/rust-lang/rust/issues/65816>.
#[cfg(feature = "alloc")]
pub fn vec_into_raw_parts<T>(v: Vec<T>) -> (*mut T, usize, usize) {
    let mut v = ManuallyDrop::new(v);
    (v.as_mut_ptr(), v.len(), v.capacity())
}
