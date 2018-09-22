use uefi::table::boot::BootServices;
use core::ptr;

pub fn test(bt: &BootServices) {
    test_watchdog(bt);
}

fn test_watchdog(bt: &BootServices) {
    bt.set_watchdog_timer(0, 0, 0, ptr::null_mut());
}
