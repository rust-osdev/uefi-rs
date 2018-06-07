use uefi::Result;
use uefi::table::boot;

use core::{mem, slice};

pub fn boot_services_test(bt: &boot::BootServices) -> Result<()> {
    let ty = boot::AllocateType::AnyPages;
    let mem_ty = boot::MemoryType::LoaderData;
    let pgs = bt.allocate_pages(ty, mem_ty, 1)?;

    info!("Allocated memory of type {:?} at {:#X}", mem_ty, pgs);

    bt.free_pages(pgs, 1)?;

    let map_sz = bt.memory_map_size();
    // 2 extra pages should be enough.
    let buf_sz = (map_sz / 4096) + 2;
    let pages = bt.allocate_pages(ty, mem_ty, buf_sz)
        .expect("Failed to allocate memory for memory map");

    let buffer = unsafe {
        let ptr = mem::transmute::<_, *mut u8>(pages);
        slice::from_raw_parts_mut(ptr, buf_sz * 4096)
    };

    let (key, mut desc_iter) = bt.memory_map(buffer)?;
    info!("Memory map key {:?}", key);
    info!("Found information for {} memory descriptors", desc_iter.len());
    info!("First descriptor: {:?}", desc_iter.next().unwrap());

    Ok(())
}
