use uefi::prelude::*;
use uefi::table::boot::{AllocateType, BootServices, MemoryDescriptor, MemoryType};

use crate::alloc::vec::Vec;
use core::mem;

pub fn test(bt: &BootServices) {
    info!("Testing memory functions");

    allocate_pages(bt);
    vec_alloc();
    alloc_alignment();
    memmove(bt);

    memory_map(bt);
}

fn allocate_pages(bt: &BootServices) {
    info!("Allocating some pages of memory");

    let ty = AllocateType::AnyPages;
    let mem_ty = MemoryType::LOADER_DATA;
    let pgs = bt
        .allocate_pages(ty, mem_ty, 1)
        .expect_success("Failed to allocate a page of memory");

    assert_eq!(pgs % 4096, 0, "Page pointer is not page-aligned");

    // Reinterprete the page as an array of bytes
    let buf = unsafe { &mut *(pgs as *mut [u8; 4096]) };

    // If these don't fail then we properly allocated some memory.
    buf[0] = 0xF0;
    buf[4095] = 0x23;

    // Clean up to avoid memory leaks.
    bt.free_pages(pgs, 1).unwrap_success();
}

// Simple test to ensure our custom allocator works with the `alloc` crate.
fn vec_alloc() {
    info!("Allocating a vector through the `alloc` crate");

    let mut values = vec![-5, 16, 23, 4, 0];

    values.sort_unstable();

    assert_eq!(values[..], [-5, 0, 4, 16, 23], "Failed to sort vector");
}

// Simple test to ensure our custom allocator works with correct alignment.
fn alloc_alignment() {
    info!("Allocating a structure with alignment to 0x100");

    #[repr(align(0x100))]
    struct Block([u8; 0x100]);

    let value = vec![Block([1; 0x100])];
    assert_eq!(value.as_ptr() as usize % 0x100, 0, "Wrong alignment");
}

// Test that the `memmove` / `memset` functions work.
fn memmove(bt: &BootServices) {
    info!("Testing the `memmove` / `memset` functions");

    let src = [1, 2, 3, 4];
    let mut dest = [0u8; 4];

    // Fill the buffer with a value
    unsafe {
        bt.memset(dest.as_mut_ptr(), dest.len(), 1);
    }

    assert_eq!(dest, [1; 4], "Failed to set memory");

    // Copy other values on it
    unsafe {
        bt.memmove(dest.as_mut_ptr(), src.as_ptr(), dest.len());
    }

    assert_eq!(dest, src, "Failed to copy memory");
}

fn memory_map(bt: &BootServices) {
    info!("Testing memory map functions");

    // Get an estimate of the memory map size.
    let map_sz = bt.memory_map_size();

    // 8 extra descriptors should be enough.
    let buf_sz = map_sz + 8 * mem::size_of::<MemoryDescriptor>();

    // We will use vectors for convencience.
    let mut buffer = Vec::with_capacity(buf_sz);

    unsafe {
        buffer.set_len(buf_sz);
    }

    let (_key, desc_iter) = bt
        .memory_map(&mut buffer)
        .expect_success("Failed to retrieve UEFI memory map");

    // Collect the descriptors into a vector
    let descriptors = desc_iter.copied().collect::<Vec<_>>();

    // Ensured we have at least one entry.
    // Real memory maps usually have dozens of entries.
    assert!(!descriptors.is_empty(), "Memory map is empty");

    // This is pretty much a sanity test to ensure returned memory isn't filled with random values.
    let first_desc = descriptors[0];

    #[cfg(target_arch = "x86_64")]
    {
        let phys_start = first_desc.phys_start;
        assert_eq!(phys_start, 0, "Memory does not start at address 0");
    }
    let page_count = first_desc.page_count;
    assert!(page_count != 0, "Memory map entry has zero size");
}
