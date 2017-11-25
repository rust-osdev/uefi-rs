use uefi::Result;
use uefi::table::boot;

pub fn boot_services_test(bt: &boot::BootServices) -> Result<()> {
    let ty = boot::AllocateType::AnyPages;
    let mem_ty = boot::MemoryType::LoaderData;
    let pgs = bt.allocate_pages(ty, mem_ty, 1)?;

    info!("Allocated memory of type {:?} at {:#X}", mem_ty, pgs);

    bt.free_pages(pgs, 1)?;

    Ok(())
}
