use uefi::prelude::*;
use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    test_watchdog(bt);
}

fn test_watchdog(bt: &BootServices) {
    // Disable the UEFI watchdog timer
    bt.set_watchdog_timer(0, 0x10000, None)
        .expect_success("Could not set watchdog timer");
}
