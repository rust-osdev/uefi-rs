use uefi::Result;
use uefi::table::boot;

pub fn boot_services_test(bt: &boot::BootServices) -> Result<()> {
    let ty = boot::AllocateType::AnyPages;
    let mem_ty = boot::MemoryType::LoaderData;
    let pgs = bt.allocate_pages(ty, mem_ty, 1)?;

    info!("Allocated memory of type {:?} at {:#X}", mem_ty, pgs);

    bt.free_pages(pgs, 1)?;

    let mut mem_desc: [boot::MemoryDescriptor; 32] = [boot::MemoryDescriptor::default(); 32];
    let mut buffer: [u8; 4096] = [0; 4096];
    let (num_desc, _) = bt.get_memory_map(&mut mem_desc,&mut buffer)?;
    info!("Found information for {} memory descriptors", num_desc);

    Ok(())
}
