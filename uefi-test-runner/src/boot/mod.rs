use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    info!("Testing boot services");
    memory::test(bt);
    misc::test(bt);
}

mod memory;
mod misc;
