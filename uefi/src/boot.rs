//! UEFI boot services.
//!
//! These functions will panic if called after exiting boot services.

use crate::data_types::PhysicalAddress;
use core::ptr::{self, NonNull};
use uefi::{table, Result, StatusExt};

#[cfg(doc)]
use uefi::Status;

pub use uefi::table::boot::AllocateType;
pub use uefi_raw::table::boot::MemoryType;

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
