// SPDX-License-Identifier: MIT OR Apache-2.0

pub fn test() {
    info!("Testing memory functions");

    bootservices::allocate_pages();
    bootservices::allocate_pool();
    bootservices::memory_map();

    global::alloc_vec();
    global::alloc_alignment();
}

/// Tests that directly use UEFI boot services to allocate memory.
mod bootservices {
    use alloc::vec::Vec;
    use uefi::boot;
    use uefi::boot::AllocateType;
    use uefi::mem::memory_map::{MemoryMap, MemoryMapMut};
    use uefi_raw::table::boot::MemoryType;

    /// Tests the `allocate_pages` boot service.
    pub fn allocate_pages() {
        let num_pages = 1;
        let ptr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, num_pages)
            .unwrap();
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % 4096, 0, "Page pointer is not page-aligned");

        // Verify the page can be written to.
        {
            let ptr = ptr.as_ptr();
            unsafe { ptr.write_volatile(0xff) };
            unsafe { ptr.add(4095).write_volatile(0xff) };
        }

        unsafe { boot::free_pages(ptr, num_pages) }.unwrap();
    }

    /// Tests the `allocate_pool` boot service.
    pub fn allocate_pool() {
        let ptr = boot::allocate_pool(MemoryType::LOADER_DATA, 10).unwrap();

        // Verify the allocation can be written to.
        {
            let ptr = ptr.as_ptr();
            unsafe { ptr.write_volatile(0xff) };
            unsafe { ptr.add(9).write_volatile(0xff) };
        }
        unsafe { boot::free_pool(ptr) }.unwrap();
    }

    /// Tests getting the memory map and performing a few sanity checks on it.
    pub fn memory_map() {
        info!("Testing memory map functions");

        let mut memory_map =
            boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to retrieve UEFI memory map");

        memory_map.sort();

        // Collect the descriptors into a vector
        let descriptors = memory_map.entries().copied().collect::<Vec<_>>();

        // Ensured we have at least one entry.
        // Real memory maps usually have dozens of entries.
        assert!(!descriptors.is_empty(), "Memory map is empty");

        let mut curr_value = descriptors[0];

        for value in descriptors.iter().skip(1) {
            if value.phys_start <= curr_value.phys_start {
                panic!("memory map sorting failed");
            }
            curr_value = *value;
        }

        // This is pretty much a basic sanity test to ensure returned memory
        // isn't filled with random values.
        let first_desc = descriptors[0];

        #[cfg(target_arch = "x86_64")]
        {
            let phys_start = first_desc.phys_start;
            assert_eq!(phys_start, 0, "Memory does not start at address 0");
        }
        let page_count = first_desc.page_count;
        assert!(page_count != 0, "Memory map entry has size zero");
    }
}

/// Tests that use [`uefi::allocator::Allocator`], which is configured as the
/// global allocator.
mod global {
    use alloc::boxed::Box;
    use uefi_raw::table::boot::PAGE_SIZE;

    /// Simple test to ensure our custom allocator works with the `alloc` crate.
    pub fn alloc_vec() {
        info!("Allocating a vector using the global allocator");

        #[allow(clippy::useless_vec)]
        let mut values = vec![-5, 16, 23, 4, 0];

        values.sort_unstable();

        assert_eq!(values[..], [-5, 0, 4, 16, 23], "Failed to sort vector");
    }

    /// Simple test to ensure our custom allocator works with correct alignment.
    #[allow(dead_code)] // Ignore warning due to field not being read.
    pub fn alloc_alignment() {
        {
            info!("Allocating a structure with alignment of 0x100 using the global allocator");
            #[repr(align(0x100))]
            struct Block([u8; 0x100]);

            let value = vec![Block([1; 0x100])];
            assert_eq!(value.as_ptr() as usize % 0x100, 0, "Wrong alignment");
        }
        {
            info!("Allocating a memory page ({PAGE_SIZE}) using the global allocator");
            #[repr(align(4096))]
            struct Page([u8; PAGE_SIZE]);
            let value = Box::new(Page([0; PAGE_SIZE]));
            assert_eq!(
                value.0.as_ptr().align_offset(PAGE_SIZE),
                0,
                "Wrong alignment"
            );
        }
    }
}
