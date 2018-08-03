use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    memory::test(bt);
}

mod memory;
