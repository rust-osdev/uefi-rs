use core::marker::PhantomData;
use core::mem::MaybeUninit;

#[cfg(feature = "exts")]
use crate::alloc_api::vec::Vec;

/// Slice backed by a potentially-unaligned pointer.
///
/// This wrapper can be used to safely expose slices that are inside a
/// [`repr(packed)`] struct. The element type must be [`Copy`].
///
/// [`repr(packed)`]: https://doc.rust-lang.org/nomicon/other-reprs.html#reprpacked
#[derive(Debug)]
pub struct UnalignedSlice<'a, T: Copy> {
    data: *const T,
    len: usize,
    _phantom_lifetime: PhantomData<&'a T>,
}

impl<'a, T: Copy> UnalignedSlice<'a, T> {
    /// Create an `UnalignedSlice` from a raw pointer. The pointer must
    /// not be dangling but can be unaligned. The `len` parameter is the
    /// number of elements in the slice (not the number of bytes).
    ///
    /// # Safety
    ///
    /// The `data` pointer must point to a packed array of at least
    /// `len` elements of type `T`. The pointer must remain valid for as
    /// long as the `'a` lifetime.
    pub unsafe fn new(data: *const T, len: usize) -> Self {
        Self {
            data,
            len,
            _phantom_lifetime: PhantomData::default(),
        }
    }

    /// Returns true if the slice has a length of 0.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the slice.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the element at `index`, or `None` if the `index` is out
    /// of bounds.
    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len {
            Some(unsafe { self.data.add(index).read_unaligned() })
        } else {
            None
        }
    }

    /// Returns an iterator over the slice.
    ///
    /// The iterator yields all items from start to end.
    pub fn iter(&'a self) -> UnalignedSliceIter<'a, T> {
        UnalignedSliceIter {
            slice: self,
            index: 0,
        }
    }

    /// Copy the data to an aligned buffer.
    ///
    /// The length of `dest` must be the same as `self`.
    ///
    /// # Panics
    ///
    /// This function will panic if the two slices have different lengths.
    pub fn copy_to(&self, dest: &mut [T]) {
        if dest.len() != self.len {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                self.len(),
                dest.len(),
            );
        }

        for (i, elem) in dest.iter_mut().enumerate() {
            *elem = unsafe { self.data.add(i).read_unaligned() };
        }
    }

    /// Copy the data to an aligned [`MaybeUninit`] buffer.
    ///
    /// The length of `dest` must be the same as `self`.
    ///
    /// This function fully initializes the `dest` slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the two slices have different lengths.
    pub fn copy_to_maybe_uninit(&self, dest: &mut [MaybeUninit<T>]) {
        if dest.len() != self.len {
            panic!(
                "source slice length ({}) does not match destination slice length ({})",
                self.len(),
                dest.len(),
            );
        }

        for (i, elem) in dest.iter_mut().enumerate() {
            unsafe { elem.write(self.data.add(i).read_unaligned()) };
        }
    }

    /// Copies `self` into a new `Vec`.
    #[cfg(feature = "exts")]
    pub fn to_vec(&self) -> Vec<T> {
        let len = self.len();
        let mut v = Vec::with_capacity(len);
        unsafe {
            self.copy_to_maybe_uninit(v.spare_capacity_mut());
            v.set_len(len);
        }
        v
    }
}

#[cfg(feature = "exts")]
impl<'a, T: Copy> From<UnalignedSlice<'a, T>> for Vec<T> {
    fn from(input: UnalignedSlice<'a, T>) -> Self {
        input.to_vec()
    }
}

impl<'a, T: Copy> IntoIterator for UnalignedSlice<'a, T> {
    type Item = T;
    type IntoIter = UnalignedSliceIntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        UnalignedSliceIntoIter {
            slice: self,
            index: 0,
        }
    }
}

impl<'a, T: Copy> IntoIterator for &'a UnalignedSlice<'a, T> {
    type Item = T;
    type IntoIter = UnalignedSliceIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator for a [`UnalignedSlice`].
pub struct UnalignedSliceIntoIter<'a, T: Copy> {
    slice: UnalignedSlice<'a, T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for UnalignedSliceIntoIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let output = self.slice.get(self.index)?;
        self.index += 1;
        Some(output)
    }
}

/// Iterator for a [`UnalignedSlice`] reference.
pub struct UnalignedSliceIter<'a, T: Copy> {
    slice: &'a UnalignedSlice<'a, T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for UnalignedSliceIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let output = self.slice.get(self.index)?;
        self.index += 1;
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc_api::vec::Vec;

    #[test]
    fn test_unaligned_slice() {
        #[rustfmt::skip]
        let bytes: [u8; 13] = [
            // Extra byte to make the rest of the data unaligned.
            0,
            // First element.
            0x10, 0x11, 0x12, 0x13,
            // Second element.
            0x20, 0x21, 0x22, 0x23,
            // Third element.
            0x30, 0x31, 0x32, 0x33,
        ];

        // Skip past the first byte and create an unaligned `*const u32` pointer.
        let bytes = &bytes[1..];
        let slice_ptr: *const u32 = bytes.as_ptr().cast();

        let slice: UnalignedSlice<u32> = unsafe { UnalignedSlice::new(slice_ptr, 0) };
        assert!(slice.is_empty());

        let slice: UnalignedSlice<u32> = unsafe { UnalignedSlice::new(slice_ptr, 3) };
        assert!(!slice.is_empty());
        assert_eq!(slice.len(), 3);

        assert_eq!(slice.get(0), Some(0x13121110));
        assert_eq!(slice.get(1), Some(0x23222120));
        assert_eq!(slice.get(2), Some(0x33323130));
        assert_eq!(slice.get(3), None);

        let mut copy = [0; 3];
        slice.copy_to(&mut copy);
        assert_eq!(copy, [0x13121110, 0x23222120, 0x33323130]);

        assert_eq!(
            slice.iter().collect::<Vec<_>>(),
            [0x13121110, 0x23222120, 0x33323130]
        );

        assert_eq!(
            slice.into_iter().collect::<Vec<_>>(),
            [0x13121110, 0x23222120, 0x33323130]
        );
    }
}
