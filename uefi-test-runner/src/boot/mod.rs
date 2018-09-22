use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    memory::test(bt);
    misc::test(bt);
}

mod memory;
mod misc;
