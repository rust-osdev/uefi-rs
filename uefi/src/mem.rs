//! This is a utility module with helper methods for allocations/memory.

use crate::ResultExt;
use crate::{Result, Status};
use ::alloc::{alloc, boxed::Box};
use core::alloc::Layout;
use core::fmt::Debug;
use core::slice;
use uefi::data_types::Align;
use uefi::Error;

/// Helper to return owned versions of certain UEFI data structures on the heap in a [`Box`]. This
/// function is intended to wrap low-level UEFI functions of this crate that
/// - can consume an empty buffer without a panic to get the required buffer size in the errors
///   payload,
/// - consume a mutable reference to a buffer that will be filled with some data if the provided
///   buffer size is sufficient, and
/// - return a mutable typed reference that points to the same memory as the input buffer on
///   success.
pub fn make_boxed<
    'a,
    Data: Align + ?Sized + Debug + 'a,
    F: FnMut(&'a mut [u8]) -> Result<&'a mut Data, Option<usize>>,
>(
    mut fetch_data_fn: F,
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

    // Allocate the buffer.
    let heap_buf: *mut u8 = unsafe {
        let ptr = alloc::alloc(layout);
        if ptr.is_null() {
            return Err(Status::OUT_OF_RESOURCES.into());
        }
        ptr
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
            unsafe { alloc::dealloc(heap_buf, layout) };
            return Err(err);
        }
    };

    let data = unsafe { Box::from_raw(data) };

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResultExt;
    use core::mem::{align_of, size_of};

    #[derive(Debug)]
    #[repr(C)]
    struct SomeData([u8; 4]);

    impl Align for SomeData {
        fn alignment() -> usize {
            align_of::<Self>()
        }
    }

    /// Function that behaves like the other UEFI functions. It takes a
    /// mutable reference to a buffer memory that represents a [`SomeData`]
    /// instance.
    fn uefi_function_stub_read(buf: &mut [u8]) -> Result<&mut SomeData, Option<usize>> {
        if buf.len() < 4 {
            return Status::BUFFER_TOO_SMALL.into_with(|| panic!(), |_| Some(4));
        };

        buf[0] = 1;
        buf[1] = 2;
        buf[2] = 3;
        buf[3] = 4;

        let data = unsafe { &mut *buf.as_mut_ptr().cast::<SomeData>() };

        Ok(data)
    }

    // Some basic checks so that miri reports everything is fine.
    #[test]
    fn some_data_type_size_constraints() {
        assert_eq!(size_of::<SomeData>(), 4);
        assert_eq!(align_of::<SomeData>(), 1);
    }

    #[test]
    fn basic_stub_read() {
        assert_eq!(
            uefi_function_stub_read(&mut []).status(),
            Status::BUFFER_TOO_SMALL
        );
        assert_eq!(
            *uefi_function_stub_read(&mut []).unwrap_err().data(),
            Some(4)
        );

        let mut buf: [u8; 4] = [0; 4];
        let data = uefi_function_stub_read(&mut buf).unwrap();

        assert_eq!(&data.0, &[1, 2, 3, 4])
    }

    #[test]
    fn make_boxed_utility() {
        let fetch_data_fn = |buf| uefi_function_stub_read(buf);
        let data: Box<SomeData> = make_boxed(fetch_data_fn).unwrap();

        assert_eq!(&data.0, &[1, 2, 3, 4])
    }
}
