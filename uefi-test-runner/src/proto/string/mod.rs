use uefi::prelude::*;

pub fn test(bt: &BootServices) {
    info!("Testing String protocols");

    unicode_collation::test(bt);
}

mod unicode_collation;
