use uefi::table::boot::BootServices;

pub fn test(bt: &BootServices) {
    test_watchdog(bt);
}

fn test_watchdog(bt: &BootServices) {
    // Disable the UEFI watchdog timer
    bt.set_watchdog_timer(0, 0x10000, None)
        .expect("Could not set watchdog timer")
        .expect("Warnings encountered while setting watchdog timer");
}
