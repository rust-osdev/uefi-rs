//! UEFI boot services.
//!
//! These functions will panic if called after exiting boot services.

use crate::data_types::PhysicalAddress;
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::{self, NonNull};
use core::slice;
use core::sync::atomic::{AtomicPtr, Ordering};
use uefi::{table, Handle, Result, StatusExt};

#[cfg(doc)]
use uefi::Status;

pub use uefi::table::boot::{AllocateType, SearchType};
pub use uefi_raw::table::boot::MemoryType;

/// Global image handle. This is only set by [`set_image_handle`], and it is
/// only read by [`image_handle`].
static IMAGE_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Get the [`Handle`] of the currently-executing image.
#[must_use]
pub fn image_handle() -> Handle {
    let ptr = IMAGE_HANDLE.load(Ordering::Acquire);
    // Safety: the image handle must be valid. We know it is, because it was set
    // by `set_image_handle`, which has that same safety requirement.
    unsafe { Handle::from_ptr(ptr) }.expect("set_image_handle has not been called")
}

/// Update the global image [`Handle`].
///
/// This is called automatically in the `main` entry point as part of
/// [`uefi::entry`]. It should not be called at any other point in time, unless
/// the executable does not use [`uefi::entry`], in which case it should be
/// called once before calling other boot services functions.
///
/// # Safety
///
/// This function should only be called as described above, and the
/// `image_handle` must be a valid image [`Handle`]. The safety guarantees of
/// `open_protocol_exclusive` rely on the global image handle being correct.
pub unsafe fn set_image_handle(image_handle: Handle) {
    IMAGE_HANDLE.store(image_handle.as_ptr(), Ordering::Release);
}

fn boot_services_raw_panicking() -> NonNull<uefi_raw::table::boot::BootServices> {
    let st = table::system_table_raw_panicking();
    // SAFETY: valid per requirements of `set_system_table`.
    let st = unsafe { st.as_ref() };
    NonNull::new(st.boot_services).expect("boot services are not active")
}

/// Allocates memory pages from the system.
///
/// UEFI OS loaders should allocate memory of the type `LoaderData`.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: allocation failed.
/// * [`Status::INVALID_PARAMETER`]: `mem_ty` is [`MemoryType::PERSISTENT_MEMORY`],
///   [`MemoryType::UNACCEPTED`], or in the range [`MemoryType::MAX`]`..=0x6fff_ffff`.
/// * [`Status::NOT_FOUND`]: the requested pages could not be found.
pub fn allocate_pages(ty: AllocateType, mem_ty: MemoryType, count: usize) -> Result<NonNull<u8>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (ty, mut addr) = match ty {
        AllocateType::AnyPages => (0, 0),
        AllocateType::MaxAddress(addr) => (1, addr),
        AllocateType::Address(addr) => (2, addr),
    };
    let addr =
        unsafe { (bt.allocate_pages)(ty, mem_ty, count, &mut addr) }.to_result_with_val(|| addr)?;
    let ptr = addr as *mut u8;
    Ok(NonNull::new(ptr).expect("allocate_pages must not return a null pointer if successful"))
}

/// Frees memory pages allocated by [`allocate_pages`].
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: `ptr` was not allocated by [`allocate_pages`].
/// * [`Status::INVALID_PARAMETER`]: `ptr` is not page aligned or is otherwise invalid.
pub unsafe fn free_pages(ptr: NonNull<u8>, count: usize) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let addr = ptr.as_ptr() as PhysicalAddress;
    unsafe { (bt.free_pages)(addr, count) }.to_result()
}

/// Allocates from a memory pool. The pointer will be 8-byte aligned.
///
/// # Errors
///
/// * [`Status::OUT_OF_RESOURCES`]: allocation failed.
/// * [`Status::INVALID_PARAMETER`]: `mem_ty` is [`MemoryType::PERSISTENT_MEMORY`],
///   [`MemoryType::UNACCEPTED`], or in the range [`MemoryType::MAX`]`..=0x6fff_ffff`.
pub fn allocate_pool(mem_ty: MemoryType, size: usize) -> Result<NonNull<u8>> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let mut buffer = ptr::null_mut();
    let ptr =
        unsafe { (bt.allocate_pool)(mem_ty, size, &mut buffer) }.to_result_with_val(|| buffer)?;

    Ok(NonNull::new(ptr).expect("allocate_pool must not return a null pointer if successful"))
}

/// Frees memory allocated by [`allocate_pool`].
///
/// # Safety
///
/// The caller must ensure that no references into the allocation remain,
/// and that the memory at the allocation is not used after it is freed.
///
/// # Errors
///
/// * [`Status::INVALID_PARAMETER`]: `ptr` is invalid.
pub unsafe fn free_pool(ptr: NonNull<u8>) -> Result {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    unsafe { (bt.free_pool)(ptr.as_ptr()) }.to_result()
}

/// Returns an array of handles that support the requested protocol in a
/// pool-allocated buffer.
///
/// See [`SearchType`] for details of the available search operations.
///
/// # Errors
///
/// * [`Status::NOT_FOUND`]: no matching handles.
/// * [`Status::OUT_OF_RESOURCES`]: out of memory.
pub fn locate_handle_buffer(search_ty: SearchType) -> Result<HandleBuffer> {
    let bt = boot_services_raw_panicking();
    let bt = unsafe { bt.as_ref() };

    let (ty, guid, key) = match search_ty {
        SearchType::AllHandles => (0, ptr::null(), ptr::null()),
        SearchType::ByRegisterNotify(registration) => {
            (1, ptr::null(), registration.0.as_ptr().cast_const())
        }
        SearchType::ByProtocol(guid) => (2, guid as *const _, ptr::null()),
    };

    let mut num_handles: usize = 0;
    let mut buffer: *mut uefi_raw::Handle = ptr::null_mut();
    unsafe { (bt.locate_handle_buffer)(ty, guid, key, &mut num_handles, &mut buffer) }
        .to_result_with_val(|| HandleBuffer {
            count: num_handles,
            buffer: NonNull::new(buffer.cast())
                .expect("locate_handle_buffer must not return a null pointer"),
        })
}

/// A buffer returned by [`locate_handle_buffer`] that contains an array of
/// [`Handle`]s that support the requested protocol.
#[derive(Debug, Eq, PartialEq)]
pub struct HandleBuffer {
    count: usize,
    buffer: NonNull<Handle>,
}

impl Drop for HandleBuffer {
    fn drop(&mut self) {
        let _ = unsafe { free_pool(self.buffer.cast::<u8>()) };
    }
}

impl Deref for HandleBuffer {
    type Target = [Handle];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buffer.as_ptr(), self.count) }
    }
}
