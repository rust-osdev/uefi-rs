//! This is a utility module with helper methods for allocations/memory.

use crate::data_types::Align;
use crate::{Error, Result, ResultExt, Status};
use ::alloc::boxed::Box;
use core::alloc::Layout;
use core::fmt::Debug;
use core::slice;

#[cfg(not(feature = "unstable"))]
use ::alloc::alloc::{alloc, dealloc};

#[cfg(feature = "unstable")]
use {core::alloc::Allocator, core::ptr::NonNull};

/// Helper to return owned versions of certain UEFI data structures on the heap in a [`Box`]. This
/// function is intended to wrap low-level UEFI functions of this crate that
/// - can consume an empty buffer without a panic to get the required buffer size in the errors
///   payload,
/// - consume a mutable reference to a buffer that will be filled with some data if the provided
///   buffer size is sufficient, and
/// - return a mutable typed reference that points to the same memory as the input buffer on
///   success.
///
/// # Feature `unstable` / `allocator_api`
/// By default, this function works with Rust's default allocation mechanism. If you activate the
/// `unstable`-feature, it uses the `allocator_api` instead. In that case, the function takes an
/// additional parameter describing the specific [`Allocator`]. You can use [`alloc::alloc::Global`]
/// as default.
///
/// [`Allocator`]: https://doc.rust-lang.org/alloc/alloc/trait.Allocator.html
/// [`alloc::alloc::Global`]: https://doc.rust-lang.org/alloc/alloc/struct.Global.html
pub(crate) fn make_boxed<
    'a,
    // The UEFI data structure.
    Data: Align + ?Sized + Debug + 'a,
    F: FnMut(&'a mut [u8]) -> Result<&'a mut Data, Option<usize>>,
    #[cfg(feature = "unstable")] A: Allocator,
>(
    // A function to read the UEFI data structure into a provided buffer.
    mut fetch_data_fn: F,
    #[cfg(feature = "unstable")]
    // Allocator of the `allocator_api` feature. You can use `Global` as default.
    allocator: A,
) -> Result<Box<Data>> {
    let required_size = match fetch_data_fn(&mut []).map_err(Error::split) {
        // This is the expected case: the empty buffer passed in is too
        // small, so we get the required size.
        Err((Status::BUFFER_TOO_SMALL, Some(required_size))) => Ok(required_size),
        // Propagate any other error.
        Err((status, _)) => Err(Error::from(status)),
        // Success is unexpected, return an error.
        Ok(_) => Err(Error::from(Status::UNSUPPORTED)),
    }?;

    // We add trailing padding because the size of a rust structure must
    // always be a multiple of alignment.
    let layout = Layout::from_size_align(required_size, Data::alignment())
        .unwrap()
        .pad_to_align();

    // Allocate the buffer on the heap.
    let heap_buf: *mut u8 = {
        #[cfg(not(feature = "unstable"))]
        {
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(Status::OUT_OF_RESOURCES.into());
            }
            ptr
        }

        #[cfg(feature = "unstable")]
        allocator
            .allocate(layout)
            .map_err(|_| <Status as Into<Error>>::into(Status::OUT_OF_RESOURCES))?
            .as_ptr()
            .cast::<u8>()
    };

    // Read the data into the provided buffer.
    let data: Result<&mut Data> = {
        let buffer = unsafe { slice::from_raw_parts_mut(heap_buf, required_size) };
        fetch_data_fn(buffer).discard_errdata()
    };

    // If an error occurred, deallocate the memory before returning.
    let data: &mut Data = match data {
        Ok(data) => data,
        Err(err) => {
            #[cfg(not(feature = "unstable"))]
            unsafe {
                dealloc(heap_buf, layout)
            };
            #[cfg(feature = "unstable")]
            unsafe {
                allocator.deallocate(NonNull::new(heap_buf).unwrap(), layout)
            }
            return Err(err);
        }
    };

    let data = unsafe { Box::from_raw(data) };

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResultExt, StatusExt};
    #[cfg(feature = "unstable")]
    use alloc::alloc::Global;
    use core::mem::{align_of, size_of};

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

        let data = unsafe { &mut *buf.as_mut_ptr().cast::<Data>() };

        Ok(data)
    }

    // Some basic sanity checks so that we can catch problems early that miri would detect
    // otherwise.
    #[test]
    fn test_some_data_type_size_constraints() {
        assert_eq!(size_of::<SomeData>(), 4);
        assert_eq!(SomeData::alignment(), 1);
        assert_eq!(
            size_of::<SomeDataAlign16>(),
            16,
            "The size must be 16 instead of 4, as in Rust the runtime size is a multiple of the alignment."
        );
        assert_eq!(SomeDataAlign16::alignment(), 16);
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
        assert_eq!(&data.0 .0, &[1, 2, 3, 4]);
    }

    /// This unit tests checks the [`make_boxed`] utility. The test has different code and behavior
    /// depending on whether the "unstable" feature is active or not.
    #[test]
    fn test_make_boxed_utility() {
        let fetch_data_fn = |buf| uefi_function_stub_read(buf);

        #[cfg(not(feature = "unstable"))]
        let data: Box<SomeData> = make_boxed(fetch_data_fn).unwrap();

        #[cfg(feature = "unstable")]
        let data: Box<SomeData> = make_boxed(fetch_data_fn, Global).unwrap();
        assert_eq!(&data.0, &[1, 2, 3, 4]);

        let fetch_data_fn = |buf| uefi_function_stub_read(buf);

        #[cfg(not(feature = "unstable"))]
        let data: Box<SomeDataAlign16> = make_boxed(fetch_data_fn).unwrap();

        #[cfg(feature = "unstable")]
        let data: Box<SomeDataAlign16> = make_boxed(fetch_data_fn, Global).unwrap();

        assert_eq!(&data.0 .0, &[1, 2, 3, 4]);
    }
}
