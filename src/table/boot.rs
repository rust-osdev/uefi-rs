//! UEFI services available during boot.

use super::Header;

/// Contains pointers to all of the boot services.
#[repr(C)]
pub struct BootServices {
    header: Header,
}

impl super::Table for BootServices {
    const SIGNATURE: u64 = 0x5652_4553_544f_4f42;
}
