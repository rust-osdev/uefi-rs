// SPDX-License-Identifier: MIT OR Apache-2.0

use uefi::mem::memory_map::*;

/// This test imitates a kernel that receives the UEFI memory map as boot
/// information.
#[test]
fn parse_boot_information_efi_mmap() {
    let desc_size = size_of::<MemoryDescriptor>();
    let mut mmap_source = [
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x3000,
            virt_start: 0x3000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x2000,
            virt_start: 0x2000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
        MemoryDescriptor {
            ty: MemoryType::CONVENTIONAL,
            phys_start: 0x1000,
            virt_start: 0x1000,
            page_count: 1,
            att: MemoryAttribute::WRITE_BACK,
        },
    ];
    let map_size = mmap_source.len() * desc_size;
    let meta = MemoryMapMeta {
        map_size,
        desc_size,
        map_key: Default::default(),
        desc_version: MemoryDescriptor::VERSION,
    };
    let mmap =
        unsafe { core::slice::from_raw_parts_mut(mmap_source.as_mut_ptr().cast::<u8>(), map_size) };

    // BOOT INFORMATION END
    //
    // BEGIN PARSING
    // This scenario is similar to what a kernel parsing a boot information
    // would do.

    let mmap = MemoryMapRefMut::new(mmap, meta).unwrap();
    assert_eq!(mmap.entries().copied().collect::<Vec<_>>(), mmap_source);
}
