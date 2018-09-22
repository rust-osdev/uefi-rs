use uefi::table::boot::{BootServices, AllocateType, MemoryType, MemoryDescriptor};

use core::mem;
use crate::alloc::vec::Vec;

pub fn test(bt: &BootServices) {
    allocate_pages(bt);
    vec_alloc();
    memmove(bt);

    memory_map(bt);
}

fn allocate_pages(bt: &BootServices) {
    let ty = AllocateType::AnyPages;
    let mem_ty = MemoryType::LoaderData;
    let pgs = bt.allocate_pages(ty, mem_ty, 1)
        .expect("Failed to allocate a page of memory");

    assert_eq!(pgs % 4096, 0, "Page pointer is not page-aligned");

    // Simple page structure to test this code.
    #[repr(C, align(4096))]
    struct Page([u8; 4096]);

    let page: &Page = unsafe { mem::transmute(pgs) };

    let mut buf = page.0;

    // If these don't fail then we properly allocated some memory.
    buf[0] = 0xF0;
    buf[4095] = 0x23;

    // Clean up to avoid memory leaks.
    bt.free_pages(pgs, 1).unwrap();
}

// Simple test to ensure our custom allocator works with the `alloc` crate.
fn vec_alloc() {
    let mut values = vec![-5, 16, 23, 4, 0];

    values.sort();

    assert_eq!(values[..], [-5, 0, 4, 16, 23], "Failed to sort vector");
}

// Test that the `memmove` / `memset` functions work.
fn memmove(bt: &BootServices) {
    let src = [1, 2, 3, 4];
    let mut dest = [0u8; 4];

    // Fill the buffer with a value
    bt.memset(dest.as_mut_ptr(), dest.len(), 1);

    assert_eq!(dest, [1; 4], "Failed to set memory");

    // Copy other values on it
    bt.memmove(dest.as_mut_ptr(), src.as_ptr(), dest.len());

    assert_eq!(dest, src, "Failed to copy memory");
}

fn memory_map(bt: &BootServices) {
    // Get an estimate of the memory map size.
    let map_sz = bt.memory_map_size();

    // 8 extra descriptors should be enough.
    let buf_sz = map_sz + 8 * mem::size_of::<MemoryDescriptor>();

    // We will use vectors for convencience.
    let mut buffer = Vec::with_capacity(buf_sz);

    unsafe {
        buffer.set_len(buf_sz);
    }

    let (_key, mut desc_iter) = bt.memory_map(&mut buffer)
        .expect("Failed to retrieve UEFI memory map");

    // Ensured we have at least one entry.
    // Real memory maps usually have dozens of entries.
    assert!(desc_iter.len() > 0, "Memory map is empty");

    // This is pretty much a sanity test to ensure returned memory isn't filled with random values.
    let first_desc = desc_iter.next().unwrap();

    let phys_start = first_desc.phys_start;

    assert_eq!(phys_start, 0, "Memory does not start at address 0");
}
