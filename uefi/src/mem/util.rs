// SPDX-License-Identifier: MIT OR Apache-2.0

//! This is a utility module with helper methods for allocations/memory.

use crate::data_types::Align;
use crate::{Error, Result, Status};
use ::alloc::alloc::{alloc, dealloc, realloc};
use ::alloc::boxed::Box;
use core::alloc::Layout;
use core::ptr::{self, NonNull};
use core::slice;
use ptr_meta::Pointee;

/// Calls `fetch_data_fn` with an empty buffer to learn the size it needs.
fn query_required_size<'a, Data: ?Sized + 'a>(
    fetch_data_fn: &mut impl FnMut(&'a mut [u8]) -> Result<&'a mut Data, Option<usize>>,
) -> Result<usize> {
    match fetch_data_fn(&mut []).map_err(Error::split) {
        // The expected case: the empty buffer is too small, and the error
        // payload tells us how much is needed. A size of zero contradicts
        // that error and cannot be allocated, so treat it as a missing size.
        Err((Status::BUFFER_TOO_SMALL, Some(size))) if size > 0 => Ok(size),
        Err((status, _)) => Err(status.into()),
        Ok(_) => {
            log::debug!("empty buffer was unexpectedly accepted");
            Err(Status::UNSUPPORTED.into())
        }
    }
}

/// Resizes the allocation `buf` so that it exactly matches `value_layout`,
/// and returns the address of the resized allocation.
///
/// # Safety
///
/// `buf` must be an allocation of `layout` that nothing else owns, and
/// `value_layout` must have the same alignment as `layout` and must not be
/// larger than it.
unsafe fn fit_allocation(
    buf: NonNull<u8>,
    layout: Layout,
    value_layout: Layout,
) -> Result<*mut u8> {
    debug_assert_eq!(value_layout.align(), layout.align());
    debug_assert!(value_layout.size() <= layout.size());

    if value_layout.size() == layout.size() {
        return Ok(buf.as_ptr());
    }

    if value_layout.size() == 0 {
        // A zero-sized value does not own an allocation, it just needs a
        // well-aligned address.
        // SAFETY: Guaranteed by the caller.
        unsafe { dealloc(buf.as_ptr(), layout) };
        return Ok(ptr::without_provenance_mut(value_layout.align()));
    }

    // SAFETY: Guaranteed by the caller. The new size is non-zero and smaller
    // than the current one, so this is a valid shrink.
    let shrunk = unsafe { realloc(buf.as_ptr(), layout, value_layout.size()) };
    if shrunk.is_null() {
        // SAFETY: A failed `realloc` leaves the original allocation intact.
        unsafe { dealloc(buf.as_ptr(), layout) };
        return Err(Status::OUT_OF_RESOURCES.into());
    }
    Ok(shrunk)
}

/// Helper to return owned versions of certain UEFI data structures on the heap
/// in a [`Box`].
///
/// This function is intended to wrap low-level UEFI functions of this crate
/// that:
/// - can consume an empty buffer without a panic to get the required buffer
///   size in the errors payload,
/// - consume a mutable reference to a buffer that will be filled with some data
///   if the provided buffer size is sufficient, and
/// - return a mutable typed reference that points to the same memory as the
///   input buffer on success.
///
/// The complexity here comes from the fact that we first allocate matching
/// memory and then we must make sure that the layout of the inner value
/// assumed by `Box` matches the allocation to fulfill all Rust guarantees. We
/// only learn the value's real size after the firmware call: we allocate what
/// the firmware said it needed, and the fetch closure returns what it actually
/// wrote, which is smaller when a file shrinks between the size query and the
/// read, or when firmware over-reports. Freeing a 112-byte allocation with a
/// 104-byte layout is undefined behavior, so the allocation is resized to match
/// before ownership is handed to `Box`. In the normal case the two sizes agree
/// and no reallocation happens at all.
pub(crate) fn make_boxed<
    'a,
    // The UEFI data structure.
    Data: Align + Pointee + ?Sized + 'a,
    F: FnMut(&'a mut [u8]) -> Result<&'a mut Data, Option<usize>>,
>(
    // A function to read the UEFI data structure into a provided buffer.
    mut fetch_data_fn: F,
) -> Result<Box<Data>> {
    let required_size = query_required_size(&mut fetch_data_fn)?;

    // The size of a Rust value is always a multiple of its alignment, so the
    // allocation needs the trailing padding on top of the reported size.
    let layout = Layout::from_size_align(required_size, Data::alignment())
        .unwrap()
        .pad_to_align();

    // SAFETY: The layout has a non-zero size.
    let buf =
        NonNull::new(unsafe { alloc(layout) }).ok_or(Error::from(Status::OUT_OF_RESOURCES))?;

    // Hand out the whole allocation, padding included, so that a reference to
    // `Data` derived from it does not exceed the provenance of this slice.
    // SAFETY: The allocation is valid for `layout.size()` bytes.
    let slice = unsafe { slice::from_raw_parts_mut(buf.as_ptr(), layout.size()) };

    let data = match fetch_data_fn(slice) {
        Ok(data) => data,
        Err(err) => {
            // SAFETY: Allocated with `layout` and unused from here on.
            unsafe { dealloc(buf.as_ptr(), layout) };
            return Err(err.to_err_without_payload());
        }
    };

    // `Box` frees using the layout of its value, which is smaller than the
    // allocation if the firmware wrote less than the size it reported. Freeing
    // with a layout that does not match the allocation is undefined behavior,
    // so resize the allocation to fit the value first.
    let value_layout = Layout::for_value(data);
    // Take the metadata before the resize invalidates `data`.
    let meta = ptr_meta::metadata(ptr::from_mut(data));
    // SAFETY: `buf` was allocated with `layout` and holds `data` alone, whose
    // alignment is `Data::alignment()` and whose size fits the allocation.
    let fitted = unsafe { fit_allocation(buf, layout, value_layout)? };

    let ptr = ptr_meta::from_raw_parts_mut(fitted.cast(), meta);

    // SAFETY: The allocation now matches the layout of the value.
    Ok(unsafe { Box::from_raw(ptr) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResultExt, StatusExt};

    /// Some simple dummy type to test [`make_boxed`].
    #[derive(Debug)]
    #[repr(C)]
    struct SomeData([u8; 4]);

    impl Align for SomeData {
        fn alignment() -> usize {
            align_of::<Self>()
        }
    }

    /// Type wrapper that ensures an alignment of 16 for the underlying data.
    #[derive(Debug)]
    #[repr(C, align(16))]
    struct Align16<T>(T);

    /// Version of [`SomeData`] that has an alignment of 16.
    type SomeDataAlign16 = Align16<SomeData>;

    impl Align for SomeDataAlign16 {
        fn alignment() -> usize {
            align_of::<Self>()
        }
    }

    // Some basic sanity checks of the helper types so that we can catch
    // problems early that miri would detect otherwise.
    const _: () = {
        assert!(size_of::<SomeData>() == 4);
        assert!(align_of::<SomeData>() == 1);
        // The size is 16 instead of 4, as in Rust the size is always a
        // multiple of the alignment.
        assert!(size_of::<SomeDataAlign16>() == 16);
        assert!(align_of::<SomeDataAlign16>() == 16);
    };

    /// Function that behaves like the other UEFI functions. It takes a
    /// mutable reference to a buffer memory that represents a [`SomeData`]
    /// instance.
    fn uefi_function_stub_read<Data: Align>(buf: &mut [u8]) -> Result<&mut Data, Option<usize>> {
        let required_size = size_of::<Data>();

        if buf.len() < required_size {
            // We can use an zero-length buffer to find the required size.
            return Status::BUFFER_TOO_SMALL.to_result_with(|| panic!(), |_| Some(required_size));
        };

        // assert alignment
        assert_eq!(
            buf.as_ptr() as usize % Data::alignment(),
            0,
            "The buffer must be correctly aligned!"
        );

        buf[0] = 1;
        buf[1] = 2;
        buf[2] = 3;
        buf[3] = 4;

        // SAFETY: The buffer is not null, aligned, and initialized.
        let data = unsafe { &mut *buf.as_mut_ptr().cast::<Data>() };

        Ok(data)
    }

    // Tests `uefi_function_stub_read` which is the foundation for the `test_make_boxed_utility`
    // test.
    #[test]
    fn test_basic_stub_read() {
        assert_eq!(
            uefi_function_stub_read::<SomeData>(&mut []).status(),
            Status::BUFFER_TOO_SMALL
        );
        assert_eq!(
            *uefi_function_stub_read::<SomeData>(&mut [])
                .unwrap_err()
                .data(),
            Some(4)
        );

        let mut buf: [u8; 4] = [0; 4];
        let data: &mut SomeData = uefi_function_stub_read(&mut buf).unwrap();
        assert_eq!(&data.0, &[1, 2, 3, 4]);

        let mut buf: Align16<[u8; 16]> = Align16([0; 16]);
        let data: &mut SomeDataAlign16 = uefi_function_stub_read(&mut buf.0).unwrap();
        assert_eq!(&data.0.0, &[1, 2, 3, 4]);
    }

    /// A reported required size of zero is inconsistent with
    /// `BUFFER_TOO_SMALL` for an empty buffer and must not lead to a
    /// zero-size allocation.
    #[test]
    fn test_make_boxed_zero_required_size() {
        let fetch_data_fn = |_buf: &mut [u8]| -> Result<&mut SomeData, Option<usize>> {
            Status::BUFFER_TOO_SMALL.to_result_with(|| panic!(), |_| Some(0))
        };

        let result = make_boxed(fetch_data_fn);
        assert_eq!(result.unwrap_err().status(), Status::BUFFER_TOO_SMALL);
    }

    /// The fetch function may return less data than it reported as
    /// required, e.g. when the firmware over-reports the size or the data
    /// shrinks between the two calls. The returned `Box` must then own an
    /// allocation that matches the value, or its deallocation is undefined
    /// behavior.
    #[test]
    fn test_make_boxed_shrinking_data() {
        /// Reports 16 bytes as required, but returns only `data_len` of them.
        fn uefi_function_stub_read_less(
            buf: &mut [u8],
            data_len: usize,
        ) -> Result<&mut [u8], Option<usize>> {
            if buf.is_empty() {
                return Status::BUFFER_TOO_SMALL.to_result_with(|| panic!(), |_| Some(16));
            }
            assert_eq!(buf.len(), 16);
            buf[..data_len].fill(7);
            Ok(&mut buf[..data_len])
        }

        for data_len in [8, 0] {
            let fetch_data_fn = |buf| uefi_function_stub_read_less(buf, data_len);

            let data: Box<[u8]> = make_boxed(fetch_data_fn).unwrap();
            assert_eq!(data.len(), data_len);
            assert!(data.iter().all(|byte| *byte == 7));
        }
    }

    /// This unit tests checks the [`make_boxed`] utility.
    ///
    /// This test is especially useful when run by miri.
    #[test]
    fn test_make_boxed_utility() {
        let fetch_data_fn = |buf| uefi_function_stub_read(buf);

        let data: Box<SomeData> = make_boxed(fetch_data_fn).unwrap();
        assert_eq!(&data.0, &[1, 2, 3, 4]);

        let fetch_data_fn = |buf| uefi_function_stub_read(buf);

        let data: Box<SomeDataAlign16> = make_boxed(fetch_data_fn).unwrap();

        assert_eq!(&data.0.0, &[1, 2, 3, 4]);
    }
}
